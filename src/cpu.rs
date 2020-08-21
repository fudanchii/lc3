use crate::register::{Flag, Mode, Register, R};
use crate::{bit, reg_1st, reg_2nd, sign_extend};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::convert::{TryFrom, TryInto};

pub type CycleResult = Result<(), String>;

#[derive(Debug, FromPrimitive)]
#[repr(u16)]
pub enum OpCode {
    BR,   // 0000
    ADD,  // 0001
    LD,   // 0010
    ST,   // 0011
    JSR,  // 0100
    AND,  // 0101
    LDR,  // 0110
    STR,  // 0111
    RTI,  // 1000
    NOT,  // 1001
    LDI,  // 1010
    STI,  // 1011
    JMP,  // 1100 // JMP R7 == RET
    RES,  // 1101 // reserved
    LEA,  // 1110
    TRAP, // 1111
}

impl TryFrom<u16> for OpCode {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        OpCode::from_u16(val).ok_or(format!("unknown opcode `{}`", val))
    }
}

pub struct CPU {
    register: Register,
    memory: [u16; u16::MAX as usize + 1],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register: Register::new(),
            memory: [0; u16::MAX as usize + 1],
        }
    }

    pub fn tick(&mut self) -> CycleResult {
        let boot_addr = self.register.read_incr(R::PC);
        let instr: u16 = self.mem_read(boot_addr);
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
            OpCode::RES => self.mnemonic_res(instr)?,
            OpCode::LEA => self.mnemonic_lea(instr)?,
            OpCode::TRAP => self.mnemonic_trap(instr)?,
        }

        Ok(())
    }

    pub fn reg_store(&mut self, r: R, val: u16) {
        self.register.write(r, val);
    }

    pub fn reg_load(&self, r: R) -> u16 {
        self.register.read(r)
    }

    pub fn mem_read(&self, addr: u16) -> u16 {
        self.memory[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, val: u16) {
        self.memory[addr as usize] = val;
    }

    fn mnemonic_br(&mut self, args: u16) -> CycleResult {
        let offset = sign_extend(args & 0x9, 9);

        let nzp = (args >> 9) & 0x7;
        if nzp == 0 {
            self.register
                .write(R::PC, self.register.read(R::PC).wrapping_add(offset));
            return Ok(());
        }

        let n: bool = bit(args, 11) == 1;
        let z: bool = bit(args, 10) == 1;
        let p: bool = bit(args, 9) == 1;

        let flag = self.register.get_flag()?;

        if (n && flag == Flag::Negative)
            || (z && flag == Flag::Zero)
            || (p && flag == Flag::Positive)
        {
            self.register
                .write(R::PC, self.register.read(R::PC).wrapping_add(offset));
        }

        Ok(())
    }

    fn mnemonic_imm5_or_sr2<F>(&mut self, args: u16, func: F) -> CycleResult
    where
        F: Fn(u16, u16) -> u16,
    {
        let r0: R = reg_1st(args)?;
        let r1: R = reg_2nd(args)?;
        let imm_flag: u16 = bit(args, 5);

        if imm_flag == 1 {
            self.register.write(
                r0,
                func(self.register.read(r1), sign_extend(args & 0x1f, 5)),
            );
        } else {
            let r2: R = (args & 0x7).try_into()?;
            self.register
                .write(r0, func(self.register.read(r1), self.register.read(r2)));
        }
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_add(&mut self, args: u16) -> CycleResult {
        self.mnemonic_imm5_or_sr2(args, |r1, r2| r1.wrapping_add(r2))
    }

    fn mnemonic_and(&mut self, args: u16) -> CycleResult {
        self.mnemonic_imm5_or_sr2(args, |r1, r2| r1 & r2)
    }

    fn mnemonic_ldi(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let pc_offset = sign_extend(args & 0x1ff, 9);
        self.register.write(
            r0,
            self.mem_read(self.mem_read(self.register.read(R::PC).wrapping_add(pc_offset))),
        );
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_ld(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let offset: u16 = sign_extend(args & 0x01ff, 9);
        self.register.write(
            r0,
            self.mem_read(self.register.read(R::PC).wrapping_add(offset)),
        );
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_st(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let offset: u16 = sign_extend(args & 0x01ff, 9);
        self.mem_write(
            self.register.read(R::PC).wrapping_add(offset),
            self.register.read(r0),
        );
        Ok(())
    }

    fn mnemonic_res(&mut self, _: u16) -> CycleResult {
        Err("reserved opcode".to_string())
    }

    fn mnemonic_jsr(&mut self, args: u16) -> CycleResult {
        let mode = bit(args, 11);

        self.register.write(R::_7, self.register.read(R::PC));

        if mode == 1 {
            self.register.write(
                R::PC,
                self.register
                    .read(R::PC)
                    .wrapping_add(sign_extend(args & 0x07ff, 11)),
            );
            return Ok(());
        }

        let r0: R = reg_2nd(args)?;
        self.register.write(R::PC, self.register.read(r0));
        Ok(())
    }

    fn mnemonic_ldr(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let r1: R = reg_2nd(args)?;
        let offset = sign_extend(args & 0x3f, 6);

        self.register.write(
            r0,
            self.mem_read(self.register.read(r1).wrapping_add(offset)),
        );
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_str(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let r1: R = reg_2nd(args)?;
        let offset = sign_extend(args & 0x01ff, 9);

        self.mem_write(
            self.register.read(r1).wrapping_add(offset),
            self.register.read(r0),
        );
        Ok(())
    }

    fn mnemonic_rti(&mut self, _: u16) -> CycleResult {
        if self.register.get_mode()? == Mode::Privilege {
            let addr = self.register.read_incr(R::_6);
            self.register.write(R::PC, self.mem_read(addr));

            let addr = self.register.read_incr(R::_6);
            self.register.write(R::PSR, self.mem_read(addr));
            return Ok(());
        }

        // abort
        // ...

        Err("illegal RTI from user mode".to_string())
    }

    fn mnemonic_not(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let r1: R = reg_2nd(args)?;
        self.register.write(r0, !self.register.read(r1));
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_sti(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let offset = sign_extend(args & 0x1ff, 9);
        self.mem_write(
            self.mem_read(self.register.read(R::PC).wrapping_add(offset)),
            self.register.read(r0),
        );
        Ok(())
    }

    fn mnemonic_jmp(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_2nd(args)?;
        self.register.write(R::PC, self.register.read(r0));
        Ok(())
    }

    fn mnemonic_lea(&mut self, args: u16) -> CycleResult {
        let r0: R = reg_1st(args)?;
        let offset = sign_extend(args & 0x01ff, 9);
        self.register
            .write(r0, self.register.read(R::PC).wrapping_add(offset));
        self.register.update_flag(r0);
        Ok(())
    }

    fn mnemonic_trap(&mut self, args: u16) -> CycleResult {
        self.register.write(R::_7, self.register.read(R::PC));
        self.register.write(R::PC, self.mem_read(args & 0x00ff));
        Ok(())
    }
}
