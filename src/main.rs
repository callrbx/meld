#![crate_name = "meld"]
use init::InitArgs;
use log::{error, info};
use structopt::StructOpt;

mod init;

/// Declare submodule argument types for matching
#[derive(Debug, StructOpt, Clone)]
pub enum Command {
    Init(InitArgs),
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "meld",
    about = "meld config management client",
    author = "drew <drew@parker.systems>"
)]
pub struct Args {
    // Path to the meld bin to use
    #[structopt(help = "path to the meld bin")]
    pub bin: String,

    // Meld command
    #[structopt(help = "meld command", subcommand)]
    pub command: Command,
}

fn main() {
    env_logger::init();

    let args = Args::from_args();

    let main_args = args.clone();

    let res = match args.command {
        Command::Init(mod_args) => init::init_core(main_args, mod_args),
    };

    match res {
        Ok(_) => {
            info!("No Errors");
            std::process::exit(0)
        }
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
}
