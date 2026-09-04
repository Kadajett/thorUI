use crate::TaskResult;
use std::ffi::OsStr;
use std::process::Command;

pub fn run<I, S>(program: &str, arguments: I) -> TaskResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program).args(arguments).status()?;
    if status.success() {
        return Ok(());
    }
    Err(format!("{program} exited with {status}").into())
}

pub fn run_with_env<I, S>(program: &str, arguments: I, environment: &[(&str, &str)]) -> TaskResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program)
        .args(arguments)
        .envs(environment.iter().copied())
        .status()?;
    if status.success() {
        return Ok(());
    }
    Err(format!("{program} exited with {status}").into())
}

pub fn output<I, S>(program: &str, arguments: I) -> TaskResult<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let result = Command::new(program).args(arguments).output()?;
    if !result.status.success() {
        return Err(format!("{program} exited with {}", result.status).into());
    }
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}
