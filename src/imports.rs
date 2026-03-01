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

#[cfg(test)]
mod tests {
    use super::{is_supported_rust_root, rust_import_binding_name, rust_use_from_import_path};

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
        assert_eq!(
            rust_import_binding_name("std::cmp::max"),
            Some("max")
        );
    }

    #[test]
    fn rust_import_root_is_whitelisted() {
        assert!(is_supported_rust_root("std::cmp::max"));
        assert!(!is_supported_rust_root("serde::Serialize"));
    }
}
