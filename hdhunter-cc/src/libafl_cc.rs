use std::env;
use std::path::Path;

use libafl_cc::{ClangWrapper, CompilerWrapper, ToolWrapper};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut dir = env::current_exe().unwrap();
        let wrapper_name = dir.file_name().unwrap().to_str().unwrap();

        let is_cpp = match wrapper_name[wrapper_name.len()-2..].to_lowercase().as_str() {
            "cc" => false,
            "++" | "pp" | "xx" => true,
            _ => panic!("Could not figure out if c or c++ warpper was called. Expected {:?} to end with c or cxx", dir),
        };

        dir.pop();

        let hdhunter_root = env::var("HDHUNTER_ROOT");
        if hdhunter_root.is_err() {
            panic!("HDHUNTER_ROOT not set");
        }

        let mut cc = ClangWrapper::new();
        if let Some(code) = cc
            .cpp(is_cpp)
            // silence the compiler wrapper output, needed for some configure scripts.
            .silence(true)
            .parse_args(&args)
            .expect("Failed to parse the command line")
            .add_arg("-fsanitize-coverage=trace-pc-guard")
            .add_arg("-g")
            .add_arg("-fPIC")
            .link_staticlib(&*Path::new(&hdhunter_root.unwrap()).join("target/debug"), "hdhunter_rt")
            .run()
            .expect("Failed to run the wrapped compiler")
        {
            std::process::exit(code);
        }
    } else {
        panic!("LibAFL CC: No Arguments given");
    }
}