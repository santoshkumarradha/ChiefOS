use anyhow::{anyhow, bail, Context, Result};
use chief_fs::aliases::AliasStore;
use chief_fs::capture_daemon;
use chief_fs::cas::{validate_cid, CasStore};
use chief_fs::fuse_shim;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    let chief_home = chief_home()?;
    match command.as_str() {
        "put" => {
            let source = required_arg(args.next(), "put <source> [alias]")?;
            let alias = args.next();
            ensure_no_extra(args)?;
            put(&chief_home, Path::new(&source), alias.as_deref())
        }
        "get" => {
            let reference = required_arg(args.next(), "get <cid-or-alias> <output>")?;
            let output = required_arg(args.next(), "get <cid-or-alias> <output>")?;
            ensure_no_extra(args)?;
            get(&chief_home, &reference, Path::new(&output))
        }
        "mount" => {
            let mountpoint = args.next().unwrap_or_else(default_mountpoint);
            ensure_no_extra(args)?;
            fuse_shim::mount_read_only(&chief_home, Path::new(&mountpoint))
        }
        "capture" => {
            let watched_dirs = args.map(PathBuf::from).collect::<Vec<_>>();
            if watched_dirs.is_empty() {
                bail!("capture requires at least one directory");
            }
            let report = capture_daemon::capture_paths(&chief_home, &watched_dirs)?;
            println!(
                "captured_files={} total_ms={:.3} p99_ms={:.3}",
                report.captured_files,
                report.total_duration_ms as f64,
                report.p99_ms()
            );
            Ok(())
        }
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        other => bail!("unknown command: {other}"),
    }
}

fn put(chief_home: &Path, source: &Path, alias: Option<&str>) -> Result<()> {
    let cas = CasStore::open(chief_home)?;
    let stat = cas.put_path(source)?;

    let alias_path = alias
        .map(ToOwned::to_owned)
        .or_else(|| {
            source
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .ok_or_else(|| anyhow!("source path has no file name; pass an explicit alias"))?;
    let mut aliases = AliasStore::open(alias_db_path(chief_home))?;
    aliases.set_alias(alias_path.as_str(), &stat.cid)?;

    println!("{} {}", stat.cid, alias_path);
    Ok(())
}

fn get(chief_home: &Path, reference: &str, output: &Path) -> Result<()> {
    let cas = CasStore::open(chief_home)?;
    let cid = if validate_cid(reference).is_ok() {
        reference.to_owned()
    } else {
        let aliases = AliasStore::open(alias_db_path(chief_home))?;
        aliases
            .get_alias(reference)?
            .ok_or_else(|| anyhow!("alias not found: {reference}"))?
            .cid
    };

    let bytes = cas.get(&cid)?;
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output parent {}", parent.display()))?;
        }
    }
    fs::write(output, bytes).with_context(|| format!("write {}", output.display()))?;
    println!("{cid} {}", output.display());
    Ok(())
}

fn chief_home() -> Result<PathBuf> {
    std::env::var_os("CHIEF_HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("CHIEF_HOME environment variable not set"))
}

fn alias_db_path(chief_home: &Path) -> PathBuf {
    chief_home.join("aliases.sqlite")
}

fn default_mountpoint() -> String {
    "/chief".to_string()
}

fn required_arg(arg: Option<String>, usage: &str) -> Result<String> {
    arg.ok_or_else(|| anyhow!("usage: {usage}"))
}

fn ensure_no_extra(args: impl Iterator) -> Result<()> {
    if args.count() > 0 {
        bail!("unexpected extra arguments");
    }
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage:
  chief-fs put <source> [alias]
  chief-fs get <cid-or-alias> <output>
  chief-fs mount [mountpoint]
  chief-fs capture <dir> [dir ...]"
    );
}
