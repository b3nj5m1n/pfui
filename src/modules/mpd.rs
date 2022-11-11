use std::{thread::sleep, time::Duration};
use anyhow::Result;

use mpd::{idle::Subsystem, Client, Idle, Song as MpdSong, State as OldMpdState, Status};
use serde::{Serialize, Serializer};

use crate::Module;

#[derive(Debug, Serialize)]
struct Data {
    song: Song,
    state: State,
    options: Options,
}
#[derive(Debug, Serialize)]
struct Song {
    file_path: Option<String>,
    title: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    date: Option<String>,
    genre: Option<String>,
}
impl Song {
    fn empty() -> Self {
        Song {
            file_path: None,
            title: None,
            album: None,
            artist: None,
            date: None,
            genre: None,
        }
    }
}
#[derive(Debug, Serialize)]
struct State {
    elapsed: Option<i64>,
    duration: Option<i64>,
    progress: Option<i8>,
    status: Option<MpdState>,
}
#[derive(Debug, Serialize)]
struct Options {
    volume: i8,
    repeat: bool,
    random: bool,
}

#[derive(Debug)]
struct MpdState(OldMpdState);
impl Serialize for MpdState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0 == OldMpdState::Play {
            return serializer.serialize_i8(0);
        } else if self.0 == OldMpdState::Pause {
            return serializer.serialize_i8(1);
        } else if self.0 == OldMpdState::Stop {
            return serializer.serialize_i8(2);
        }
        Err(serde::ser::Error::custom("Error serializing MpdState"))
    }
}

impl From<&MpdSong> for Song {
    fn from(value: &MpdSong) -> Self {
        Song {
            file_path: Some(value.file.clone()),
            title: value.title.clone(),
            album: value.tags.get("Album").cloned(),
            artist: value.tags.get("Artist").cloned(),
            date: value.tags.get("Date").cloned(),
            genre: value.tags.get("Genre").cloned(),
        }
    }
}

impl From<&Status> for State {
    fn from(value: &Status) -> Self {
        let elapsed = if let Some(elapsed) = value.elapsed {
            Some(elapsed.num_seconds())
        } else {
            None
        };
        let duration = if let Some(duration) = value.duration {
            Some(duration.num_seconds())
        } else {
            None
        };
        let progress = if let (Some(elapsed), Some(duration)) = (elapsed, duration) {
            if let (Ok(elapsed), Ok(duration)) = (i32::try_from(elapsed), i32::try_from(duration)) {
                i8::try_from(((f64::from(elapsed) / f64::from(duration)) * 100.0).round() as i64)
                    .ok()
            } else {
                None
            }
        } else {
            None
        };
        return State {
            elapsed,
            duration,
            progress,
            status: Some(MpdState(value.state)),
        };
    }
}

impl From<&Status> for Options {
    fn from(value: &Status) -> Self {
        Options {
            volume: value.volume,
            repeat: value.repeat,
            random: value.random,
        }
    }
}

impl
    TryFrom<(
        Result<std::option::Option<MpdSong>, mpd::error::Error>,
        Result<Status, mpd::error::Error>,
    )> for Data
{
    type Error = mpd::error::Error;
    fn try_from(
        value: (
            Result<std::option::Option<MpdSong>, mpd::error::Error>,
            Result<Status, mpd::error::Error>,
        ),
    ) -> Result<Self, Self::Error> {
        let status = value.1?;
        if let Ok(Some(current_song)) = value.0 {
            return Ok(Data {
                song: Song::from(&current_song),
                state: State::from(&status),
                options: Options::from(&status),
            });
        } else {
            return Ok(Data {
                song: Song::empty(),
                state: State::from(&status),
                options: Options::from(&status),
            });
        }
    }
}

fn get_info(conn: &mut Client) -> Option<Data> {
    let current_song = conn.currentsong();
    let status = conn.status();
    Data::try_from((current_song, status)).ok()
}

pub struct Mpd {}

impl Module for Mpd {
    type Connection = Client;
    fn connect(&mut self, timeout: u64) -> Result<Self::Connection> {
        let mut conn_ = Client::connect("127.0.0.1:6600");
        while let Err(..) = conn_ {
            conn_ = Client::connect("127.0.0.1:6600");
            crate::print(&None::<Data>);
            sleep(Duration::new(timeout, 0));
        }
        return Ok(conn_?);
    }
    fn output(&self, conn: &mut Self::Connection) {
        let info = get_info(conn);
        crate::print(&info);
    }
    fn start(&mut self, timeout: u64) -> Result<()> {
        let mut conn = self.connect(timeout)?;
        self.output(&mut conn);
        loop {
            let guard = conn.idle(&[Subsystem::Player, Subsystem::Mixer, Subsystem::Options])?;
            match guard.get() {
                Ok(_) => self.output(&mut conn),
                Err(_) => {}
            }
        }
    }
}
