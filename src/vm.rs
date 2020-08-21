use crate::cpu::{CycleResult, CPU};
use crate::register::R;

pub struct Args {
    pub image: Option<String>,
    pub offset: u16,
}

pub struct VM {
    cpu: CPU,
    running: bool,
}

impl VM {
    pub fn boot(args: Args) -> Self {
        let mut vm = VM {
            cpu: CPU::new(),
            running: true,
        };
        vm.cpu.reg_store(R::PC, args.offset);
        vm
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
