#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
struct SourceMapping {
    location: SourceLocation,
    rust_column: usize,
}

pub struct SourceMap {
    // rs_line -> Desert source location
    mappings: Vec<Option<SourceMapping>>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, rs_line: usize, location: SourceLocation) {
        self.add_mapping_with_rust_column(rs_line, location, 1);
    }

    pub fn add_mapping_with_rust_column(
        &mut self,
        rs_line: usize,
        location: SourceLocation,
        rust_column: usize,
    ) {
        if rs_line >= self.mappings.len() {
            self.mappings.resize(rs_line + 1, None);
        }
        self.mappings[rs_line] = Some(SourceMapping {
            location,
            rust_column: rust_column.max(1),
        });
    }

    pub fn get_location(&self, rs_line: usize) -> Option<&SourceLocation> {
        self.mappings
            .get(rs_line)
            .and_then(|loc| loc.as_ref())
            .map(|mapping| &mapping.location)
    }

    pub fn get_location_for_span(
        &self,
        rs_line: usize,
        rs_column: usize,
    ) -> Option<SourceLocation> {
        let mapping = self.mappings.get(rs_line).and_then(|loc| loc.as_ref())?;
        let normalized_column = rs_column.max(mapping.rust_column);
        let delta = normalized_column - mapping.rust_column;

        Some(SourceLocation {
            file: mapping.location.file.clone(),
            line: mapping.location.line,
            column: mapping.location.column.saturating_add(delta),
        })
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}
