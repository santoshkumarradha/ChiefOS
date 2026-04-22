use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, SourceMap, Span};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem};
use swc_core::ecma::parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub pack_root: PathBuf,
    pub allowlist: Allowlist,
}

#[derive(Debug, Clone, Default)]
pub struct ScanReport {
    pub violations: Vec<Violation>,
}

impl ScanReport {
    pub fn exit_code(&self) -> i32 {
        if self.violations.is_empty() {
            0
        } else {
            1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Violation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    #[serde(rename = "import")]
    pub import_specifier: String,
    pub category: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputReport {
    pub violations: Vec<Violation>,
    pub total: usize,
}

impl From<&ScanReport> for OutputReport {
    fn from(report: &ScanReport) -> Self {
        Self {
            violations: report.violations.clone(),
            total: report.violations.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Allowlist {
    exact: BTreeSet<String>,
    wildcards: Vec<String>,
}

impl Default for Allowlist {
    fn default() -> Self {
        let mut allowlist = Self {
            exact: BTreeSet::new(),
            wildcards: Vec::new(),
        };

        for item in [
            "@chief-os/sdk",
            "@chief-os/ui",
            "zod",
            "date-fns",
            "ulid",
            "yaml",
            "lodash",
            "classnames",
            "react",
            "react-jsx-runtime",
        ] {
            allowlist.allow_exact(item);
        }
        allowlist.allow_wildcard("./*");
        allowlist.allow_wildcard("../*");
        allowlist
    }
}

impl Allowlist {
    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self, ScanError> {
        let content = fs::read_to_string(path.as_ref()).map_err(|source| ScanError::Io {
            path: path.as_ref().to_path_buf(),
            source,
        })?;
        let config: AllowlistConfig = toml::from_str(&content)?;
        let mut allowlist = Self::default();

        if let Some(allowed) = config.allowed {
            for item in allowed.exact {
                allowlist.allow_exact(&item);
            }
            for item in allowed.wildcard {
                allowlist.allow_wildcard(&item);
            }
            for item in allowed.patterns {
                if item.ends_with('*') {
                    allowlist.allow_wildcard(&item);
                } else {
                    allowlist.allow_exact(&item);
                }
            }
        }

        Ok(allowlist)
    }

    pub fn is_allowed(&self, import_specifier: &str) -> bool {
        self.exact.contains(import_specifier)
            || self
                .wildcards
                .iter()
                .any(|pattern| wildcard_matches(pattern, import_specifier))
    }

    fn allow_exact(&mut self, value: &str) {
        self.exact.insert(value.to_owned());
    }

    fn allow_wildcard(&mut self, value: &str) {
        self.wildcards.push(value.to_owned());
    }
}

#[derive(Debug, Deserialize)]
struct AllowlistConfig {
    allowed: Option<AllowedSection>,
}

#[derive(Debug, Default, Deserialize)]
struct AllowedSection {
    #[serde(default)]
    exact: Vec<String>,
    #[serde(default)]
    wildcard: Vec<String>,
    #[serde(default)]
    patterns: Vec<String>,
}

#[derive(Debug)]
pub enum ScanError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Toml(toml::de::Error),
    Parse {
        path: PathBuf,
        message: String,
    },
    Walk(walkdir::Error),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Toml(source) => write!(formatter, "invalid allowlist TOML: {source}"),
            Self::Parse { path, message } => write!(formatter, "{}: {message}", path.display()),
            Self::Walk(source) => write!(formatter, "{source}"),
        }
    }
}

impl std::error::Error for ScanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Toml(source) => Some(source),
            Self::Walk(source) => Some(source),
            Self::Parse { .. } => None,
        }
    }
}

impl From<toml::de::Error> for ScanError {
    fn from(source: toml::de::Error) -> Self {
        Self::Toml(source)
    }
}

impl From<walkdir::Error> for ScanError {
    fn from(source: walkdir::Error) -> Self {
        Self::Walk(source)
    }
}

pub fn scan_pack(options: ScanOptions) -> Result<ScanReport, ScanError> {
    let mut report = ScanReport::default();
    let pack_root = options
        .pack_root
        .canonicalize()
        .map_err(|source| ScanError::Io {
            path: options.pack_root.clone(),
            source,
        })?;

    for entry in WalkDir::new(&pack_root)
        .into_iter()
        .filter_entry(|entry| !is_skipped_dir(entry))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        match path.extension().and_then(|value| value.to_str()) {
            Some("ts" | "tsx") => {
                report.violations.extend(scan_typescript_file(
                    &pack_root,
                    path,
                    &options.allowlist,
                )?);
            }
            Some("css") => {
                report.violations.extend(scan_css_file(&pack_root, path)?);
            }
            _ => {}
        }
    }

    report.violations.sort_by(|left, right| {
        (&left.file, left.line, left.column).cmp(&(&right.file, right.line, right.column))
    });

    Ok(report)
}

pub fn format_text_report(report: &ScanReport) -> String {
    let mut output = String::new();
    for violation in &report.violations {
        output.push_str("error: Forbidden import detected\n");
        output.push_str(&format!(
            "  file: {}:{}:{}\n",
            violation.file, violation.line, violation.column
        ));
        output.push_str(&format!("  import: {}\n", violation.import_specifier));
        output.push_str(&format!("  suggestion: {}\n", violation.suggestion));
    }
    output
}

fn scan_typescript_file(
    pack_root: &Path,
    path: &Path,
    allowlist: &Allowlist,
) -> Result<Vec<Violation>, ScanError> {
    let source = fs::read_to_string(path).map_err(|source| ScanError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = source_map.new_source_file(FileName::Real(path.to_path_buf()).into(), source);
    let syntax = Syntax::Typescript(TsSyntax {
        tsx: path.extension().and_then(|value| value.to_str()) == Some("tsx"),
        decorators: true,
        ..Default::default()
    });
    let lexer = Lexer::new(
        syntax,
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().map_err(|err| ScanError::Parse {
        path: path.to_path_buf(),
        message: format!("{err:?}"),
    })?;

    let mut violations = Vec::new();
    for item in module.body {
        let Some((specifier, span)) = import_from_module_item(item) else {
            continue;
        };
        if let Some((category, suggestion)) = classify_banned_import(&specifier) {
            violations.push(make_violation(
                pack_root,
                path,
                &source_map,
                span,
                &specifier,
                category,
                suggestion,
            ));
        } else if !allowlist.is_allowed(&specifier) {
            violations.push(make_violation(
                pack_root,
                path,
                &source_map,
                span,
                &specifier,
                "allowlist",
                "Import from @chief-os/sdk, @chief-os/ui, a relative module, or add an approved utility to the allowlist",
            ));
        }
    }

    Ok(violations)
}

fn scan_css_file(pack_root: &Path, path: &Path) -> Result<Vec<Violation>, ScanError> {
    let source = fs::read_to_string(path).map_err(|source| ScanError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut violations = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let normalized = line.to_ascii_lowercase();
        let is_import = normalized.contains("@import");
        let has_url_or_font = normalized.contains("url(")
            || normalized.contains("font")
            || normalized.contains("fonts.google")
            || normalized.contains("fonts.googleapis")
            || normalized.contains("fonts.gstatic");
        if is_import && has_url_or_font {
            violations.push(Violation {
                file: relative_file(pack_root, path),
                line: index + 1,
                column: line.find('@').map_or(1, |column| column + 1),
                import_specifier: line.trim().to_owned(),
                category: "font-url".to_owned(),
                suggestion:
                    "Bundle fonts through @chief-os/ui assets instead of loading runtime font URLs"
                        .to_owned(),
            });
        }
    }

    Ok(violations)
}

fn import_from_module_item(item: ModuleItem) -> Option<(String, Span)> {
    match item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => Some((
            decl.src.value.as_wtf8().to_string_lossy().into_owned(),
            decl.src.span,
        )),
        _ => None,
    }
}

fn classify_banned_import(import_specifier: &str) -> Option<(&'static str, &'static str)> {
    if import_specifier.starts_with("@radix-ui/") {
        Some(("radix-ui", "Use @chief-os/ui Button component instead"))
    } else if import_specifier.starts_with("@chief-os/ui/internal/")
        || import_specifier == "@chief-os/ui/internal"
        || import_specifier.contains("/shadcn/")
        || import_specifier.ends_with("/shadcn")
    {
        Some((
            "chief-ui-internal",
            "Use the public @chief-os/ui entry point instead of internal components",
        ))
    } else if import_specifier == "tauri"
        || import_specifier.starts_with("tauri/")
        || import_specifier.starts_with("@tauri-apps/")
    {
        Some((
            "tauri",
            "Tauri APIs are kit-internal; request the capability through @chief-os/sdk",
        ))
    } else if import_specifier == "react-dom" || import_specifier.starts_with("react-dom/") {
        Some((
            "react-dom",
            "chief-ui owns rendering; compose primitives from @chief-os/ui instead",
        ))
    } else if import_specifier.to_ascii_lowercase().contains("font")
        || import_specifier.to_ascii_lowercase().contains("url")
    {
        Some((
            "font-url",
            "Bundle fonts through @chief-os/ui assets instead of loading runtime font URLs",
        ))
    } else {
        None
    }
}

fn make_violation(
    pack_root: &Path,
    path: &Path,
    source_map: &SourceMap,
    span: Span,
    import_specifier: &str,
    category: &str,
    suggestion: &str,
) -> Violation {
    let position = source_map.lookup_char_pos(span.lo);
    Violation {
        file: relative_file(pack_root, path),
        line: position.line,
        column: position.col_display + 1,
        import_specifier: import_specifier.to_owned(),
        category: category.to_owned(),
        suggestion: suggestion.to_owned(),
    }
}

fn relative_file(pack_root: &Path, path: &Path) -> String {
    path.strip_prefix(pack_root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        value.starts_with(prefix)
    } else {
        pattern == value
    }
}

fn is_skipped_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
        && matches!(
            entry.file_name().to_str(),
            Some("node_modules" | ".git" | "target")
        )
}
