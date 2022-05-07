use structopt::StructOpt;

mod init;
mod util;

#[derive(Debug, StructOpt, Clone)]
pub enum Command {
    #[structopt(external_subcommand)]
    Init(Vec<String>),
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "meld",
    about = "meld config management client",
    author = "drew <drew@parker.systems>"
)]
pub struct MainArgs {
    // Activate debug mode
    #[structopt(short, long, help = "enable debug/verbose messages")]
    pub debug: bool,

    // Path to the meld bin to use
    #[structopt(help = "path to the meld bin")]
    pub bin: String,

    // Meld command
    #[structopt(help = "meld command", subcommand)]
    pub command: Command,
}

fn main() {
    let args = MainArgs::from_args();

    let margs = args.clone();

    let res = match args.command {
        Command::Init(mode_args) => init::init_core(margs, mode_args),
    };

    if res {
        util::good_message("Completed Successfully");
    }
}
