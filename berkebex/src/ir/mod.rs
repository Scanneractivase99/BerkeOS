//! IR module - transforms Python AST to Intermediate Representation
//!
//! This module provides the `ast_to_ir()` function that converts Python AST
//! (from rustpython-parser) into a simple SSA-like IR form.

mod types;

pub use types::*;

use rustpython_parser::ast;
use std::collections::HashMap;

/// Scope information for variable resolution
#[derive(Debug, Clone, Default)]
struct Scope {
    variables: HashMap<String, usize>,
}

impl Scope {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    fn insert(&mut self, name: String) -> usize {
        let id = self.variables.len();
        self.variables.insert(name, id);
        id
    }

    fn get(&self, name: &str) -> Option<usize> {
        self.variables.get(name).copied()
    }
}

/// IR builder state during AST transformation
struct IrBuilder {
    value_counter: usize,
    block_counter: usize,
    current_block: usize,
    scopes: Vec<Scope>,
    locals: Vec<(String, IrType)>,
}

impl IrBuilder {
    fn new() -> Self {
        Self {
            value_counter: 0,
            block_counter: 0,
            current_block: 0,
            scopes: vec![Scope::new()],
            locals: Vec::new(),
        }
    }

    fn next_value(&mut self) -> usize {
        let v = self.value_counter;
        self.value_counter += 1;
        v
    }

    fn next_block(&mut self) -> usize {
        let b = self.block_counter;
        self.block_counter += 1;
        b
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(id);
            }
        }
        None
    }

    fn define(&mut self, name: String) -> usize {
        self.scopes.last_mut().unwrap().insert(name)
    }

    fn current_block_mut(&mut self) -> usize {
        self.current_block
    }

    fn set_current_block(&mut self, block: usize) {
        self.current_block = block;
    }
}

/// Transform a Python AST expression into IR value
fn transform_expr(
    builder: &mut IrBuilder,
    expr: &ast::Expr,
    blocks: &mut Vec<IrBlock>,
) -> Result<usize, String> {
    match expr {
        ast::Expr::Constant(c) => {
            let dest = builder.next_value();
            match &c.value {
                ast::Constant::Int(i) => {
                    let val = i64::try_from(i.clone()).unwrap_or(0);
                    blocks[builder.current_block]
                        .push(IrInstruction::ConstInt { dest, value: val });
                }
                ast::Constant::Float(f) => {
                    let val = *f as f64;
                    blocks[builder.current_block]
                        .push(IrInstruction::ConstFloat { dest, value: val });
                }
                ast::Constant::Str(s) => {
                    blocks[builder.current_block].push(IrInstruction::ConstString {
                        dest,
                        value: s.clone(),
                    });
                }
                ast::Constant::Bool(b) => {
                    blocks[builder.current_block]
                        .push(IrInstruction::ConstBool { dest, value: *b });
                }
                ast::Constant::None => {
                    blocks[builder.current_block].push(IrInstruction::ConstInt { dest, value: 0 });
                }
                _ => {
                    blocks[builder.current_block].push(IrInstruction::ConstInt { dest, value: 0 });
                }
            }
            Ok(dest)
        }
        ast::Expr::Name(n) => {
            if let Some(var_id) = builder.lookup(&n.id) {
                let dest = builder.next_value();
                blocks[builder.current_block].push(IrInstruction::Load { dest, var_id });
                Ok(dest)
            } else {
                Err(format!("Unknown variable: {}", n.id))
            }
        }
        ast::Expr::BinOp(b) => {
            let lhs = transform_expr(builder, &b.left, blocks)?;
            let rhs = transform_expr(builder, &b.right, blocks)?;
            let dest = builder.next_value();
            let inst = match b.op {
                ast::Operator::Add => IrInstruction::Add { dest, lhs, rhs },
                ast::Operator::Sub => IrInstruction::Sub { dest, lhs, rhs },
                ast::Operator::Mult => IrInstruction::Mul { dest, lhs, rhs },
                ast::Operator::Div => IrInstruction::Div { dest, lhs, rhs },
                ast::Operator::Mod => IrInstruction::Mod { dest, lhs, rhs },
                _ => return Err(format!("Unsupported binary operator: {:?}", b.op)),
            };
            blocks[builder.current_block].push(inst);
            Ok(dest)
        }
        ast::Expr::UnaryOp(u) => {
            let operand = transform_expr(builder, &u.operand, blocks)?;
            let dest = builder.next_value();
            let inst = match u.op {
                ast::UnaryOp::USub => IrInstruction::Neg { dest, operand },
                ast::UnaryOp::Not => IrInstruction::Not { dest, operand },
                ast::UnaryOp::Invert => IrInstruction::Not { dest, operand },
                ast::UnaryOp::UAdd => {
                    blocks[builder.current_block].push(IrInstruction::ConstInt { dest, value: 0 });
                    blocks[builder.current_block].push(IrInstruction::Add {
                        dest,
                        lhs: dest,
                        rhs: operand,
                    });
                    return Ok(dest);
                }
            };
            blocks[builder.current_block].push(inst);
            Ok(dest)
        }
        ast::Expr::Compare(c) => {
            let lhs = transform_expr(builder, &c.left, blocks)?;
            let mut current = lhs;
            for (op, comp) in c.ops.iter().zip(c.comparators.iter()) {
                let rhs = transform_expr(builder, comp, blocks)?;
                let dest = builder.next_value();
                let inst = match op {
                    ast::CmpOp::Eq => IrInstruction::CmpEq {
                        dest,
                        lhs: current,
                        rhs,
                    },
                    ast::CmpOp::NotEq => IrInstruction::CmpNe {
                        dest,
                        lhs: current,
                        rhs,
                    },
                    ast::CmpOp::Lt => IrInstruction::CmpLt {
                        dest,
                        lhs: current,
                        rhs,
                    },
                    ast::CmpOp::LtE => IrInstruction::CmpLe {
                        dest,
                        lhs: current,
                        rhs,
                    },
                    ast::CmpOp::Gt => IrInstruction::CmpGt {
                        dest,
                        lhs: current,
                        rhs,
                    },
                    ast::CmpOp::GtE => IrInstruction::CmpGe {
                        dest,
                        lhs: current,
                        rhs,
                    },
                    ast::CmpOp::Is | ast::CmpOp::IsNot | ast::CmpOp::In | ast::CmpOp::NotIn => {
                        return Err(format!("Unsupported compare operator: {:?}", op))
                    }
                };
                blocks[builder.current_block].push(inst);
                current = dest;
            }
            Ok(current)
        }
        ast::Expr::Call(c) => {
            let mut args = Vec::new();
            for arg in &c.args {
                args.push(transform_expr(builder, arg, blocks)?);
            }
            let dest = builder.next_value();
            let func_name = match c.func.as_ref() {
                ast::Expr::Name(n) => n.id.to_string(),
                _ => return Err("Only simple function calls supported".to_string()),
            };
            blocks[builder.current_block].push(IrInstruction::Call {
                dest,
                func: func_name,
                args,
            });
            Ok(dest)
        }
        ast::Expr::Subscript(s) => {
            let _value = transform_expr(builder, &s.value, blocks)?;
            let _slice = transform_expr(builder, &s.slice, blocks)?;
            let dest = builder.next_value();
            blocks[builder.current_block].push(IrInstruction::ConstInt { dest, value: 0 });
            Ok(dest)
        }
        _ => Err(format!("Unsupported expression type: {:?}", expr)),
    }
}

/// Transform a Python AST statement into IR instructions
fn transform_stmt(
    builder: &mut IrBuilder,
    stmt: &ast::Stmt,
    blocks: &mut Vec<IrBlock>,
    func_locals: &mut Vec<(String, IrType)>,
) -> Result<(), String> {
    match stmt {
        ast::Stmt::FunctionDef(f) => {
            let mut func = IrFunction::new(f.name.to_string());
            builder.push_scope();
            for (i, arg) in f.args.args.iter().enumerate() {
                let arg_name = arg.def.arg.to_string();
                let var_id = builder.define(arg_name.clone());
                let dest = builder.next_value();
                blocks[builder.current_block].push(IrInstruction::Arg { dest, index: i });
                blocks[builder.current_block].push(IrInstruction::Store {
                    var_id,
                    value: dest,
                });
                func.add_param(arg_name.clone());
                func_locals.push((arg_name, IrType::Unknown));
            }
            for stmt in &f.body {
                transform_stmt(builder, stmt, blocks, func_locals)?;
            }
            builder.pop_scope();
            Ok(())
        }
        ast::Stmt::Assign(a) => {
            let value = transform_expr(builder, &a.value, blocks)?;
            for target in &a.targets {
                match target {
                    ast::Expr::Name(n) => {
                        let var_name = n.id.to_string();
                        let var_id = if let Some(id) = builder.lookup(&var_name) {
                            id
                        } else {
                            let id = builder.define(var_name.clone());
                            func_locals.push((var_name, IrType::Unknown));
                            id
                        };
                        blocks[builder.current_block].push(IrInstruction::Store { var_id, value });
                    }
                    _ => return Err("Only simple assignments supported".to_string()),
                }
            }
            Ok(())
        }
        ast::Stmt::Return(r) => {
            if let Some(value) = &r.value {
                let val = transform_expr(builder, value, blocks)?;
                blocks[builder.current_block].push(IrInstruction::Return { value: Some(val) });
            } else {
                blocks[builder.current_block].push(IrInstruction::ReturnVoid);
            }
            Ok(())
        }
        ast::Stmt::Expr(e) => {
            transform_expr(builder, &e.value, blocks)?;
            Ok(())
        }
        ast::Stmt::If(i) => {
            let cond = transform_expr(builder, &i.test, blocks)?;
            let else_block = builder.next_block();
            let end_block = builder.next_block();
            blocks[builder.current_block].push(IrInstruction::Branch {
                cond,
                then_block: builder.current_block + 1,
                else_block,
            });
            let then_block_idx = builder.current_block + 1;
            builder.set_current_block(then_block_idx);
            blocks.push(IrBlock::new(format!("then_{}", then_block_idx)));
            for stmt in &i.body {
                transform_stmt(builder, stmt, blocks, func_locals)?;
            }
            blocks[builder.current_block].push(IrInstruction::Jump { target: end_block });
            let after_then = builder.current_block;
            builder.set_current_block(else_block);
            blocks.push(IrBlock::new(format!("else_{}", else_block)));
            for stmt in &i.orelse {
                transform_stmt(builder, stmt, blocks, func_locals)?;
            }
            blocks[builder.current_block].push(IrInstruction::Jump { target: end_block });
            let after_else = builder.current_block;
            builder.set_current_block(end_block);
            blocks.push(IrBlock::new(format!("end_{}", end_block)));
            let _ = (after_then, after_else);
            Ok(())
        }
        ast::Stmt::While(w) => {
            let loop_block = builder.next_block();
            let body_block = builder.next_block();
            let end_block = builder.next_block();
            blocks[builder.current_block].push(IrInstruction::Jump { target: loop_block });
            builder.set_current_block(loop_block);
            blocks.push(IrBlock::new(format!("loop_{}", loop_block)));
            let cond = transform_expr(builder, &w.test, blocks)?;
            blocks[builder.current_block].push(IrInstruction::Branch {
                cond,
                then_block: body_block,
                else_block: end_block,
            });
            builder.set_current_block(body_block);
            blocks.push(IrBlock::new(format!("body_{}", body_block)));
            for stmt in &w.body {
                transform_stmt(builder, stmt, blocks, func_locals)?;
            }
            blocks[builder.current_block].push(IrInstruction::Jump { target: loop_block });
            builder.set_current_block(end_block);
            blocks.push(IrBlock::new(format!("endloop_{}", end_block)));
            Ok(())
        }
        ast::Stmt::Pass(_) | ast::Stmt::Break(_) | ast::Stmt::Continue(_) => {
            blocks[builder.current_block].push(IrInstruction::Nop);
            Ok(())
        }
        _ => Err(format!("Unsupported statement type: {:?}", stmt)),
    }
}

/// Transform a Python AST suite into an IR module
///
/// # Arguments
/// * `module` - The Python AST suite (list of statements)
/// * `name` - The name for the IR module
///
/// # Returns
/// * `Ok(IrModule)` - The transformed IR module
/// * `Err(String)` - Error message if transformation fails
pub fn ast_to_ir(module: &ast::Suite, name: &str) -> Result<IrModule, String> {
    let mut ir_module = IrModule::new(name.to_string());
    let mut builder = IrBuilder::new();
    let mut blocks = vec![IrBlock::new("entry".to_string())];
    let mut func_locals = Vec::new();
    let mut in_function = false;

    for stmt in module {
        if matches!(stmt, ast::Stmt::FunctionDef(_)) {
            if in_function {
                builder.pop_scope();
            }
            in_function = true;
            builder.push_scope();
            func_locals.clear();
            blocks = vec![IrBlock::new("entry".to_string())];
            builder.value_counter = 0;
            builder.block_counter = 0;
        }
        transform_stmt(&mut builder, stmt, &mut blocks, &mut func_locals)?;
    }

    if in_function {
        builder.pop_scope();
    }

    let mut func = IrFunction::new(
        module
            .iter()
            .find_map(|s| {
                if let ast::Stmt::FunctionDef(f) = s {
                    Some(f.name.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "main".to_string()),
    );
    func.locals = func_locals;
    func.blocks = blocks;
    ir_module.add_function(func);

    Ok(ir_module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_simple_assignment() {
        let source = "x = 1";
        let suite = parser::parse_python(source).expect("Parse failed");
        let result = ast_to_ir(&suite, "test");
        assert!(result.is_ok());
        let ir = result.unwrap();
        assert_eq!(ir.functions.len(), 1);
    }

    #[test]
    fn test_binary_operation() {
        let source = "x = 1 + 2";
        let suite = parser::parse_python(source).expect("Parse failed");
        let result = ast_to_ir(&suite, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_def() {
        let source = "def add(a, b):\n    return a + b";
        let suite = parser::parse_python(source).expect("Parse failed");
        let result = ast_to_ir(&suite, "test");
        assert!(result.is_ok());
    }
}
