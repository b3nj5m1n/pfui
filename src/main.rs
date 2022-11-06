use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

mod modules;
use modules::mpd;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Mpd { },
}

trait Module {
    fn start(&self);

    fn output<T: serde::Serialize>(&self, info: &T) {
        println!("\n{}", serde_json::to_string(info).unwrap());
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Mpd { .. }) => {
            if cfg!(feature = "mpd") {
                let module = mpd::Mpd {}.start();
            } else {
                println!("Feature not enabled");
            }
        }
        None => {}
    }
}
