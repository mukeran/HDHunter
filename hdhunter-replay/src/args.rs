use clap::Parser;
use hdhunter::mode::Mode;

#[derive(Parser, Debug)]
pub(crate) struct Args {
    pub(crate) workspace: String,
    pub(crate) input: String,

    #[clap(short, long, default_value = "request")]
    pub(crate) mode: Mode,

    #[clap(short, long)]
    pub(crate) print_edgemap: bool,
}