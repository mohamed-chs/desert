use std::collections::HashSet;

pub struct Resolver {
    types: HashSet<String>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut types = HashSet::new();
        // Add built-in types
        types.insert("i32".to_string());
        types.insert("i64".to_string());
        types.insert("f32".to_string());
        types.insert("f64".to_string());
        types.insert("bool".to_string());
        types.insert("Str".to_string());
        types.insert("List".to_string());
        types.insert("Dict".to_string());

        Self { types }
    }

    pub fn is_type(&self, name: &str) -> bool {
        self.types.contains(name) || name.chars().next().is_some_and(|c| c.is_uppercase())
    }

    // This is a simplified resolution strategy:
    // If the left side of a dot starts with an uppercase letter or is a known type, it's a static call.
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
