use crate::Args;
use libmeld::Bin;
use structopt::StructOpt;

// Define Module Arguments
#[derive(Debug, StructOpt, Clone)]
pub struct InitArgs {
    #[structopt(
        short = "p",
        long = "parents",
        help = "no error if existing, make parent directories as needed"
    )]
    pub(crate) make_parents: bool,

    #[structopt(
        short = "f",
        long = "force",
        help = "force delete and init of an existing folder"
    )]
    pub(crate) force: bool,
}

/// Main handler for Meld Bin Init
pub fn init_core(main_args: Args, args: InitArgs) -> Result<(), libmeld::Error> {
    Bin::new(main_args.bin, args.force, args.make_parents)?;

    return Ok(());
}
