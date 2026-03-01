use std::collections::HashSet;

use crate::ast::Expression;

pub const BUILTIN_TYPE_SYMBOLS: &[&str] = &[
    "i32", "i64", "f32", "f64", "bool", "Str", "List", "Dict", "Box",
];
pub const BUILTIN_VALUE_SYMBOLS: &[&str] = &["Some", "None", "Ok", "Err", "true", "false"];

pub struct Resolver {
    type_names: HashSet<String>,
    scopes: Vec<HashSet<String>>,
}

impl Resolver {
    pub fn new() -> Self {
        let mut type_names = HashSet::new();
        // Add built-in types
        for symbol in BUILTIN_TYPE_SYMBOLS {
            type_names.insert((*symbol).to_string());
        }

        Self {
            type_names,
            scopes: vec![HashSet::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    pub fn leave_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn declare_value(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    pub fn declare_type(&mut self, name: &str) {
        self.type_names.insert(name.to_string());
    }

    fn is_value(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }

    fn is_type_name(&self, name: &str) -> bool {
        self.type_names.contains(name)
    }

    pub fn is_static_receiver(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Ident(name) => self.is_type_name(name) && !self.is_value(name),
            Expression::MemberAccess(inner, _) => self.is_static_receiver(inner),
            _ => false,
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Resolver;
    use crate::ast::Expression;

    #[test]
    fn type_resolution_respects_value_shadowing() {
        let mut resolver = Resolver::new();
        resolver.declare_type("Path");
        assert!(resolver.is_static_receiver(&Expression::Ident("Path".to_string())));

        resolver.declare_value("Path");
        assert!(!resolver.is_static_receiver(&Expression::Ident("Path".to_string())));
    }

    #[test]
    fn value_shadowing_is_scoped() {
        let mut resolver = Resolver::new();
        resolver.declare_type("Path");
        resolver.enter_scope();
        resolver.declare_value("Path");
        assert!(!resolver.is_static_receiver(&Expression::Ident("Path".to_string())));

        resolver.leave_scope();
        assert!(resolver.is_static_receiver(&Expression::Ident("Path".to_string())));
    }

    #[test]
    fn uppercase_name_is_not_treated_as_type_without_declaration() {
        let resolver = Resolver::new();
        assert!(!resolver.is_static_receiver(&Expression::Ident("Path".to_string())));
    }
}
