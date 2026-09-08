//! T012 - Unit tests for ImportPath construction from file structure.

use pydoc_gen::model::ImportPath;
use pydoc_gen::path::module_path_from_relative;

#[test]
fn path_from_nested_module() {
    let rel = std::path::Path::new("src/pkg/sub/mod.py");
    let segments = module_path_from_relative(rel).expect("module");
    assert_eq!(ImportPath::new(segments).text(), "src.pkg.sub.mod");
}

#[test]
fn path_from_init_is_package() {
    let rel = std::path::Path::new("src/pkg/sub/__init__.py");
    let segments = module_path_from_relative(rel).expect("package");
    assert_eq!(ImportPath::new(segments).text(), "src.pkg.sub");
}

#[test]
fn child_appends_segment() {
    let parent = ImportPath::new(vec!["pkg".into(), "mod".into()]);
    assert_eq!(parent.child("Class").text(), "pkg.mod.Class");
    assert_eq!(parent.child("Class").name(), "Class");
}

#[test]
fn full_path_uniquely_identifies_same_names() {
    // SC-006: two same-named elements in different modules stay distinct.
    let a = ImportPath::new(vec!["pkg".into(), "a".into(), "Helper".into()]);
    let b = ImportPath::new(vec!["pkg".into(), "b".into(), "Helper".into()]);
    assert_ne!(a, b);
    assert_eq!(a.text(), "pkg.a.Helper");
    assert_eq!(b.text(), "pkg.b.Helper");
}
