use mpd::{Client, Idle, State};
use serde::Serialize;

use crate::Module;

#[derive(Debug, Serialize)]
struct Info {
    filename: String,
    title: Option<String>,
    album: Option<String>,
    artist: Option<String>,
    date: Option<String>,
    genre: Option<String>,
    volume: i8,
    repeat: bool,
    random: bool,
    /* state: State,
    elapsed: Option<time::Duration>,
    duration: Option<time::Duration>, */
}

fn get_info(conn: &mut Client) -> Option<Info> {
    let current_song = conn.currentsong();
    let status = conn.status();
    if let (Ok(Some(song)), Ok(status)) = (current_song, status) {
        let song_info = Info {
            filename: song.file,
            title: song.title,
            album: song.tags.get("Album").cloned(),
            artist: song.tags.get("Artist").cloned(),
            date: song.tags.get("Date").cloned(),
            genre: song.tags.get("Genre").cloned(),
            volume: status.volume,
            repeat: status.repeat,
            random: status.random,
            /* state: status.state,
            elapsed: status.elapsed,
            duration: status.duration, */
        };
        return Some(song_info);
    }
    return None;
}

pub struct Mpd {}

impl Module for Mpd {
    fn start(&self) {
        let mut conn = Client::connect("127.0.0.1:6600").unwrap();
        loop {
            let guard = conn.idle(&[]).unwrap();
            match guard.get() {
                Ok(_) => {
                    let info = get_info(&mut conn);
                    self.output(&info);
                }
                Err(_) => continue,
            }
        }
    }
}
