pub mod ast;
pub mod formatter;
pub mod imports;
pub mod lexer;
pub mod mirage;
pub mod parser;
pub mod resolver;
pub mod sourcemap;
pub mod transpiler;

use crate::lexer::Lexer;
use crate::parser::parse_program;
use crate::sourcemap::SourceLocation;
use crate::transpiler::Transpiler;

type ParserError<'a> = NomErr<NomError<&'a [crate::parser::TokenSpan]>>;
use clap::{Parser, ValueEnum};
use nom::Err as NomErr;
use nom::error::Error as NomError;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Transpile a .ds file to .rs
    Transpile {
        /// Input .ds file or project directory (defaults to current directory)
        #[arg(default_value = ".")]
        input: PathBuf,
        /// Output .rs file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Check a .ds file for errors
    Check {
        /// Input .ds file or project directory (defaults to current directory)
        #[arg(default_value = ".")]
        input: PathBuf,
        /// Check stage: syntax-only, syntax+semantic, or full rustc-backed check
        #[arg(long, value_enum, default_value_t = CheckStage::Rust)]
        stage: CheckStage,
    },
    /// Compile and run a .ds file or project
    Run {
        /// Input .ds file or project directory (defaults to current directory)
        #[arg(default_value = ".")]
        input: PathBuf,
        /// Arguments passed through to the program (`desert run app.ds -- arg1 arg2`)
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Create a new Desert project scaffold
    New {
        /// Path to create (project name if relative)
        path: PathBuf,
        /// Allow scaffolding into an existing non-empty directory
        #[arg(long)]
        force: bool,
    },
    /// Format Desert source files
    Fmt {
        /// Input .ds file or directory (defaults to current directory)
        #[arg(default_value = ".")]
        input: PathBuf,
        /// Check mode: fail if any file would be reformatted
        #[arg(long)]
        check: bool,
    },
    /// Run environment and project preflight diagnostics
    Doctor {
        /// Optional .ds file or project directory to validate
        input: Option<PathBuf>,
    },
    /// Print resolved import graph order for a project
    Graph {
        /// Project directory containing desert.toml/Desert.toml (defaults to current directory)
        #[arg(default_value = ".")]
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transpile { input, output } => {
            let input_source = load_input_source(&input)?;
            let (rust_code, _) = transpile_file(&input_source)?;
            if let Some(output_path) = output {
                fs::write(output_path, rust_code)?;
            } else {
                println!("{}", rust_code);
            }
        }
        Commands::Check { input, stage } => match stage {
            CheckStage::Syntax => {
                check_syntax_input(&input)?;
                println!("Syntax check passed.");
            }
            CheckStage::Semantic => {
                let input_source = load_input_source(&input)?;
                validate_input_semantics(&input_source)?;
                println!("Semantic check passed.");
            }
            CheckStage::Rust => {
                let input_source = load_input_source(&input)?;
                let (rust_code, source_map) = transpile_file(&input_source)?;

                let output = run_rustc_check(&rust_code)?;
                let saw_diagnostic = emit_translated_diagnostics(&output, &source_map)?;

                if !output.status.success() {
                    if !saw_diagnostic {
                        anyhow::bail!("Rust check failed.");
                    }
                    anyhow::bail!("Rust check failed with translated diagnostics.");
                }

                println!("Check passed.");
            }
        },
        Commands::Run { input, args } => {
            let input_source = load_input_source(&input)?;
            let (rust_code, source_map) = transpile_file(&input_source)?;

            with_temp_dir(|temp_dir| {
                let rs_file = temp_dir.join("main.rs");
                let binary_file = compiled_binary_path(temp_dir);
                fs::write(&rs_file, &rust_code)?;

                let output = std::process::Command::new("rustc")
                    .arg("--edition=2024")
                    .arg("--error-format=json")
                    .arg("-o")
                    .arg(&binary_file)
                    .arg(&rs_file)
                    .output()?;

                let saw_diagnostic = emit_translated_diagnostics(&output, &source_map)?;
                if !output.status.success() {
                    if !saw_diagnostic {
                        anyhow::bail!("Rust compile failed.");
                    }
                    anyhow::bail!("Rust compile failed with translated diagnostics.");
                }

                let status = std::process::Command::new(&binary_file)
                    .args(&args)
                    .status()?;
                if !status.success() {
                    match status.code() {
                        Some(code) => anyhow::bail!("Program exited with status code {}", code),
                        None => anyhow::bail!("Program terminated by signal"),
                    }
                }
                Ok(())
            })?;
        }
        Commands::Graph { input } => {
            let (project_root, ordered_files) = resolve_project_graph(&input)?;
            for file in ordered_files {
                let display_path = file.strip_prefix(&project_root).unwrap_or(file.as_path());
                println!("{}", display_path.display());
            }
        }
        Commands::New { path, force } => {
            scaffold_project(&path, force)?;
            println!("Created Desert project at {}", path.display());
            println!("Next steps:");
            println!("  cd {}", path.display());
            println!("  desert run .");
        }
        Commands::Fmt { input, check } => {
            let files = collect_ds_files(&input)?;
            let mut changed_files = Vec::new();
            for file in files {
                let source = fs::read_to_string(&file)?;
                let formatted = format_source(&source)?;
                if formatted != source {
                    changed_files.push(file.clone());
                    if !check {
                        fs::write(&file, formatted)?;
                    }
                }
            }

            if check {
                if changed_files.is_empty() {
                    println!("Formatting check passed.");
                } else {
                    for file in &changed_files {
                        println!("{}", file.display());
                    }
                    anyhow::bail!(
                        "format check failed: {} file(s) need formatting",
                        changed_files.len()
                    );
                }
            } else {
                println!("Formatted {} file(s).", changed_files.len());
            }
        }
        Commands::Doctor { input } => {
            run_doctor(input.as_deref())?;
        }
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum CheckStage {
    Syntax,
    Semantic,
    Rust,
}

#[derive(Debug, Deserialize, Default)]
struct DesertManifest {
    package: Option<ManifestPackage>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestPackage {
    entry: Option<PathBuf>,
}

struct InputSource {
    content: String,
    line_origins: Vec<SourceLocation>,
}

fn load_input_source(input: &Path) -> anyhow::Result<InputSource> {
    if input.is_file() {
        return load_file_source(input);
    }
    if input.is_dir() {
        return load_project_source(input);
    }

    anyhow::bail!(
        "input path '{}' is neither a file nor a directory",
        input.display()
    )
}

fn load_file_source(entry_file: &Path) -> anyhow::Result<InputSource> {
    let ordered_files = resolve_file_graph(entry_file)?;
    build_input_source(&ordered_files)
}

fn resolve_project_entry(project_root: &Path) -> anyhow::Result<PathBuf> {
    let manifest_path = ["desert.toml", "Desert.toml"]
        .iter()
        .map(|name| project_root.join(name))
        .find(|path| path.is_file());

    if manifest_path.is_none() {
        let fallback_candidates = [project_root.join("src/main.ds"), project_root.join("main.ds")];
        if let Some(entry_path) = fallback_candidates.iter().find(|path| path.is_file()) {
            return Ok(entry_path.to_path_buf());
        }
        anyhow::bail!(
            "project input '{}' is missing desert.toml/Desert.toml and no fallback entry was found (expected 'src/main.ds' or 'main.ds')",
            project_root.display()
        );
    }

    let manifest_path = manifest_path.unwrap();

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: DesertManifest = toml::from_str(&manifest_content)
        .map_err(|err| anyhow::anyhow!("failed to parse '{}': {}", manifest_path.display(), err))?;

    let entry_rel = manifest
        .package
        .and_then(|pkg| pkg.entry)
        .unwrap_or_else(|| PathBuf::from("src/main.ds"));

    let entry_path = project_root.join(entry_rel);
    if entry_path.is_file() {
        Ok(entry_path)
    } else {
        anyhow::bail!(
            "project entry '{}' does not exist as a file",
            entry_path.display()
        )
    }
}

fn load_project_source(project_root: &Path) -> anyhow::Result<InputSource> {
    let (_, ordered_files) = resolve_project_graph(project_root)?;
    build_input_source(&ordered_files)
}

fn build_input_source(ordered_files: &[PathBuf]) -> anyhow::Result<InputSource> {
    let mut content = String::new();
    let mut line_origins = Vec::new();

    for file in ordered_files {
        let piece = fs::read_to_string(file)?;
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&piece);
        line_origins.extend(line_origins_for_file(file, &piece)?);
    }

    Ok(InputSource {
        content,
        line_origins,
    })
}

fn line_origins_for_file(file_path: &Path, content: &str) -> anyhow::Result<Vec<SourceLocation>> {
    let file_name = format_display_path(file_path)?;
    let mut origins = Vec::new();
    for (idx, _) in content.lines().enumerate() {
        origins.push(SourceLocation {
            file: file_name.clone(),
            line: idx + 1,
            column: 1,
        });
    }
    Ok(origins)
}

fn format_display_path(path: &Path) -> anyhow::Result<String> {
    let canonical = path
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to canonicalize '{}': {}", path.display(), err))?;
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(relative) = canonical.strip_prefix(&cwd) {
            return Ok(relative.display().to_string());
        }
    }
    Ok(canonical.display().to_string())
}

fn resolve_project_graph(project_root: &Path) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    if !project_root.is_dir() {
        anyhow::bail!(
            "graph input '{}' must be a project directory",
            project_root.display()
        );
    }

    let canonical_root = project_root.canonicalize().map_err(|err| {
        anyhow::anyhow!(
            "failed to resolve project root '{}': {}",
            project_root.display(),
            err
        )
    })?;

    let entry_path = resolve_project_entry(&canonical_root)?;
    let mut visited = HashSet::new();
    let mut loading = Vec::new();
    let mut ordered_files = Vec::new();

    collect_project_sources(
        &entry_path,
        &canonical_root,
        &mut visited,
        &mut loading,
        &mut ordered_files,
    )?;

    Ok((canonical_root, ordered_files))
}

fn resolve_file_graph(entry_file: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if !entry_file.is_file() {
        anyhow::bail!(
            "input file '{}' does not exist as a file",
            entry_file.display()
        );
    }

    let canonical_entry = entry_file.canonicalize().map_err(|err| {
        anyhow::anyhow!(
            "failed to resolve input file '{}': {}",
            entry_file.display(),
            err
        )
    })?;

    let mut visited = HashSet::new();
    let mut loading = Vec::new();
    let mut ordered_files = Vec::new();
    collect_file_sources(&canonical_entry, &mut visited, &mut loading, &mut ordered_files)?;
    Ok(ordered_files)
}

fn check_import_cycle(canonical_file: &Path, loading: &[PathBuf]) -> anyhow::Result<()> {
    if let Some(cycle_idx) = loading.iter().position(|path| path == canonical_file) {
        let mut cycle = loading[cycle_idx..]
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        cycle.push(canonical_file.display().to_string());
        anyhow::bail!("import cycle detected: {}", cycle.join(" -> "));
    }
    Ok(())
}

fn collect_file_sources(
    file_path: &Path,
    visited: &mut HashSet<PathBuf>,
    loading: &mut Vec<PathBuf>,
    ordered_files: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    let canonical_file = file_path.canonicalize().map_err(|err| {
        anyhow::anyhow!(
            "failed to resolve import '{}': {}",
            file_path.display(),
            err
        )
    })?;

    if visited.contains(&canonical_file) {
        return Ok(());
    }

    check_import_cycle(&canonical_file, loading)?;

    let source = fs::read_to_string(&canonical_file)?;
    loading.push(canonical_file.clone());
    let program = parse_source(&source)?;

    for stmt in &program.statements {
        match &stmt.kind {
            crate::ast::StatementKind::Import { path, .. } => {
                if crate::imports::rust_use_from_import_path(path).is_some() {
                    continue;
                }
                let import_path = resolve_import_path(&canonical_file, path, None)?;
                collect_file_sources(&import_path, visited, loading, ordered_files)?;
            }
            crate::ast::StatementKind::FromImport { .. } => {
                // Non-rust from-import is currently unsupported and is validated
                // semantically; keep graph loading aligned with that rule.
            }
            _ => {}
        }
    }

    loading.pop();
    visited.insert(canonical_file.clone());
    ordered_files.push(canonical_file);
    Ok(())
}

fn collect_project_sources(
    file_path: &Path,
    project_root: &Path,
    visited: &mut HashSet<PathBuf>,
    loading: &mut Vec<PathBuf>,
    ordered_files: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    let canonical_file = file_path.canonicalize().map_err(|err| {
        anyhow::anyhow!(
            "failed to resolve import '{}': {}",
            file_path.display(),
            err
        )
    })?;

    if !canonical_file.starts_with(project_root) {
        anyhow::bail!(
            "import '{}' resolves outside project root '{}'",
            canonical_file.display(),
            project_root.display()
        );
    }

    if visited.contains(&canonical_file) {
        return Ok(());
    }

    check_import_cycle(&canonical_file, loading)?;

    let source = fs::read_to_string(&canonical_file)?;
    loading.push(canonical_file.clone());
    let program = parse_source(&source)?;

    for stmt in &program.statements {
        match &stmt.kind {
            crate::ast::StatementKind::Import { path, .. } => {
                if crate::imports::rust_use_from_import_path(path).is_some() {
                    continue;
                }
                let import_path = resolve_import_path(&canonical_file, path, Some(project_root))?;
                collect_project_sources(&import_path, project_root, visited, loading, ordered_files)?;
            }
            crate::ast::StatementKind::FromImport { .. } => {
                // Non-rust from-import is currently unsupported and is validated
                // semantically; keep graph loading aligned with that rule.
            }
            _ => {}
        }
    }

    loading.pop();
    visited.insert(canonical_file.clone());
    ordered_files.push(canonical_file);
    Ok(())
}

fn resolve_import_path(
    current_file: &Path,
    import_path: &str,
    project_root: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    let mut resolved = PathBuf::from(import_path);
    if resolved.extension().is_none() {
        resolved.set_extension("ds");
    }

    let joined = if resolved.is_absolute() {
        resolved
    } else {
        current_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(resolved)
    };

    let canonical = joined.canonicalize().map_err(|err| {
        anyhow::anyhow!(
            "failed to resolve import '{}' from '{}': {}",
            import_path,
            current_file.display(),
            err
        )
    })?;

    if let Some(project_root) = project_root {
        if !canonical.starts_with(project_root) {
            anyhow::bail!(
                "import '{}' from '{}' resolves outside project root '{}': '{}'",
                import_path,
                current_file.display(),
                project_root.display(),
                canonical.display()
            );
        }
    }
    Ok(canonical)
}

fn transpile_file(
    input_source: &InputSource,
) -> anyhow::Result<(String, crate::sourcemap::SourceMap)> {
    let program = parse_and_validate_program(input_source)?;
    let transpiler = Transpiler::new();
    Ok(transpiler.transpile(&program, &input_source.content, &input_source.line_origins))
}

fn check_syntax_input(input: &Path) -> anyhow::Result<()> {
    if input.is_file() {
        let ordered_files = resolve_file_graph(input)?;
        for file in ordered_files {
            let source = fs::read_to_string(&file)?;
            parse_source(&source).map_err(|err| anyhow::anyhow!("{}: {}", file.display(), err))?;
        }
        return Ok(());
    }

    if input.is_dir() {
        let (_, ordered_files) = resolve_project_graph(input)?;
        for file in ordered_files {
            let source = fs::read_to_string(&file)?;
            parse_source(&source).map_err(|err| anyhow::anyhow!("{}: {}", file.display(), err))?;
        }
        return Ok(());
    }

    anyhow::bail!(
        "input path '{}' is neither a file nor a directory",
        input.display()
    )
}

fn validate_input_semantics(input_source: &InputSource) -> anyhow::Result<()> {
    parse_and_validate_program(input_source).map(|_| ())
}

fn parse_and_validate_program(input_source: &InputSource) -> anyhow::Result<crate::ast::Program> {
    let program = parse_source(&input_source.content)?;
    validate_program(&input_source.content, &program)?;
    Ok(program)
}

fn format_source(input_content: &str) -> anyhow::Result<String> {
    let program = parse_source(input_content)?;
    Ok(crate::formatter::format_program(&program))
}

fn parse_source(input_content: &str) -> anyhow::Result<crate::ast::Program> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(input_content);
    while let Some(result) = lexer.next() {
        match result {
            Ok(token_span) => tokens.push(token_span),
            Err(_) => {
                let span = lexer.span();
                let (line, col) = line_col_from_offset(input_content, span.start);
                let snippet = input_content
                    .get(span.clone())
                    .map(str::trim)
                    .unwrap_or("<unknown>");
                anyhow::bail!(
                    "Lexing error at line {}, column {} near '{}'",
                    line,
                    col,
                    snippet
                );
            }
        }
    }
    let (_, program) = parse_program(&tokens).map_err(|e| format_parse_error(input_content, e))?;
    Ok(program)
}

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("desert_check_{}_{}", std::process::id(), nanos))
}

fn run_doctor(input: Option<&Path>) -> anyhow::Result<()> {
    let rustc_version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|err| anyhow::anyhow!("failed to execute rustc: {}", err))?;
    if !rustc_version.status.success() {
        anyhow::bail!("rustc is installed but not runnable");
    }
    let version_text = String::from_utf8(rustc_version.stdout)?.trim().to_string();
    println!("rustc: {version_text}");

    if let Some(input) = input {
        if input.is_file() {
            let input_source = load_file_source(input)?;
            let program = parse_source(&input_source.content)?;
            validate_program(&input_source.content, &program)?;
            println!("source: ok ({})", input.display());
            return Ok(());
        }

        if input.is_dir() {
            let (project_root, ordered_files) = resolve_project_graph(input)?;
            for file in ordered_files {
                let source = fs::read_to_string(&file)?;
                let program = parse_source(&source)?;
                validate_program(&source, &program)?;
            }
            println!("project: ok ({})", project_root.display());
            return Ok(());
        }

        anyhow::bail!(
            "doctor input '{}' is neither a file nor a directory",
            input.display()
        );
    }

    println!("environment: ok");
    Ok(())
}

fn collect_ds_files(input: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if input.is_file() {
        if input.extension().is_some_and(|ext| ext == "ds") {
            return Ok(vec![input.to_path_buf()]);
        }
        anyhow::bail!("format input '{}' must be a .ds file", input.display());
    }

    if !input.is_dir() {
        anyhow::bail!(
            "format input '{}' is neither a file nor a directory",
            input.display()
        );
    }

    let mut files = Vec::new();
    collect_ds_files_recursive(input, &mut files)?;
    files.sort();
    if files.is_empty() {
        anyhow::bail!("no .ds files found under '{}'", input.display());
    }
    Ok(files)
}

fn collect_ds_files_recursive(root: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ds_files_recursive(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "ds") {
            files.push(path);
        }
    }
    Ok(())
}

fn scaffold_project(path: &Path, force: bool) -> anyhow::Result<()> {
    if path.exists() {
        if !path.is_dir() {
            anyhow::bail!("target '{}' exists and is not a directory", path.display());
        }
        if !force && path.read_dir()?.next().is_some() {
            anyhow::bail!(
                "target directory '{}' is not empty (use --force to overwrite files)",
                path.display()
            );
        }
    } else {
        fs::create_dir_all(path)?;
    }

    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir)?;

    let project_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("desert_app")
        .replace('-', "_");

    let manifest = format!(
        "[package]\nname = \"{}\"\nentry = \"src/main.ds\"\n",
        project_name
    );
    fs::write(path.join("desert.toml"), manifest)?;
    fs::write(
        src_dir.join("main.ds"),
        "def main():\n    $print(\"Hello from Desert!\")\n",
    )?;
    Ok(())
}

fn with_temp_dir<T>(f: impl FnOnce(&Path) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let temp_dir = unique_temp_dir();
    fs::create_dir_all(&temp_dir)?;
    let result = f(&temp_dir);
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn compiled_binary_path(temp_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        temp_dir.join("desert_program.exe")
    } else {
        temp_dir.join("desert_program")
    }
}

fn run_rustc_check(rust_code: &str) -> anyhow::Result<std::process::Output> {
    with_temp_dir(|temp_dir| {
        let rs_file = temp_dir.join("main.rs");
        fs::write(&rs_file, rust_code)?;
        let output = std::process::Command::new("rustc")
            .arg("--edition=2024")
            .arg("--error-format=json")
            .arg("--emit=metadata")
            .arg("--out-dir")
            .arg(temp_dir)
            .arg(&rs_file)
            .output()?;
        Ok(output)
    })
}

fn emit_translated_diagnostics(
    output: &std::process::Output,
    source_map: &crate::sourcemap::SourceMap,
) -> anyhow::Result<bool> {
    let stderr = String::from_utf8(output.stderr.clone())?;
    let mut saw_diagnostic = false;
    for line in stderr.lines() {
        if let Ok(msg) = serde_json::from_str::<crate::mirage::Diagnostic>(line) {
            let translated = crate::mirage::Mirage::translate_error(&msg, source_map);
            println!("{}", translated);
            saw_diagnostic = true;
        } else if !line.trim().is_empty() {
            eprintln!("{}", line);
        }
    }
    Ok(saw_diagnostic)
}

fn line_col_from_offset(source: &str, offset: usize) -> (usize, usize) {
    let safe_offset = offset.min(source.len());
    let prefix = &source[..safe_offset];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
    let col = prefix
        .rsplit('\n')
        .next()
        .map_or(1, |s| s.chars().count() + 1);
    (line, col)
}

fn format_parse_error(input_content: &str, err: ParserError<'_>) -> anyhow::Error {
    match err {
        NomErr::Error(e) | NomErr::Failure(e) => {
            if let Some((token, span)) = e.input.first() {
                let (line, col) = line_col_from_offset(input_content, span.start);
                anyhow::anyhow!(
                    "Parsing error at line {}, column {} near token {:?}",
                    line,
                    col,
                    token
                )
            } else {
                anyhow::anyhow!("Parsing error at end of file")
            }
        }
        NomErr::Incomplete(_) => anyhow::anyhow!("Parsing error: incomplete input"),
    }
}

#[derive(Debug)]
struct SemanticError {
    offset: usize,
    message: String,
}

#[derive(Debug, Clone, Copy)]
struct BindingInfo {
    is_mut: bool,
    can_write_through: bool,
    callable_arity: Option<usize>,
}

#[derive(Debug, Default)]
struct SemanticIndex {
    struct_names: HashSet<String>,
    protocol_methods: HashMap<String, ProtocolInfo>,
}

#[derive(Debug, Default)]
struct ProtocolInfo {
    method_order: Vec<String>,
    methods: HashMap<String, MethodSignature>,
}

#[derive(Debug, Clone)]
struct MethodSignature {
    params: Vec<ParamSignature>,
    return_ty: Option<crate::ast::Type>,
}

#[derive(Debug, Clone)]
struct ParamSignature {
    is_mut: bool,
    ty: Option<crate::ast::Type>,
}

fn validate_program(input_content: &str, program: &crate::ast::Program) -> anyhow::Result<()> {
    validate_top_level_declarations(program).map_err(|err| {
        let (line, col) = line_col_from_offset(input_content, err.offset);
        anyhow::anyhow!(
            "Semantic error at line {}, column {}: {}",
            line,
            col,
            err.message
        )
    })?;
    let semantic_index = collect_semantic_index(program);
    let struct_fields = collect_struct_fields(program);
    let mut scopes = vec![HashMap::new()];
    validate_statements(
        &program.statements,
        &mut scopes,
        &semantic_index,
        &struct_fields,
        0,
        0,
        true,
    )
    .map_err(|err| {
        let (line, col) = line_col_from_offset(input_content, err.offset);
        anyhow::anyhow!(
            "Semantic error at line {}, column {}: {}",
            line,
            col,
            err.message
        )
    })
}

fn validate_top_level_declarations(program: &crate::ast::Program) -> Result<(), SemanticError> {
    let mut function_names = HashSet::new();
    let mut struct_names = HashSet::new();
    let mut protocol_names = HashSet::new();
    let mut declarations = HashMap::new();

    for stmt in &program.statements {
        match &stmt.kind {
            crate::ast::StatementKind::Def { name, .. } => {
                if !function_names.insert(name.as_str()) {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!("duplicate top-level function `{}`", name),
                    });
                }
                if let Some(previous) = declarations.insert(name.as_str(), "function") {
                    if previous != "function" {
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: format!(
                                "top-level name `{}` is already declared as {}",
                                name, previous
                            ),
                        });
                    }
                }
            }
            crate::ast::StatementKind::Struct { name, .. } => {
                if !struct_names.insert(name.as_str()) {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!("duplicate top-level struct `{}`", name),
                    });
                }
                if let Some(previous) = declarations.insert(name.as_str(), "struct") {
                    if previous != "struct" {
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: format!(
                                "top-level name `{}` is already declared as {}",
                                name, previous
                            ),
                        });
                    }
                }
            }
            crate::ast::StatementKind::Protocol { name, .. } => {
                if !protocol_names.insert(name.as_str()) {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!("duplicate top-level protocol `{}`", name),
                    });
                }
                if let Some(previous) = declarations.insert(name.as_str(), "protocol") {
                    if previous != "protocol" {
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: format!(
                                "top-level name `{}` is already declared as {}",
                                name, previous
                            ),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn validate_statements(
    statements: &[crate::ast::Statement],
    scopes: &mut Vec<HashMap<String, BindingInfo>>,
    semantic_index: &SemanticIndex,
    struct_fields: &HashMap<String, Vec<String>>,
    nesting_depth: usize,
    function_depth: usize,
    predeclare_defs: bool,
) -> Result<(), SemanticError> {
    if predeclare_defs {
        predeclare_block_symbols(statements, scopes)?;
    }

    for stmt in statements {
        use crate::ast::StatementKind;
        match &stmt.kind {
            StatementKind::Let { name, value, .. } => {
                if current_scope_contains(scopes, name) {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!("duplicate local binding `{}` in same scope", name),
                    });
                }
                validate_expression(
                    value,
                    stmt.span.start,
                    scopes,
                    semantic_index,
                    struct_fields,
                )?;
                declare_binding(
                    scopes,
                    name,
                    BindingInfo {
                        is_mut: false,
                        can_write_through: expression_is_unique_ref(value),
                        callable_arity: None,
                    },
                );
            }
            StatementKind::Mut { name, value, .. } => {
                if current_scope_contains(scopes, name) {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!("duplicate local binding `{}` in same scope", name),
                    });
                }
                validate_expression(
                    value,
                    stmt.span.start,
                    scopes,
                    semantic_index,
                    struct_fields,
                )?;
                declare_binding(
                    scopes,
                    name,
                    BindingInfo {
                        is_mut: true,
                        can_write_through: true,
                        callable_arity: None,
                    },
                );
            }
            StatementKind::Def { params, body, .. } => {
                let mut seen_params = HashSet::new();
                for param in params {
                    if !seen_params.insert(param.name.as_str()) {
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: format!(
                                "duplicate parameter `{}` in function signature",
                                param.name
                            ),
                        });
                    }
                }
                scopes.push(HashMap::new());
                for param in params {
                    declare_binding(
                        scopes,
                        &param.name,
                        BindingInfo {
                            is_mut: param.is_mut,
                            can_write_through: param.is_mut
                                || param.ty.as_ref().is_some_and(type_is_unique_ref),
                            callable_arity: None,
                        },
                    );
                }
                validate_statements(
                    body,
                    scopes,
                    semantic_index,
                    struct_fields,
                    nesting_depth + 1,
                    function_depth + 1,
                    true,
                )?;
                scopes.pop();
            }
            StatementKind::If {
                condition,
                then_block,
                else_block,
            } => {
                validate_expression(
                    condition,
                    stmt.span.start,
                    scopes,
                    semantic_index,
                    struct_fields,
                )?;
                scopes.push(HashMap::new());
                validate_statements(
                    then_block,
                    scopes,
                    semantic_index,
                    struct_fields,
                    nesting_depth + 1,
                    function_depth,
                    true,
                )?;
                scopes.pop();
                if let Some(block) = else_block {
                    scopes.push(HashMap::new());
                    validate_statements(
                        block,
                        scopes,
                        semantic_index,
                        struct_fields,
                        nesting_depth + 1,
                        function_depth,
                        true,
                    )?;
                    scopes.pop();
                }
            }
            StatementKind::For {
                var,
                iterable,
                body,
            } => {
                validate_expression(
                    iterable,
                    stmt.span.start,
                    scopes,
                    semantic_index,
                    struct_fields,
                )?;
                scopes.push(HashMap::new());
                declare_binding(
                    scopes,
                    var,
                    BindingInfo {
                        is_mut: false,
                        can_write_through: false,
                        callable_arity: None,
                    },
                );
                validate_statements(
                    body,
                    scopes,
                    semantic_index,
                    struct_fields,
                    nesting_depth + 1,
                    function_depth,
                    true,
                )?;
                scopes.pop();
            }
            StatementKind::Match { expression, arms } => {
                validate_expression(
                    expression,
                    stmt.span.start,
                    scopes,
                    semantic_index,
                    struct_fields,
                )?;
                for (pattern, body) in arms {
                    scopes.push(HashMap::new());
                    let pattern_bindings =
                        collect_pattern_bindings(pattern, scopes, semantic_index);
                    for binding in pattern_bindings {
                        if current_scope_contains(scopes, &binding) {
                            return Err(SemanticError {
                                offset: stmt.span.start,
                                message: format!(
                                    "duplicate match pattern binding `{}` in same arm",
                                    binding
                                ),
                            });
                        }
                        declare_binding(
                            scopes,
                            &binding,
                            BindingInfo {
                                is_mut: false,
                                can_write_through: false,
                                callable_arity: None,
                            },
                        );
                    }
                    validate_expression(
                        pattern,
                        stmt.span.start,
                        scopes,
                        semantic_index,
                        struct_fields,
                    )?;
                    validate_statements(
                        body,
                        scopes,
                        semantic_index,
                        struct_fields,
                        nesting_depth + 1,
                        function_depth,
                        true,
                    )?;
                    scopes.pop();
                }
            }
            StatementKind::Return(Some(expr)) => {
                if function_depth == 0 {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: "`return` is only allowed inside `def` bodies".to_string(),
                    });
                }
                validate_expression(expr, stmt.span.start, scopes, semantic_index, struct_fields)?;
            }
            StatementKind::Expr(expr) => {
                validate_expression(expr, stmt.span.start, scopes, semantic_index, struct_fields)?;
            }
            StatementKind::Impl {
                for_type, methods, ..
            } => {
                validate_method_block_shapes(
                    methods,
                    stmt.span.start,
                    &format!("impl for `{}`", for_type),
                )?;
                validate_method_name_uniqueness(
                    methods,
                    stmt.span.start,
                    &format!("impl for `{}`", for_type),
                )?;
                validate_impl_declaration(stmt, semantic_index)?;
                validate_statements(
                    methods,
                    scopes,
                    semantic_index,
                    struct_fields,
                    nesting_depth + 1,
                    function_depth,
                    false,
                )?;
            }
            StatementKind::Protocol { name, methods } => {
                validate_method_block_shapes(
                    methods,
                    stmt.span.start,
                    &format!("protocol `{}`", name),
                )?;
                validate_method_name_uniqueness(
                    methods,
                    stmt.span.start,
                    &format!("protocol `{}`", name),
                )?;
                validate_statements(
                    methods,
                    scopes,
                    semantic_index,
                    struct_fields,
                    nesting_depth + 1,
                    function_depth,
                    false,
                )?;
            }
            StatementKind::Import { path, .. } => {
                if nesting_depth > 0 {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: "`import` is only allowed at top level".to_string(),
                    });
                }
                if crate::imports::rust_use_from_import_path(path).is_none()
                    && matches!(&stmt.kind, StatementKind::Import { alias: Some(_), .. })
                {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message:
                            "aliasing non-rust imports is unsupported (use plain `import \"path\"`)"
                                .to_string(),
                    });
                }
                if let Some(use_path) = crate::imports::rust_use_from_import_path(path)
                    && !crate::imports::is_supported_rust_root(&use_path)
                {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!(
                            "unsupported rust import root `{}` (only std/core/alloc are supported)",
                            use_path.split("::").next().unwrap_or(&use_path)
                        ),
                    });
                }
            }
            StatementKind::FromImport { path, .. } => {
                if nesting_depth > 0 {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: "`from ... import ...` is only allowed at top level".to_string(),
                    });
                }
                if let StatementKind::FromImport { items, .. } = &stmt.kind {
                    validate_from_import_items(items, stmt.span.start)?;
                    let rust_path = crate::imports::rust_use_from_import_path(path);
                    if rust_path.is_none() {
                        if items.iter().any(|item| item.alias.is_some()) {
                            return Err(SemanticError {
                                offset: stmt.span.start,
                                message: "aliasing non-rust from-import items is unsupported (remove `as ...`)"
                                    .to_string(),
                            });
                        }
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: "non-rust `from ... import ...` is unsupported (use plain `import \"path\"`)"
                                .to_string(),
                        });
                    }
                    if let Some(use_path) = rust_path
                        && !crate::imports::is_supported_rust_root(&use_path)
                    {
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: format!(
                                "unsupported rust import root `{}` (only std/core/alloc are supported)",
                                use_path.split("::").next().unwrap_or(&use_path)
                            ),
                        });
                    }
                }
            }
            StatementKind::Return(None) => {
                if function_depth == 0 {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: "`return` is only allowed inside `def` bodies".to_string(),
                    });
                }
            }
            StatementKind::Struct { name, fields } => {
                let mut seen_fields = HashSet::new();
                for field in fields {
                    if !seen_fields.insert(field.name.as_str()) {
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: format!(
                                "duplicate field `{}` in struct `{}`",
                                field.name, name
                            ),
                        });
                    }
                }
            }
            StatementKind::PyImport(_) => {}
        }
    }
    Ok(())
}

fn validate_expression(
    expr: &crate::ast::Expression,
    offset: usize,
    scopes: &[HashMap<String, BindingInfo>],
    semantic_index: &SemanticIndex,
    struct_fields: &HashMap<String, Vec<String>>,
) -> Result<(), SemanticError> {
    use crate::ast::Expression;
    match expr {
        Expression::BinaryOp(left, crate::ast::BinaryOp::Assign, right) => {
            validate_assignment_target(left, offset, scopes, semantic_index, struct_fields)?;
            validate_expression(right, offset, scopes, semantic_index, struct_fields)
        }
        Expression::BinaryOp(left, _, right) => {
            validate_expression(left, offset, scopes, semantic_index, struct_fields)?;
            validate_expression(right, offset, scopes, semantic_index, struct_fields)
        }
        Expression::Call(callee, args) => {
            validate_expression(callee, offset, scopes, semantic_index, struct_fields)?;
            if let Some(struct_name) = constructor_name(callee, struct_fields) {
                validate_constructor_call(
                    struct_name,
                    args,
                    offset,
                    scopes,
                    semantic_index,
                    struct_fields,
                )?;
            } else {
                validate_declared_call_arity(callee, args.len(), offset, scopes)?;
                for arg in args {
                    validate_expression(arg, offset, scopes, semantic_index, struct_fields)?;
                }
            }
            Ok(())
        }
        Expression::GenericCall(callee, _, args) => {
            validate_expression(callee, offset, scopes, semantic_index, struct_fields)?;
            validate_declared_call_arity(callee, args.len(), offset, scopes)?;
            for arg in args {
                validate_expression(arg, offset, scopes, semantic_index, struct_fields)?;
            }
            Ok(())
        }
        Expression::MacroCall(_, args) | Expression::Literal(crate::ast::Literal::List(args)) => {
            for arg in args {
                validate_expression(arg, offset, scopes, semantic_index, struct_fields)?;
            }
            Ok(())
        }
        Expression::Move(inner) => {
            validate_mutable_binding(inner, scopes, "move", offset)?;
            validate_expression(inner, offset, scopes, semantic_index, struct_fields)
        }
        Expression::UniqueRef(inner) => {
            validate_mutable_binding(inner, scopes, "~", offset)?;
            validate_expression(inner, offset, scopes, semantic_index, struct_fields)
        }
        Expression::MemberAccess(inner, _)
        | Expression::SharedRef(inner)
        | Expression::Question(inner)
        | Expression::Unwrap(inner) => {
            validate_expression(inner, offset, scopes, semantic_index, struct_fields)
        }
        Expression::Index(inner, index) => {
            validate_expression(inner, offset, scopes, semantic_index, struct_fields)?;
            validate_expression(index, offset, scopes, semantic_index, struct_fields)
        }
        Expression::Ident(name) => {
            if is_resolvable_ident(name, scopes, semantic_index) {
                Ok(())
            } else {
                Err(SemanticError {
                    offset,
                    message: format!("unknown identifier `{}`", name),
                })
            }
        }
        Expression::Literal(_) => Ok(()),
    }
}

fn validate_mutable_binding(
    expr: &crate::ast::Expression,
    scopes: &[HashMap<String, BindingInfo>],
    op_name: &str,
    offset: usize,
) -> Result<(), SemanticError> {
    if !is_place_expression(expr) {
        return Err(SemanticError {
            offset,
            message: format!(
                "`{}` expects a mutable place expression (`x`, `obj.field`, or `items[i]`)",
                op_name
            ),
        });
    }

    if let Some(root) = place_root_ident(expr) {
        let Some(binding) = lookup_binding(root, scopes) else {
            return Err(SemanticError {
                offset,
                message: format!("`{}` requires declared binding `{}`", op_name, root),
            });
        };

        let can_mutate_place = match expr {
            crate::ast::Expression::Ident(_) => binding.is_mut,
            _ => binding.can_write_through,
        };
        if can_mutate_place {
            return Ok(());
        }

        let message = match expr {
            crate::ast::Expression::Ident(_) => format!(
                "`{}` requires mutable binding `{}` (declare with `mut` first)",
                op_name, root
            ),
            _ => format!(
                "`{}` through `{}` requires mutable root binding or a unique reference (`~`)",
                op_name, root
            ),
        };
        Err(SemanticError { offset, message })
    } else {
        Err(SemanticError {
            offset,
            message: format!("`{}` requires a mutable local binding", op_name),
        })
    }
}

fn is_place_expression(expr: &crate::ast::Expression) -> bool {
    match expr {
        crate::ast::Expression::Ident(_) => true,
        crate::ast::Expression::MemberAccess(inner, _) => is_place_expression(inner),
        crate::ast::Expression::Index(inner, _) => is_place_expression(inner),
        _ => false,
    }
}

fn place_root_ident(expr: &crate::ast::Expression) -> Option<&str> {
    match expr {
        crate::ast::Expression::Ident(name) => Some(name.as_str()),
        crate::ast::Expression::MemberAccess(inner, _)
        | crate::ast::Expression::Index(inner, _) => place_root_ident(inner),
        _ => None,
    }
}

fn validate_assignment_target(
    expr: &crate::ast::Expression,
    offset: usize,
    scopes: &[HashMap<String, BindingInfo>],
    semantic_index: &SemanticIndex,
    struct_fields: &HashMap<String, Vec<String>>,
) -> Result<(), SemanticError> {
    if !is_place_expression(expr) {
        return Err(SemanticError {
            offset,
            message:
                "assignment expects a place expression on the left side (`x`, `obj.field`, or `items[i]`)"
                    .to_string(),
        });
    }

    validate_place_subexpressions(expr, offset, scopes, semantic_index, struct_fields)?;

    let Some(root) = place_root_ident(expr) else {
        return Err(SemanticError {
            offset,
            message: "assignment requires a declared local binding on the left side".to_string(),
        });
    };

    let Some(binding) = lookup_binding(root, scopes) else {
        return Err(SemanticError {
            offset,
            message: format!("assignment requires declared binding `{}`", root),
        });
    };

    let can_assign = match expr {
        crate::ast::Expression::Ident(_) => binding.is_mut,
        _ => binding.can_write_through,
    };
    if can_assign {
        return Ok(());
    }

    let message = match expr {
        crate::ast::Expression::Ident(_) => format!(
            "assignment requires mutable binding `{}` (declare with `mut` first)",
            root
        ),
        _ => format!(
            "assignment through `{}` requires mutable root binding or a unique reference (`~`)",
            root
        ),
    };
    Err(SemanticError { offset, message })
}

fn validate_place_subexpressions(
    expr: &crate::ast::Expression,
    offset: usize,
    scopes: &[HashMap<String, BindingInfo>],
    semantic_index: &SemanticIndex,
    struct_fields: &HashMap<String, Vec<String>>,
) -> Result<(), SemanticError> {
    match expr {
        crate::ast::Expression::MemberAccess(inner, _) => {
            validate_place_subexpressions(inner, offset, scopes, semantic_index, struct_fields)
        }
        crate::ast::Expression::Index(inner, index) => {
            validate_place_subexpressions(inner, offset, scopes, semantic_index, struct_fields)?;
            validate_expression(index, offset, scopes, semantic_index, struct_fields)
        }
        crate::ast::Expression::Ident(_) => Ok(()),
        _ => Err(SemanticError {
            offset,
            message:
                "assignment expects a place expression on the left side (`x`, `obj.field`, or `items[i]`)"
                    .to_string(),
        }),
    }
}

fn predeclare_block_symbols(
    statements: &[crate::ast::Statement],
    scopes: &mut [HashMap<String, BindingInfo>],
) -> Result<(), SemanticError> {
    for stmt in statements {
        match &stmt.kind {
            crate::ast::StatementKind::Def { name, params, .. } => {
                if current_scope_contains(scopes, name) {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!("duplicate local name `{}` in same scope", name),
                    });
                }
                declare_binding(
                    scopes,
                    name,
                    BindingInfo {
                        is_mut: false,
                        can_write_through: false,
                        callable_arity: Some(params.len()),
                    },
                );
            }
            crate::ast::StatementKind::Import { path, alias } => {
                let Some(use_path) =
                    crate::imports::rust_use_from_import(path, alias.as_deref())
                else {
                    continue;
                };
                let Some(name) = crate::imports::rust_import_binding_name(&use_path) else {
                    continue;
                };
                if current_scope_contains(scopes, name) {
                    return Err(SemanticError {
                        offset: stmt.span.start,
                        message: format!("duplicate local name `{}` in same scope", name),
                    });
                }
                declare_binding(
                    scopes,
                    name,
                    BindingInfo {
                        is_mut: false,
                        can_write_through: false,
                        callable_arity: None,
                    },
                );
            }
            crate::ast::StatementKind::FromImport { path, items } => {
                validate_from_import_items(items, stmt.span.start)?;
                if crate::imports::rust_use_from_import_path(path).is_none() {
                    continue;
                }
                for item in items {
                    let name = item.alias.as_deref().unwrap_or(&item.name);
                    if current_scope_contains(scopes, name) {
                        return Err(SemanticError {
                            offset: stmt.span.start,
                            message: format!("duplicate local name `{}` in same scope", name),
                        });
                    }
                    declare_binding(
                        scopes,
                        name,
                        BindingInfo {
                            is_mut: false,
                            can_write_through: false,
                            callable_arity: None,
                        },
                    );
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_declared_call_arity(
    callee: &crate::ast::Expression,
    actual_arg_count: usize,
    offset: usize,
    scopes: &[HashMap<String, BindingInfo>],
) -> Result<(), SemanticError> {
    let crate::ast::Expression::Ident(name) = callee else {
        return Ok(());
    };

    let Some(binding) = lookup_binding(name, scopes) else {
        return Ok(());
    };
    let Some(expected_arg_count) = binding.callable_arity else {
        return Ok(());
    };

    if expected_arg_count == actual_arg_count {
        return Ok(());
    }

    Err(SemanticError {
        offset,
        message: format!(
            "call to `{}` expects {} argument(s), found {}",
            name, expected_arg_count, actual_arg_count
        ),
    })
}

fn validate_method_name_uniqueness(
    methods: &[crate::ast::Statement],
    fallback_offset: usize,
    owner_label: &str,
) -> Result<(), SemanticError> {
    let mut seen = HashSet::new();
    for method in methods {
        if let crate::ast::StatementKind::Def { name, .. } = &method.kind {
            if !seen.insert(name.as_str()) {
                return Err(SemanticError {
                    offset: method.span.start.max(fallback_offset),
                    message: format!("duplicate method `{}` in {}", name, owner_label),
                });
            }
        }
    }
    Ok(())
}

fn validate_from_import_items(
    items: &[crate::ast::ImportItem],
    offset: usize,
) -> Result<(), SemanticError> {
    let mut seen_items = HashSet::new();
    let mut seen_bindings = HashSet::new();
    for item in items {
        if !seen_items.insert(item.name.as_str()) {
            return Err(SemanticError {
                offset,
                message: format!("duplicate from-import item `{}`", item.name),
            });
        }

        let binding_name = item.alias.as_deref().unwrap_or(&item.name);
        if !seen_bindings.insert(binding_name) {
            return Err(SemanticError {
                offset,
                message: format!(
                    "from-import introduces duplicate local name `{}` in one statement",
                    binding_name
                ),
            });
        }
    }
    Ok(())
}

fn validate_method_block_shapes(
    methods: &[crate::ast::Statement],
    fallback_offset: usize,
    owner_label: &str,
) -> Result<(), SemanticError> {
    for method in methods {
        if !matches!(method.kind, crate::ast::StatementKind::Def { .. }) {
            return Err(SemanticError {
                offset: method.span.start.max(fallback_offset),
                message: format!("{} can only contain `def` method declarations", owner_label),
            });
        }
    }
    Ok(())
}

fn validate_impl_declaration(
    stmt: &crate::ast::Statement,
    semantic_index: &SemanticIndex,
) -> Result<(), SemanticError> {
    let crate::ast::StatementKind::Impl {
        protocol,
        for_type,
        methods,
    } = &stmt.kind
    else {
        return Ok(());
    };

    if !semantic_index.struct_names.contains(for_type) {
        return Err(SemanticError {
            offset: stmt.span.start,
            message: format!("impl target `{}` must be a declared struct", for_type),
        });
    }

    let Some(protocol_name) = protocol.as_ref() else {
        return Ok(());
    };

    let Some(protocol_info) = semantic_index.protocol_methods.get(protocol_name) else {
        return Err(SemanticError {
            offset: stmt.span.start,
            message: format!("impl references unknown protocol `{}`", protocol_name),
        });
    };

    let required_set: HashSet<&str> = protocol_info
        .method_order
        .iter()
        .map(String::as_str)
        .collect();
    let mut provided = HashSet::new();
    for method in methods {
        if let crate::ast::StatementKind::Def { name, .. } = &method.kind {
            provided.insert(name.as_str());
            if !required_set.contains(name.as_str()) {
                return Err(SemanticError {
                    offset: method.span.start.max(stmt.span.start),
                    message: format!(
                        "impl for protocol `{}` on `{}` defines unknown method `{}`",
                        protocol_name, for_type, name
                    ),
                });
            }

            let Some(required_signature) = protocol_info.methods.get(name) else {
                continue;
            };
            let actual_signature = method_signature_from_statement(method);
            if let Some(reason) =
                validate_method_signature_match(required_signature, &actual_signature)
            {
                return Err(SemanticError {
                    offset: method.span.start.max(stmt.span.start),
                    message: format!(
                        "impl for protocol `{}` on `{}` has incompatible signature for method `{}`: {}",
                        protocol_name, for_type, name, reason
                    ),
                });
            }
        }
    }

    let missing: Vec<&str> = protocol_info
        .method_order
        .iter()
        .map(String::as_str)
        .filter(|name| !provided.contains(name))
        .collect();
    if !missing.is_empty() {
        return Err(SemanticError {
            offset: stmt.span.start,
            message: format!(
                "impl for protocol `{}` on `{}` is missing methods: {}",
                protocol_name,
                for_type,
                missing.join(", ")
            ),
        });
    }

    Ok(())
}

fn validate_method_signature_match(
    expected: &MethodSignature,
    actual: &MethodSignature,
) -> Option<String> {
    if expected.params.len() != actual.params.len() {
        return Some(format!(
            "expected {} parameter(s), found {}",
            expected.params.len(),
            actual.params.len()
        ));
    }

    for (idx, (expected_param, actual_param)) in
        expected.params.iter().zip(&actual.params).enumerate()
    {
        let position = idx + 1;
        if expected_param.is_mut != actual_param.is_mut {
            return Some(format!(
                "parameter {} mutability mismatch (expected `{}`, found `{}`)",
                position,
                if expected_param.is_mut {
                    "mut"
                } else {
                    "non-mut"
                },
                if actual_param.is_mut {
                    "mut"
                } else {
                    "non-mut"
                }
            ));
        }
        if expected_param.ty != actual_param.ty {
            return Some(format!(
                "parameter {} type mismatch (expected `{}`, found `{}`)",
                position,
                format_optional_type(expected_param.ty.as_ref()),
                format_optional_type(actual_param.ty.as_ref())
            ));
        }
    }

    if expected.return_ty != actual.return_ty {
        return Some(format!(
            "return type mismatch (expected `{}`, found `{}`)",
            format_optional_type(expected.return_ty.as_ref()),
            format_optional_type(actual.return_ty.as_ref())
        ));
    }

    None
}

fn format_optional_type(ty: Option<&crate::ast::Type>) -> String {
    match ty {
        Some(value) => format!("{:?}", value),
        None => "unspecified".to_string(),
    }
}

fn validate_constructor_call(
    struct_name: &str,
    args: &[crate::ast::Expression],
    offset: usize,
    scopes: &[HashMap<String, BindingInfo>],
    semantic_index: &SemanticIndex,
    struct_fields: &HashMap<String, Vec<String>>,
) -> Result<(), SemanticError> {
    let Some(fields) = struct_fields.get(struct_name) else {
        return Ok(());
    };
    let mut named_fields = HashSet::new();
    let mut positional_count = 0usize;

    for arg in args {
        if let crate::ast::Expression::BinaryOp(left, crate::ast::BinaryOp::Assign, value) = arg {
            let crate::ast::Expression::Ident(field_name) = left.as_ref() else {
                return Err(SemanticError {
                    offset,
                    message: "named constructor arguments must be in `field = value` form"
                        .to_string(),
                });
            };
            if !fields.iter().any(|field| field == field_name) {
                return Err(SemanticError {
                    offset,
                    message: format!(
                        "constructor `{}` has no field `{}`",
                        struct_name, field_name
                    ),
                });
            }
            if !named_fields.insert(field_name.clone()) {
                return Err(SemanticError {
                    offset,
                    message: format!(
                        "constructor `{}` received duplicate field `{}`",
                        struct_name, field_name
                    ),
                });
            }
            validate_expression(value, offset, scopes, semantic_index, struct_fields)?;
            continue;
        }

        positional_count += 1;
        validate_expression(arg, offset, scopes, semantic_index, struct_fields)?;
    }

    let available_positional = fields.len().saturating_sub(named_fields.len());
    if positional_count > available_positional {
        return Err(SemanticError {
            offset,
            message: format!(
                "constructor `{}` received too many positional arguments",
                struct_name
            ),
        });
    }

    let mut remaining_positional = positional_count;
    let mut missing_fields = Vec::new();
    for field in fields {
        if named_fields.contains(field) {
            continue;
        }
        if remaining_positional > 0 {
            remaining_positional -= 1;
            continue;
        }
        missing_fields.push(field.clone());
    }

    if !missing_fields.is_empty() {
        return Err(SemanticError {
            offset,
            message: format!(
                "constructor `{}` is missing fields: {}",
                struct_name,
                missing_fields.join(", ")
            ),
        });
    }

    Ok(())
}

fn constructor_name<'a>(
    callee: &'a crate::ast::Expression,
    struct_fields: &HashMap<String, Vec<String>>,
) -> Option<&'a str> {
    match callee {
        crate::ast::Expression::Ident(name) if struct_fields.contains_key(name) => Some(name),
        _ => None,
    }
}

fn declare_binding(scopes: &mut [HashMap<String, BindingInfo>], name: &str, binding: BindingInfo) {
    if let Some(scope) = scopes.last_mut() {
        scope.insert(name.to_string(), binding);
    }
}

fn current_scope_contains(scopes: &[HashMap<String, BindingInfo>], name: &str) -> bool {
    scopes.last().is_some_and(|scope| scope.contains_key(name))
}

fn lookup_binding<'a>(
    name: &str,
    scopes: &'a [HashMap<String, BindingInfo>],
) -> Option<&'a BindingInfo> {
    scopes.iter().rev().find_map(|scope| scope.get(name))
}

fn is_resolvable_ident(
    name: &str,
    scopes: &[HashMap<String, BindingInfo>],
    semantic_index: &SemanticIndex,
) -> bool {
    lookup_binding(name, scopes).is_some()
        || semantic_index.struct_names.contains(name)
        || crate::resolver::BUILTIN_TYPE_SYMBOLS.contains(&name)
        || crate::resolver::BUILTIN_VALUE_SYMBOLS.contains(&name)
}

fn collect_pattern_bindings(
    expr: &crate::ast::Expression,
    scopes: &[HashMap<String, BindingInfo>],
    semantic_index: &SemanticIndex,
) -> HashSet<String> {
    let mut bindings = HashSet::new();
    collect_pattern_bindings_inner(expr, scopes, semantic_index, &mut bindings);
    bindings
}

fn collect_pattern_bindings_inner(
    expr: &crate::ast::Expression,
    scopes: &[HashMap<String, BindingInfo>],
    semantic_index: &SemanticIndex,
    bindings: &mut HashSet<String>,
) {
    match expr {
        crate::ast::Expression::Ident(name) => {
            if !is_resolvable_ident(name, scopes, semantic_index) {
                bindings.insert(name.clone());
            }
        }
        crate::ast::Expression::BinaryOp(left, _, right) => {
            collect_pattern_bindings_inner(left, scopes, semantic_index, bindings);
            collect_pattern_bindings_inner(right, scopes, semantic_index, bindings);
        }
        crate::ast::Expression::Call(callee, args)
        | crate::ast::Expression::GenericCall(callee, _, args) => {
            collect_pattern_bindings_inner(callee, scopes, semantic_index, bindings);
            for arg in args {
                collect_pattern_bindings_inner(arg, scopes, semantic_index, bindings);
            }
        }
        crate::ast::Expression::MacroCall(_, args)
        | crate::ast::Expression::Literal(crate::ast::Literal::List(args)) => {
            for arg in args {
                collect_pattern_bindings_inner(arg, scopes, semantic_index, bindings);
            }
        }
        crate::ast::Expression::MemberAccess(inner, _)
        | crate::ast::Expression::Move(inner)
        | crate::ast::Expression::SharedRef(inner)
        | crate::ast::Expression::UniqueRef(inner)
        | crate::ast::Expression::Question(inner)
        | crate::ast::Expression::Unwrap(inner) => {
            collect_pattern_bindings_inner(inner, scopes, semantic_index, bindings);
        }
        crate::ast::Expression::Index(inner, index) => {
            collect_pattern_bindings_inner(inner, scopes, semantic_index, bindings);
            collect_pattern_bindings_inner(index, scopes, semantic_index, bindings);
        }
        crate::ast::Expression::Literal(_) => {}
    }
}

fn expression_is_unique_ref(expr: &crate::ast::Expression) -> bool {
    matches!(expr, crate::ast::Expression::UniqueRef(_))
}

fn type_is_unique_ref(ty: &crate::ast::Type) -> bool {
    matches!(ty, crate::ast::Type::UniqueRef(_))
}

fn collect_struct_fields(program: &crate::ast::Program) -> HashMap<String, Vec<String>> {
    let mut fields = HashMap::new();
    for stmt in &program.statements {
        if let crate::ast::StatementKind::Struct {
            name,
            fields: struct_fields,
        } = &stmt.kind
        {
            fields.insert(
                name.clone(),
                struct_fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect(),
            );
        }
    }
    fields
}

fn collect_semantic_index(program: &crate::ast::Program) -> SemanticIndex {
    let mut index = SemanticIndex::default();
    for stmt in &program.statements {
        match &stmt.kind {
            crate::ast::StatementKind::Struct { name, .. } => {
                index.struct_names.insert(name.clone());
            }
            crate::ast::StatementKind::Protocol { name, methods } => {
                let mut info = ProtocolInfo::default();
                for method in methods {
                    if let crate::ast::StatementKind::Def { name, .. } = &method.kind {
                        info.method_order.push(name.clone());
                        info.methods
                            .insert(name.clone(), method_signature_from_statement(method));
                    }
                }
                index.protocol_methods.insert(name.clone(), info);
            }
            _ => {}
        }
    }
    index
}

fn method_signature_from_statement(stmt: &crate::ast::Statement) -> MethodSignature {
    match &stmt.kind {
        crate::ast::StatementKind::Def {
            params, return_ty, ..
        } => MethodSignature {
            params: params
                .iter()
                .map(|param| ParamSignature {
                    is_mut: param.is_mut,
                    ty: param.ty.clone(),
                })
                .collect(),
            return_ty: return_ty.clone(),
        },
        _ => MethodSignature {
            params: Vec::new(),
            return_ty: None,
        },
    }
}
