use crate::ast::ImportItem;

pub fn rust_use_from_import_path(path: &str) -> Option<String> {
    let raw = if let Some(rest) = path.strip_prefix("rust/") {
        rest
    } else if let Some(rest) = path.strip_prefix("rust:") {
        rest
    } else {
        return None;
    };

    let normalized = raw.replace('/', "::");
    let trimmed = normalized.trim_matches(':').to_string();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed)
}

pub fn rust_import_binding_name(use_path: &str) -> Option<&str> {
    if let Some((_, alias)) = use_path.rsplit_once(" as ") {
        if !alias.is_empty() && alias != "*" {
            return Some(alias);
        }
    }
    let name = use_path.rsplit("::").next().unwrap_or(use_path);
    if name.is_empty() || name == "*" {
        return None;
    }
    Some(name)
}

pub fn is_supported_rust_root(use_path: &str) -> bool {
    let root = use_path.split("::").next().unwrap_or(use_path);
    matches!(root, "std" | "core" | "alloc")
}

pub fn rust_use_from_import(path: &str, alias: Option<&str>) -> Option<String> {
    let base = rust_use_from_import_path(path)?;
    if let Some(alias) = alias {
        return Some(format!("{base} as {alias}"));
    }
    Some(base)
}

pub fn rust_use_from_from_import(path: &str, items: &[ImportItem]) -> Option<String> {
    let base = rust_use_from_import_path(path)?;
    let rendered = items
        .iter()
        .map(|item| match &item.alias {
            Some(alias) => format!("{} as {}", item.name, alias),
            None => item.name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("{base}::{{{rendered}}}"))
}

#[cfg(test)]
mod tests {
    use super::{
        is_supported_rust_root, rust_import_binding_name, rust_use_from_from_import,
        rust_use_from_import, rust_use_from_import_path,
    };
    use crate::ast::ImportItem;

    #[test]
    fn rust_import_from_dotted_path() {
        assert_eq!(
            rust_use_from_import_path("rust/std/cmp/max"),
            Some("std::cmp::max".to_string())
        );
    }

    #[test]
    fn rust_import_from_string_path() {
        assert_eq!(
            rust_use_from_import_path("rust:std::io::Read"),
            Some("std::io::Read".to_string())
        );
    }

    #[test]
    fn rust_import_binding_extracts_leaf() {
        assert_eq!(rust_import_binding_name("std::cmp::max"), Some("max"));
        assert_eq!(
            rust_import_binding_name("std::collections::HashMap as Map"),
            Some("Map")
        );
    }

    #[test]
    fn rust_import_root_is_whitelisted() {
        assert!(is_supported_rust_root("std::cmp::max"));
        assert!(!is_supported_rust_root("serde::Serialize"));
    }

    #[test]
    fn rust_use_with_alias() {
        assert_eq!(
            rust_use_from_import("rust/std/collections/HashMap", Some("Map")),
            Some("std::collections::HashMap as Map".to_string())
        );
    }

    #[test]
    fn rust_use_from_list() {
        let items = vec![
            ImportItem {
                name: "max".to_string(),
                alias: Some("maximum".to_string()),
            },
            ImportItem {
                name: "min".to_string(),
                alias: None,
            },
        ];
        assert_eq!(
            rust_use_from_from_import("rust/std/cmp", &items),
            Some("std::cmp::{max as maximum, min}".to_string())
        );
    }
}
