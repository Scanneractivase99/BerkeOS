//! GUI Framework Runtime Hooks for BerkeBex
//!
//! This module provides runtime hooks that detect when a program attempts to
//! initialize GUI frameworks. When detected, it emits a special bytecode
//! instruction that causes BerkeOS to show a popup warning.
//!
//! # Hooked GUI Frameworks
//! - tkinter: `tkinter.Tk()`, `tkinter.Toplevel()`
//! - pygame: `pygame.init()`, `pygame.display.set_mode()`
//! - matplotlib: `matplotlib.pyplot.show()`, `FigureCanvasTkAgg`
//! - Qt: `QApplication([])`, `QWidget()`
//!
//! # Usage
//!
//! The hooks are enabled by default. To disable:
//! ```bash
//! berkebex compile --no-gui-guard program.py
//! ```

/// GUI framework hook registry
#[derive(Debug, Clone)]
pub struct GuiHookRegistry {
    /// Whether GUI guards are enabled
    pub enabled: bool,
    /// List of hooked function patterns
    pub hooks: Vec<GuiHook>,
}

#[derive(Debug, Clone)]
pub struct GuiHook {
    /// Module name (e.g., "tkinter", "pygame")
    pub module: String,
    /// Function/Class name (e.g., "Tk", "init")
    pub name: String,
    /// Optional warning message override
    pub warning: Option<String>,
    /// Syscall number to emit when hooked
    pub syscall_id: u64,
}

impl Default for GuiHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiHookRegistry {
    /// Create a new GUI hook registry with default hooks
    pub fn new() -> Self {
        Self {
            enabled: true,
            hooks: Self::default_hooks(),
        }
    }

    /// Create a registry with GUI guards disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            hooks: Vec::new(),
        }
    }

    /// Default GUI hooks for common frameworks
    fn default_hooks() -> Vec<GuiHook> {
        vec![
            // Tkinter hooks
            GuiHook {
                module: "tkinter".to_string(),
                name: "Tk".to_string(),
                warning: Some("Tkinter GUI initialization detected".to_string()),
                syscall_id: 70,
            },
            GuiHook {
                module: "tkinter".to_string(),
                name: "Toplevel".to_string(),
                warning: Some("Tkinter Toplevel window created".to_string()),
                syscall_id: 70,
            },
            GuiHook {
                module: "tkinter".to_string(),
                name: "Frame".to_string(),
                warning: Some("Tkinter Frame created".to_string()),
                syscall_id: 70,
            },
            GuiHook {
                module: "tkinter".to_string(),
                name: "Canvas".to_string(),
                warning: Some("Tkinter Canvas created".to_string()),
                syscall_id: 70,
            },
            // Pygame hooks
            GuiHook {
                module: "pygame".to_string(),
                name: "init".to_string(),
                warning: Some("Pygame initialization detected".to_string()),
                syscall_id: 71,
            },
            GuiHook {
                module: "pygame".to_string(),
                name: "display".to_string(),
                warning: Some("Pygame display module accessed".to_string()),
                syscall_id: 71,
            },
            GuiHook {
                module: "pygame".to_string(),
                name: "set_mode".to_string(),
                warning: Some("Pygame display mode set".to_string()),
                syscall_id: 71,
            },
            GuiHook {
                module: "pygame".to_string(),
                name: "SCREEN".to_string(),
                warning: Some("Pygame screen created".to_string()),
                syscall_id: 71,
            },
            // Matplotlib hooks
            GuiHook {
                module: "matplotlib".to_string(),
                name: "pyplot".to_string(),
                warning: Some("Matplotlib pyplot accessed".to_string()),
                syscall_id: 72,
            },
            GuiHook {
                module: "matplotlib.pyplot".to_string(),
                name: "show".to_string(),
                warning: Some("Matplotlib plt.show() called - GUI window will open".to_string()),
                syscall_id: 72,
            },
            GuiHook {
                module: "matplotlib".to_string(),
                name: "figure".to_string(),
                warning: Some("Matplotlib figure created".to_string()),
                syscall_id: 72,
            },
            GuiHook {
                module: "matplotlib".to_string(),
                name: "FigureCanvasTkAgg".to_string(),
                warning: Some("Matplotlib Tkinter backend initialized".to_string()),
                syscall_id: 72,
            },
            // Qt/PyQt/PySide hooks
            GuiHook {
                module: "PyQt5".to_string(),
                name: "QApplication".to_string(),
                warning: Some("PyQt5 QApplication created".to_string()),
                syscall_id: 73,
            },
            GuiHook {
                module: "PyQt5.QtWidgets".to_string(),
                name: "QApplication".to_string(),
                warning: Some("PyQt5 QApplication created".to_string()),
                syscall_id: 73,
            },
            GuiHook {
                module: "PyQt5.QtWidgets".to_string(),
                name: "QWidget".to_string(),
                warning: Some("PyQt5 QWidget created".to_string()),
                syscall_id: 73,
            },
            GuiHook {
                module: "PySide2".to_string(),
                name: "QApplication".to_string(),
                warning: Some("PySide2 QApplication created".to_string()),
                syscall_id: 73,
            },
            GuiHook {
                module: "PySide2.QtWidgets".to_string(),
                name: "QApplication".to_string(),
                warning: Some("PySide2 QApplication created".to_string()),
                syscall_id: 73,
            },
            GuiHook {
                module: "PySide6".to_string(),
                name: "QApplication".to_string(),
                warning: Some("PySide6 QApplication created".to_string()),
                syscall_id: 73,
            },
            GuiHook {
                module: "PySide6.QtWidgets".to_string(),
                name: "QApplication".to_string(),
                warning: Some("PySide6 QApplication created".to_string()),
                syscall_id: 73,
            },
            // Turtle graphics (common in educational Python)
            GuiHook {
                module: "turtle".to_string(),
                name: "Screen".to_string(),
                warning: Some("Turtle graphics Screen() called".to_string()),
                syscall_id: 74,
            },
            GuiHook {
                module: "turtle".to_string(),
                name: "Pen".to_string(),
                warning: Some("Turtle Pen() created".to_string()),
                syscall_id: 74,
            },
            // GUIZero hooks
            GuiHook {
                module: "guizero".to_string(),
                name: "App".to_string(),
                warning: Some("Guizero App created".to_string()),
                syscall_id: 75,
            },
            GuiHook {
                module: "guizero".to_string(),
                name: "Window".to_string(),
                warning: Some("Guizero Window created".to_string()),
                syscall_id: 75,
            },
            // DearPyGui hooks
            GuiHook {
                module: " dearpygui".to_string(),
                name: "start_dearpygui".to_string(),
                warning: Some("DearPyGui started".to_string()),
                syscall_id: 76,
            },
            GuiHook {
                module: " dearpygui".to_string(),
                name: "create_viewport".to_string(),
                warning: Some("DearPyGui viewport created".to_string()),
                syscall_id: 76,
            },
            // Custom warning syscall
            GuiHook {
                module: "__berkebex__".to_string(),
                name: "warn_gui".to_string(),
                warning: Some("GUI warning triggered".to_string()),
                syscall_id: 77,
            },
        ]
    }

    /// Check if a function call matches any GUI hook
    pub fn match_hook(&self, module: &str, name: &str) -> Option<&GuiHook> {
        if !self.enabled {
            return None;
        }

        for hook in &self.hooks {
            if Self::module_matches(&hook.module, module) && &hook.name == name {
                return Some(hook);
            }
        }
        None
    }

    /// Check if a function call matches any GUI hook by full qualified name
    pub fn match_qualified(&self, qualified_name: &str) -> Option<&GuiHook> {
        if !self.enabled {
            return None;
        }

        // qualified_name format: "module.submodule.ClassName" or "module.function"
        for hook in &self.hooks {
            if qualified_name == hook.module
                || qualified_name.starts_with(&format!("{}.", hook.module))
            {
                // Check if the name matches
                let remaining = &qualified_name[hook.module.len()..];
                if remaining.starts_with('.') {
                    let name_part = remaining.trim_start_matches('.');
                    if name_part == hook.name || name_part.starts_with(&format!("{}.", hook.name)) {
                        return Some(hook);
                    }
                }
            }
        }
        None
    }

    /// Module matching with support for submodules
    fn module_matches(hook_module: &str, called_module: &str) -> bool {
        if hook_module == called_module {
            return true;
        }
        // Support submodule matching
        called_module.starts_with(&format!("{}.", hook_module))
    }
}

/// GUI Popup types that can be emitted via bytecode
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GuiPopupType {
    Warning = 0,
    Info = 1,
    Error = 2,
}

impl GuiPopupType {
    pub fn from_syscall_id(id: u64) -> Option<Self> {
        match id {
            70 => Some(Self::Warning), // Tkinter
            71 => Some(Self::Warning), // Pygame
            72 => Some(Self::Info),    // Matplotlib
            73 => Some(Self::Warning), // Qt
            74 => Some(Self::Info),    // Turtle
            75 => Some(Self::Warning), // Guizero
            76 => Some(Self::Info),    // DearPyGui
            _ => None,
        }
    }
}

/// Generate bytecode that emits a GUI warning syscall
///
/// Returns (syscall_opcode, syscall_id) for the bytecode emitter
pub fn emit_gui_warning_syscall(hook: &GuiHook) -> (u8, i32) {
    // Using OP_SYSCALL (18) with the hook's syscall_id
    (18, hook.syscall_id as i32)
}

/// Parse a qualified name into module and name parts
pub fn parse_qualified_name(name: &str) -> Option<(String, String)> {
    if let Some(last_dot) = name.rfind('.') {
        let module = name[..last_dot].to_string();
        let func_name = name[last_dot + 1..].to_string();
        Some((module, func_name))
    } else {
        None
    }
}

/// Check if a name looks like a GUI init function
pub fn is_gui_init_pattern(name: &str) -> bool {
    let gui_patterns = [
        "init",
        "show",
        "display",
        "set_mode",
        "create_window",
        "Tk",
        "Toplevel",
        "QApplication",
        "QWidget",
        "Screen",
        "App",
        "Window",
        "Figure",
        "run",
        "mainloop",
        "main_loop",
    ];
    gui_patterns.iter().any(|p| name == *p)
}

/// Global GUI guard state
static mut GUI_GUARD_ENABLED: bool = true;

/// Enable GUI guards
pub fn enable_gui_guard() {
    unsafe { GUI_GUARD_ENABLED = true };
}

/// Disable GUI guards (--no-gui-guard flag)
pub fn disable_gui_guard() {
    unsafe { GUI_GUARD_ENABLED = false };
}

/// Check if GUI guards are enabled
pub fn is_gui_guard_enabled() -> bool {
    unsafe { GUI_GUARD_ENABLED }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_hooks_not_empty() {
        let registry = GuiHookRegistry::new();
        assert!(!registry.hooks.is_empty());
        assert!(registry.enabled);
    }

    #[test]
    fn test_disabled_registry() {
        let registry = GuiHookRegistry::disabled();
        assert!(!registry.enabled);
        assert!(registry.hooks.is_empty());
    }

    #[test]
    fn test_match_tkinter() {
        let registry = GuiHookRegistry::new();
        let hook = registry.match_hook("tkinter", "Tk");
        assert!(hook.is_some());
        assert_eq!(hook.unwrap().syscall_id, 70);
    }

    #[test]
    fn test_match_pygame() {
        let registry = GuiHookRegistry::new();
        let hook = registry.match_hook("pygame", "init");
        assert!(hook.is_some());
        assert_eq!(hook.unwrap().syscall_id, 71);
    }

    #[test]
    fn test_match_matplotlib() {
        let registry = GuiHookRegistry::new();
        let hook = registry.match_hook("matplotlib.pyplot", "show");
        assert!(hook.is_some());
        assert_eq!(hook.unwrap().syscall_id, 72);
    }

    #[test]
    fn test_no_match_when_disabled() {
        let registry = GuiHookRegistry::disabled();
        let hook = registry.match_hook("tkinter", "Tk");
        assert!(hook.is_none());
    }

    #[test]
    fn test_parse_qualified_name() {
        assert_eq!(
            parse_qualified_name("tkinter.Tk"),
            Some(("tkinter".to_string(), "Tk".to_string()))
        );
        assert_eq!(
            parse_qualified_name("PyQt5.QtWidgets.QApplication"),
            Some(("PyQt5.QtWidgets".to_string(), "QApplication".to_string()))
        );
        assert_eq!(parse_qualified_name("init"), None);
    }

    #[test]
    fn test_gui_init_pattern() {
        assert!(is_gui_init_pattern("init"));
        assert!(is_gui_init_pattern("show"));
        assert!(is_gui_init_pattern("Tk"));
        assert!(is_gui_init_pattern("QApplication"));
        assert!(!is_gui_init_pattern("len"));
        assert!(!is_gui_init_pattern("print"));
    }

    #[test]
    fn test_popup_type_from_syscall() {
        assert_eq!(
            GuiPopupType::from_syscall_id(70),
            Some(GuiPopupType::Warning)
        );
        assert_eq!(
            GuiPopupType::from_syscall_id(71),
            Some(GuiPopupType::Warning)
        );
        assert_eq!(GuiPopupType::from_syscall_id(72), Some(GuiPopupType::Info));
        assert_eq!(GuiPopupType::from_syscall_id(99), None);
    }

    #[test]
    fn test_module_matches() {
        let registry = GuiHookRegistry::new();
        assert!(registry.match_hook("tkinter", "Tk").is_some());
        assert!(registry.match_hook("tkinter.foo", "Tk").is_some());
        assert!(registry.match_hook("tkinter.foo.bar", "Tk").is_some());
    }

    #[test]
    fn test_emit_gui_warning_syscall() {
        let hook = GuiHook {
            module: "tkinter".to_string(),
            name: "Tk".to_string(),
            warning: Some("test".to_string()),
            syscall_id: 70,
        };
        let (opcode, id) = emit_gui_warning_syscall(&hook);
        assert_eq!(opcode, 18); // OP_SYSCALL
        assert_eq!(id, 70);
    }
}
