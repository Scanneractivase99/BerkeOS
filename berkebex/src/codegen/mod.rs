//! Code generation module: IR to BexVM bytecode.

use crate::{
    BytecodeFunction, BytecodeModule, Constant, OP_ADD, OP_CALL, OP_CATCH, OP_CMP, OP_DIV,
    OP_GETATTR, OP_JIF, OP_JMP, OP_MAKE_OBJECT, OP_METHOD_CALL, OP_MOD, OP_MUL, OP_NEGATE, OP_NEW,
    OP_NOP, OP_POP, OP_PRINT, OP_PRINTLN, OP_PUSH, OP_PUSHLOCAL, OP_RAISE, OP_RET, OP_SETATTR,
    OP_STORELOCAL, OP_SUB, OP_SYSCALL, OP_TRY,
};

use crate::ir::{IrClassDef, IrFunction, IrInstruction, IrModule};

const SYSCALL_LEN: i32 = 1;
const SYSCALL_PUSH: i32 = 2;
const SYSCALL_RANGE: i32 = 3;

const SYS_SLEEP: i32 = 4;
const SYS_GETPID: i32 = 3;
const SYS_FOPEN: i32 = 12;
const SYS_FREAD: i32 = 13;
const SYS_FWRITE: i32 = 14;
const SYS_FCLOSE: i32 = 15;
const SYS_FB_CLEAR: i32 = 33;
const SYS_FB_PIXEL: i32 = 31;
const SYS_FB_RECT: i32 = 32;
const SYS_FB_TEXT: i32 = 34;
const SYS_READ_KEY: i32 = 40;
const SYS_WINDOW_NEW: i32 = 60;

const CMP_EQ: i32 = 0;
const CMP_NE: i32 = 1;
const CMP_LT: i32 = 2;
const CMP_LE: i32 = 3;
const CMP_GT: i32 = 4;
const CMP_GE: i32 = 5;

#[derive(Default)]
struct ValueStack {
    stack: Vec<usize>,
}

impl ValueStack {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn push(&mut self, id: usize) {
        self.stack.push(id);
    }

    fn pop(&mut self) -> Option<usize> {
        self.stack.pop()
    }
}

struct FunctionCodeGen {
    bytecode_func: BytecodeFunction,
    value_stack: ValueStack,
    local_vars: Vec<usize>,
    block_start: usize,
    pending_jumps: Vec<(String, usize)>,
}

impl FunctionCodeGen {
    fn new(name: String, params: usize, locals: usize) -> Self {
        let mut bytecode_func = BytecodeFunction::new(&name);
        bytecode_func.params = params;
        bytecode_func.locals = locals;
        Self {
            bytecode_func,
            value_stack: ValueStack::new(),
            local_vars: Vec::new(),
            block_start: 0,
            pending_jumps: Vec::new(),
        }
    }

    fn emit(&mut self, opcode: u8) -> usize {
        self.bytecode_func.emit(opcode)
    }

    fn emit_op(&mut self, opcode: u8, operand: i32) -> usize {
        self.bytecode_func.emit_op(opcode, operand)
    }

    fn add_constant(&mut self, value: Constant) -> usize {
        self.bytecode_func.add_constant(value)
    }

    fn push_value(&mut self, id: usize) {
        self.value_stack.push(id);
    }

    fn pop_value(&mut self) -> Option<usize> {
        self.value_stack.pop()
    }

    fn emit_load(&mut self, var_idx: usize) {
        self.emit_op(OP_PUSHLOCAL, var_idx as i32);
    }

    fn emit_store(&mut self, var_idx: usize) {
        self.emit_op(OP_STORELOCAL, var_idx as i32);
    }
}

pub fn ir_to_bex(ir: &IrModule) -> BytecodeModule {
    let mut module = BytecodeModule::new(&ir.name);

    // First pass: compile all class definitions to constants
    for class_def in &ir.classes {
        let class_const = compile_class_def(class_def, &mut module);
        module.add_constant(class_const);
    }

    // Second pass: compile all functions (including class methods)
    for ir_func in &ir.functions {
        let bytecode_func = compile_function(ir_func, &mut module);
        module.add_function(bytecode_func);
    }

    module
}

fn compile_class_def(class_def: &IrClassDef, module: &mut BytecodeModule) -> Constant {
    // Compile all methods first to get their indices
    let mut method_names: Vec<String> = Vec::new();

    for method in &class_def.methods {
        let bytecode_func = compile_function(method, module);
        module.add_function(bytecode_func);
        method_names.push(method.name.clone());
    }

    Constant::Class {
        name: class_def.name.clone(),
        methods: method_names,
        attributes: class_def.attributes.clone(),
        parent: class_def.parent.clone(),
    }
}

fn compile_function(ir_func: &IrFunction, module: &mut BytecodeModule) -> BytecodeFunction {
    let mut codegen = FunctionCodeGen::new(
        ir_func.name.clone(),
        ir_func.params.len(),
        ir_func.locals.len(),
    );

    codegen.local_vars.resize(ir_func.locals.len(), 0);
    for (i, _) in ir_func.locals.iter().enumerate() {
        codegen.local_vars[i] = i;
    }

    for block in &ir_func.blocks {
        codegen.block_start = codegen.bytecode_func.instructions.len();
        resolve_jumps_to_block(&mut codegen, block.label.clone());

        for inst in &block.instructions {
            compile_instruction(&mut codegen, inst, module);
        }
    }

    codegen.bytecode_func
}

fn resolve_jumps_to_block(codegen: &mut FunctionCodeGen, block_label: String) {
    let target_pos = codegen.block_start;
    codegen.pending_jumps.retain(|(label, inst_idx)| {
        if *label == block_label {
            codegen.bytecode_func.instructions[*inst_idx].operand = Some(target_pos as i32);
            false
        } else {
            true
        }
    });
}

fn compile_instruction(
    codegen: &mut FunctionCodeGen,
    inst: &IrInstruction,
    _module: &mut BytecodeModule,
) {
    match inst {
        IrInstruction::ConstInt { dest, value } => {
            let const_idx = codegen.add_constant(Constant::Integer(*value as i32));
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.push_value(*dest);
        }
        IrInstruction::ConstFloat { dest, value } => {
            let const_idx = codegen.add_constant(Constant::Float(*value));
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.push_value(*dest);
        }
        IrInstruction::ConstString { dest, value } => {
            let const_idx = codegen.add_constant(Constant::String(value.clone()));
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.push_value(*dest);
        }
        IrInstruction::ConstBool { dest, value } => {
            let const_idx = codegen.add_constant(Constant::Boolean(*value));
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.push_value(*dest);
        }

        IrInstruction::Add { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_ADD);
            codegen.push_value(*dest);
        }
        IrInstruction::Sub { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_SUB);
            codegen.push_value(*dest);
        }
        IrInstruction::Mul { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_MUL);
            codegen.push_value(*dest);
        }
        IrInstruction::Div { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_DIV);
            codegen.push_value(*dest);
        }
        IrInstruction::Mod { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_MOD);
            codegen.push_value(*dest);
        }

        IrInstruction::Neg { dest, operand } => {
            codegen.emit_load(*operand);
            codegen.emit(OP_NEGATE);
            codegen.push_value(*dest);
        }
        IrInstruction::Not { dest, operand } => {
            codegen.emit_load(*operand);
            let const_idx = codegen.add_constant(Constant::Integer(0));
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.emit(OP_CMP);
            codegen.emit_op(OP_CMP, CMP_EQ);
            codegen.emit(OP_NEGATE);
            codegen.emit(OP_PUSH);
            let const_idx = codegen.add_constant(Constant::Integer(1));
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.emit(OP_ADD);
            codegen.push_value(*dest);
        }

        IrInstruction::CmpEq { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_CMP);
            codegen.emit_op(OP_CMP, CMP_EQ);
            codegen.push_value(*dest);
        }
        IrInstruction::CmpNe { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_CMP);
            codegen.emit_op(OP_CMP, CMP_NE);
            codegen.push_value(*dest);
        }
        IrInstruction::CmpLt { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_CMP);
            codegen.emit_op(OP_CMP, CMP_LT);
            codegen.push_value(*dest);
        }
        IrInstruction::CmpLe { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_CMP);
            codegen.emit_op(OP_CMP, CMP_LE);
            codegen.push_value(*dest);
        }
        IrInstruction::CmpGt { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_CMP);
            codegen.emit_op(OP_CMP, CMP_GT);
            codegen.push_value(*dest);
        }
        IrInstruction::CmpGe { dest, lhs, rhs } => {
            compile_binary_op(codegen, *lhs, *rhs, OP_CMP);
            codegen.emit_op(OP_CMP, CMP_GE);
            codegen.push_value(*dest);
        }

        IrInstruction::Call { dest, func, args } => {
            compile_call(codegen, func, args, _module);
            codegen.push_value(*dest);
        }

        IrInstruction::Branch {
            cond: _,
            then_block: _,
            else_block: _,
        } => {
            codegen.emit(OP_POP);
            let jmp_pos = codegen.emit_op(OP_JIF, 0);
            codegen.pending_jumps.push(("then".to_string(), jmp_pos));
        }
        IrInstruction::Jump { target } => {
            let jmp_pos = codegen.emit_op(OP_JMP, 0);
            codegen
                .pending_jumps
                .push((format!("block_{}", target), jmp_pos));
        }
        IrInstruction::Return { value } => {
            if let Some(val) = value {
                codegen.emit_load(*val);
            } else {
                let const_idx = codegen.add_constant(Constant::Integer(0));
                codegen.emit_op(OP_PUSH, const_idx as i32);
            }
            codegen.emit(OP_RET);
        }
        IrInstruction::ReturnVoid => {
            let const_idx = codegen.add_constant(Constant::Integer(0));
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.emit(OP_RET);
        }

        IrInstruction::Load { dest, var_id } => {
            codegen.emit_load(*var_id);
            codegen.push_value(*dest);
        }
        IrInstruction::Store { var_id, value } => {
            codegen.emit_load(*value);
            codegen.emit_store(*var_id);
            codegen.pop_value();
        }

        IrInstruction::Arg { dest, index } => {
            codegen.emit_load(*index);
            codegen.push_value(*dest);
        }

        IrInstruction::Phi { dest, args } => {
            if let Some((val, _)) = args.first() {
                codegen.emit_load(*val);
                codegen.push_value(*dest);
            }
        }

        IrInstruction::Nop => {
            codegen.emit(OP_NOP);
        }

        // --- Exception Handling ---
        IrInstruction::Try { handler_block } => {
            codegen.emit_op(OP_TRY, *handler_block as i32);
        }
        IrInstruction::Catch {
            exc_type,
            exc_value,
        } => {
            codegen.emit_load(*exc_type);
            if let Some(val) = exc_value {
                codegen.emit_load(*val);
            }
            codegen.emit(OP_CATCH);
        }
        IrInstruction::Raise { exc_type, message } => {
            codegen.emit_load(*exc_type);
            if let Some(msg) = message {
                codegen.emit_load(*msg);
            } else {
                let const_idx = codegen.add_constant(Constant::String(String::new()));
                codegen.emit_op(OP_PUSH, const_idx as i32);
            }
            codegen.emit(OP_RAISE);
        }
        IrInstruction::EndTry => {
            codegen.emit(OP_CATCH);
        }

        // --- With Statement (Context Manager) ---
        IrInstruction::WithSetup { dest, ctx_manager } => {
            codegen.emit_load(*ctx_manager);
            let const_str = codegen.add_constant(Constant::String("__enter__".to_string()));
            codegen.emit_op(OP_PUSH, const_str as i32);
            codegen.emit(OP_SYSCALL);
            codegen.emit_op(OP_SYSCALL, 10);
            codegen.push_value(*dest);
        }
        IrInstruction::WithCleanup { ctx_manager } => {
            codegen.emit_load(*ctx_manager);
            let const_str = codegen.add_constant(Constant::String("__exit__".to_string()));
            codegen.emit_op(OP_PUSH, const_str as i32);
            codegen.emit(OP_SYSCALL);
            codegen.emit_op(OP_SYSCALL, 11);
            codegen.pop_value();
        }

        // --- Class and Object Operations ---
        IrInstruction::NewClass {
            dest,
            class_name,
            args,
        } => {
            // Push args in reverse order (callee expects them in order)
            for arg in args.iter().rev() {
                codegen.emit_load(*arg);
            }
            // Push class name as constant
            let class_idx = codegen.add_constant(Constant::String(class_name.clone()));
            codegen.emit_op(OP_PUSH, class_idx as i32);
            // Push arg count
            let argc_idx = codegen.add_constant(Constant::Integer(args.len() as i32));
            codegen.emit_op(OP_PUSH, argc_idx as i32);
            // Emit NEW opcode to instantiate class
            codegen.emit(OP_NEW);
            codegen.push_value(*dest);
        }
        IrInstruction::GetAttr {
            dest,
            obj,
            attr_name,
        } => {
            // Push object reference
            codegen.emit_load(*obj);
            // Push attribute name
            let attr_idx = codegen.add_constant(Constant::String(attr_name.clone()));
            codegen.emit_op(OP_PUSH, attr_idx as i32);
            // Emit GETATTR opcode
            codegen.emit(OP_GETATTR);
            codegen.push_value(*dest);
        }
        IrInstruction::SetAttr {
            obj,
            attr_name,
            value,
        } => {
            // Push object reference
            codegen.emit_load(*obj);
            // Push attribute name
            let attr_idx = codegen.add_constant(Constant::String(attr_name.clone()));
            codegen.emit_op(OP_PUSH, attr_idx as i32);
            // Push value
            codegen.emit_load(*value);
            // Emit SETATTR opcode
            codegen.emit(OP_SETATTR);
            codegen.pop_value();
        }
        IrInstruction::MethodCall {
            dest,
            obj,
            method_name,
            args,
        } => {
            // Push object reference
            codegen.emit_load(*obj);
            // Push method name
            let method_idx = codegen.add_constant(Constant::String(method_name.clone()));
            codegen.emit_op(OP_PUSH, method_idx as i32);
            // Push args in reverse order
            for arg in args.iter().rev() {
                codegen.emit_load(*arg);
            }
            // Push arg count
            let argc_idx = codegen.add_constant(Constant::Integer(args.len() as i32));
            codegen.emit_op(OP_PUSH, argc_idx as i32);
            // Emit METHOD_CALL opcode
            codegen.emit(OP_METHOD_CALL);
            codegen.push_value(*dest);
        }
        IrInstruction::Lambda { dest, params, body } => {
            let mut lambda_func = BytecodeFunction::new(&format!("lambda_{}", dest));
            lambda_func.params = params.len();
            lambda_func.locals = 0;

            for inst in body {
                compile_instruction_to_func(&mut lambda_func, inst);
            }

            let func_idx = _module.add_function(lambda_func);
            let lambda_const = Constant::Lambda {
                params: params.clone(),
                func_idx,
            };
            let const_idx = codegen.add_constant(lambda_const);
            codegen.emit_op(OP_PUSH, const_idx as i32);
            codegen.push_value(*dest);
        }
    }
}

fn compile_binary_op(codegen: &mut FunctionCodeGen, lhs: usize, rhs: usize, opcode: u8) {
    codegen.emit_load(lhs);
    codegen.emit_load(rhs);
    codegen.emit(opcode);
}

fn compile_instruction_to_func(func: &mut BytecodeFunction, inst: &IrInstruction) {
    match inst {
        IrInstruction::ConstInt { dest: _, value } => {
            let const_idx = func.add_constant(Constant::Integer(*value as i32));
            func.emit_op(OP_PUSH, const_idx as i32);
        }
        IrInstruction::ConstFloat { dest: _, value } => {
            let const_idx = func.add_constant(Constant::Float(*value));
            func.emit_op(OP_PUSH, const_idx as i32);
        }
        IrInstruction::ConstString { dest: _, value } => {
            let const_idx = func.add_constant(Constant::String(value.clone()));
            func.emit_op(OP_PUSH, const_idx as i32);
        }
        IrInstruction::ConstBool { dest: _, value } => {
            let const_idx = func.add_constant(Constant::Boolean(*value));
            func.emit_op(OP_PUSH, const_idx as i32);
        }
        IrInstruction::Add { dest: _, lhs, rhs } => {
            func.emit_op(OP_PUSHLOCAL, *lhs as i32);
            func.emit_op(OP_PUSHLOCAL, *rhs as i32);
            func.emit(OP_ADD);
        }
        IrInstruction::Return { value } => {
            if let Some(val) = value {
                func.emit_op(OP_PUSHLOCAL, *val as i32);
            } else {
                func.emit_op(OP_PUSH, 0);
            }
            func.emit(OP_RET);
        }
        IrInstruction::ReturnVoid => {
            func.emit_op(OP_PUSH, 0);
            func.emit(OP_RET);
        }
        IrInstruction::Load { dest: _, var_id } => {
            func.emit_op(OP_PUSHLOCAL, *var_id as i32);
        }
        IrInstruction::Store { var_id, value } => {
            func.emit_op(OP_PUSHLOCAL, *value as i32);
            func.emit_op(OP_STORELOCAL, *var_id as i32);
        }
        IrInstruction::Nop => {
            func.emit(OP_NOP);
        }
        _ => {
            func.emit(OP_NOP);
        }
    }
}

fn compile_call(
    codegen: &mut FunctionCodeGen,
    func_name: &str,
    args: &[usize],
    module: &BytecodeModule,
) {
    for arg in args.iter().rev() {
        codegen.emit_load(*arg);
    }

    if func_name.starts_with("berkeos.") {
        compile_berkeos_call(codegen, func_name, args);
        return;
    }

    match func_name {
        "print" => {
            codegen.emit(OP_PRINT);
        }
        "println" => {
            codegen.emit(OP_PRINTLN);
        }
        "input" => {
            codegen.emit(OP_SYSCALL);
            codegen.emit_op(OP_SYSCALL, 0);
        }
        "len" => {
            codegen.emit(OP_SYSCALL);
            codegen.emit_op(OP_SYSCALL, SYSCALL_LEN);
        }
        "push" => {
            codegen.emit(OP_SYSCALL);
            codegen.emit_op(OP_SYSCALL, SYSCALL_PUSH);
        }
        "range" => {
            codegen.emit(OP_SYSCALL);
            codegen.emit_op(OP_SYSCALL, SYSCALL_RANGE);
        }
        _ => {
            let func_idx = find_function_index(func_name, module);
            codegen.emit_op(OP_CALL, func_idx);
        }
    }
}

fn find_function_index(func_name: &str, module: &BytecodeModule) -> i32 {
    module
        .functions
        .iter()
        .position(|f| f.name == func_name)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

fn compile_berkeos_call(codegen: &mut FunctionCodeGen, func_name: &str, _args: &[usize]) {
    let parts: Vec<&str> = func_name.split('.').collect();
    if parts.len() < 2 {
        return;
    }

    match parts[1] {
        "process" => {
            if parts.len() == 3 {
                match parts[2] {
                    "sleep" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_SLEEP);
                    }
                    "getpid" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_GETPID);
                    }
                    _ => {}
                }
            }
        }
        "file" => {
            if parts.len() == 3 {
                match parts[2] {
                    "open" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FOPEN);
                    }
                    "read" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FREAD);
                    }
                    "write" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FWRITE);
                    }
                    "close" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FCLOSE);
                    }
                    _ => {}
                }
            }
        }
        "display" => {
            if parts.len() == 3 {
                match parts[2] {
                    "clear" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FB_CLEAR);
                    }
                    "draw_pixel" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FB_PIXEL);
                    }
                    "draw_rect" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FB_RECT);
                    }
                    "draw_text" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_FB_TEXT);
                    }
                    _ => {}
                }
            }
        }
        "input" => {
            if parts.len() == 3 {
                match parts[2] {
                    "key" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_READ_KEY);
                    }
                    _ => {}
                }
            }
        }
        "window" => {
            if parts.len() == 3 {
                match parts[2] {
                    "new" => {
                        codegen.emit(OP_SYSCALL);
                        codegen.emit_op(OP_SYSCALL, SYS_WINDOW_NEW);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IrBlock, IrFunction, IrModule};

    #[test]
    fn test_ir_to_bex_basic() {
        let mut module = IrModule::new("test".to_string());

        let mut func = IrFunction::new("main".to_string());
        func.add_param("a".to_string());

        let mut block = IrBlock::new("entry".to_string());

        block.push(IrInstruction::ConstInt { dest: 0, value: 42 });
        block.push(IrInstruction::ConstInt { dest: 1, value: 10 });
        block.push(IrInstruction::Add {
            dest: 2,
            lhs: 0,
            rhs: 1,
        });
        block.push(IrInstruction::Return { value: Some(2) });

        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);

        assert_eq!(bytecode.name, "test");
        assert_eq!(bytecode.functions.len(), 1);
        assert_eq!(bytecode.functions[0].name, "main");
    }

    #[test]
    fn test_ir_to_bex_comparisons() {
        let mut module = IrModule::new("test".to_string());

        let mut func = IrFunction::new("main".to_string());
        let mut block = IrBlock::new("entry".to_string());

        block.push(IrInstruction::ConstInt { dest: 0, value: 5 });
        block.push(IrInstruction::ConstInt { dest: 1, value: 3 });
        block.push(IrInstruction::CmpLt {
            dest: 2,
            lhs: 0,
            rhs: 1,
        });
        block.push(IrInstruction::Return { value: Some(2) });

        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);
        assert!(!bytecode.functions.is_empty());
    }

    #[test]
    fn test_ir_to_bex_booleans() {
        let mut module = IrModule::new("test".to_string());

        let mut func = IrFunction::new("main".to_string());
        let mut block = IrBlock::new("entry".to_string());

        block.push(IrInstruction::ConstBool {
            dest: 0,
            value: true,
        });
        block.push(IrInstruction::ConstBool {
            dest: 1,
            value: false,
        });
        block.push(IrInstruction::Not {
            dest: 2,
            operand: 0,
        });
        block.push(IrInstruction::Return { value: Some(2) });

        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);
        assert!(!bytecode.functions.is_empty());
    }

    #[test]
    fn test_ir_to_bex_class_def() {
        let mut module = IrModule::new("test_class".to_string());

        let mut class_def = IrClassDef::new("Point".to_string());
        class_def.add_attribute("x".to_string());
        class_def.add_attribute("y".to_string());
        module.add_class(class_def);

        let mut func = IrFunction::new("main".to_string());
        let mut block = IrBlock::new("entry".to_string());
        block.push(IrInstruction::ConstInt { dest: 0, value: 42 });
        block.push(IrInstruction::Return { value: Some(0) });
        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);

        assert_eq!(bytecode.name, "test_class");
        assert!(!bytecode.functions.is_empty());
    }

    #[test]
    fn test_ir_to_bex_class_with_inheritance() {
        let mut module = IrModule::new("test_inherit".to_string());

        let mut child_class = IrClassDef::new("Child".to_string());
        child_class.set_parent("Parent".to_string());
        module.add_class(child_class);

        let mut func = IrFunction::new("main".to_string());
        let mut block = IrBlock::new("entry".to_string());
        block.push(IrInstruction::ConstInt { dest: 0, value: 0 });
        block.push(IrInstruction::Return { value: Some(0) });
        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);
        assert!(!bytecode.functions.is_empty());
    }

    #[test]
    fn test_ir_to_bex_newclass_instruction() {
        let mut module = IrModule::new("test_new".to_string());

        let mut func = IrFunction::new("main".to_string());
        let mut block = IrBlock::new("entry".to_string());

        block.push(IrInstruction::NewClass {
            dest: 0,
            class_name: "Foo".to_string(),
            args: vec![],
        });
        block.push(IrInstruction::Return { value: Some(0) });

        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);
        assert!(!bytecode.functions.is_empty());
    }

    #[test]
    fn test_ir_to_bex_getattr_instruction() {
        let mut module = IrModule::new("test_getattr".to_string());

        let mut func = IrFunction::new("main".to_string());
        let mut block = IrBlock::new("entry".to_string());

        block.push(IrInstruction::ConstInt { dest: 0, value: 1 });
        block.push(IrInstruction::GetAttr {
            dest: 1,
            obj: 0,
            attr_name: "x".to_string(),
        });
        block.push(IrInstruction::Return { value: Some(1) });

        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);
        assert!(!bytecode.functions.is_empty());
    }

    #[test]
    fn test_ir_to_bex_method_call() {
        let mut module = IrModule::new("test_method".to_string());

        let mut func = IrFunction::new("main".to_string());
        let mut block = IrBlock::new("entry".to_string());

        block.push(IrInstruction::ConstInt { dest: 0, value: 1 });
        block.push(IrInstruction::MethodCall {
            dest: 1,
            obj: 0,
            method_name: "method".to_string(),
            args: vec![],
        });
        block.push(IrInstruction::Return { value: Some(1) });

        func.add_block(block);
        module.add_function(func);

        let bytecode = ir_to_bex(&module);
        assert!(!bytecode.functions.is_empty());
    }
}
