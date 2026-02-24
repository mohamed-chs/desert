#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
}

pub struct SourceMap {
    // rs_line -> Desert source location
    mappings: Vec<Option<SourceLocation>>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, rs_line: usize, location: SourceLocation) {
        if rs_line >= self.mappings.len() {
            self.mappings.resize(rs_line + 1, None);
        }
        self.mappings[rs_line] = Some(location);
    }

    pub fn get_location(&self, rs_line: usize) -> Option<&SourceLocation> {
        self.mappings.get(rs_line).and_then(|loc| loc.as_ref())
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}
