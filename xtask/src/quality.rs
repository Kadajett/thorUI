use crate::{TaskResult, command};
use std::fs;
use std::path::Path;

const SOURCE_ROOTS: [&str; 2] = ["tools", "xtask"];
const FORBIDDEN: [&str; 4] = [
    concat!(".unwrap", "()"),
    concat!(".expect", "("),
    concat!("todo", "!"),
    concat!("unimplemented", "!"),
];

pub fn check() -> TaskResult {
    validate_sources()?;
    command::run("cargo", ["fmt", "--all", "--check"])?;
    command::run("cargo", ["check", "--workspace", "--all-targets"])?;
    command::run(
        "cargo",
        [
            "check",
            "-p",
            "thorui-lab",
            "--target",
            "wasm32-unknown-unknown",
        ],
    )?;
    command::run(
        "cargo",
        [
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    command::run(
        "cargo",
        [
            "clippy",
            "-p",
            "thorui-lab",
            "--target",
            "wasm32-unknown-unknown",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    command::run("cargo", ["test", "--workspace"])?;
    Ok(())
}

fn validate_sources() -> TaskResult {
    for root in SOURCE_ROOTS {
        visit(Path::new(root))?;
    }
    Ok(())
}

fn visit(path: &Path) -> TaskResult {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            visit(&entry?.path())?;
        }
        return Ok(());
    }
    if path.extension().is_some_and(|extension| extension == "rs") {
        validate_file(path)?;
    }
    Ok(())
}

fn validate_file(path: &Path) -> TaskResult {
    let source = fs::read_to_string(path)?;
    let lines: Vec<&str> = source.lines().collect();
    if lines.len() > 600 {
        return Err(format!("{} has {} lines; split it", path.display(), lines.len()).into());
    }
    for token in FORBIDDEN {
        if source.contains(token) {
            return Err(format!("{} contains forbidden token {token}", path.display()).into());
        }
    }
    validate_comments(path, &lines)
}

fn validate_comments(path: &Path, lines: &[&str]) -> TaskResult {
    let mut consecutive = 0;
    for (index, line) in lines.iter().enumerate() {
        consecutive = if line.trim_start().starts_with("//") {
            consecutive + 1
        } else {
            0
        };
        if consecutive > 2 {
            return Err(format!(
                "{}:{} has over two comment lines",
                path.display(),
                index + 1
            )
            .into());
        }
    }
    Ok(())
}
