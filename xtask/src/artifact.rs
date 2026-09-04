use crate::{TaskResult, command};
use brotli::CompressorWriter;
use flate2::{Compression, write::GzEncoder};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct BuildMetadata<'a> {
    schema_version: u16,
    revision: &'a str,
    build_time_epoch_seconds: u64,
    channel: &'a str,
    report_schema_version: u16,
}

#[derive(Serialize)]
struct Asset {
    path: String,
    bytes: u64,
    gzip_bytes: u64,
    brotli_bytes: u64,
    sha256: String,
}

#[derive(Serialize)]
struct SizeReport {
    schema_version: u16,
    executable_raw_bytes: u64,
    executable_gzip_bytes: u64,
    executable_brotli_bytes: u64,
    raw_budget_bytes: u64,
    gzip_budget_bytes: u64,
}

pub fn build() -> TaskResult {
    let revision = revision()?;
    let channel = env::var("THORUI_CHANNEL").unwrap_or_else(|_| "local".to_owned());
    command::run_with_env(
        "trunk",
        ["build", "--release"],
        &[
            ("THORUI_BUILD_REVISION", &revision),
            ("THORUI_CHANNEL", &channel),
            ("NO_COLOR", "true"),
        ],
    )?;
    write_version(&revision)?;
    let assets = describe_dist()?;
    write_size_report(&assets)?;
    write_manifest(&assets)?;
    Ok(())
}

fn revision() -> TaskResult<String> {
    env::var("THORUI_BUILD_REVISION")
        .or_else(|_| command::output("git", ["rev-parse", "--short=12", "HEAD"]))
}

fn write_version(revision: &str) -> TaskResult {
    let built_at = env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
    let channel = env::var("THORUI_CHANNEL").unwrap_or_else(|_| "local".to_owned());
    let version = BuildMetadata {
        schema_version: 1,
        revision,
        build_time_epoch_seconds: built_at,
        channel: &channel,
        report_schema_version: 1,
    };
    fs::write("dist/version.json", serde_json::to_vec_pretty(&version)?)?;
    Ok(())
}

fn describe_dist() -> TaskResult<Vec<Asset>> {
    let mut files = Vec::new();
    collect_files(Path::new("dist"), &mut files)?;
    files.retain(|path| {
        !path.ends_with("asset-manifest.json") && !path.ends_with("size-report.json")
    });
    files.sort();
    files
        .iter()
        .map(|path| describe(path))
        .collect::<TaskResult<Vec<_>>>()
}

fn write_manifest(assets: &[Asset]) -> TaskResult {
    fs::write(
        "dist/asset-manifest.json",
        serde_json::to_vec_pretty(assets)?,
    )?;
    Ok(())
}

fn write_size_report(assets: &[Asset]) -> TaskResult {
    let executable = assets.iter().filter(|asset| is_executable(&asset.path));
    let report = SizeReport {
        schema_version: 1,
        executable_raw_bytes: executable.clone().map(|asset| asset.bytes).sum(),
        executable_gzip_bytes: executable.clone().map(|asset| asset.gzip_bytes).sum(),
        executable_brotli_bytes: executable.map(|asset| asset.brotli_bytes).sum(),
        raw_budget_bytes: 524_288,
        gzip_budget_bytes: 196_608,
    };
    if report.executable_raw_bytes > report.raw_budget_bytes
        || report.executable_gzip_bytes > report.gzip_budget_bytes
    {
        return Err("optimized executable bundle exceeds its size budget".into());
    }
    fs::write("dist/size-report.json", serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> TaskResult {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn describe(path: &Path) -> TaskResult<Asset> {
    let bytes = fs::read(path)?;
    let relative = path
        .strip_prefix("dist")?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(Asset {
        path: format!("/{relative}"),
        bytes: u64::try_from(bytes.len())?,
        gzip_bytes: gzip_size(&bytes)?,
        brotli_bytes: brotli_size(&bytes)?,
        sha256: format!("{:x}", Sha256::digest(bytes)),
    })
}

fn gzip_size(bytes: &[u8]) -> TaskResult<u64> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    Ok(u64::try_from(encoder.finish()?.len())?)
}

fn brotli_size(bytes: &[u8]) -> TaskResult<u64> {
    let mut compressed = Vec::new();
    {
        let mut encoder = CompressorWriter::new(&mut compressed, 4_096, 11, 22);
        encoder.write_all(bytes)?;
    }
    Ok(u64::try_from(compressed.len())?)
}

fn is_executable(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["wasm", "js", "css"]
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
        })
}
