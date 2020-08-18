use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Copy, FromPrimitive)]
#[repr(u16)]
pub enum R {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    PC,
    PSR,
}

impl TryFrom<u16> for R {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        R::from_u16(val).ok_or(format!("register index out of bound: {}", val))
    }
}

#[derive(FromPrimitive, PartialEq)]
#[repr(u16)]
pub enum Flag {
    Positive = 1 << 0,
    Zero = 1 << 1,
    Negative = 1 << 2,
}

impl TryFrom<u16> for Flag {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        Flag::from_u16(val & 7).ok_or(format!("wrong overflow flag `{}`", val & 7))
    }
}

#[derive(FromPrimitive)]
#[repr(u16)]
pub enum PL {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
}

impl TryFrom<u16> for PL {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        PL::from_u16((val >> 7) & 7).ok_or(format!("wrong priority level `{}`", (val >> 7) & 7))
    }
}

#[derive(FromPrimitive)]
#[repr(u16)]
pub enum Mode {
    Privilege = 0,
    User,
}

impl TryFrom<u16> for Mode {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        Mode::from_u16(val >> 15).ok_or(format!("wrong mode `{}`", val >> 15))
    }
}

pub struct Register([u16; 10]);

impl Register {
    pub fn new() -> Self {
        Register([0; 10])
    }

    pub fn write(&mut self, r: R, val: u16) {
        self.0[r as usize] = val;
    }

    pub fn read(&self, r: R) -> u16 {
        self.0[r as usize]
    }

    pub fn read_incr(&mut self, r: R) -> u16 {
        let val = self.read(r);
        self.incr(r);
        val
    }

    pub fn update_flag(&mut self, r: R) {
        match self.0[r as usize] {
            0 => self.set_flag(Flag::Zero),
            x if x >> 15 == 1 => self.set_flag(Flag::Negative),
            _ => self.set_flag(Flag::Positive),
        }
    }

    pub fn set_flag(&mut self, f: Flag) {
        self.write(R::PSR, self.read(R::PSR) & 0xfff8 | (f as u16));
    }

    pub fn get_flag(&self) -> Result<Flag, String> {
        (self.read(R::PSR) & 7).try_into()
    }

    pub fn set_level(&mut self, lv: PL) {
        self.write(R::PSR, self.read(R::PSR) & 0xfc7f | ((lv as u16) << 7));
    }

    pub fn get_level(&self) -> Result<PL, String> {
        ((self.read(R::PSR) >> 7) & 7).try_into()
    }

    pub fn set_mode(&mut self, m: Mode) {
        self.write(R::PSR, self.read(R::PSR) & 0x7fff | ((m as u16) << 15))
    }

    pub fn get_mode(&self) -> Result<Mode, String> {
        (self.read(R::PSR) >> 15).try_into()
    }

    pub fn incr(&mut self, r: R) {
        self.write(r, self.read(r) + 1);
    }
}
