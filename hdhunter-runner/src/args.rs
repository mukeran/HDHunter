use clap::Parser;
use hdhunter::mode::Mode;

#[derive(Parser, Debug)]
pub(crate) struct Args {
    #[arg(short = '1', long)]
    pub(crate) first_path: String,

    #[arg(short = '2', long)]
    pub(crate) second_path: String,

    #[arg(short, long, required = true)]
    pub(crate) seeds: Vec<String>,

    #[arg(short, long, default_value_t = Mode::Request)]
    pub(crate) mode: Mode,

    #[arg(short, long, default_value_t = String::from("./corpus_discovered"))]
    pub(crate) corpus: String,

    #[arg(short, long, default_value_t = String::from("./solutions"))]
    pub(crate) output: String,

    #[arg(short, long, default_value_t = String::from("./tokens.json"))]
    pub(crate) tokens: String,
}