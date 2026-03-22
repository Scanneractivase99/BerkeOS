//! IR (Intermediate Representation) types for Python AST transformation.
//!
//! This module defines the types used to represent code in a simple SSA-like IR form.

/// IR types for Python dynamic types
#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    /// Integer type (i32)
    Integer,
    /// Float type (f64)
    Float,
    /// String type
    String,
    /// Boolean type
    Boolean,
    /// Array/List type
    Array,
    /// Dictionary type
    Dict,
    /// Object/Instance type
    Object,
    /// Unknown/dynamic type
    Unknown,
    /// No return type (void)
    Void,
}

impl Default for IrType {
    fn default() -> Self {
        IrType::Unknown
    }
}

/// Represents an IR value (temporary register)
#[derive(Debug, Clone)]
pub struct IrValue {
    /// Unique identifier for this value
    pub id: usize,
    /// Inferred type of this value
    pub ir_type: IrType,
}

impl IrValue {
    /// Create a new IR value with the given id and type
    pub fn new(id: usize, ir_type: IrType) -> Self {
        Self { id, ir_type }
    }

    /// Create a new IR value with unknown type
    pub fn unknown(id: usize) -> Self {
        Self {
            id,
            ir_type: IrType::Unknown,
        }
    }
}

/// IR instructions - single static assignment form
#[derive(Debug, Clone)]
pub enum IrInstruction {
    // --- Constants and Memory ---
    /// `%dest = const i32 <value>`
    ConstInt { dest: usize, value: i64 },
    /// `%dest = const f64 <value>`
    ConstFloat { dest: usize, value: f64 },
    /// `%dest = const string "<value>"`
    ConstString { dest: usize, value: String },
    /// `%dest = const bool <value>`
    ConstBool { dest: usize, value: bool },

    // --- Binary Operations ---
    /// `%dest = add %lhs, %rhs`
    Add { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = sub %lhs, %rhs`
    Sub { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = mul %lhs, %rhs`
    Mul { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = div %lhs, %rhs`
    Div { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = mod %lhs, %rhs`
    Mod { dest: usize, lhs: usize, rhs: usize },

    // --- Unary Operations ---
    /// `%dest = neg %operand`
    Neg { dest: usize, operand: usize },
    /// `%dest = not %operand`
    Not { dest: usize, operand: usize },

    // --- Comparison Operations ---
    /// `%dest = cmp_eq %lhs, %rhs`
    CmpEq { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = cmp_ne %lhs, %rhs`
    CmpNe { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = cmp_lt %lhs, %rhs`
    CmpLt { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = cmp_le %lhs, %rhs`
    CmpLe { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = cmp_gt %lhs, %rhs`
    CmpGt { dest: usize, lhs: usize, rhs: usize },
    /// `%dest = cmp_ge %lhs, %rhs`
    CmpGe { dest: usize, lhs: usize, rhs: usize },

    // --- Function Operations ---
    /// `%dest = call <func_name> [%arg1, %arg2, ...]`
    Call {
        dest: usize,
        func: String,
        args: Vec<usize>,
    },
    /// `ret %value` (optional return value)
    Return { value: Option<usize> },
    /// `ret` (no value)
    ReturnVoid,

    // --- Control Flow ---
    /// `br %cond, block_then, block_else`
    Branch {
        cond: usize,
        then_block: usize,
        else_block: usize,
    },
    /// `jmp block`
    Jump { target: usize },

    // --- Phi Function (SSA merge) ---
    /// `%dest = phi [%val1, block1], [%val2, block2], ...`
    Phi {
        dest: usize,
        args: Vec<(usize, usize)>,
    },

    // --- Local Variables ---
    /// `%dest = load %var_id` (load local variable)
    Load { dest: usize, var_id: usize },
    /// `store %var_id, %value` (store to local variable)
    Store { var_id: usize, value: usize },

    // --- Function Arguments ---
    /// `%dest = arg %index` (get function argument at index)
    Arg { dest: usize, index: usize },

    // --- Class and Object Operations ---
    /// `%dest = new <class_name> [%arg1, %arg2, ...]` (instantiate class)
    NewClass {
        dest: usize,
        class_name: String,
        args: Vec<usize>,
    },
    /// `%dest = getattr %obj, "<attr_name>"` (get instance attribute)
    GetAttr {
        dest: usize,
        obj: usize,
        attr_name: String,
    },
    /// `setattr %obj, "<attr_name>", %value` (set instance attribute)
    SetAttr {
        obj: usize,
        attr_name: String,
        value: usize,
    },
    /// `%dest = call_method %obj, "<method>" [%arg1, %arg2, ...]`
    MethodCall {
        dest: usize,
        obj: usize,
        method_name: String,
        args: Vec<usize>,
    },
    /// `%dest = lambda [%param1, %param2, ...] { <body> }` (anonymous function)
    Lambda {
        dest: usize,
        params: Vec<String>,
        body: Vec<IrInstruction>,
    },

    // --- Special Operations ---
    /// `nop`
    Nop,

    // --- Exception Handling ---
    /// `try handler_block` - Setup exception handler
    Try { handler_block: usize },
    /// `catch exc_type, exc_value` - Catch exception
    Catch {
        exc_type: usize,
        exc_value: Option<usize>,
    },
    /// `raise exc_type, message` - Raise an exception
    Raise {
        exc_type: usize,
        message: Option<usize>,
    },
    /// `end_try` - End try block (pop exception frame)
    EndTry,

    // --- With Statement (Context Manager) ---
    /// `with_setup %dest, %ctx_manager` - Call __enter__, push to with stack
    WithSetup { dest: usize, ctx_manager: usize },
    /// `with_cleanup %ctx_manager` - Call __exit__
    WithCleanup { ctx_manager: usize },
}

impl IrInstruction {
    /// Get the destination value id if this instruction has one
    pub fn dest(&self) -> Option<usize> {
        match self {
            IrInstruction::ConstInt { dest, .. } => Some(*dest),
            IrInstruction::ConstFloat { dest, .. } => Some(*dest),
            IrInstruction::ConstString { dest, .. } => Some(*dest),
            IrInstruction::ConstBool { dest, .. } => Some(*dest),
            IrInstruction::Add { dest, .. } => Some(*dest),
            IrInstruction::Sub { dest, .. } => Some(*dest),
            IrInstruction::Mul { dest, .. } => Some(*dest),
            IrInstruction::Div { dest, .. } => Some(*dest),
            IrInstruction::Mod { dest, .. } => Some(*dest),
            IrInstruction::Neg { dest, .. } => Some(*dest),
            IrInstruction::Not { dest, .. } => Some(*dest),
            IrInstruction::CmpEq { dest, .. } => Some(*dest),
            IrInstruction::CmpNe { dest, .. } => Some(*dest),
            IrInstruction::CmpLt { dest, .. } => Some(*dest),
            IrInstruction::CmpLe { dest, .. } => Some(*dest),
            IrInstruction::CmpGt { dest, .. } => Some(*dest),
            IrInstruction::CmpGe { dest, .. } => Some(*dest),
            IrInstruction::Call { dest, .. } => Some(*dest),
            IrInstruction::Load { dest, .. } => Some(*dest),
            IrInstruction::Arg { dest, .. } => Some(*dest),
            IrInstruction::Phi { dest, .. } => Some(*dest),
            IrInstruction::WithSetup { dest, .. } => Some(*dest),
            IrInstruction::NewClass { dest, .. } => Some(*dest),
            IrInstruction::GetAttr { dest, .. } => Some(*dest),
            IrInstruction::MethodCall { dest, .. } => Some(*dest),
            IrInstruction::Lambda { dest, .. } => Some(*dest),
            _ => None,
        }
    }
}

/// A basic block in the IR
#[derive(Debug, Clone)]
pub struct IrBlock {
    /// Block label (unique identifier)
    pub label: String,
    /// Instructions in this block
    pub instructions: Vec<IrInstruction>,
}

impl IrBlock {
    /// Create a new empty block with the given label
    pub fn new(label: String) -> Self {
        Self {
            label,
            instructions: Vec::new(),
        }
    }

    /// Add an instruction to this block
    pub fn push(&mut self, inst: IrInstruction) {
        self.instructions.push(inst);
    }
}

/// Represents a function in IR
#[derive(Debug, Clone)]
pub struct IrFunction {
    /// Function name
    pub name: String,
    /// Parameter names
    pub params: Vec<String>,
    /// Local variables (name -> type)
    pub locals: Vec<(String, IrType)>,
    /// Basic blocks in this function
    pub blocks: Vec<IrBlock>,
}

impl IrFunction {
    /// Create a new function with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            params: Vec::new(),
            locals: Vec::new(),
            blocks: Vec::new(),
        }
    }

    /// Add a parameter to this function
    pub fn add_param(&mut self, name: String) {
        self.params.push(name);
    }

    /// Add a local variable
    pub fn add_local(&mut self, name: String, ir_type: IrType) {
        self.locals.push((name, ir_type));
    }

    /// Add a basic block
    pub fn add_block(&mut self, block: IrBlock) {
        self.blocks.push(block);
    }
}

/// Represents a class definition in IR
#[derive(Debug, Clone)]
pub struct IrClassDef {
    /// Class name
    pub name: String,
    /// Parent class name (single inheritance, None for no parent)
    pub parent: Option<String>,
    /// Methods defined in this class (name -> function)
    pub methods: Vec<IrFunction>,
    /// Instance attribute names
    pub attributes: Vec<String>,
}

impl IrClassDef {
    /// Create a new class definition
    pub fn new(name: String) -> Self {
        Self {
            name,
            parent: None,
            methods: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Set the parent class for inheritance
    pub fn set_parent(&mut self, parent: String) {
        self.parent = Some(parent);
    }

    /// Add a method to this class
    pub fn add_method(&mut self, method: IrFunction) {
        self.methods.push(method);
    }

    /// Add an instance attribute
    pub fn add_attribute(&mut self, attr: String) {
        self.attributes.push(attr);
    }
}

/// Represents a complete IR module
#[derive(Debug, Clone)]
pub struct IrModule {
    /// Module name
    pub name: String,
    /// Functions in this module
    pub functions: Vec<IrFunction>,
    /// Class definitions in this module
    pub classes: Vec<IrClassDef>,
}

impl IrModule {
    /// Create a new empty module
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }

    /// Add a function to this module
    pub fn add_function(&mut self, func: IrFunction) {
        self.functions.push(func);
    }

    /// Add a class definition to this module
    pub fn add_class(&mut self, class: IrClassDef) {
        self.classes.push(class);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_value() {
        let val = IrValue::new(0, IrType::Integer);
        assert_eq!(val.id, 0);
        assert_eq!(val.ir_type, IrType::Integer);
    }

    #[test]
    fn test_ir_instruction_dest() {
        let inst = IrInstruction::ConstInt { dest: 1, value: 42 };
        assert_eq!(inst.dest(), Some(1));

        let inst = IrInstruction::Return { value: None };
        assert_eq!(inst.dest(), None);
    }

    #[test]
    fn test_ir_function() {
        let mut func = IrFunction::new("test".to_string());
        func.add_param("a".to_string());
        func.add_param("b".to_string());
        func.add_local("x".to_string(), IrType::Float);

        assert_eq!(func.name, "test");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.locals.len(), 1);
    }

    #[test]
    fn test_ir_module() {
        let mut module = IrModule::new("test_module".to_string());
        let func = IrFunction::new("main".to_string());
        module.add_function(func);

        assert_eq!(module.name, "test_module");
        assert_eq!(module.functions.len(), 1);
    }
}
