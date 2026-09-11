//! pydoc-gen: Generate HTML documentation for Python source from
//! YAML docstrings.
//!
//! The three-stage pipeline is:
//! 1. Parse Python source into an AST using `rustpython_parser`.
//! 2. Extract YAML-structured docstrings into an intermediary representation.
//! 3. Generate a navigable HTML site (index + per-module pages) from that
//!    representation.

pub mod cli;
pub mod docstring;
pub mod error;
pub mod extract;
pub mod index;
pub mod model;
pub mod parser;
pub mod path;
pub mod pipeline;
pub mod style;
pub mod theme;
