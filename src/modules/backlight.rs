use anyhow::Result;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use std::{
    fs::read_to_string,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    process::exit,
};
pub struct Backlight {
    notifier: Inotify,
    max_brightness: PathBuf,
    brightness: PathBuf,
}

impl Backlight {
    pub fn new() -> Self {
        let notifier = Inotify::init(InitFlags::empty()).unwrap();
        // assuming first entry in /sys/class/backlight/ is the display path,
        let Some(Ok(display_path)) = std::fs::read_dir("/sys/class/backlight/").unwrap_or_else(|error| {
            eprintln!("Backlight not found: {error:?}");
                exit(1);
        }).next() else{
            eprintln!("No Backlight device found");
            exit(1);
        };
        let max_brightness = display_path.path().join("max_brightness");
        let actual_brightness = display_path.path().join("actual_brightness");
        // listen only for brightness changes, ie if the file modified
        notifier
            .add_watch(&actual_brightness, AddWatchFlags::IN_MODIFY)
            .unwrap_or_else(|error_nu| {
                eprintln!("Failed to listen on {display_path:?}: {error_nu}");
                exit(1);
            });
        Self {
            notifier,
            max_brightness,
            brightness: actual_brightness,
        }
    }
    pub fn listen(&mut self) -> Result<()> {
        let max_bright: u64 = read_to_string(&self.max_brightness)?
            .trim()
            .parse::<u64>()?;
        let mut bright_fd = std::fs::OpenOptions::new()
            .read(true)
            .open(&self.brightness)?;
        let mut bright_buf = String::new();
        bright_fd.read_to_string(&mut bright_buf)?;
        let mut bright_val: u64 = bright_buf.trim().parse::<u64>()?;
        let mut bright_perc = ((bright_val as f64 / max_bright as f64) * 100.0) as u64;
        crate::print(&Some(bright_perc));
        loop {
            self.notifier.read_events()?.iter().for_each(|_event| {
                bright_buf.clear();
                bright_fd.seek(SeekFrom::Start(0)).unwrap();
                bright_fd.read_to_string(&mut bright_buf).unwrap();
                bright_val = bright_buf.trim().parse::<u64>().unwrap();
                bright_perc = ((bright_val as f64 / max_bright as f64) * 100.0) as u64;
                crate::print(&Some(bright_perc));
            })
        }
    }
}
