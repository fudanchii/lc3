use crate::register::{Register, R};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, FromPrimitive)]
#[repr(u16)]
pub enum OpCode {
    BR,
    ADD,
    LD,
    ST,
    JSR,
    AND,
    LDR,
    STR,
    RTI,
    NOT,
    LDI,
    STI,
    JMP,
    RES,
    LEA,
    TRAP,
}

impl TryFrom<u16> for OpCode {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        OpCode::from_u16(val).ok_or(format!("unknown opcode `{}`", val))
    }
}

pub struct Args {
    pub image: Option<String>,
    pub offset: u16,
}

pub struct VM {
    memory: [u16; u16::MAX as usize],
    register: Register,
    running: bool,
}

impl Default for VM {
    fn default() -> Self {
        VM {
            memory: [0; u16::MAX as usize],
            register: Register::new(),
            running: false,
        }
    }
}

impl VM {
    pub fn boot(&mut self, args: Args) {
        self.register.write(R::PC, args.offset);
        self.running = true;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn next(&mut self) -> Result<(), String> {
        let instr: u16 = self.read_memory(self.register.read(R::PC));
        let opcode: OpCode = (instr >> 12).try_into()?;

        match opcode {
            OpCode::ADD => self.mnemonic_add(instr)?,
            OpCode::LDI => self.mnemonic_ldi(instr)?,
            OpCode::AND => self.mnemonic_and(instr)?,
            _ => return Err(format!("opcode `{:?}` not implemented", opcode)),
        }

        self.register.incr(R::PC);
        Ok(())
    }

    pub fn abort(&mut self) {
        self.running = false
    }

    pub fn read_memory(&self, addr: u16) -> u16 {
        self.memory[addr as usize]
    }

    fn sign_extend(x: u16, bitcount: u16) -> u16 {
        if ((x >> (bitcount - 1)) & 1) == 1 {
            x | 0xffff << bitcount
        } else {
            x
        }
    }

    fn mnemonic_add(&mut self, args: u16) -> Result<(), String> {
        let r0: R = ((args >> 9) & 0x7).try_into()?;
        let r1: R = ((args >> 6) & 0x7).try_into()?;
        let imm_flag: u16 = (args >> 5) & 0x1;

        if imm_flag == 1 {
            let imm5: u16 = Self::sign_extend(args & 0x1f, 5);
            self.register.write(r0, self.register.read(r1) + imm5);
        } else {
            let r2: R = (args & 0x7).try_into()?;
            self.register
                .write(r0, self.register.read(r1) + self.register.read(r2));
        }
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_and(&mut self, args: u16) -> Result<(), String> {
        let r0: R = ((args >> 9) & 0x7).try_into()?;
        let r1: R = ((args >> 6) & 0x7).try_into()?;
        let imm_flag: u16 = (args >> 5) & 0x1;

        if imm_flag == 1 {
            let imm5: u16 = Self::sign_extend(args & 0x1f, 5);
            self.register.write(r0, self.register.read(r1) & imm5);
        } else {
            let r2: R = (args & 0x7).try_into()?;
            self.register
                .write(r0, self.register.read(r1) & self.register.read(r2));
        }
        self.register.update_flag(r0);
        Ok(())

    }

    fn mnemonic_ldi(&mut self, args: u16) -> Result<(), String> {
        let r0: R = ((args >> 9) & 0x7).try_into()?;
        let pc_offset = Self::sign_extend(args & 0x1ff, 9);
        self.register.write(
            r0,
            self.read_memory(self.read_memory(self.register.read(R::PC) + pc_offset)),
        );
        self.register.update_flag(r0);
        Ok(())
    }
}
