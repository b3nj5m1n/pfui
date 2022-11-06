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
    ok: bool,
    data: Option<T>,
}

trait Module {
    fn start(&self, timeout: u64);

    fn output<T: serde::Serialize>(&self, info: &Option<T>) {
        let output = if let Some(data) = info {
            Output { ok: true, data: Some(data) }
        } else {
            Output {
                ok: false,
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
                    mpd::Mpd {}.start(10);
                } else {
                    println!("Feature not enabled");
                }
            }
        },
        None => {}
    }
}
