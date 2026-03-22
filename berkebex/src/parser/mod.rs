//! Parser modules for berkebex
//!
//! Provides parsers for different input languages:
//! - Python: Uses rustpython-parser crate

pub mod python;

pub use python::{is_valid_python, parse_python, ParseError};
