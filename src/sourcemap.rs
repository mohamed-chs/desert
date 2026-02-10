pub struct SourceMap {
    // rs_line -> ds_line
    mappings: Vec<usize>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { mappings: Vec::new() }
    }

    pub fn add_mapping(&mut self, rs_line: usize, ds_line: usize) {
        if rs_line >= self.mappings.len() {
            self.mappings.resize(rs_line + 1, 0);
        }
        self.mappings[rs_line] = ds_line;
    }

    pub fn get_ds_line(&self, rs_line: usize) -> Option<usize> {
        self.mappings.get(rs_line).copied()
    }
}
