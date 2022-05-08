use init::InitArgs;
use structopt::StructOpt;

mod init;
mod util;

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
    let args = Args::from_args();

    let margs = args.clone();

    let res = match args.command {
        Command::Init(mod_args) => init::init_core(margs, mod_args),
    };

    if res {
        util::good_message("Completed Successfully");
    }
}
