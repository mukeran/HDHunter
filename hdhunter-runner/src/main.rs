mod args;
mod run;


use clap::Parser;
use crate::args::Args;
use crate::run::run;

fn main() {
    pretty_env_logger::init_timed();
    let args = Args::parse();

    run(&args.mode, &args.first_path, &args.second_path, &args.seeds, &args.corpus, &args.output, &args.tokens);
}
