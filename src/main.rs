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
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transpile { input, output } => {
            let input_content = fs::read_to_string(&input)?;
            let (rust_code, _) = transpile_file(&input_content)?;
            if let Some(output_path) = output {
                fs::write(output_path, rust_code)?;
            } else {
                println!("{}", rust_code);
            }
        }
        Commands::Check { input } => {
            let input_content = fs::read_to_string(&input)?;
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
    }

    Ok(())
}

fn transpile_file(input_content: &str) -> anyhow::Result<(String, crate::sourcemap::SourceMap)> {
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
    validate_program(input_content, &program)?;
    let transpiler = Transpiler::new();
    Ok(transpiler.transpile(&program, input_content))
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

fn validate_program(input_content: &str, program: &crate::ast::Program) -> anyhow::Result<()> {
    let mut scopes = vec![HashMap::new()];
    validate_statements(&program.statements, &mut scopes).map_err(|err| {
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
    scopes: &mut Vec<HashMap<String, bool>>,
) -> Result<(), SemanticError> {
    for stmt in statements {
        use crate::ast::StatementKind;
        match &stmt.kind {
            StatementKind::Let { name, value, .. } => {
                validate_expression(value, stmt.span.start, scopes)?;
                declare_mutability(scopes, name, false);
            }
            StatementKind::Mut { name, value, .. } => {
                validate_expression(value, stmt.span.start, scopes)?;
                declare_mutability(scopes, name, true);
            }
            StatementKind::Ref { name, value, .. } | StatementKind::MutRef { name, value, .. } => {
                validate_expression(value, stmt.span.start, scopes)?;
                declare_mutability(scopes, name, false);
            }
            StatementKind::Def { params, body, .. } => {
                scopes.push(HashMap::new());
                for param in params {
                    declare_mutability(scopes, &param.name, param.is_mut);
                }
                validate_statements(body, scopes)?;
                scopes.pop();
            }
            StatementKind::If {
                condition,
                then_block,
                else_block,
            } => {
                validate_expression(condition, stmt.span.start, scopes)?;
                scopes.push(HashMap::new());
                validate_statements(then_block, scopes)?;
                scopes.pop();
                if let Some(block) = else_block {
                    scopes.push(HashMap::new());
                    validate_statements(block, scopes)?;
                    scopes.pop();
                }
            }
            StatementKind::For {
                var,
                iterable,
                body,
            } => {
                validate_expression(iterable, stmt.span.start, scopes)?;
                scopes.push(HashMap::new());
                declare_mutability(scopes, var, false);
                validate_statements(body, scopes)?;
                scopes.pop();
            }
            StatementKind::Match { expression, arms } => {
                validate_expression(expression, stmt.span.start, scopes)?;
                for (pattern, body) in arms {
                    validate_expression(pattern, stmt.span.start, scopes)?;
                    scopes.push(HashMap::new());
                    validate_statements(body, scopes)?;
                    scopes.pop();
                }
            }
            StatementKind::Return(Some(expr)) | StatementKind::Expr(expr) => {
                validate_expression(expr, stmt.span.start, scopes)?;
            }
            StatementKind::Impl { methods, .. } | StatementKind::Protocol { methods, .. } => {
                validate_statements(methods, scopes)?;
            }
            StatementKind::Struct { .. } | StatementKind::PyImport(_) | StatementKind::Return(None) => {}
        }
    }
    Ok(())
}

fn validate_expression(
    expr: &crate::ast::Expression,
    offset: usize,
    scopes: &[HashMap<String, bool>],
) -> Result<(), SemanticError> {
    use crate::ast::Expression;
    match expr {
        Expression::BinaryOp(left, _, right) => {
            validate_expression(left, offset, scopes)?;
            validate_expression(right, offset, scopes)
        }
        Expression::Call(callee, args) | Expression::GenericCall(callee, _, args) => {
            validate_expression(callee, offset, scopes)?;
            for arg in args {
                validate_expression(arg, offset, scopes)?;
            }
            Ok(())
        }
        Expression::MacroCall(_, args) | Expression::Literal(crate::ast::Literal::List(args)) => {
            for arg in args {
                validate_expression(arg, offset, scopes)?;
            }
            Ok(())
        }
        Expression::Move(inner) => {
            validate_mutable_binding(inner, scopes, "move", offset)?;
            validate_expression(inner, offset, scopes)
        }
        Expression::UniqueRef(inner) => {
            validate_mutable_binding(inner, scopes, "~", offset)?;
            validate_expression(inner, offset, scopes)
        }
        Expression::MemberAccess(inner, _)
        | Expression::SharedRef(inner)
        | Expression::Question(inner)
        | Expression::Unwrap(inner) => validate_expression(inner, offset, scopes),
        Expression::Index(inner, index) => {
            validate_expression(inner, offset, scopes)?;
            validate_expression(index, offset, scopes)
        }
        Expression::Literal(_) | Expression::Ident(_) => Ok(()),
    }
}

fn validate_mutable_binding(
    expr: &crate::ast::Expression,
    scopes: &[HashMap<String, bool>],
    op_name: &str,
    offset: usize,
) -> Result<(), SemanticError> {
    match expr {
        crate::ast::Expression::Ident(name) => {
            if is_mutable(name, scopes) {
                Ok(())
            } else {
                Err(SemanticError {
                    offset,
                    message: format!(
                        "`{}` requires mutable binding `{}` (declare with `mut` first)",
                        op_name, name
                    ),
                })
            }
        }
        _ => Ok(()),
    }
}

fn declare_mutability(scopes: &mut [HashMap<String, bool>], name: &str, is_mut: bool) {
    if let Some(scope) = scopes.last_mut() {
        scope.insert(name.to_string(), is_mut);
    }
}

fn is_mutable(name: &str, scopes: &[HashMap<String, bool>]) -> bool {
    scopes
        .iter()
        .rev()
        .find_map(|scope| scope.get(name).copied())
        .unwrap_or(false)
}
