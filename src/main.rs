pub mod lexer;
pub mod ast;
pub mod parser;
pub mod transpiler;
pub mod resolver;
pub mod sourcemap;
pub mod mirage;

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use crate::lexer::Lexer;
use crate::parser::parse_program;
use crate::transpiler::Transpiler;

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
            
            let temp_dir = std::env::temp_dir().join("desert_check");
            fs::create_dir_all(&temp_dir)?;
            let rs_file = temp_dir.join("main.rs");
            fs::write(&rs_file, rust_code)?;
            
            // Just use rustc for simple checks without a full cargo project
            let output = std::process::Command::new("rustc")
                .args(["--error-format=json", rs_file.to_str().unwrap()])
                .output()?;
            
            let stderr = String::from_utf8(output.stderr)?;
            for line in stderr.lines() {
                if let Ok(msg) = serde_json::from_str::<crate::mirage::Diagnostic>(line) {
                    let translated = crate::mirage::Mirage::translate_error(&msg, &source_map);
                    println!("{}", translated);
                }
            }
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
                let snippet = &input_content[span.clone()];
                anyhow::bail!("Lexing error at span {:?}: '{}'", span, snippet);
            }
        }
    }
    let (_, program) = parse_program(&tokens).map_err(|e| anyhow::anyhow!("Parsing error: {:?}", e))?;
    let transpiler = Transpiler::new();
    Ok(transpiler.transpile(&program, input_content))
}
