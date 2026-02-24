pub mod ast;
pub mod lexer;
pub mod mirage;
pub mod parser;
pub mod resolver;
pub mod sourcemap;
pub mod transpiler;

use crate::lexer::Lexer;
use crate::parser::parse_program;
use crate::transpiler::Transpiler;

type ParserError<'a> = NomErr<NomError<&'a [crate::parser::TokenSpan]>>;
use clap::Parser;
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
        /// Input .ds file
        input: PathBuf,
        /// Output .rs file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Check a .ds file for errors
    Check {
        /// Input .ds file
        input: PathBuf,
    },
    /// Print resolved import graph order for a project
    Graph {
        /// Project directory containing desert.toml/Desert.toml
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transpile { input, output } => {
            let input_content = load_input_source(&input)?;
            let (rust_code, _) = transpile_file(&input_content)?;
            if let Some(output_path) = output {
                fs::write(output_path, rust_code)?;
            } else {
                println!("{}", rust_code);
            }
        }
        Commands::Check { input } => {
            let input_content = load_input_source(&input)?;
            let (rust_code, source_map) = transpile_file(&input_content)?;

            let temp_dir = unique_temp_dir();
            fs::create_dir_all(&temp_dir)?;
            let rs_file = temp_dir.join("main.rs");
            fs::write(&rs_file, rust_code)?;

            // Just use rustc for simple checks without a full cargo project
            let output = std::process::Command::new("rustc")
                .arg("--edition=2024")
                .arg("--error-format=json")
                .arg("--emit=metadata")
                .arg("--out-dir")
                .arg(&temp_dir)
                .arg(&rs_file)
                .output()?;

            let stderr = String::from_utf8(output.stderr)?;
            let mut saw_diagnostic = false;
            for line in stderr.lines() {
                if let Ok(msg) = serde_json::from_str::<crate::mirage::Diagnostic>(line) {
                    let translated = crate::mirage::Mirage::translate_error(&msg, &source_map);
                    println!("{}", translated);
                    saw_diagnostic = true;
                } else if !line.trim().is_empty() {
                    eprintln!("{}", line);
                }
            }

            let _ = fs::remove_dir_all(&temp_dir);
            if !output.status.success() {
                if !saw_diagnostic {
                    anyhow::bail!("Rust check failed.");
                }
                anyhow::bail!("Rust check failed with translated diagnostics.");
            }

            println!("Check passed.");
        }
        Commands::Graph { input } => {
            let (project_root, ordered_files) = resolve_project_graph(&input)?;
            for file in ordered_files {
                let display_path = file.strip_prefix(&project_root).unwrap_or(file.as_path());
                println!("{}", display_path.display());
            }
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Default)]
struct DesertManifest {
    package: Option<ManifestPackage>,
}

#[derive(Debug, Deserialize, Default)]
struct ManifestPackage {
    entry: Option<PathBuf>,
}

fn load_input_source(input: &Path) -> anyhow::Result<String> {
    if input.is_file() {
        return fs::read_to_string(input).map_err(Into::into);
    }
    if input.is_dir() {
        return load_project_source(input);
    }

    anyhow::bail!(
        "input path '{}' is neither a file nor a directory",
        input.display()
    )
}

fn resolve_project_entry(project_root: &Path) -> anyhow::Result<PathBuf> {
    let manifest_path = ["desert.toml", "Desert.toml"]
        .iter()
        .map(|name| project_root.join(name))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "project input '{}' is missing desert.toml or Desert.toml",
                project_root.display()
            )
        })?;

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

fn load_project_source(project_root: &Path) -> anyhow::Result<String> {
    let (_, ordered_files) = resolve_project_graph(project_root)?;
    let mut pieces = Vec::with_capacity(ordered_files.len());
    for file in ordered_files {
        pieces.push(fs::read_to_string(file)?);
    }
    Ok(pieces.join("\n\n"))
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

    if let Some(cycle_idx) = loading.iter().position(|path| path == &canonical_file) {
        let mut cycle = loading[cycle_idx..]
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        cycle.push(canonical_file.display().to_string());
        anyhow::bail!("import cycle detected: {}", cycle.join(" -> "));
    }

    let source = fs::read_to_string(&canonical_file)?;
    loading.push(canonical_file.clone());
    let program = parse_source(&source)?;

    for stmt in &program.statements {
        if let crate::ast::StatementKind::Import(path) = &stmt.kind {
            let import_path = resolve_import_path(&canonical_file, path, project_root)?;
            collect_project_sources(&import_path, project_root, visited, loading, ordered_files)?;
        }
    }

    loading.pop();
    visited.insert(canonical_file);
    ordered_files.push(file_path.canonicalize()?);
    Ok(())
}

fn resolve_import_path(
    current_file: &Path,
    import_path: &str,
    project_root: &Path,
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

    if !canonical.starts_with(project_root) {
        anyhow::bail!(
            "import '{}' from '{}' resolves outside project root '{}': '{}'",
            import_path,
            current_file.display(),
            project_root.display(),
            canonical.display()
        );
    }
    Ok(canonical)
}

fn transpile_file(input_content: &str) -> anyhow::Result<(String, crate::sourcemap::SourceMap)> {
    let program = parse_source(input_content)?;
    validate_program(input_content, &program)?;
    let transpiler = Transpiler::new();
    Ok(transpiler.transpile(&program, input_content))
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
}

fn validate_program(input_content: &str, program: &crate::ast::Program) -> anyhow::Result<()> {
    let struct_names = collect_struct_names(program);
    let mut scopes = vec![HashMap::new()];
    validate_statements(&program.statements, &mut scopes, &struct_names).map_err(|err| {
        let (line, col) = line_col_from_offset(input_content, err.offset);
        anyhow::anyhow!(
            "Semantic error at line {}, column {}: {}",
            line,
            col,
            err.message
        )
    })
}

fn validate_statements(
    statements: &[crate::ast::Statement],
    scopes: &mut Vec<HashMap<String, BindingInfo>>,
    struct_names: &HashSet<String>,
) -> Result<(), SemanticError> {
    predeclare_block_symbols(statements, scopes);

    for stmt in statements {
        use crate::ast::StatementKind;
        match &stmt.kind {
            StatementKind::Let { name, value, .. } => {
                validate_expression(value, stmt.span.start, scopes, struct_names)?;
                declare_binding(
                    scopes,
                    name,
                    BindingInfo {
                        is_mut: false,
                        can_write_through: expression_is_unique_ref(value),
                    },
                );
            }
            StatementKind::Mut { name, value, .. } => {
                validate_expression(value, stmt.span.start, scopes, struct_names)?;
                declare_binding(
                    scopes,
                    name,
                    BindingInfo {
                        is_mut: true,
                        can_write_through: true,
                    },
                );
            }
            StatementKind::Def { params, body, .. } => {
                scopes.push(HashMap::new());
                for param in params {
                    declare_binding(
                        scopes,
                        &param.name,
                        BindingInfo {
                            is_mut: param.is_mut,
                            can_write_through: param.is_mut
                                || param.ty.as_ref().is_some_and(type_is_unique_ref),
                        },
                    );
                }
                validate_statements(body, scopes, struct_names)?;
                scopes.pop();
            }
            StatementKind::If {
                condition,
                then_block,
                else_block,
            } => {
                validate_expression(condition, stmt.span.start, scopes, struct_names)?;
                scopes.push(HashMap::new());
                validate_statements(then_block, scopes, struct_names)?;
                scopes.pop();
                if let Some(block) = else_block {
                    scopes.push(HashMap::new());
                    validate_statements(block, scopes, struct_names)?;
                    scopes.pop();
                }
            }
            StatementKind::For {
                var,
                iterable,
                body,
            } => {
                validate_expression(iterable, stmt.span.start, scopes, struct_names)?;
                scopes.push(HashMap::new());
                declare_binding(
                    scopes,
                    var,
                    BindingInfo {
                        is_mut: false,
                        can_write_through: false,
                    },
                );
                validate_statements(body, scopes, struct_names)?;
                scopes.pop();
            }
            StatementKind::Match { expression, arms } => {
                validate_expression(expression, stmt.span.start, scopes, struct_names)?;
                for (pattern, body) in arms {
                    validate_expression(pattern, stmt.span.start, scopes, struct_names)?;
                    scopes.push(HashMap::new());
                    validate_statements(body, scopes, struct_names)?;
                    scopes.pop();
                }
            }
            StatementKind::Return(Some(expr)) | StatementKind::Expr(expr) => {
                validate_expression(expr, stmt.span.start, scopes, struct_names)?;
            }
            StatementKind::Impl { methods, .. } | StatementKind::Protocol { methods, .. } => {
                validate_statements(methods, scopes, struct_names)?;
            }
            StatementKind::Struct { .. }
            | StatementKind::Import(_)
            | StatementKind::PyImport(_)
            | StatementKind::Return(None) => {}
        }
    }
    Ok(())
}

fn validate_expression(
    expr: &crate::ast::Expression,
    offset: usize,
    scopes: &[HashMap<String, BindingInfo>],
    struct_names: &HashSet<String>,
) -> Result<(), SemanticError> {
    use crate::ast::Expression;
    match expr {
        Expression::BinaryOp(left, crate::ast::BinaryOp::Assign, right) => {
            validate_assignment_target(left, offset, scopes, struct_names)?;
            validate_expression(right, offset, scopes, struct_names)
        }
        Expression::BinaryOp(left, _, right) => {
            validate_expression(left, offset, scopes, struct_names)?;
            validate_expression(right, offset, scopes, struct_names)
        }
        Expression::Call(callee, args) => {
            validate_expression(callee, offset, scopes, struct_names)?;
            let constructor_call = is_struct_constructor_callee(callee, struct_names);
            for arg in args {
                if constructor_call {
                    validate_constructor_arg(arg, offset, scopes, struct_names)?;
                } else {
                    validate_expression(arg, offset, scopes, struct_names)?;
                }
            }
            Ok(())
        }
        Expression::GenericCall(callee, _, args) => {
            validate_expression(callee, offset, scopes, struct_names)?;
            for arg in args {
                validate_expression(arg, offset, scopes, struct_names)?;
            }
            Ok(())
        }
        Expression::MacroCall(_, args) | Expression::Literal(crate::ast::Literal::List(args)) => {
            for arg in args {
                validate_expression(arg, offset, scopes, struct_names)?;
            }
            Ok(())
        }
        Expression::Move(inner) => {
            validate_mutable_binding(inner, scopes, "move", offset)?;
            validate_expression(inner, offset, scopes, struct_names)
        }
        Expression::UniqueRef(inner) => {
            validate_mutable_binding(inner, scopes, "~", offset)?;
            validate_expression(inner, offset, scopes, struct_names)
        }
        Expression::MemberAccess(inner, _)
        | Expression::SharedRef(inner)
        | Expression::Question(inner)
        | Expression::Unwrap(inner) => validate_expression(inner, offset, scopes, struct_names),
        Expression::Index(inner, index) => {
            validate_expression(inner, offset, scopes, struct_names)?;
            validate_expression(index, offset, scopes, struct_names)
        }
        Expression::Literal(_) | Expression::Ident(_) => Ok(()),
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

        if !binding.is_mut {
            return Err(SemanticError {
                offset,
                message: format!(
                    "`{}` requires mutable binding `{}` (declare with `mut` first)",
                    op_name, root
                ),
            });
        }

        Ok(())
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
    struct_names: &HashSet<String>,
) -> Result<(), SemanticError> {
    if !is_place_expression(expr) {
        return Err(SemanticError {
            offset,
            message:
                "assignment expects a place expression on the left side (`x`, `obj.field`, or `items[i]`)"
                    .to_string(),
        });
    }

    validate_place_subexpressions(expr, offset, scopes, struct_names)?;

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
    struct_names: &HashSet<String>,
) -> Result<(), SemanticError> {
    match expr {
        crate::ast::Expression::MemberAccess(inner, _) => {
            validate_place_subexpressions(inner, offset, scopes, struct_names)
        }
        crate::ast::Expression::Index(inner, index) => {
            validate_place_subexpressions(inner, offset, scopes, struct_names)?;
            validate_expression(index, offset, scopes, struct_names)
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
) {
    for stmt in statements {
        if let crate::ast::StatementKind::Def { name, .. } = &stmt.kind {
            declare_binding(
                scopes,
                name,
                BindingInfo {
                    is_mut: false,
                    can_write_through: false,
                },
            );
        }
    }
}

fn validate_constructor_arg(
    arg: &crate::ast::Expression,
    offset: usize,
    scopes: &[HashMap<String, BindingInfo>],
    struct_names: &HashSet<String>,
) -> Result<(), SemanticError> {
    if let crate::ast::Expression::BinaryOp(left, crate::ast::BinaryOp::Assign, value) = arg {
        if !matches!(left.as_ref(), crate::ast::Expression::Ident(_)) {
            return Err(SemanticError {
                offset,
                message: "named constructor arguments must be in `field = value` form".to_string(),
            });
        }
        return validate_expression(value, offset, scopes, struct_names);
    }

    validate_expression(arg, offset, scopes, struct_names)
}

fn is_struct_constructor_callee(
    callee: &crate::ast::Expression,
    struct_names: &HashSet<String>,
) -> bool {
    matches!(callee, crate::ast::Expression::Ident(name) if struct_names.contains(name))
}

fn declare_binding(scopes: &mut [HashMap<String, BindingInfo>], name: &str, binding: BindingInfo) {
    if let Some(scope) = scopes.last_mut() {
        scope.insert(name.to_string(), binding);
    }
}

fn lookup_binding<'a>(
    name: &str,
    scopes: &'a [HashMap<String, BindingInfo>],
) -> Option<&'a BindingInfo> {
    scopes.iter().rev().find_map(|scope| scope.get(name))
}

fn expression_is_unique_ref(expr: &crate::ast::Expression) -> bool {
    matches!(expr, crate::ast::Expression::UniqueRef(_))
}

fn type_is_unique_ref(ty: &crate::ast::Type) -> bool {
    matches!(ty, crate::ast::Type::UniqueRef(_))
}

fn collect_struct_names(program: &crate::ast::Program) -> HashSet<String> {
    let mut names = HashSet::new();
    for stmt in &program.statements {
        if let crate::ast::StatementKind::Struct { name, .. } = &stmt.kind {
            names.insert(name.clone());
        }
    }
    names
}
