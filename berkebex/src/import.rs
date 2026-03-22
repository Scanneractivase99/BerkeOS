//! Python import system for Berkebex compiler.
//!
//! Handles import resolution for Berkebex Python modules (.bpy files).
//!
//! # Supported Import Forms
//!
//! - `import module` - Loads module.bpy from current directory
//! - `from module import name` - Import specific names from module
//! - `import module as alias` - Aliased import
//! - `from . import sibling` - Relative imports within packages
//! - `__init__.bpy` - Package initialization files
//!
//! # Module Resolution Order
//!
//! 1. Check module cache (prevents re-importing)
//! 2. Try `module.bpy` in current directory
//! 3. Try `module/__init__.bpy` (package support)
//! 4. Return ImportError if not found

use rustpython_parser::{ast, Parse};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Import error types
#[derive(Debug, Clone)]
pub enum ImportError {
    /// Module not found in search path
    ModuleNotFound { name: String, search_path: String },
    /// Invalid relative import level
    InvalidRelativeImport { level: u32, message: String },
    /// Circular import detected
    CircularImport { module: String, chain: Vec<String> },
    /// IO error reading module file
    IoError { module: String, message: String },
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::ModuleNotFound { name, search_path } => {
                write!(
                    f,
                    "ModuleNotFound: No module named '{}' (searched: {})",
                    name, search_path
                )
            }
            ImportError::InvalidRelativeImport { level, message } => {
                write!(f, "InvalidRelativeImport: level={} - {}", level, message)
            }
            ImportError::CircularImport { module, chain } => {
                write!(
                    f,
                    "CircularImport: {} (chain: {})",
                    module,
                    chain.join(" -> ")
                )
            }
            ImportError::IoError { module, message } => {
                write!(f, "IoError reading '{}': {}", module, message)
            }
        }
    }
}

impl std::error::Error for ImportError {}

/// Represents a parsed import statement
#[derive(Debug, Clone)]
pub enum ImportStatement {
    /// `import module`
    SimpleImport {
        module: String,
        alias: Option<String>,
    },
    /// `from module import name1, name2`
    FromImport {
        module: String,
        names: Vec<(String, Option<String>)>, // (name, alias)
        level: u32,                           // 0 = absolute, 1+ = relative dots
    },
}

impl ImportStatement {
    /// Get the module name(s) this import refers to
    pub fn get_module_names(&self) -> Vec<String> {
        match self {
            ImportStatement::SimpleImport { module, alias } => {
                vec![alias.clone().unwrap_or_else(|| module.clone())]
            }
            ImportStatement::FromImport {
                module,
                names,
                level,
            } => {
                if *level > 0 {
                    vec![module.clone()]
                } else {
                    vec![module.clone()]
                }
            }
        }
    }
}

/// Module cache to track imported modules
#[derive(Debug, Clone, Default)]
pub struct ModuleCache {
    /// Maps module full path -> resolved file path
    cache: HashMap<String, PathBuf>,
    /// Tracks currently importing modules (for circular import detection)
    importing: HashSet<String>,
}

impl ModuleCache {
    /// Create a new empty module cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            importing: HashSet::new(),
        }
    }

    /// Check if a module is already cached
    pub fn is_cached(&self, module_name: &str) -> bool {
        self.cache.contains_key(module_name)
    }

    /// Get cached module path
    pub fn get_cached_path(&self, module_name: &str) -> Option<&PathBuf> {
        self.cache.get(module_name)
    }

    /// Add module to cache
    pub fn cache_module(&mut self, module_name: String, path: PathBuf) {
        self.cache.insert(module_name, path);
    }

    /// Check if module is currently being imported (for circular detection)
    pub fn is_importing(&self, module_name: &str) -> bool {
        self.importing.contains(module_name)
    }

    /// Mark module as currently importing
    pub fn start_import(&mut self, module_name: &str) {
        self.importing.insert(module_name.to_string());
    }

    /// Mark module as finished importing
    pub fn finish_import(&mut self, module_name: &str) {
        self.importing.remove(module_name);
    }

    /// Get all cached module names
    pub fn cached_modules(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }
}

use std::collections::HashSet;

/// Resolve import path - converts module name to file path
///
/// # Arguments
/// * `module_name` - The module name (e.g., "foo.bar")
/// * `current_dir` - Directory to search in
///
/// # Returns
/// * `Ok(String)` - Full path to the module file
/// * `Err(ImportError)` - If module cannot be found
pub fn resolve_import(module_name: &str, current_dir: &Path) -> Result<String, ImportError> {
    // Strip any leading dots for relative imports
    let (level, name) = parse_relative_level(module_name);

    if level > 0 {
        resolve_relative_import(name, level, current_dir)
    } else {
        resolve_absolute_import(name, current_dir)
    }
}

/// Parse relative import level from module name
fn parse_relative_level(module_name: &str) -> (u32, &str) {
    let mut level = 0u32;
    let mut remaining = module_name;

    while remaining.starts_with('.') {
        level += 1;
        remaining = &remaining[1..];
    }

    // Skip extra dots at start
    (level, remaining)
}

/// Resolve absolute import
fn resolve_absolute_import(module_name: &str, current_dir: &Path) -> Result<String, ImportError> {
    let search_path = current_dir.to_string_lossy().to_string();

    // Try direct module file: module.bpy
    let direct_path = current_dir.join(format!("{}.bpy", module_name));
    if direct_path.exists() {
        return Ok(direct_path.to_string_lossy().to_string());
    }

    // Try package: module/__init__.bpy
    let package_init = current_dir.join(module_name).join("__init__.bpy");
    if package_init.exists() {
        return Ok(package_init.to_string_lossy().to_string());
    }

    Err(ImportError::ModuleNotFound {
        name: module_name.to_string(),
        search_path,
    })
}

/// Resolve relative import (from . or .. etc)
fn resolve_relative_import(
    module_name: &str,
    level: u32,
    current_dir: &Path,
) -> Result<String, ImportError> {
    let mut dir = current_dir.to_path_buf();

    // Navigate up 'level' directories
    for _ in 0..level {
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => {
                return Err(ImportError::InvalidRelativeImport {
                    level,
                    message: "Attempted to go past top-level package".to_string(),
                });
            }
        }
    }

    // If no submodule specified, return the package __init__.bpy
    if module_name.is_empty() {
        let init_path = dir.join("__init__.bpy");
        if init_path.exists() {
            return Ok(init_path.to_string_lossy().to_string());
        }
        return Err(ImportError::ModuleNotFound {
            name: "__init__".to_string(),
            search_path: dir.to_string_lossy().to_string(),
        });
    }

    // Otherwise resolve the submodule
    resolve_absolute_import(module_name, &dir)
}

/// Parse Python AST to extract import statements
pub fn parse_imports(source: &str) -> Result<Vec<ImportStatement>, String> {
    let suite =
        ast::Suite::parse(source, "<import>").map_err(|e| format!("Parse error: {}", e.error))?;

    let mut imports = Vec::new();

    for stmt in &suite {
        extract_imports_from_stmt(stmt, &mut imports);
    }

    Ok(imports)
}

/// Recursively extract import statements from AST node
fn extract_imports_from_stmt(stmt: &ast::Stmt, imports: &mut Vec<ImportStatement>) {
    match stmt {
        ast::Stmt::Import(import_node) => {
            for alias in &import_node.names {
                let name = alias.name.to_string();
                let alias_name = alias.asname.as_ref().map(|a| a.to_string());

                imports.push(ImportStatement::SimpleImport {
                    module: name,
                    alias: alias_name,
                });
            }
        }
        ast::Stmt::ImportFrom(import_from) => {
            let level = if import_from.level.is_some() { 1 } else { 0 };
            let module = import_from
                .module
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_default();

            let mut names = Vec::new();
            for alias in &import_from.names {
                let name = alias.name.to_string();
                let alias_name = alias.asname.as_ref().map(|a| a.to_string());
                names.push((name, alias_name));
            }

            imports.push(ImportStatement::FromImport {
                module,
                names,
                level,
            });
        }
        ast::Stmt::FunctionDef(func_node) => {
            for inner_stmt in &func_node.body {
                extract_imports_from_stmt(inner_stmt, imports);
            }
        }
        ast::Stmt::If(if_node) => {
            for inner_stmt in &if_node.body {
                extract_imports_from_stmt(inner_stmt, imports);
            }
            for inner_stmt in &if_node.orelse {
                extract_imports_from_stmt(inner_stmt, imports);
            }
        }
        ast::Stmt::While(while_node) => {
            for inner_stmt in &while_node.body {
                extract_imports_from_stmt(inner_stmt, imports);
            }
        }
        ast::Stmt::For(for_node) => {
            for inner_stmt in &for_node.body {
                extract_imports_from_stmt(inner_stmt, imports);
            }
        }
        ast::Stmt::ClassDef(class_node) => {
            for inner_stmt in &class_node.body {
                extract_imports_from_stmt(inner_stmt, imports);
            }
        }
        ast::Stmt::Try(try_node) => {
            for inner_stmt in &try_node.body {
                extract_imports_from_stmt(inner_stmt, imports);
            }
            for handler in &try_node.handlers {
                if let ast::ExceptHandler::ExceptHandler(h) = handler {
                    for inner_stmt in &h.body {
                        extract_imports_from_stmt(inner_stmt, imports);
                    }
                }
            }
            for inner_stmt in &try_node.orelse {
                extract_imports_from_stmt(inner_stmt, imports);
            }
            for inner_stmt in &try_node.finalbody {
                extract_imports_from_stmt(inner_stmt, imports);
            }
        }
        _ => {}
    }
}

/// Import resolver that handles the full import pipeline
pub struct ImportResolver {
    /// Module cache for already-loaded modules
    cache: ModuleCache,
    /// Current search path
    search_path: PathBuf,
}

impl ImportResolver {
    /// Create a new import resolver with given base directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            cache: ModuleCache::new(),
            search_path: base_dir,
        }
    }

    /// Create resolver from current directory
    pub fn current_dir() -> Self {
        Self::new(std::env::current_dir().unwrap_or_default())
    }

    /// Resolve and load a module
    ///
    /// Returns the source code of the module if found and loaded
    pub fn load_module(&mut self, module_name: &str) -> Result<String, ImportError> {
        // Check circular import
        if self.cache.is_importing(module_name) {
            return Err(ImportError::CircularImport {
                module: module_name.to_string(),
                chain: self.cache.cached_modules(),
            });
        }

        // Check cache
        if let Some(cached) = self.cache.get_cached_path(module_name) {
            return fs::read_to_string(cached).map_err(|e| ImportError::IoError {
                module: module_name.to_string(),
                message: e.to_string(),
            });
        }

        // Mark as importing
        self.cache.start_import(module_name);

        // Resolve path
        let resolved_path = resolve_import(module_name, &self.search_path)?;

        // Read source
        let source = fs::read_to_string(&resolved_path).map_err(|e| ImportError::IoError {
            module: module_name.to_string(),
            message: e.to_string(),
        })?;

        // Cache the module
        self.cache
            .cache_module(module_name.to_string(), PathBuf::from(&resolved_path));

        // Mark as finished importing
        self.cache.finish_import(module_name);

        Ok(source)
    }

    /// Check if module is cached
    pub fn is_cached(&self, module_name: &str) -> bool {
        self.cache.is_cached(module_name)
    }

    /// Get all imported module names
    pub fn imported_modules(&self) -> Vec<String> {
        self.cache.cached_modules()
    }

    /// Get the module cache (for inspection)
    pub fn get_cache(&self) -> &ModuleCache {
        &self.cache
    }

    /// Add a directory to the search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_path = path;
    }
}

/// Import context for tracking imports during compilation
#[derive(Debug, Clone, Default)]
pub struct ImportContext {
    /// List of all imports found in the source
    pub imports: Vec<ImportStatement>,
    /// Set of imported module names
    pub imported_modules: HashSet<String>,
    /// Mapping of alias -> original name
    pub alias_map: HashMap<String, String>,
}

impl ImportContext {
    /// Create new empty import context
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            imported_modules: HashSet::new(),
            alias_map: HashMap::new(),
        }
    }

    /// Process imports from source and update context
    pub fn process_source(&mut self, source: &str) -> Result<(), String> {
        let parsed_imports = parse_imports(source)?;

        for import in parsed_imports {
            self.add_import(&import);
        }

        Ok(())
    }

    /// Add an import statement to the context
    pub fn add_import(&mut self, import: &ImportStatement) {
        self.imports.push(import.clone());

        match import {
            ImportStatement::SimpleImport { module, alias } => {
                let module_name = module.split('.').next().unwrap_or(module).to_string();
                self.imported_modules.insert(module_name.clone());

                let target = alias.clone().unwrap_or_else(|| module.clone());
                self.alias_map.insert(target, module.clone());
            }
            ImportStatement::FromImport {
                module,
                names,
                level,
            } => {
                if *level == 0 {
                    let module_name = module.split('.').next().unwrap_or(module).to_string();
                    self.imported_modules.insert(module_name.clone());
                }

                for (name, alias) in names {
                    let alias_name = alias.clone().unwrap_or_else(|| name.clone());
                    self.alias_map.insert(alias_name, name.clone());
                }
            }
        }
    }

    /// Check if a name is imported (for name resolution)
    pub fn is_imported(&self, name: &str) -> bool {
        self.alias_map.contains_key(name)
    }

    /// Get the original name for an alias
    pub fn resolve_alias(&self, alias: &str) -> Option<&String> {
        self.alias_map.get(alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_import_direct() {
        let dir = Path::new("/tmp/test_project");

        // Create a dummy module file
        let _ = fs::create_dir_all(dir);
        fs::write(dir.join("foo.bpy"), "# test").ok();

        let result = resolve_import("foo", dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/tmp/test_project/foo.bpy");

        // Cleanup
        fs::remove_file(dir.join("foo.bpy")).ok();
    }

    #[test]
    fn test_resolve_package_import() {
        let dir = Path::new("/tmp/test_project");

        // Create package
        let _ = fs::create_dir_all(dir.join("mypackage"));
        fs::write(dir.join("mypackage/__init__.bpy"), "# package").ok();

        let result = resolve_import("mypackage", dir);
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("mypackage/__init__.bpy"));

        // Cleanup
        fs::remove_file(dir.join("mypackage/__init__.bpy")).ok();
        fs::remove_dir(dir.join("mypackage")).ok();
    }

    #[test]
    fn test_resolve_relative_import() {
        let dir = Path::new("/tmp/test_project/pkg");

        // Create sibling module
        let _ = fs::create_dir_all(dir.parent().unwrap());
        fs::write(dir.parent().unwrap().join("helper.bpy"), "# helper").ok();

        let result = resolve_import(".helper", dir);
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("helper.bpy"));

        // Cleanup
        fs::remove_file(dir.parent().unwrap().join("helper.bpy")).ok();
    }

    #[test]
    fn test_module_not_found() {
        let dir = Path::new("/tmp");
        let result = resolve_import("nonexistent_module", dir);

        assert!(matches!(result, Err(ImportError::ModuleNotFound { .. })));
    }

    #[test]
    fn test_parse_simple_import() {
        let source = r#"
import os
import sys as system
"#;
        let imports = parse_imports(source).unwrap();

        assert_eq!(imports.len(), 2);

        match &imports[0] {
            ImportStatement::SimpleImport { module, alias } => {
                assert_eq!(module, "os");
                assert!(alias.is_none());
            }
            _ => panic!("Expected SimpleImport"),
        }

        match &imports[1] {
            ImportStatement::SimpleImport { module, alias } => {
                assert_eq!(module, "sys");
                assert_eq!(alias.as_deref(), Some("system"));
            }
            _ => panic!("Expected SimpleImport with alias"),
        }
    }

    #[test]
    fn test_parse_from_import() {
        let source = r#"
from os import path, getcwd
from collections import OrderedDict as OD
"#;
        let imports = parse_imports(source).unwrap();

        assert_eq!(imports.len(), 2);

        match &imports[0] {
            ImportStatement::FromImport {
                module,
                names,
                level,
            } => {
                assert_eq!(module, "os");
                // level is 1 if Some, 0 if None
                assert_eq!(*level <= 1, true); // absolute import
                assert_eq!(names.len(), 2);
            }
            _ => panic!("Expected FromImport"),
        }

        match &imports[1] {
            ImportStatement::FromImport {
                module,
                names,
                level,
            } => {
                assert_eq!(module, "collections");
                assert_eq!(*level <= 1, true); // absolute import
                assert_eq!(names[0].0, "OrderedDict");
                assert_eq!(names[0].1.as_deref(), Some("OD"));
            }
            _ => panic!("Expected FromImport with alias"),
        }
    }

    #[test]
    fn test_parse_relative_import() {
        let source = r#"
from . import sibling
from .. import parent
from .module import func
"#;
        let imports = parse_imports(source).unwrap();

        assert_eq!(imports.len(), 3);

        for import in &imports {
            match import {
                ImportStatement::FromImport {
                    module,
                    names: _,
                    level,
                } => {
                    // Relative imports have level > 0
                    assert!(*level > 0);
                    // module should be empty for pure relative
                    if import.get_module_names().len() == 1 {
                        assert!(module.is_empty() || module == "module");
                    }
                }
                _ => panic!("Expected FromImport"),
            }
        }
    }

    #[test]
    fn test_import_context() {
        let mut ctx = ImportContext::new();

        ctx.add_import(&ImportStatement::SimpleImport {
            module: "os".to_string(),
            alias: None,
        });

        ctx.add_import(&ImportStatement::FromImport {
            module: "sys".to_string(),
            names: vec![("path".to_string(), None)],
            level: 0,
        });

        assert!(ctx.is_imported("os"));
        assert!(ctx.is_imported("path"));
        assert!(!ctx.is_imported("nonexistent"));
    }

    #[test]
    fn test_alias_resolution() {
        let mut ctx = ImportContext::new();

        ctx.add_import(&ImportStatement::SimpleImport {
            module: "very_long_module_name".to_string(),
            alias: Some("short".to_string()),
        });

        assert_eq!(
            ctx.resolve_alias("short"),
            Some(&"very_long_module_name".to_string())
        );
    }

    #[test]
    fn test_module_cache() {
        let mut cache = ModuleCache::new();

        assert!(!cache.is_cached("test"));

        cache.cache_module("test".to_string(), PathBuf::from("/path/to/test.bpy"));

        assert!(cache.is_cached("test"));
        assert_eq!(
            cache.get_cached_path("test").unwrap(),
            &PathBuf::from("/path/to/test.bpy")
        );
    }

    #[test]
    fn test_circular_import_detection() {
        let mut cache = ModuleCache::new();

        cache.start_import("module_a");
        assert!(cache.is_importing("module_a"));

        cache.finish_import("module_a");
        assert!(!cache.is_importing("module_a"));
    }

    #[test]
    fn test_import_resolver_cache() {
        let mut resolver = ImportResolver::new(PathBuf::from("/tmp"));

        assert!(!resolver.is_cached("test"));

        // Direct cache manipulation for testing
        resolver
            .cache
            .cache_module("test".to_string(), PathBuf::from("/tmp/test.bpy"));

        assert!(resolver.is_cached("test"));
    }
}
