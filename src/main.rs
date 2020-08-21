use lc3::vm::{Args, VM};

const PC_START: u16 = 0x3000;

fn main() {
    let mut vm = VM::boot(Args {
        offset: PC_START,
        image: None,
    }).unwrap();

    while vm.is_running() {
        if let Err(code) = vm.next() {
            eprintln!("error {}, halting...", code);
            vm.abort();
        }
    }
}
