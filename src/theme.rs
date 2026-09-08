//! Theme resolution per `data-model.md` (FR-012, FR-013).
//!
//! A `Theme` is the complete stylesheet text embedded into every generated
//! page. The default theme comes from the bundled `assets/default.css`;
//! a user-provided stylesheet (via `--theme`) is appended after it so its
//! rules take precedence through the CSS cascade (research D-004).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Where the theme's CSS came from (for diagnostics).
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeSource {
    /// The built-in hand-written theme authored in `assets/default.css`,
    /// bundled into the tool (FR-013).
    Default,
    /// A user-provided stylesheet read from `path` at build time.
    Custom { path: PathBuf },
}

/// The complete stylesheet text embedded into every generated page.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Full CSS text for the `<style>` block (default + optional user CSS).
    pub css: String,
    /// Where the CSS came from.
    pub source: ThemeSource,
}

impl Theme {
    /// The bundled default theme (light + dark) with no user overrides.
    pub fn default_theme() -> Self {
        Theme {
            css: crate::style::default_theme_css().to_string(),
            source: ThemeSource::Default,
        }
    }
}

/// Resolve the theme for a run.
///
/// - `None` → the bundled default theme.
/// - `Some(path)` → the default theme with the user stylesheet's bytes
///   appended (empty/whitespace-only files are allowed and degrade
///   gracefully to the default theme). A missing/unreadable file is a
///   fatal `Error::ReadTheme` (contracts/cli.md).
pub fn resolve_theme(theme_path: Option<&Path>) -> Result<Theme> {
    match theme_path {
        None => Ok(Theme::default_theme()),
        Some(path) => {
            let user_css = fs::read_to_string(path).map_err(|e| {
                Error::ReadTheme(path.display().to_string(), e)
            })?;
            let mut css = crate::style::default_theme_css().to_string();
            css.push('\n');
            css.push_str(&user_css);
            Ok(Theme {
                css,
                source: ThemeSource::Custom {
                    path: path.to_path_buf(),
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn no_theme_uses_bundled_default() {
        let theme = resolve_theme(None).expect("default resolves");
        assert_eq!(theme.source, ThemeSource::Default);
        assert_eq!(theme.css, crate::style::default_theme_css());
    }

    #[test]
    fn custom_theme_is_appended_after_default() {
        let dir = std::env::temp_dir().join(format!(
            "pydoc_theme_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("user.css");
        std::fs::write(&path, "/* marker */ .x { color: red; }\n").unwrap();

        let theme = resolve_theme(Some(&path)).expect("custom resolves");
        assert_eq!(
            theme.source,
            ThemeSource::Custom {
                path: path.clone()
            }
        );
        assert_eq!(
            theme.css,
            format!(
                "{}\n/* marker */ .x {{ color: red; }}\n",
                crate::style::default_theme_css()
            )
        );
        // User css must come after the default css.
        let default_at = theme.css.find(":root").unwrap();
        let user_at = theme.css.find("/* marker */").unwrap();
        assert!(default_at < user_at, "user css must follow default");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_custom_theme_is_allowed() {
        let dir = std::env::temp_dir().join(format!(
            "pydoc_theme_empty_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.css");
        std::fs::write(&path, "   \n").unwrap();

        let theme = resolve_theme(Some(&path)).expect("empty resolves");
        assert_eq!(theme.css, crate::style::default_theme_css().to_owned() + "\n   \n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_custom_theme_is_fatal_error() {
        let err = resolve_theme(Some(Path::new("/nonexistent/theme.css")))
            .expect_err("missing theme file must error");
        assert!(matches!(err, Error::ReadTheme(_, _)));
    }

    #[test]
    fn default_theme_css_is_consistent() {
        let theme = Theme::default_theme();
        assert!(theme.css.contains("prefers-color-scheme: dark"));
        assert!(theme.css.contains("--color-text"));
    }
}