use anyhow::{anyhow, Result};
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use pulse::{
    context::Context,
    mainloop::standard::{IterateResult, Mainloop},
};
use serde::Serialize;

use crate::Module;

#[derive(Debug, Serialize)]
struct Data {
    volume: u32,
    muted: bool,
}

pub struct Connection {
    cnxt: Context,
    mnlp: Mainloop,
}

impl Connection {
    fn new(timeout: u64) -> Result<Self> {
        let mnlp = Mainloop::new().unwrap();
        for _ in 0..10 {
            let mut cnxt = Context::new(&mnlp, "pfui_listener").unwrap();
            if cnxt
                .connect(None, pulse::context::FlagSet::NOAUTOSPAWN, None)
                .is_ok()
            {
                return Ok(Self { cnxt, mnlp });
            }
            sleep(Duration::from_secs(timeout));
        }
        Err(anyhow!("Timed out creating connection"))
    }
    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match self.mnlp.iterate(false) {
                IterateResult::Err(e) => {
                    return Err(Box::new(e));
                }
                IterateResult::Quit(_) => {
                    return Err(Box::new(pulse::error::Code::BadState));
                }
                IterateResult::Success(_) => {}
            }
            match self.cnxt.get_state() {
                pulse::context::State::Ready => {
                    return Ok(());
                }
                pulse::context::State::Failed | pulse::context::State::Terminated => {
                    return Err(Box::new(pulse::error::Code::BadState));
                }
                _ => {}
            }
        }
    }
}

fn print_state(avg: u32, state: bool) {
    let percent = u32::try_from((f64::from(avg) / f64::from(0x10000) * 100.0).round() as i64);
    match percent {
        Ok(percent) => crate::print(&Some(Data {
            volume: percent,
            muted: state,
        })),
        Err(_) => crate::print::<Data>(&None),
    }
}

pub struct PulseAudio {}

impl Module for PulseAudio {
    type Connection = Connection;
    fn connect(&mut self, timeout: u64) -> Result<Self::Connection> {
        Connection::new(timeout)
    }

    fn start(&mut self, timeout: u64) -> Result<()> {
        let default_sink_index: Arc<std::sync::RwLock<Option<u32>>> =
            Arc::new(std::sync::RwLock::new(None));
        let mut conn = self.connect(timeout)?;
        if conn.connect().is_err() {
            return Err(anyhow!("Error establishing connection"));
        }
        let interest = pulse::context::subscribe::InterestMaskSet::SINK;
        conn.cnxt.subscribe(interest, |_| {});
        // FIXME This is quite hacky, and tbh the api is quite confusing, I'm not sure how to
        // identify the default sink
        // (To be clear I'm not talking about the next section but this whole module in general)
        // One possible way to solve this would be to output the data for each available sink and
        // let the user figure out which one to use.
        let introspector = Rc::new(Mutex::new(conn.cnxt.introspect()));
        // print the data for initialization
        {
            let i = Rc::clone(&introspector);
            let d = i.lock().unwrap();
            let index_c = default_sink_index.clone();
            d.get_sink_info_by_name("@DEFAULT_SINK@", move |list| {
                if let pulse::callbacks::ListResult::Item(sink) = list {
                    if let Ok(mut sink_index) = index_c.write() {
                        *sink_index = Some(sink.index);
                    }
                    print_state(sink.volume.avg().0, sink.mute);
                }
            });
        }
        conn.cnxt
            .set_subscribe_callback(Some(Box::new(move |facility, _operation, index| {
                if let Ok(sink_index_guard) = default_sink_index.read() {
                    if let Some(sink_index) = *sink_index_guard {
                        if index == sink_index {
                            let i = Rc::clone(&introspector);
                            let d = i.lock().unwrap();
                            if let Some(pulse::context::subscribe::Facility::Sink) = facility {
                                // eprintln!("Event facility: {facility:?}, _operation: {_operation:?}");
                                d.get_sink_info_by_index(index, |s| {
                                    if let pulse::callbacks::ListResult::Item(item) = s {
                                        print_state(item.volume.avg().0, item.mute);
                                    }
                                });
                            }
                        }
                    } else {
                        eprintln!("Error: Failed to get default sink");
                        std::process::exit(1);
                    }
                }
            })));
        match conn.mnlp.run() {
            Ok(_retval) => Ok(()),
            Err((e, _retval)) => Err(anyhow::Error::new(e)),
        }
    }

    #[allow(unused)]
    fn output(&self, conn: &mut Self::Connection) {}
}
