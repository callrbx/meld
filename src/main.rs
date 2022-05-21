use init::InitArgs;
use pull::PullArgs;
use push::PushArgs;
use structopt::StructOpt;

mod init;
mod meld;
mod pull;
mod push;
mod util;

#[derive(Debug, StructOpt, Clone)]
pub enum Command {
    Init(InitArgs),
    Push(PushArgs),
    Pull(PullArgs),
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
        Command::Push(mod_args) => push::push_core(margs, mod_args),
        Command::Pull(mod_args) => pull::pull_core(margs, mod_args),
    };

    if res {
        util::good_message("Completed Successfully");
        std::process::exit(0);
    } else {
        util::error_message("Failed to Meld");
        std::process::exit(1);
    }
}
