use clap::{Parser, Subcommand, ColorChoice};
use serde::{Deserialize, Serialize};

mod modules;
use modules::mpd;

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

trait Module {
    fn start(&self);

    fn output<T: serde::Serialize>(&self, info: &T) {
        println!("\n{}", serde_json::to_string(info).unwrap());
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start(start)) => match start.module {
            Modules::Mpd {} => {
                if cfg!(feature = "mpd") {
                    mpd::Mpd {}.start();
                } else {
                    println!("Feature not enabled");
                }
            }
        },
        None => {}
    }
}
