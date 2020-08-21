use crate::cpu::{CycleResult, CPU};
use crate::register::R;
use std::fs::File;
use std::io::Read;

pub struct Args {
    pub image: Option<String>,
    pub offset: u16,
}

pub struct VM {
    cpu: CPU,
    running: bool,
}

impl VM {
    pub fn boot(args: Args) -> Result<Self, String> {
        let mut vm = VM {
            cpu: CPU::new(),
            running: true,
        };
        if let Some(image) = args.image {
            let offset = vm.read_image_from_file(&image)?;
            vm.cpu.reg_store(R::PC, offset);
        } else {
            vm.cpu.reg_store(R::PC, args.offset);
        }
        Ok(vm)
    }

    pub fn read_image_from_file(&mut self, image: &str) -> Result<u16, String> {
        let mut baseloc: [u8; 2] = [0, 0];
        let mut objfile = File::open(image)
            .map_err(|err| format!("{:?}", err.kind()))?;

        objfile.read_exact(&mut baseloc)
            .map_err(|err| format!("{:?}", err.kind()))?;

        let baseloc = u16::from_be_bytes(baseloc);
        let mut instrbuff: [u8; 2] = [0, 0];
        let mut loc = baseloc;
        while let Ok(_) = objfile.read_exact(&mut instrbuff) {
            self.cpu.mem_write(loc, u16::from_be_bytes(instrbuff));
            loc += 1;
        }
        Ok(baseloc)
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn next(&mut self) -> CycleResult {
        self.cpu.tick()
        // h/w interrupt
    }

    pub fn abort(&mut self) {
        self.running = false
    }
}
