use serde::Deserialize;
use crate::sourcemap::SourceMap;

#[derive(Deserialize, Debug)]
pub struct Diagnostic {
    pub message: String,
    pub spans: Vec<DiagnosticSpan>,
}

#[derive(Deserialize, Debug)]
pub struct DiagnosticSpan {
    pub line_start: usize,
    pub line_end: usize,
    pub file_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "reason")]
pub enum CargoMessage {
    #[serde(rename = "compiler-message")]
    CompilerMessage {
        message: Diagnostic,
    },
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
        for span in &msg.spans {
            // rustc lines are 1-based. SourceMap uses 0-based.
            let rs_line = span.line_start.saturating_sub(1);
            if let Some(ds_line) = source_map.get_ds_line(rs_line) {
                locations.push_str(&format!("
  Line {}: in Desert source", ds_line + 1));
            }
        }
        
        format!("{}{}", translated, locations)
    }
}
