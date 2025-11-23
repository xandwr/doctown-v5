//! Language detection for source files.

use doctown_common::Language;
use std::path::Path;

/// Detects language from a file path using extension and shebang.
pub fn detect_language(path: &Path, content: Option<&str>) -> Option<Language> {
    // First try extension
    if let Some(lang) = Language::from_path(path) {
        return Some(lang);
    }

    // Then try shebang
    if let Some(content) = content {
        if let Some(lang) = detect_from_shebang(content) {
            return Some(lang);
        }
    }

    None
}

/// Detects language from shebang line.
fn detect_from_shebang(content: &str) -> Option<Language> {
    let first_line = content.lines().next()?;

    if !first_line.starts_with("#!") {
        return None;
    }

    let shebang = first_line.trim_start_matches("#!");

    // Handle /usr/bin/env style shebangs
    let interpreter = if shebang.contains("env ") {
        shebang.split_whitespace().nth(1)?
    } else {
        shebang.split('/').next_back()?.split_whitespace().next()?
    };

    match interpreter {
        "python" | "python3" | "python2" => Some(Language::Python),
        "node" | "nodejs" => Some(Language::JavaScript),
        "deno" | "ts-node" => Some(Language::TypeScript),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_from_extension() {
        let path = PathBuf::from("src/main.rs");
        assert_eq!(detect_language(&path, None), Some(Language::Rust));

        let path = PathBuf::from("script.py");
        assert_eq!(detect_language(&path, None), Some(Language::Python));
    }

    #[test]
    fn test_detect_from_shebang_python() {
        let path = PathBuf::from("script");
        let content = "#!/usr/bin/env python3\nprint('hello')";
        assert_eq!(
            detect_language(&path, Some(content)),
            Some(Language::Python)
        );
    }

    #[test]
    fn test_detect_from_shebang_python_direct() {
        let path = PathBuf::from("script");
        let content = "#!/usr/bin/python\nprint('hello')";
        assert_eq!(
            detect_language(&path, Some(content)),
            Some(Language::Python)
        );
    }

    #[test]
    fn test_detect_from_shebang_node() {
        let path = PathBuf::from("script");
        let content = "#!/usr/bin/env node\nconsole.log('hello')";
        assert_eq!(
            detect_language(&path, Some(content)),
            Some(Language::JavaScript)
        );
    }

    #[test]
    fn test_detect_from_shebang_deno() {
        let path = PathBuf::from("script");
        let content = "#!/usr/bin/env deno\nconsole.log('hello')";
        assert_eq!(
            detect_language(&path, Some(content)),
            Some(Language::TypeScript)
        );
    }

    #[test]
    fn test_no_detection() {
        let path = PathBuf::from("Makefile");
        assert_eq!(detect_language(&path, None), None);
    }

    #[test]
    fn test_extension_takes_precedence() {
        let path = PathBuf::from("script.rs");
        let content = "#!/usr/bin/env python3\nfn main() {}";
        // Extension should win over shebang
        assert_eq!(detect_language(&path, Some(content)), Some(Language::Rust));
    }
}
