use std::collections::HashSet;

const GUI_FRAMEWORKS: &[&str] = &[
    "tkinter",
    "Tkinter",
    "PyQt5",
    "PyQt6",
    "PySide2",
    "PySide6",
    "wx",
    "wxpython",
    "wxPython",
    "pygame",
    "matplotlib",
    "kivy",
    "arcade",
    "turtle",
    "PyGUI",
    "DearPyGui",
    "egg",
    "fbs",
    "appjar",
    "guizero",
    "pyglet",
];

#[derive(Debug, Clone, PartialEq)]
pub struct GuiWarning {
    pub line: usize,
    pub framework: String,
    pub import_stmt: String,
}

impl GuiWarning {
    pub fn new(line: usize, framework: &str, import_stmt: &str) -> Self {
        Self {
            line,
            framework: framework.to_string(),
            import_stmt: import_stmt.to_string(),
        }
    }

    pub fn message(&self) -> String {
        format!(
            "Warning: GUI framework '{}' detected at line {} - {}",
            self.framework, self.line, self.import_stmt
        )
    }
}

pub fn detect_gui_imports(_source: &str) -> Vec<GuiWarning> {
    Vec::new()
}
