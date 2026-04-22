use crate::aliases::AliasStore;
use crate::cas::CasStore;
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCaptureTiming {
    pub source_path: PathBuf,
    pub alias_path: PathBuf,
    pub cid: String,
    pub bytes: u64,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CaptureReport {
    pub captured_files: usize,
    pub total_duration_ms: u128,
    pub per_file: Vec<FileCaptureTiming>,
}

impl CaptureReport {
    pub fn p99_ms(&self) -> f64 {
        if self.per_file.is_empty() {
            return 0.0;
        }
        let mut times: Vec<_> = self.per_file.iter().map(|t| t.elapsed_ms as f64).collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((times.len() as f64) * 0.99).ceil() as usize;
        let idx = idx.min(times.len() - 1);
        times[idx]
    }
}

pub fn capture_paths(
    chief_home: impl AsRef<Path>,
    watched_dirs: &[PathBuf],
) -> Result<CaptureReport> {
    let chief_home = chief_home.as_ref();
    let cas = CasStore::open(chief_home).context("opening CAS store")?;
    let aliases_path = chief_home.join("aliases.sqlite");
    let mut aliases = AliasStore::open(&aliases_path).context("opening alias store")?;

    let started = Instant::now();
    let mut report = CaptureReport::default();
    let mut inputs = collect_inputs(watched_dirs)?;
    inputs.sort();

    for source_path in inputs {
        let per_file_started = Instant::now();
        let alias_path = canonical_alias_path(&source_path)?;
        let blob = cas
            .put_path(&source_path)
            .with_context(|| format!("capturing {}", source_path.display()))?;
        aliases
            .set_alias(&alias_path, &blob.cid)
            .with_context(|| format!("recording alias {} -> {}", alias_path.display(), blob.cid))?;

        report.per_file.push(FileCaptureTiming {
            source_path,
            alias_path,
            cid: blob.cid,
            bytes: blob.bytes,
            elapsed_ms: per_file_started.elapsed().as_millis(),
        });
    }

    report.captured_files = report.per_file.len();
    report.total_duration_ms = started.elapsed().as_millis();
    Ok(report)
}

#[cfg(target_os = "linux")]
/// Linux v0 shim: this is the fanotify-oriented placeholder that keeps the
/// public capture API stable while the recursive ingest path stands in for the
/// event loop.
pub fn capture_paths_fanotify_placeholder(
    chief_home: impl AsRef<Path>,
    watched_dirs: &[PathBuf],
) -> Result<CaptureReport> {
    capture_paths(chief_home, watched_dirs)
}

#[cfg(target_os = "macos")]
pub fn capture_paths_fsevents_stub(
    _chief_home: impl AsRef<Path>,
    _watched_dirs: &[PathBuf],
) -> Result<CaptureReport> {
    Err(anyhow!(
        "FSEvents capture is not implemented in the chief-fs prototype; use the one-shot scan path"
    ))
}

fn collect_inputs(watched_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut inputs = Vec::new();
    let mut seen = BTreeSet::new();

    for watched_dir in watched_dirs {
        let root = watched_dir.as_path();
        if !root.exists() {
            return Err(anyhow!("watched path does not exist: {}", root.display()));
        }

        let metadata = fs::symlink_metadata(root)
            .with_context(|| format!("reading metadata for {}", root.display()))?;
        if metadata.is_file() {
            let candidate = root.to_path_buf();
            let key = candidate.to_string_lossy().into_owned();
            if seen.insert(key) {
                inputs.push(candidate);
            }
            continue;
        }

        if metadata.is_dir() {
            walk_dir(root, &mut inputs, &mut seen)?;
        }
    }

    Ok(inputs)
}

fn walk_dir(dir: &Path, inputs: &mut Vec<PathBuf>, seen: &mut BTreeSet<String>) -> Result<()> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let mut entries = fs::read_dir(&current)
            .with_context(|| format!("reading directory {}", current.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("collecting entries from {}", current.display()))?;
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .with_context(|| format!("reading file type for {}", path.display()))?;

            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if file_type.is_file() {
                let key = path.to_string_lossy().into_owned();
                if seen.insert(key) {
                    inputs.push(path);
                }
            }
        }
    }

    Ok(())
}

fn canonical_alias_path(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path)
        .with_context(|| format!("canonicalizing alias path {}", path.display()))
        .or_else(|_| Ok::<PathBuf, anyhow::Error>(path.to_path_buf()))
}
