pub mod cpu;
pub mod register;
pub mod vm;

use register::R;
use std::convert::TryInto;

pub fn reg_1st(instr: u16) -> Result<R, String> {
    ((instr >> 9) & 0x7).try_into()
}

pub fn reg_2nd(instr: u16) -> Result<R, String> {
    ((instr >> 6) & 0x7).try_into()
}

pub fn bit(args: u16, nth: u16) -> u16 {
    (args >> nth) & 0x1
}

pub fn sign_extend(x: u16, bitcount: u16) -> u16 {
    if bit(x, bitcount - 1) == 1 {
        x | 0xffff << bitcount
    } else {
        x
    }
}
