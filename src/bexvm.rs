// BerkeOS — bexvm.rs
// BerkeBex Virtual Machine - Execute .bex bytecode programs

pub struct BexVM {
    stack: [i64; 256],
    sp: usize,
    locals: [i64; 32],
}

impl BexVM {
    pub fn new() -> Self {
        Self {
            stack: [0; 256],
            sp: 0,
            locals: [0; 32],
        }
    }

    fn push(&mut self, val: i64) {
        if self.sp < 256 {
            self.stack[self.sp] = val;
            self.sp += 1;
        }
    }

    fn pop(&mut self) -> i64 {
        if self.sp > 0 {
            self.sp -= 1;
            self.stack[self.sp]
        } else {
            0
        }
    }

    fn print_str(&self, s: &[u8]) {
        for &b in s {
            crate::serial::write_byte(b);
        }
    }

    fn print_i64(&self, mut val: i64) {
        if val == 0 {
            crate::serial::write_byte(b'0');
            return;
        }
        let neg = val < 0;
        if neg {
            val = -val;
        }
        let mut buf = [0u8; 32];
        let mut idx = 0usize;
        while val > 0 {
            buf[idx] = b'0' + (val % 10) as u8;
            val /= 10;
            idx += 1;
        }
        if neg {
            crate::serial::write_byte(b'-');
        }
        for i in (0..idx).rev() {
            crate::serial::write_byte(buf[i]);
        }
    }

    fn print_const(&self, data: &[u8], offset: &mut usize) {
        let tag = data[*offset];
        *offset += 1;
        match tag {
            1 => {
                let val = i32::from_le_bytes([
                    data[*offset],
                    data[*offset + 1],
                    data[*offset + 2],
                    data[*offset + 3],
                ]);
                *offset += 4;
                self.print_i64(val as i64);
            }
            2 => {
                let val = f64::from_le_bytes([
                    data[*offset],
                    data[*offset + 1],
                    data[*offset + 2],
                    data[*offset + 3],
                    data[*offset + 4],
                    data[*offset + 5],
                    data[*offset + 6],
                    data[*offset + 7],
                ]);
                *offset += 8;
                self.print_i64(val as i64);
                crate::serial::write_byte(b'.');
                crate::serial::write_byte(b'0');
            }
            3 => {
                let len = u32::from_le_bytes([
                    data[*offset],
                    data[*offset + 1],
                    data[*offset + 2],
                    data[*offset + 3],
                ]) as usize;
                *offset += 4;
                for i in 0..len {
                    crate::serial::write_byte(data[*offset + i]);
                }
                *offset += len;
            }
            4 => {
                let b = data[*offset] != 0;
                *offset += 1;
                if b {
                    self.print_str(b"true");
                } else {
                    self.print_str(b"false");
                }
            }
            _ => {}
        }
    }

    pub fn run(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() < 8 {
            return Err("File too small");
        }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != 0x42455831 {
            return Err("Not .bex");
        }
        let version = u16::from_le_bytes([data[4], data[5]]);
        if version != 1 {
            return Err("Unsupported version");
        }
        let name_len = u16::from_le_bytes([data[6], data[7]]) as usize;

        crate::serial::write_str("\r\n[BERUN] Running: ");
        for i in 0..name_len {
            crate::serial::write_byte(data[8 + i]);
        }
        crate::serial::write_str("\r\n\r\n");

        let mut offset = 8 + name_len;
        let const_count = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        for _ in 0..const_count {
            if offset >= data.len() {
                break;
            }
            let tag = data[offset];
            offset += 1;
            match tag {
                1 => offset += 4,
                2 => offset += 8,
                3 => {
                    let len = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]) as usize;
                    offset += 4 + len;
                }
                4 => offset += 1,
                _ => break,
            }
        }

        let func_count = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        for _ in 0..func_count {
            if offset >= data.len() {
                break;
            }
            let fn_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2 + fn_len + 4;
            let instr_count = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4 + (instr_count * 5);
        }

        let mut ip = 0usize;
        let code_start = offset;

        loop {
            let pos = code_start + ip;
            if pos >= data.len() {
                break;
            }
            let opcode = data[pos];
            ip += 1;

            match opcode {
                0 => {} // NOP
                1 => {
                    // PUSH
                    let idx = i32::from_le_bytes([
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                    ]) as usize;
                    ip += 4;
                    let const_off = 8 + name_len + 4;
                    let mut off = const_off;
                    for _ in 0..idx {
                        if off >= data.len() {
                            break;
                        }
                        let t = data[off];
                        off += 1;
                        match t {
                            1 => off += 4,
                            2 => off += 8,
                            3 => {
                                let l = u32::from_le_bytes([
                                    data[off],
                                    data[off + 1],
                                    data[off + 2],
                                    data[off + 3],
                                ]) as usize;
                                off += 4 + l;
                            }
                            4 => off += 1,
                            _ => break,
                        }
                    }
                    if off < data.len() {
                        self.print_const(data, &mut off);
                    }
                }
                2 => {
                    self.push(self.locals[0]);
                } // PUSHLOCAL
                3 => {
                    let v = self.pop();
                    if self.sp < 32 {
                        self.locals[0] = v;
                    }
                } // STORELOCAL
                4 => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                } // ADD
                5 => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a - b);
                } // SUB
                6 => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a * b);
                } // MUL
                7 => {
                    let b = self.pop();
                    let a = self.pop();
                    if b != 0 {
                        self.push(a / b);
                    }
                } // DIV
                8 => {
                    let b = self.pop();
                    let a = self.pop();
                    if b != 0 {
                        self.push(a % b);
                    }
                } // MOD
                9 => {
                    let v = self.pop();
                    self.push(-v);
                } // NEGATE
                10 => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(if a == b { 1 } else { 0 });
                } // CMP
                11 => {
                    let target = i32::from_le_bytes([
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                    ]);
                    ip = target as usize;
                } // JMP
                12 => {
                    let target = i32::from_le_bytes([
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                    ]);
                    ip += 4;
                    if self.pop() == 0 {
                        ip = target as usize;
                    }
                } // JIF
                13 => {
                    let _ = self.pop();
                } // PRINT
                14 => {
                    let _ = self.pop();
                    crate::serial::write_str("\r\n");
                } // PRINTLN
                15 | 16 => {
                    break;
                } // RET/HALT
                17 => {
                    let _ = self.pop();
                } // POP
                18 => {
                    let syscall_id = i32::from_le_bytes([
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                    ]);
                    ip += 4;
                    match syscall_id {
                        0 => {
                            let _ = self.pop();
                        }
                        1 => {
                            self.push(0);
                        }
                        2 => {
                            let _ = self.pop();
                            let _ = self.pop();
                        }
                        _ => {}
                    }
                }
                19 => {
                    let val = self.stack[self.sp - 1];
                    self.push(val);
                }
                20 => {
                    self.push(0);
                }
                21 => {
                    let _ms = self.pop();
                }
                22 => {
                    use core::num::Wrapping;
                    static mut SEED: u64 = 12345;
                    unsafe {
                        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
                        let val = ((SEED >> 16) & 0x7FFF) as i64;
                        self.push(val);
                    }
                }
                _ => {}
            }
        }
        crate::serial::write_str("\r\n[BERUN] Done.\r\n");
        Ok(())
    }
}

impl Default for BexVM {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_bex_file(data: &[u8]) -> Result<(), &'static str> {
    let mut vm = BexVM::new();
    vm.run(data)
}
