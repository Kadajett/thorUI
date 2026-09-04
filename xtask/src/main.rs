#![forbid(unsafe_code)]

mod artifact;
mod command;
mod quality;
mod report;

use std::env;

type TaskResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> TaskResult {
    let mut arguments = env::args().skip(1);
    let task = arguments.next().unwrap_or_else(|| "help".to_owned());
    match task.as_str() {
        "build" => artifact::build(),
        "check" => quality::check(),
        "validate-reports" => report::validate(&arguments.collect::<Vec<_>>()),
        "help" | "--help" | "-h" => {
            println!("usage: cargo run -p xtask -- <build|check|validate-reports> [reports...]");
            Ok(())
        }
        unknown => Err(format!("unknown task: {unknown}").into()),
    }
}
