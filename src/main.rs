#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod hal;
mod sys;

use hal::{HTIFConsole, HTIFPowerDown};
use sys::{exit, init_heap, ClintIpi, ClintTimer};

global_asm!(include_str!("crt.S"));

fn init() {
    init_heap();
    use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal;
    init_legacy_stdio_embedded_hal(HTIFConsole {});

    use rustsbi::init_ipi;
    init_ipi(ClintIpi);
    use rustsbi::init_timer;
    init_timer(ClintTimer);

    use rustsbi::init_reset;
    init_reset(HTIFPowerDown {});
}

#[export_name = "main"]
fn main() -> ! {
    init();
    println!("terminus boot loader!");
    exit(0);
}
