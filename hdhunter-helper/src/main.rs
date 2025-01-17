mod inputs;
mod deduplicate;

use std::io::{Read, Write};
use std::net::TcpStream;
use clap::{Parser, Subcommand};
use libafl::inputs::{HasTargetBytes, Input};
use hdhunter::input::HttpSequenceInput;
use deduplicate::deduplicate;
use crate::inputs::{HttpInputBuilder, inputs};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    PrintInput {
        path: String,
        #[arg(short, long, default_value_t = false)]
        no_raw: bool,
    },
    WritePresetInputs { dir: String },
    SendInput {
        path: String,
        ip: String,
        port: u16,
    },
    ConvertInput {
        source: String,
        output: String,
        #[arg(short, long, default_value_t = false)]
        directory: bool,
        #[arg(short, long, default_value_t = false)]
        response: bool,
    },
    Deduplicate {
        first: String,
        second: String,
        solutions: String,
        output: String,
    }
}

fn main() {
    pretty_env_logger::init_timed();
    let cli = Cli::parse();

    match &cli.command {
        Commands::PrintInput { path, no_raw } => {
            let input = HttpSequenceInput::from_file(path).unwrap();
            let target_bytes = input.target_bytes();
            println!("length: {}", target_bytes.iter().len());
            println!("escaped: {:?}", escaped!(target_bytes.iter().as_slice()));
            if !no_raw {
                println!("raw: {}", std::str::from_utf8(target_bytes.iter().as_slice()).unwrap())
            }
        }
        Commands::WritePresetInputs { dir } => {
            for (idx, input) in inputs().iter().enumerate() {
                let path = format!("{}/input{}.json", dir, idx + 1);
                input.to_file(&path).unwrap();
            }
        }
        Commands::SendInput { path, ip, port } => {
            let input = HttpSequenceInput::from_file(path).unwrap();
            let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).unwrap();
            stream.write_all(input.target_bytes().iter().as_slice()).unwrap();
            let mut buf = [0; 1024];
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n <= 0 {
                    break;
                }
                print!("{}", unsafe { std::str::from_utf8_unchecked(&buf[..n]) });
            }
        }
        Commands::ConvertInput { source, output, directory, response } => {
            fn convert(source: &str, output: &str, response: bool) {
                let content = std::fs::read(source).unwrap();
                let input = HttpSequenceInput::new(vec![HttpInputBuilder::from_raw(&content, response).build()]);
                input.to_file(output).unwrap()
            }

            if *directory {
                for entry in std::fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir() {
                        continue;
                    }
                    let output = format!("{}/{}", output, path.file_name().unwrap().to_str().unwrap());
                    println!("Converting {} to {}", path.to_str().unwrap(), output);
                    convert(path.to_str().unwrap(), &output, *response);
                }
            } else {
                convert(source, output, *response);
            }
        }
        Commands::Deduplicate { first, second, solutions, output } => {
            deduplicate(first, second, &solutions, output)
        }
    }
}
