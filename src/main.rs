#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod hal;
mod sys;
mod trap;

use crate::sys::init_pmp;
use crate::trap::delege_trap;
use hal::{HTIFConsole, HTIFPowerDown};
use sys::{init_heap, ClintIpi, ClintTimer};
use trap::init_trap;

global_asm!(include_str!("crt.S"));
//generate in build.rs
global_asm!(include_str!(concat!(env!("OUT_DIR"), "/payload.S")));

fn init() {
    init_heap();
    use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal;
    init_legacy_stdio_embedded_hal(HTIFConsole {});

    init_trap();

    use rustsbi::init_ipi;
    init_ipi(ClintIpi);
    use rustsbi::init_timer;
    init_timer(ClintTimer);

    use rustsbi::init_reset;
    init_reset(HTIFPowerDown {});

    init_pmp();
    delege_trap();
}

#[export_name = "main"]
fn main(_hartid: usize, dtb_pa: usize) -> ! {
    use riscv::register::{
        mepc, mhartid,
        mstatus::{self, MPP},
    };
    init();
    println!(include_str!("logo.txt"));
    // exit(0);
    extern "C" {
        static payload_bin: usize;
    }
    let payload_start = unsafe { &payload_bin } as *const _ as usize;
    unsafe {
        mepc::write(payload_start);
        mstatus::set_mpp(MPP::Supervisor);
        rustsbi::enter_privileged(mhartid::read(), dtb_pa)
    }
    // unsafe {
    //     asm!(
    //         "
    // j payload_bin
    // ",
    //         options(noreturn)
    //     )
    // }
}
