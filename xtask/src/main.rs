#![forbid(unsafe_code)]

mod artifact;
mod command;
mod quality;

use std::env;

type TaskResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> TaskResult {
    let task = env::args().nth(1).unwrap_or_else(|| "help".to_owned());
    match task.as_str() {
        "build" => artifact::build(),
        "check" => quality::check(),
        "help" | "--help" | "-h" => {
            println!("usage: cargo run -p xtask -- <build|check>");
            Ok(())
        }
        unknown => Err(format!("unknown task: {unknown}").into()),
    }
}
