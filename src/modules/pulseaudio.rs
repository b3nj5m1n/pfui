use anyhow::{anyhow, Result};
use std::{rc::Rc, sync::Mutex, thread::sleep, time::Duration};

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
            match cnxt.connect(None, pulse::context::FlagSet::NOAUTOSPAWN, None) {
                Ok(_) => return Ok(Self { cnxt, mnlp }),
                Err(_) => {}
            }
            sleep(Duration::from_secs(timeout));
        }
        return Err(anyhow!("Timed out creating connection"));
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

pub struct PulseAudio {}

impl Module for PulseAudio {
    type Connection = Connection;
    fn connect(&mut self, timeout: u64) -> Result<Self::Connection> {
        return Ok(Connection::new(timeout)?);
    }

    fn start(&mut self, timeout: u64) -> Result<()> {
        let mut conn = self.connect(timeout)?;
        if let Err(_) = conn.connect() {
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
        {
            let i = Rc::clone(&introspector);
            let d = i.lock().unwrap();
            d.get_sink_info_list(|list| match list {
                pulse::callbacks::ListResult::Item(sink) => {
                    let avg = sink.volume.avg().0;
                    let percent =
                        u32::try_from((f64::from(avg) / f64::from(0x10000) * 100.0).round() as i64);
                    match percent {
                        Ok(percent) => crate::print(&Some(Data { volume: percent, muted: sink.mute })),
                        Err(_) => crate::print::<Data>(&None),
                    }
                }
                _ => {}
            });
        }
        conn.cnxt
            .set_subscribe_callback(Some(Box::new(move |_facility, _operation, index| {
                let i = Rc::clone(&introspector);
                let d = i.lock().unwrap();
                d.get_sink_info_by_index(index, |s| match s {
                    pulse::callbacks::ListResult::Item(item) => {
                        let avg = item.volume.avg().0;
                        let percent = u32::try_from(
                            (f64::from(avg) / f64::from(0x10000) * 100.0).round() as i64,
                        );
                        match percent {
                            Ok(percent) => crate::print(&Some(Data { volume: percent, muted: item.mute })),
                            Err(_) => crate::print::<Data>(&None),
                        }
                    }
                    _ => {}
                });
            })));
        match conn.mnlp.run() {
            Ok(_retval) => return Ok(()),
            Err((e, _retval)) => return Err(anyhow::Error::new(e)),
        }
    }

    fn output(&self, conn: &mut Self::Connection) {}
}
