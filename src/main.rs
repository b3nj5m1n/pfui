use std::{fmt, thread::sleep, process::exit};

use clap::{ColorChoice, Parser, Subcommand};

mod modules;
use modules::mpd;
use serde::Serialize;

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
    Mpd {},
}

#[derive(Debug, Serialize)]
struct Output<T: serde::Serialize> {
    ok: u8,
    data: Option<T>,
}

trait Module {
    type Connection;

    /// This starts the event listening loop
    fn start(&self, timeout: u64) -> Result<(), Box<dyn std::error::Error>>;

    /// This connects to a server or similar, returns whatever is necessary to communicate with the
    /// server
    fn connect(&self, timeout: u64) -> Self::Connection;

    /// This generates the data and calls print
    fn output(&self, conn: &mut Self::Connection);

    /// This actually prints the json representation of the data
    fn print<T: serde::Serialize>(&self, info: &Option<T>) {
        let output = if let Some(data) = info {
            Output {
                ok: 1,
                data: Some(data),
            }
        } else {
            Output {
                ok: 0,
                data: None,
            }
        };
        println!("\n{}", serde_json::to_string(&output).unwrap());
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start(start)) => match start.module {
            Modules::Mpd {} => {
                if cfg!(feature = "mpd") {
                    while let Err(..) = (mpd::Mpd {}.start(5)) { }
                    exit(0);
                } else {
                    println!("Feature not enabled");
                }
            }
        },
        None => {}
    }
}
