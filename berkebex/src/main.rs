use std::env;
use std::fs;
use std::path::Path;

mod builtins;
mod codegen;
mod gui_detector;
mod import;
mod ir;
mod parser;

#[derive(Debug, Clone)]
pub enum Constant {
    Integer(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Dict(Vec<(String, i32)>),
    Class {
        name: String,
        methods: Vec<String>,
        attributes: Vec<String>,
        parent: Option<String>,
    },
    Object(Vec<(String, i32)>),
    Lambda {
        params: Vec<String>,
        func_idx: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub operand: Option<i32>,
}

impl Instruction {
    pub fn new(opcode: u8) -> Self {
        Self {
            opcode,
            operand: None,
        }
    }
    pub fn with_operand(opcode: u8, operand: i32) -> Self {
        Self {
            opcode,
            operand: Some(operand),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: String,
    pub params: usize,
    pub locals: usize,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
}

impl BytecodeFunction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            params: 0,
            locals: 0,
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }
    pub fn add_constant(&mut self, value: Constant) -> usize {
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }
    pub fn emit(&mut self, opcode: u8) -> usize {
        let pos = self.instructions.len();
        self.instructions.push(Instruction::new(opcode));
        pos
    }
    pub fn emit_op(&mut self, opcode: u8, operand: i32) -> usize {
        let pos = self.instructions.len();
        self.instructions
            .push(Instruction::with_operand(opcode, operand));
        pos
    }
}

#[derive(Debug, Clone)]
pub struct BytecodeModule {
    pub magic: u32,
    pub version: u16,
    pub name: String,
    pub constants: Vec<Constant>,
    pub functions: Vec<BytecodeFunction>,
}

impl BytecodeModule {
    pub fn new(name: &str) -> Self {
        Self {
            magic: 0x42455831,
            version: 1,
            name: name.to_string(),
            constants: Vec::new(),
            functions: Vec::new(),
        }
    }
    pub fn add_constant(&mut self, value: Constant) -> usize {
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }
    pub fn add_function(&mut self, func: BytecodeFunction) -> usize {
        let idx = self.functions.len();
        self.functions.push(func);
        idx
    }
    pub fn emit_bex(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.magic.to_le_bytes());
        bytes.extend_from_slice(&self.version.to_le_bytes());
        let name_bytes = self.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name_bytes);
        bytes.extend_from_slice(&(self.constants.len() as u32).to_le_bytes());
        for c in &self.constants {
            match c {
                Constant::Integer(i) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&i.to_le_bytes());
                }
                Constant::Float(f) => {
                    bytes.push(2);
                    bytes.extend_from_slice(&f.to_le_bytes());
                }
                Constant::String(s) => {
                    bytes.push(3);
                    let s_bytes = s.as_bytes();
                    bytes.extend_from_slice(&(s_bytes.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(s_bytes);
                }
                Constant::Boolean(b) => {
                    bytes.push(4);
                    bytes.push(*b as u8);
                }
                Constant::Dict(entries) => {
                    bytes.push(5);
                    bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
                    for (key, val_idx) in entries {
                        let key_bytes = key.as_bytes();
                        bytes.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
                        bytes.extend_from_slice(key_bytes);
                        bytes.extend_from_slice(&val_idx.to_le_bytes());
                    }
                }
                Constant::Class {
                    name,
                    methods,
                    attributes,
                    parent,
                } => {
                    bytes.push(6);
                    let name_bytes = name.as_bytes();
                    bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(name_bytes);
                    bytes.extend_from_slice(&(methods.len() as u32).to_le_bytes());
                    for method in methods {
                        let m_bytes = method.as_bytes();
                        bytes.extend_from_slice(&(m_bytes.len() as u32).to_le_bytes());
                        bytes.extend_from_slice(m_bytes);
                    }
                    bytes.extend_from_slice(&(attributes.len() as u32).to_le_bytes());
                    for attr in attributes {
                        let a_bytes = attr.as_bytes();
                        bytes.extend_from_slice(&(a_bytes.len() as u32).to_le_bytes());
                        bytes.extend_from_slice(a_bytes);
                    }
                    if let Some(p) = parent {
                        bytes.push(1);
                        let p_bytes = p.as_bytes();
                        bytes.extend_from_slice(&(p_bytes.len() as u32).to_le_bytes());
                        bytes.extend_from_slice(p_bytes);
                    } else {
                        bytes.push(0);
                    }
                }
                Constant::Object(fields) => {
                    bytes.push(7);
                    bytes.extend_from_slice(&(fields.len() as u32).to_le_bytes());
                    for (key, val_idx) in fields {
                        let key_bytes = key.as_bytes();
                        bytes.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
                        bytes.extend_from_slice(key_bytes);
                        bytes.extend_from_slice(&val_idx.to_le_bytes());
                    }
                }
                Constant::Lambda { params, func_idx } => {
                    bytes.push(8);
                    bytes.extend_from_slice(&(params.len() as u32).to_le_bytes());
                    for param in params {
                        let p_bytes = param.as_bytes();
                        bytes.extend_from_slice(&(p_bytes.len() as u32).to_le_bytes());
                        bytes.extend_from_slice(p_bytes);
                    }
                    bytes.extend_from_slice(&func_idx.to_le_bytes());
                }
            }
        }
        bytes.extend_from_slice(&(self.functions.len() as u32).to_le_bytes());
        for func in &self.functions {
            bytes.extend_from_slice(&(func.name.len() as u16).to_le_bytes());
            bytes.extend_from_slice(func.name.as_bytes());
            bytes.extend_from_slice(&(func.params as u16).to_le_bytes());
            bytes.extend_from_slice(&(func.locals as u16).to_le_bytes());
            bytes.extend_from_slice(&(func.instructions.len() as u32).to_le_bytes());
            for inst in &func.instructions {
                bytes.push(inst.opcode);
                if let Some(op) = inst.operand {
                    bytes.extend_from_slice(&op.to_le_bytes());
                } else {
                    bytes.extend_from_slice(&(-1i32).to_le_bytes());
                }
            }
        }
        bytes
    }
}

const OP_NOP: u8 = 0;
const OP_PUSH: u8 = 1;
const OP_PUSHLOCAL: u8 = 2;
const OP_STORELOCAL: u8 = 3;
const OP_ADD: u8 = 4;
const OP_SUB: u8 = 5;
const OP_MUL: u8 = 6;
const OP_DIV: u8 = 7;
const OP_MOD: u8 = 8;
const OP_NEGATE: u8 = 9;
const OP_CMP: u8 = 10;
const OP_JMP: u8 = 11;
const OP_JIF: u8 = 12;
const OP_PRINT: u8 = 13;
const OP_PRINTLN: u8 = 14;
const OP_RET: u8 = 15;
const OP_HALT: u8 = 16;
const OP_POP: u8 = 17;
const OP_SYSCALL: u8 = 18;
const OP_DUP: u8 = 19;
const OP_INPUT: u8 = 20;
const OP_SLEEP: u8 = 21;
const OP_RANDOM: u8 = 22;
const OP_CALL: u8 = 23;
const OP_ARRAY_INDEX: u8 = 27;
const OP_ARRAY_STORE: u8 = 28;
const OP_DICT_ACCESS: u8 = 29;
const OP_DICT_NEW: u8 = 30;

// Exception handling opcodes (match bexvm.rs)
pub const OP_TRY: u8 = 24;
pub const OP_CATCH: u8 = 25;
pub const OP_RAISE: u8 = 26;

// Object/Class opcodes
pub const OP_NEW: u8 = 31;
pub const OP_GETATTR: u8 = 32;
pub const OP_SETATTR: u8 = 33;
pub const OP_METHOD_CALL: u8 = 34;
pub const OP_MAKE_OBJECT: u8 = 35;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(i32),
    Float(f64),
    String(String),
    Identifier(String),
    Print,
    PrintLn,
    PrintInt,
    PrintChar,
    Input,
    Sleep,
    Len,
    Append,
    Return,
    If,
    Else,
    While,
    For,
    Fn,
    Let,
    True,
    False,
    Int,
    Char,
    Void,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Newline,
    Dict,
    Class,
    Lambda,
    Eof,
}

struct Lexer {
    source: Vec<char>,
    pos: usize,
}
impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
        }
    }
    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }
    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    fn next_token(&mut self) -> Token {
        self.skip_ws();
        let c = match self.advance() {
            Some(c) => c,
            None => return Token::Eof,
        };
        if c == '\n' {
            return Token::Newline;
        }
        if c == '/' && self.peek() == Some('/') {
            while self.peek() != Some('\n') && self.peek() != None {
                self.advance();
            }
            return self.next_token();
        }
        if c.is_ascii_digit() {
            let mut num = String::from(c);
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '.' {
                    num.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if num.contains('.') {
                return Token::Float(num.parse().unwrap_or(0.0));
            }
            return Token::Number(num.parse().unwrap_or(0));
        }
        if c == '"' {
            let mut s = String::new();
            while let Some(ch) = self.peek() {
                if ch == '"' {
                    self.advance();
                    break;
                }
                s.push(ch);
                self.advance();
            }
            return Token::String(s);
        }
        if c == '\'' {
            let ch = self.advance().unwrap_or(' ');
            self.advance();
            return Token::Number(ch as i32);
        }
        if c.is_alphabetic() || c == '_' {
            let mut ident = String::from(c);
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            return match ident.as_str() {
                "print" => Token::Print,
                "println" => Token::PrintLn,
                "printi" => Token::PrintInt,
                "input" => Token::Input,
                "sleep" => Token::Sleep,
                "len" => Token::Len,
                "push" => Token::Append,
                "let" => Token::Let,
                "if" => Token::If,
                "else" => Token::Else,
                "while" => Token::While,
                "for" => Token::For,
                "fn" => Token::Fn,
                "class" => Token::Class,
                "lambda" => Token::Lambda,
                "return" => Token::Return,
                "true" => Token::True,
                "false" => Token::False,
                "int" => Token::Int,
                "char" => Token::Char,
                _ => Token::Identifier(ident),
            };
        }
        match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EqEq
                } else {
                    Token::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::NotEq
                } else {
                    Token::Identifier(String::from("!"))
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LtEq
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GtEq
                } else {
                    Token::Gt
                }
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            _ => Token::Identifier(String::from(c)),
        }
    }
    fn tokens(&mut self) -> Vec<Token> {
        let mut toks = Vec::new();
        loop {
            let t = self.next_token();
            toks.push(t.clone());
            if matches!(t, Token::Eof) {
                break;
            }
        }
        toks
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    fn advance(&mut self) -> Token {
        let tok = self.current().clone();
        self.pos += 1;
        tok
    }
    fn skip_nl(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.advance();
        }
    }
    fn parse_fn(&mut self, module: &mut BytecodeModule) -> Result<(), String> {
        self.advance();
        let name = match self.advance() {
            Token::Identifier(n) => n,
            t => return Err(format!("Expected fn name, got {:?}", t)),
        };
        self.advance();
        while !matches!(self.current(), Token::RParen) {
            self.advance();
        }
        self.advance();
        let mut func = BytecodeFunction::new(&name);
        self.skip_nl();
        self.advance();
        self.skip_nl();
        let mut locals = Vec::new();
        while !matches!(self.current(), Token::RBrace) {
            self.skip_nl();
            if matches!(self.current(), Token::RBrace) {
                break;
            }
            self.parse_stmt(&mut func, &mut locals, module)?;
            self.skip_nl();
        }
        self.advance();
        func.locals = locals.len();
        module.add_function(func);
        Ok(())
    }
    fn parse_stmt(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<(), String> {
        match self.current() {
            Token::Let => {
                self.advance();
                if let Token::Identifier(name) = self.advance() {
                    locals.push(name.clone());
                    self.advance();
                    self.parse_expr(func, locals, module)?;
                    func.emit(OP_STORELOCAL);
                    func.emit_op(OP_PUSHLOCAL, locals.len() as i32 - 1);
                }
            }
            Token::Print => {
                self.advance();
                self.advance();
                self.parse_expr(func, locals, module)?;
                self.advance();
                func.emit(OP_PRINT);
            }
            Token::PrintLn => {
                self.advance();
                self.advance();
                self.parse_expr(func, locals, module)?;
                self.advance();
                func.emit(OP_PRINTLN);
            }
            Token::PrintInt => {
                self.advance();
                self.advance();
                self.parse_expr(func, locals, module)?;
                self.advance();
                func.emit(OP_PRINT);
            }
            Token::Input => {
                self.advance();
                self.advance();
                self.advance();
                func.emit(OP_INPUT);
            }
            Token::Sleep => {
                self.advance();
                self.advance();
                self.parse_expr(func, locals, module)?;
                self.advance();
                func.emit(OP_SLEEP);
            }
            Token::If => {
                self.advance();
                self.parse_expr(func, locals, module)?;
                let jmp = func.emit_op(OP_JIF, 0);
                self.skip_nl();
                self.advance();
                self.skip_nl();
                while !matches!(self.current(), Token::RBrace) {
                    self.skip_nl();
                    self.parse_stmt(func, locals, module)?;
                    self.skip_nl();
                }
                self.advance();
                func.instructions[jmp].operand = Some(func.instructions.len() as i32);
            }
            Token::While => {
                self.advance();
                let start = func.instructions.len();
                self.parse_expr(func, locals, module)?;
                let jmp = func.emit_op(OP_JIF, 0);
                self.skip_nl();
                self.advance();
                self.skip_nl();
                while !matches!(self.current(), Token::RBrace) {
                    self.skip_nl();
                    self.parse_stmt(func, locals, module)?;
                    self.skip_nl();
                }
                self.advance();
                func.emit_op(OP_JMP, start as i32);
                func.instructions[jmp].operand = Some(func.instructions.len() as i32);
            }
            Token::For => {
                self.advance();
                let var = if let Token::Identifier(n) = self.advance() {
                    n
                } else {
                    return Err("Expected var".to_string());
                };
                self.advance();
                self.parse_expr(func, locals, module)?;
                locals.push(var.clone());
                func.emit(OP_STORELOCAL);
                let start = func.instructions.len();
                func.emit_op(OP_PUSHLOCAL, locals.len() as i32 - 1);
                self.advance();
                self.parse_expr(func, locals, module)?;
                func.emit(OP_CMP);
                let jmp = func.emit_op(OP_JIF, 0);
                self.skip_nl();
                self.advance();
                self.skip_nl();
                while !matches!(self.current(), Token::RBrace) {
                    self.skip_nl();
                    self.parse_stmt(func, locals, module)?;
                    self.skip_nl();
                }
                self.advance();
                func.emit_op(OP_PUSHLOCAL, locals.len() as i32 - 1);
                let const_idx = func.add_constant(Constant::Integer(1));
                func.emit_op(OP_PUSH, const_idx as i32);
                func.emit(OP_ADD);
                func.emit(OP_STORELOCAL);
                func.emit_op(OP_JMP, start as i32);
                func.instructions[jmp].operand = Some(func.instructions.len() as i32);
            }
            Token::Return => {
                self.advance();
                if !matches!(self.current(), Token::Newline)
                    && !matches!(self.current(), Token::RBrace)
                {
                    self.parse_expr(func, locals, module)?;
                } else {
                    func.emit_op(OP_PUSH, module.add_constant(Constant::Integer(0)) as i32);
                }
                func.emit(OP_RET);
            }
            _ => {
                if matches!(self.current(), Token::Identifier(_)) {
                    if let Token::Identifier(ref name) = self.current() {
                        let name_str = name.clone();
                        self.advance();
                        if self.current() == &Token::LBracket {
                            self.advance();
                            if let Some(idx) = locals.iter().position(|n| *n == name_str) {
                                func.emit_op(OP_PUSHLOCAL, idx as i32);
                            } else {
                                return Err("Unknown variable for array assignment".to_string());
                            }
                            self.parse_expr(func, locals, module)?;
                            self.advance();
                            self.advance();
                            self.parse_expr(func, locals, module)?;
                            func.emit(OP_ARRAY_STORE);
                            func.emit(OP_POP);
                            return Ok(());
                        }
                    }
                }
                self.parse_expr(func, locals, module)?;
                func.emit(OP_POP);
            }
        }
        Ok(())
    }
    fn parse_expr(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<(), String> {
        self.parse_cmp(func, locals, module)
    }
    fn parse_cmp(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<(), String> {
        self.parse_term(func, locals, module)?;
        loop {
            match self.current() {
                Token::EqEq | Token::NotEq | Token::Lt | Token::LtEq | Token::Gt | Token::GtEq => {
                    self.advance();
                    self.parse_term(func, locals, module)?;
                    func.emit(OP_CMP);
                }
                _ => break,
            }
        }
        Ok(())
    }
    fn parse_term(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<(), String> {
        self.parse_factor(func, locals, module)?;
        loop {
            match self.current() {
                Token::Plus => {
                    self.advance();
                    self.parse_factor(func, locals, module)?;
                    func.emit(OP_ADD);
                }
                Token::Minus => {
                    self.advance();
                    self.parse_factor(func, locals, module)?;
                    func.emit(OP_SUB);
                }
                _ => break,
            }
        }
        Ok(())
    }
    fn handle_string_concat(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
        s1: String,
    ) -> Result<(), String> {
        let mut result = s1;
        loop {
            if self.current() == &Token::Plus {
                self.advance();
                if let Token::String(s2) = self.current() {
                    let s2 = s2.clone();
                    self.advance();
                    result.push_str(&s2);
                } else {
                    return Err("String concat requires string operand".to_string());
                }
            } else {
                break;
            }
        }
        let const_idx = module.add_constant(Constant::String(result));
        func.emit_op(OP_PUSH, const_idx as i32);
        Ok(())
    }
    fn parse_factor(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<(), String> {
        self.parse_unary(func, locals, module)?;
        loop {
            match self.current() {
                Token::Star => {
                    self.advance();
                    self.parse_unary(func, locals, module)?;
                    func.emit(OP_MUL);
                }
                Token::Slash => {
                    self.advance();
                    self.parse_unary(func, locals, module)?;
                    func.emit(OP_DIV);
                }
                Token::Percent => {
                    self.advance();
                    self.parse_unary(func, locals, module)?;
                    func.emit(OP_MOD);
                }
                _ => break,
            }
        }
        Ok(())
    }
    fn parse_unary(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<(), String> {
        match self.current() {
            Token::Minus => {
                self.advance();
                self.parse_unary(func, locals, module)?;
                func.emit(OP_NEGATE);
            }
            _ => self.parse_primary(func, locals, module)?,
        }
        Ok(())
    }
    fn parse_primary(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<(), String> {
        let tok = self.advance();
        match tok {
            Token::Number(n) => {
                let _ = func.emit_op(OP_PUSH, module.add_constant(Constant::Integer(n)) as i32);
            }
            Token::Float(f) => {
                let _ = func.emit_op(OP_PUSH, module.add_constant(Constant::Float(f)) as i32);
            }
            Token::String(s) => {
                if self.current() == &Token::Plus {
                    self.handle_string_concat(func, locals, module, s)?;
                } else {
                    let _ = func.emit_op(OP_PUSH, module.add_constant(Constant::String(s)) as i32);
                }
            }
            Token::True => {
                let _ = func.emit_op(OP_PUSH, module.add_constant(Constant::Boolean(true)) as i32);
            }
            Token::False => {
                let _ = func.emit_op(
                    OP_PUSH,
                    module.add_constant(Constant::Boolean(false)) as i32,
                );
            }
            Token::LBrace => {
                let entries = self.parse_dict_entries(func, locals, module)?;
                let const_idx = module.add_constant(Constant::Dict(entries));
                func.emit_op(OP_PUSH, const_idx as i32);
            }
            Token::LBracket => {
                self.parse_expr(func, locals, module)?;
                self.advance();
                func.emit(OP_ARRAY_INDEX);
            }
            Token::Identifier(name) => {
                if self.current() == &Token::LParen {
                    self.advance();
                    let mut args = 0;
                    while !matches!(self.current(), Token::RParen) {
                        self.parse_expr(func, locals, module)?;
                        args += 1;
                        if matches!(self.current(), Token::Comma) {
                            self.advance();
                        }
                    }
                    self.advance();
                    match name.as_str() {
                        "input" => {
                            func.emit(OP_INPUT);
                        }
                        "sleep" => {
                            func.emit(OP_SLEEP);
                        }
                        "rand" => {
                            func.emit(OP_RANDOM);
                            func.emit(OP_PUSH);
                            let const_idx = func.add_constant(Constant::Integer(100));
                            func.emit_op(OP_PUSH, const_idx as i32);
                            func.emit(OP_MOD);
                        }
                        "len" => {
                            func.emit(OP_SYSCALL);
                            func.emit_op(OP_SYSCALL, 1);
                        }
                        "push" => {
                            func.emit(OP_SYSCALL);
                            func.emit_op(OP_SYSCALL, 2);
                        }
                        _ => {
                            if let Some(func_idx) =
                                module.functions.iter().position(|f| f.name == name)
                            {
                                func.emit_op(OP_CALL, func_idx as i32);
                            } else {
                                func.emit(OP_SYSCALL);
                                func.emit_op(OP_SYSCALL, 0);
                            }
                        }
                    }
                } else if self.current() == &Token::LBracket {
                    if let Some(idx) = locals.iter().position(|n| n == &name) {
                        func.emit_op(OP_PUSHLOCAL, idx as i32);
                    } else {
                        func.emit_op(OP_PUSH, module.add_constant(Constant::Integer(0)) as i32);
                    }
                    self.advance();
                    if matches!(self.current(), Token::String(_)) {
                        if let Token::String(ref s) = self.current() {
                            let key_idx = module.add_constant(Constant::String(s.clone()));
                            self.advance();
                            if self.current() == &Token::RBracket {
                                self.advance();
                                let const_idx = module.add_constant(Constant::Integer(0));
                                func.emit_op(OP_PUSH, const_idx as i32);
                                func.emit_op(OP_PUSH, key_idx as i32);
                                func.emit(OP_DICT_ACCESS);
                            } else {
                                return Err("Expected ']'".to_string());
                            }
                        } else {
                            return Err("Dict key must be string".to_string());
                        }
                    } else {
                        self.parse_expr(func, locals, module)?;
                        self.advance();
                        func.emit(OP_ARRAY_INDEX);
                    }
                } else if let Some(idx) = locals.iter().position(|n| n == &name) {
                    func.emit_op(OP_PUSHLOCAL, idx as i32);
                } else {
                    func.emit_op(OP_PUSH, module.add_constant(Constant::Integer(0)) as i32);
                }
            }
            Token::LParen => {
                self.parse_expr(func, locals, module)?;
                self.advance();
            }
            _ => return Err(format!("Unexpected: {:?}", tok)),
        }
        Ok(())
    }
    fn parse_dict_entries(
        &mut self,
        func: &mut BytecodeFunction,
        locals: &mut Vec<String>,
        module: &mut BytecodeModule,
    ) -> Result<Vec<(String, i32)>, String> {
        let mut entries = Vec::new();
        while !matches!(self.current(), Token::RBrace) && !matches!(self.current(), Token::Eof) {
            if matches!(self.current(), Token::Newline) {
                self.advance();
                continue;
            }
            let key = match self.current().clone() {
                Token::String(s) => {
                    self.advance();
                    s
                }
                Token::Identifier(name) => {
                    self.advance();
                    name
                }
                _ => return Err("Expected string or identifier for dict key".to_string()),
            };
            self.advance();
            let val_const_idx = self.parse_simple_constant(func, &locals, module)?;
            entries.push((key, val_const_idx));
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }
        }
        if !matches!(self.current(), Token::RBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance();
        Ok(entries)
    }
    fn parse_simple_constant(
        &mut self,
        func: &mut BytecodeFunction,
        _locals: &[String],
        module: &mut BytecodeModule,
    ) -> Result<i32, String> {
        match self.current() {
            Token::Number(n) => {
                let idx = module.add_constant(Constant::Integer(*n));
                self.advance();
                let _ = func.emit_op(OP_PUSH, idx as i32);
                Ok(idx as i32)
            }
            Token::String(s) => {
                let idx = module.add_constant(Constant::String(s.clone()));
                self.advance();
                let _ = func.emit_op(OP_PUSH, idx as i32);
                Ok(idx as i32)
            }
            Token::True => {
                let idx = module.add_constant(Constant::Boolean(true));
                self.advance();
                let _ = func.emit_op(OP_PUSH, idx as i32);
                Ok(idx as i32)
            }
            Token::False => {
                let idx = module.add_constant(Constant::Boolean(false));
                self.advance();
                let _ = func.emit_op(OP_PUSH, idx as i32);
                Ok(idx as i32)
            }
            _ => Err("Dict values must be constant (number, string, boolean)".to_string()),
        }
    }
}

pub fn compile(source: &str, name: &str) -> Result<BytecodeModule, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokens();
    let mut parser = Parser::new(tokens);
    let mut module = BytecodeModule::new(name);
    while !matches!(parser.current(), Token::Eof) {
        parser.skip_nl();
        if matches!(parser.current(), Token::Eof) {
            break;
        }
        parser.parse_fn(&mut module)?;
        parser.skip_nl();
    }
    Ok(module)
}

fn print_usage() {
    println!("BerkeBex v0.2 - BerkeOS Executable Compiler");
    println!("Usage: berkebex compile <input> [-o <output.bex>]");
    println!("       berkebex --python <input.py> [-o <output.bex>] [--verbose]");
    println!("       berkebex info <file.bex>");
    println!("       berkebex parse [-s <source>] | <file>");
    println!("Languages: .bepy (Python subset) | .c (C subset)");
    println!("Functions: print(), println(), input(), sleep(), rand()");
    println!("Options: --no-gui-guard  Disable GUI framework runtime hooks");
}

fn check_gui_warnings(source: &str, verbose: bool) {
    let patterns = [
        ("tkinter", "Tkinter GUI not supported in BerkeOS"),
        ("pygame", "Pygame not supported in BerkeOS"),
        ("pyqt", "PyQt not supported in BerkeOS"),
        ("pyside", "PySide not supported in BerkeOS"),
        ("wxpython", "wxPython not supported in BerkeOS"),
        ("gtk", "GTK not supported in BerkeOS"),
        ("matplotlib", "matplotlib GUI not supported in BerkeOS"),
        ("turtle", "turtle graphics not supported in BerkeOS"),
        ("cv2", "OpenCV GUI not supported in BerkeOS"),
    ];

    let lower = source.to_lowercase();
    for (pattern, msg) in patterns {
        if lower.contains(pattern) {
            eprintln!("Warning: {}", msg);
            if verbose {
                eprintln!("  (Hint: Remove '{}' for BerkeOS compatibility)", pattern);
            }
        }
    }
}

/// Compile Python source to .bex bytecode using the full pipeline
fn compile_python(input_file: &str, output_file: &str, verbose: bool) -> Result<(), String> {
    // Step 1: Read source
    if verbose {
        eprintln!("[VERBOSE] Reading source from: {}", input_file);
    }
    let source =
        fs::read_to_string(input_file).map_err(|e| format!("Failed to read file: {}", e))?;

    // Step 2: Check for GUI patterns
    if verbose {
        eprintln!("[VERBOSE] Checking for unsupported GUI patterns...");
    }
    check_gui_warnings(&source, verbose);

    // Step 3: Parse Python source with rustpython-parser
    if verbose {
        eprintln!("[VERBOSE] Parsing Python source...");
    }
    let ast = parser::parse_python(&source).map_err(|e| format!("Parse error: {}", e))?;
    if verbose {
        eprintln!("[VERBOSE] Parsed {} statements", ast.len());
    }

    // Step 4: Convert AST to IR
    if verbose {
        eprintln!("[VERBOSE] Converting AST to IR...");
    }
    let name = Path::new(input_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program");
    let ir_module = ir::ast_to_ir(&ast, name).map_err(|e| format!("IR conversion error: {}", e))?;
    if verbose {
        eprintln!(
            "[VERBOSE] IR module has {} functions",
            ir_module.functions.len()
        );
    }

    // Step 5: Generate bytecode
    if verbose {
        eprintln!("[VERBOSE] Generating bytecode...");
    }
    let bytecode_module = codegen::ir_to_bex(&ir_module);

    // Step 6: Emit .bex binary
    if verbose {
        eprintln!("[VERBOSE] Emitting .bex binary...");
    }
    let bytes = bytecode_module.emit_bex();
    if verbose {
        eprintln!("[VERBOSE] Generated {} bytes", bytes.len());
    }

    // Step 7: Write output file
    if verbose {
        eprintln!("[VERBOSE] Writing to: {}", output_file);
    }
    fs::write(output_file, &bytes).map_err(|e| format!("Failed to write output: {}", e))?;

    Ok(())
}

fn parse_python_source(source: &str) -> Result<(), String> {
    match parser::parse_python(source) {
        Ok(suite) => {
            println!("Parse successful!");
            println!("Statements: {}", suite.len());
            Ok(())
        }
        Err(e) => Err(format!("{}", e)),
    }
}

fn handle_parse(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: berkebex parse [-s <source>] | <file>");
        return;
    }

    if args[0] == "-s" {
        if args.len() < 2 {
            eprintln!("Error: -s requires source argument");
            return;
        }
        match parse_python_source(&args[1]) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        }
    } else {
        let file = &args[0];
        match fs::read_to_string(file) {
            Ok(source) => match parse_python_source(&source) {
                Ok(_) => {}
                Err(e) => eprintln!("Error: {}", e),
            },
            Err(e) => eprintln!("Error reading file: {}", e),
        }
    }
}

fn compile_file(input_file: &str) -> Result<Vec<u8>, String> {
    let source = fs::read_to_string(input_file).map_err(|e| e.to_string())?;
    let ext = Path::new(input_file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let name = Path::new(input_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program");
    match ext {
        "py" | "bepy" | "c" | "bexs" => {
            let m = compile(&source, name)?;
            Ok(m.emit_bex())
        }
        _ => Err(format!("Unknown: {}", ext)),
    }
}

fn print_info(file: &str) -> Result<(), String> {
    let bytes = fs::read(file).map_err(|e| e.to_string())?;
    if bytes.len() < 8 {
        return Err("Too short".to_string());
    }
    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if magic != 0x42455831 {
        return Err("Invalid .bex".to_string());
    }
    let name_len = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
    let name = String::from_utf8_lossy(&bytes[8..8 + name_len]).to_string();
    println!("Magic: 0x{:08X} | Name: {}", magic, name);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    if args[1] == "--python" {
        if args.len() < 3 {
            eprintln!("Error: --python requires an input file");
            print_usage();
            return;
        }
        let input = &args[2];
        let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());
        let no_gui_guard = args.contains(&"--no-gui-guard".to_string());
        if no_gui_guard {
            eprintln!("Note: GUI checks disabled");
        }
        let output = extract_output_file(&args, "--python");

        match compile_python(input, &output, verbose) {
            Ok(()) => println!("{} -> {}", input, output),
            Err(e) => eprintln!("Error: {}", e),
        }
        return;
    }

    if args.len() < 3 {
        print_usage();
        return;
    }
    match args[1].as_str() {
        "compile" | "c" => {
            let input = &args[2];
            let no_gui_guard = args.contains(&"--no-gui-guard".to_string());
            if no_gui_guard {
                eprintln!("Note: GUI checks disabled");
            }
            let output = if args.len() > 4 && args[3] == "-o" {
                args[4].clone()
            } else {
                format!(
                    "{}.bex",
                    Path::new(input).file_stem().unwrap().to_str().unwrap()
                )
            };
            match compile_file(input) {
                Ok(bytes) => {
                    fs::write(&output, &bytes).expect("Write error");
                    println!("{} -> {}", input, output);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "info" => {
            if args.len() > 2 {
                if let Err(e) = print_info(&args[2]) {
                    eprintln!("Error: {}", e);
                }
            } else {
                print_usage();
            }
        }
        "parse" => {
            handle_parse(&args[2..]);
        }
        "help" | "-h" | "--help" => print_usage(),
        _ => {
            eprintln!("Unknown: {}", args[1]);
            print_usage();
        }
    }
}

fn extract_output_file(args: &[String], mode: &str) -> String {
    let mut found_o = false;
    for arg in args.iter().skip(1) {
        if found_o {
            return arg.clone();
        }
        if arg == "-o" {
            found_o = true;
        }
    }
    match mode {
        "--python" => {
            let input_idx = args.iter().position(|a| a == "--python").unwrap_or(1);
            let input = args
                .get(input_idx + 1)
                .map(|s| s.as_str())
                .unwrap_or("program");
            format!(
                "{}.bex",
                Path::new(input).file_stem().unwrap().to_str().unwrap()
            )
        }
        _ => "a.out".to_string(),
    }
}
