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

fn rustsbi_info() {
    println!("[rustsbi] RustSBI version {}", rustsbi::VERSION);
    println!("{}", rustsbi::LOGO);
}

fn platform_info(kernal_entry: usize) {
    use riscv::register::{
        medeleg, mideleg,
        misa::{self, MXL},
    };
    println!(include_str!("logo.txt"));
    println!("Platform: Terminus (Version {})", env!("CARGO_PKG_VERSION"));
    let isa = misa::read();
    if let Some(isa) = isa {
        let mxl_str = match isa.mxl() {
            MXL::XLEN32 => "RV32",
            MXL::XLEN64 => "RV64",
            MXL::XLEN128 => "RV128",
        };
        print!("[rustsbi] misa: {}", mxl_str);
        for ext in 'A'..='Z' {
            if isa.has_extension(ext) {
                print!("{}", ext);
            }
        }
        println!("");
    }
    println!("mideleg: {:#x}", mideleg::read().bits());
    println!("medeleg: {:#x}", medeleg::read().bits());
    println!("Kernel entry: {:x}", kernal_entry);
}

#[export_name = "main"]
fn main(_hartid: usize, dtb_pa: usize) -> ! {
    use riscv::register::{
        mepc, mhartid,
        mstatus::{self, MPP},
    };
    init();
    extern "C" {
        static payload_bin: usize;
    }
    let payload_start = unsafe { &payload_bin } as *const _ as usize;
    rustsbi_info();
    platform_info(payload_start);
    unsafe {
        mepc::write(payload_start);
        mstatus::set_mpp(MPP::Supervisor);
        rustsbi::enter_privileged(mhartid::read(), dtb_pa)
    }
}
