//! Python parser module using rustpython-parser
//!
//! This module provides Python syntax parsing using the rustpython-parser crate.
//! It only imports the parser, not the full RustPython VM.

use rustpython_parser::{ast, Parse};

/// Parse error with location information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(line), Some(col)) = (self.line, self.column) {
            write!(f, "SyntaxError at {}:{}: {}", line, col, self.message)
        } else {
            write!(f, "SyntaxError: {}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a Python source string and return the AST suite (list of statements).
///
/// # Arguments
/// * `source` - The Python source code to parse
///
/// # Returns
/// * `Ok(ast::Suite)` - The parsed AST suite on success
/// * `Err(ParseError)` - A parse error with location info on failure
pub fn parse_python(source: &str) -> Result<ast::Suite, ParseError> {
    match ast::Suite::parse(source, "<input>") {
        Ok(suite) => Ok(suite),
        Err(e) => Err(ParseError {
            message: e.error.to_string(),
            line: None,
            column: None,
        }),
    }
}

/// Check if Python source is syntactically valid.
///
/// # Arguments
/// * `source` - The Python source code to check
///
/// # Returns
/// * `true` if the source is valid Python syntax
/// * `false` if there are syntax errors
pub fn is_valid_python(source: &str) -> bool {
    parse_python(source).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_assignment() {
        let result = parse_python("x = 1");
        assert!(result.is_ok());
        let suite = result.unwrap();
        assert!(!suite.is_empty());
    }

    #[test]
    fn test_parse_expression() {
        let result = parse_python("x = 1 + 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_multiple_statements() {
        let result = parse_python("x = 1\ny = 2\nz = x + y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function_def() {
        let result = parse_python("def add(a, b):\n    return a + b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let result = parse_python("x = ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_paren() {
        let result = parse_python("x = (1 + 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_python() {
        assert!(is_valid_python("x = 1"));
        assert!(!is_valid_python("x = "));
    }

    #[test]
    fn test_parse_error_contains_message() {
        let result = parse_python("x = ");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.message.is_empty());
    }
}
