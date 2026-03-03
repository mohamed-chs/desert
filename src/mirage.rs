use crate::sourcemap::SourceMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Diagnostic {
    pub message: String,
    pub code: Option<DiagnosticCode>,
    pub spans: Vec<DiagnosticSpan>,
}

#[derive(Deserialize, Debug)]
pub struct DiagnosticCode {
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct DiagnosticSpan {
    pub line_start: usize,
    pub line_end: usize,
    pub file_name: String,
    #[serde(default = "default_column_start")]
    pub column_start: usize,
}

fn default_column_start() -> usize {
    1
}

#[derive(Deserialize, Debug)]
#[serde(tag = "reason")]
pub enum CargoMessage {
    #[serde(rename = "compiler-message")]
    CompilerMessage { message: Diagnostic },
    #[serde(other)]
    Other,
}

pub struct Mirage;

impl Mirage {
    pub fn translate_error(msg: &Diagnostic, source_map: &SourceMap) -> String {
        let mut translated = msg.message.clone();

        // Basic symbol replacement
        translated = translated.replace("Vec", "List");
        translated = translated.replace("&mut ", "~");
        translated = translated.replace("::", ".");

        let mut locations = String::new();
        let mut seen = std::collections::HashSet::new();
        for span in &msg.spans {
            // rustc lines are 1-based. SourceMap uses 0-based.
            let rs_line = span.line_start.saturating_sub(1);
            if let Some(ds_loc) = source_map.get_location_for_span(rs_line, span.column_start) {
                if !seen.insert((ds_loc.file.clone(), ds_loc.line, ds_loc.column)) {
                    continue;
                }
                locations.push_str(&format!(
                    "\n  {}:{}:{}: in Desert source",
                    ds_loc.file, ds_loc.line, ds_loc.column
                ));
            }
        }

        if let Some(hint) = Self::hint_for(msg) {
            translated.push_str("\nHint: ");
            translated.push_str(hint);
        }

        format!("{}{}", translated, locations)
    }

    fn hint_for(msg: &Diagnostic) -> Option<&'static str> {
        match msg.code.as_ref().map(|c| c.code.as_str()) {
            Some("E0596") => Some("Declare the binding with `mut` before using `~` or `move`."),
            Some("E0599") => Some(
                "Call a method that exists for this type, or implement a matching `protocol`/`impl`.",
            ),
            Some("E0308") => {
                Some("Match the declared type annotation with the assigned expression.")
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticCode, DiagnosticSpan, Mirage};
    use crate::sourcemap::{SourceLocation, SourceMap};

    fn span(rs_line_1_based: usize, rs_column_1_based: usize) -> DiagnosticSpan {
        DiagnosticSpan {
            line_start: rs_line_1_based,
            line_end: rs_line_1_based,
            file_name: "main.rs".to_string(),
            column_start: rs_column_1_based,
        }
    }

    #[test]
    fn translate_error_adds_mutability_hint_and_desert_line() {
        let mut source_map = SourceMap::new();
        source_map.add_mapping(
            2,
            SourceLocation {
                file: "example.ds".to_string(),
                line: 9,
                column: 5,
            },
        );
        let msg = Diagnostic {
            message: "cannot borrow `xs` as mutable, as it is not declared as mutable".to_string(),
            code: Some(DiagnosticCode {
                code: "E0596".to_string(),
            }),
            spans: vec![span(3, 1)],
        };

        let translated = Mirage::translate_error(&msg, &source_map);
        assert!(
            translated.contains("Hint: Declare the binding with `mut` before using `~` or `move`.")
        );
        assert!(translated.contains("example.ds:9:5: in Desert source"));
    }

    #[test]
    fn translate_error_adds_method_resolution_hint() {
        let source_map = SourceMap::new();
        let msg = Diagnostic {
            message: "no method named `nope` found for type `i32` in the current scope".to_string(),
            code: Some(DiagnosticCode {
                code: "E0599".to_string(),
            }),
            spans: vec![],
        };

        let translated = Mirage::translate_error(&msg, &source_map);
        assert!(
            translated.contains(
                "Hint: Call a method that exists for this type, or implement a matching `protocol`/`impl`."
            )
        );
    }

    #[test]
    fn translate_error_offsets_desert_column_using_rust_span_column() {
        let mut source_map = SourceMap::new();
        source_map.add_mapping_with_rust_column(
            1,
            SourceLocation {
                file: "sample.ds".to_string(),
                line: 3,
                column: 5,
            },
            5,
        );
        let msg = Diagnostic {
            message: "cannot find value `missing` in this scope".to_string(),
            code: None,
            spans: vec![span(2, 11)],
        };

        let translated = Mirage::translate_error(&msg, &source_map);
        assert!(translated.contains("sample.ds:3:11: in Desert source"));
    }
}
