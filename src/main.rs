use anyhow::Result;
use std::process::exit;

use clap::{ColorChoice, Parser, Subcommand};

mod modules;
use modules::{hyprland, mpd, pulseaudio, sway};
use serde::Serialize;

use crate::modules::backlight;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Efficiently generate content for statusbars",
    long_about = None,
    subcommand_required = true,
    arg_required_else_help = true,
    color = ColorChoice::Auto,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Start(Start),
}

#[derive(Parser)]
#[command(about = "Start a module, do `pfui start --help` for list of available modules")]
struct Start {
    #[structopt(subcommand)]
    pub module: Modules,
}

#[derive(Subcommand)]
enum Modules {
    Mpd,
    #[command(name = "pulseaudio")]
    PulseAudio,
    #[command(alias = "i3")]
    Sway,
    #[command(subcommand)]
    Hyprland(hyprland::HyprlandOpts),
    Backlight,
}

#[derive(Debug, Serialize)]
struct Output<T: serde::Serialize> {
    ok: u8,
    data: Option<T>,
}

trait Module {
    type Connection;

    /// This starts the event listening loop
    fn start(&mut self, timeout: u64) -> Result<()>;

    /// This connects to a server or similar, returns whatever is necessary to communicate with the
    /// server
    fn connect(&mut self, timeout: u64) -> Result<Self::Connection>;

    /// This generates the data and calls print
    fn output(&self, conn: &mut Self::Connection);
}
/// This actually prints the json representation of the data
pub fn print<T: serde::Serialize>(info: &Option<T>) {
    let output = if let Some(data) = info {
        Output {
            ok: 1,
            data: Some(data),
        }
    } else {
        Output { ok: 0, data: None }
    };
    println!("{}", serde_json::to_string(&output).unwrap());
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start(start)) => match start.module {
            Modules::Mpd => {
                if cfg!(feature = "mpd") {
                    while let Err(..) = (mpd::Mpd {}.start(5)) {}
                    exit(0);
                } else {
                    println!("Feature not enabled");
                }
            }
            Modules::PulseAudio => {
                if cfg!(feature = "pulseaudio") {
                    while let Err(..) = (pulseaudio::PulseAudio {}.start(5)) {}
                    exit(0);
                } else {
                    println!("Feature not enabled");
                }
            }
            Modules::Sway => {
                if cfg!(feature = "sway") {
                    while let Err(..) = (sway::Sway {}.start(5)) {}
                    exit(0);
                } else {
                    println!("Feature not enabled");
                }
            }
            Modules::Hyprland(ref opts) => {
                if cfg!(feature = "hyprland") {
                    while let Err(..) = hyprland::HyprlandListener::new(opts).listen() {}
                    exit(0);
                } else {
                    println!("Feature not enabled");
                }
            }
            Modules::Backlight => {
                if cfg!(feature = "backlight") {
                    backlight::Backlight::new().listen().unwrap();
                } else {
                    eprintln!("Feature not enabled");
                }
            }
        },
        None => {}
    }
}
