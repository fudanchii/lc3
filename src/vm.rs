use crate::{bit, reg_1st, reg_2nd, sign_extend};
use crate::register::{Flag, Register, R};
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
    RET,
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
        let instr: u16 = self.read_memory(self.register.read_incr(R::PC));
        let opcode: OpCode = (instr >> 12).try_into()?;

        match opcode {
            OpCode::BR => self.mnemonic_br(instr)?,
            OpCode::ADD => self.mnemonic_add(instr)?,
            OpCode::LD => self.mnemonic_ld(instr)?,
            OpCode::ST => self.mnemonic_st(instr)?,
            OpCode::JSR => self.mnemonic_jsr(instr)?,
            OpCode::AND => self.mnemonic_and(instr)?,
            OpCode::LDR => self.mnemonic_ldr(instr)?,
            OpCode::STR => self.mnemonic_str(instr)?,
            OpCode::RTI => self.mnemonic_rti(instr)?,
            OpCode::NOT => self.mnemonic_not(instr)?,
            OpCode::LDI => self.mnemonic_ldi(instr)?,
            OpCode::STI => self.mnemonic_sti(instr)?,
            OpCode::JMP => self.mnemonic_jmp(instr)?,
            OpCode::RET => self.mnemonic_ret(instr)?,
            OpCode::LEA => self.mnemonic_lea(instr)?,
            OpCode::TRAP => self.mnemonic_trap(instr)?,
            _ => return Err(format!("opcode `{:?}` not implemented", opcode)),
        }

        Ok(())
    }

    pub fn abort(&mut self) {
        self.running = false
    }

    pub fn read_memory(&self, addr: u16) -> u16 {
        self.memory[addr as usize]
    }

    pub fn write_memory(&mut self, addr: u16, val: u16) {
        self.memory[addr as usize] = val;
    }

    fn mnemonic_br(&mut self, args: u16) -> Result<(), String> {
        let offset = sign_extend(args & 0x9, 9);

        let nzp = (args >> 9) & 0x7;
        if nzp == 0 {
            self.register.write(
                R::PC,
                self.register.read(R::PC) + offset,
            );
            return Ok(());
        }

        let n: bool = bit(args, 11) == 1;
        let z: bool = bit(args, 10) == 1;
        let p: bool = bit(args,  9) == 1;

        let flag = self.register.get_flag()?;

        if (n && flag == Flag::Negative)
            || (z && flag == Flag::Zero)
            || (p && flag == Flag::Positive)
        {
            self.register.write(
                R::PC,
                self.register.read(R::PC) + sign_extend(offset, 9),
            );
        }

        Ok(())
    }

    fn mnemonic_imm5_or_sr2<F>(&mut self, args: u16, func: F) -> Result<(), String>
    where
        F: Fn(u16, u16) -> u16,
    {
        let r0: R = reg_1st(args)?;
        let r1: R = reg_2nd(args)?;
        let imm_flag: u16 = bit(args, 5);

        if imm_flag == 1 {
            self.register.write(r0, func(self.register.read(r1), sign_extend(args & 0x1f, 5)));
        } else {
            let r2: R = (args & 0x7).try_into()?;
            self.register
                .write(r0, func(self.register.read(r1), self.register.read(r2)));
        }
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_add(&mut self, args: u16) -> Result<(), String> {
        self.mnemonic_imm5_or_sr2(args, |r1, r2| r1 + r2)
    }

    fn mnemonic_and(&mut self, args: u16) -> Result<(), String> {
        self.mnemonic_imm5_or_sr2(args, |r1, r2| r1 & r2)
    }

    fn mnemonic_ldi(&mut self, args: u16) -> Result<(), String> {
        let r0: R = reg_1st(args)?;
        let pc_offset = sign_extend(args & 0x1ff, 9);
        self.register.write(
            r0,
            self.read_memory(self.read_memory(self.register.read(R::PC) + pc_offset)),
        );
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_ld(&mut self, args: u16) -> Result<(), String> {
        let r0: R = reg_1st(args)?;
        let offset: u16 = sign_extend(args & 0x01ff, 9);
        self.register.write(
            r0,
            self.read_memory(self.register.read(R::PC) + offset),
        );
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_st(&mut self, args: u16) -> Result<(), String> {
        let r0: R = reg_1st(args)?;
        let offset: u16 = sign_extend(args & 0x1ff, 9);
        self.write_memory(self.register.read(R::PC) + offset, self.register.read(r0));
        Ok(())
    }

    fn mnemonic_ret(&mut self, _: u16) -> Result<(), String> {
        self.register.write(R::PC, self.register.read(R::_7));
        Ok(())
    }
}
