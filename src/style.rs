//! Styling: bundles and emits the authored default theme (FR-010, FR-013).
//!
//! The default theme is maintained as a real CSS file, `assets/default.css`
//! (authored with normal CSS tooling), and compiled into the tool at build
//! time. The generated pages embed the theme inline so the output stays
//! self-contained and offline-capable.

/// The authored default-theme stylesheet, bundled at compile time (FR-013).
pub fn default_theme_css() -> &'static str {
    include_str!("../assets/default.css")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_non_empty() {
        assert!(
            !default_theme_css().trim().is_empty(),
            "the bundled default theme must not be empty"
        );
    }

    #[test]
    fn bundled_file_is_the_authoring_source() {
        // FR-013: the theme lives in a real CSS file, not a string literal.
        let from_cargo = include_str!("../assets/default.css");
        assert_eq!(default_theme_css(), from_cargo);
    }
}