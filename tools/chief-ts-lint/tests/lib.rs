use std::path::{Path, PathBuf};

use chief_ts_lint::{format_text_report, scan_pack, Allowlist, OutputReport, ScanOptions};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn scan_fixture(name: &str) -> chief_ts_lint::ScanReport {
    scan_pack(ScanOptions {
        pack_root: fixture_path(name),
        allowlist: Allowlist::default(),
    })
    .expect("fixture scan should succeed")
}

#[test]
fn good_pack_exits_zero_with_valid_imports() {
    let report = scan_fixture("good-pack");

    assert_eq!(report.exit_code(), 0);
    assert!(report.violations.is_empty());
}

#[test]
fn radix_import_exits_one_with_diagnostic() {
    let report = scan_fixture("bad-pack-radix");

    assert_eq!(report.exit_code(), 1);
    assert_eq!(report.violations.len(), 1);

    let violation = &report.violations[0];
    assert_eq!(violation.file, "src/Button.tsx");
    assert_eq!(violation.line, 1);
    assert_eq!(violation.category, "radix-ui");
    assert_eq!(violation.import_specifier, "@radix-ui/tooltip");

    let output = format_text_report(&report);
    assert!(output.contains("error: Forbidden import detected"));
    assert!(output.contains("file: src/Button.tsx:1:"));
    assert!(output.contains("import: @radix-ui/tooltip"));
    assert!(output.contains("suggestion: Use @chief-os/ui Button component instead"));

    let json = serde_json::to_string(&OutputReport::from(&report)).unwrap();
    assert!(json.contains(r#"total":1"#));
    assert!(json.contains(r#"category":"radix-ui""#));
    assert!(json.contains(r#"import":"@radix-ui/tooltip""#));
}

#[test]
fn tauri_import_exits_one() {
    let report = scan_fixture("bad-pack-tauri");

    assert_eq!(report.exit_code(), 1);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].category, "tauri");
    assert_eq!(
        report.violations[0].import_specifier,
        "@tauri-apps/api/core"
    );
}

#[test]
fn font_url_css_import_exits_one() {
    let report = scan_fixture("bad-pack-font-url");

    assert_eq!(report.exit_code(), 1);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].file, "src/styles.css");
    assert_eq!(report.violations[0].line, 1);
    assert_eq!(report.violations[0].category, "font-url");
    assert!(report.violations[0]
        .import_specifier
        .contains("https://fonts.google.com/css"));
}
