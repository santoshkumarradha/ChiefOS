use std::path::PathBuf;
use std::process::ExitCode;

use chief_ts_lint::{format_text_report, scan_pack, Allowlist, OutputReport, ScanOptions};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "chief-ts-lint",
    about = "Scan TypeScript pack imports for ADR-0011 build constraints"
)]
struct Cli {
    #[arg(long, default_value = ".")]
    pack_root: PathBuf,

    #[arg(long)]
    allowlist: Option<PathBuf>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    fail_on_violation: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let allowlist = match cli.allowlist {
        Some(path) => Allowlist::from_toml_file(path)?,
        None => Allowlist::default(),
    };

    let report = scan_pack(ScanOptions {
        pack_root: cli.pack_root,
        allowlist,
    })?;

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&OutputReport::from(&report))?
        );
    } else if report.violations.is_empty() {
        println!("chief-ts-lint: no forbidden imports found");
    } else {
        print!("{}", format_text_report(&report));
    }

    let should_fail = cli.fail_on_violation || !report.violations.is_empty();
    if should_fail && !report.violations.is_empty() {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
