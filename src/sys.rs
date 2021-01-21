extern crate alloc;

use alloc::boxed::Box;
use core::alloc::Layout;
use core::fmt;
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;
use spin::Mutex;

use crate::hal::{Clint, HTIFConsole, HTIFPowerDown};

pub const CLINT_BASE: usize = 0x02000000;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn oom(_layout: Layout) -> ! {
    loop {}
}

pub fn init_heap() {
    extern "C" {
        static mut _sheap: u8;
        static _heap_size: u8;
    }
    let m_sheap = unsafe { &mut _sheap } as *mut _ as usize;
    let m_heap_size = unsafe { &_heap_size } as *const u8 as usize;
    unsafe {
        ALLOCATOR.lock().init(m_sheap, m_heap_size);
    }
}

//only use in boot loader internally
lazy_static::lazy_static! {
    static ref HTIF_CONSOLE: Mutex<Box<HTIFConsole>> = Mutex::new(Box::new(HTIFConsole {}));
}
struct EarlyConsole;

impl fmt::Write for EarlyConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut stdio = HTIF_CONSOLE.lock();
        for byte in s.as_bytes() {
            use embedded_hal::serial::Write;
            nb::block!(stdio.as_mut().try_write(*byte)).ok();
            nb::block!(stdio.as_mut().try_flush()).ok();
        }
        Ok(())
    }
}

pub(crate) fn _print(args: fmt::Arguments) {
    use fmt::Write;
    EarlyConsole.write_fmt(args).unwrap();
}

pub(crate) fn _print_num(num: u8) {
    let mut stdio = HTIF_CONSOLE.lock();
    use embedded_hal::serial::Write;
    nb::block!(stdio.as_mut().try_write('0' as u8 + num)).ok();
    nb::block!(stdio.as_mut().try_flush()).ok();
}

pub(crate) fn _print_str(s: &str) {
    use fmt::Write;
    EarlyConsole.write_str(s).unwrap();
}

#[macro_export(local_inner_macros)]
macro_rules! print {
    ($fmt:expr) => ($crate::sys::_print_str($fmt));
    ($fmt:expr, $($arg:tt)*) => ({
        $crate::sys::_print(core::format_args!($fmt, $($arg)*));
    });
}

#[macro_export(local_inner_macros)]
macro_rules! println {
    ($fmt:expr) => ({
        $crate::sys::_print_str($fmt);
        $crate::sys::_print_str("\n");
    });
    ($fmt:expr, $($arg:tt)*) => (print!(core::concat!($fmt, "\n"), $($arg)*));
}

//only use in boot loader internally
pub fn exit(code: u8) -> ! {
    print!("exit with ");
    _print_num(code);
    print!("\n");
    use rustsbi::Reset;
    HTIFPowerDown.system_reset(rustsbi::reset::RESET_TYPE_SHUTDOWN, code as usize);
    loop {}
}

#[cfg(feature = "panic-full")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    println!("panic!");
    exit(1);
}

#[cfg(not(feature = "panic-full"))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("panic!");
    exit(1);
}

lazy_static::lazy_static! {
    static ref  CLINT: Clint = Clint::new(CLINT_BASE);
}

use rustsbi::{HartMask, Ipi, Timer};

pub struct ClintIpi;

impl Ipi for ClintIpi {
    fn max_hart_id(&self) -> usize {
        0
    }

    fn send_ipi_many(&mut self, hart_mask: HartMask) {
        for i in 0..=self.max_hart_id() {
            if hart_mask.has_bit(i) {
                CLINT.send_soft(i);
            }
        }
    }
}

pub struct ClintTimer;

impl ClintTimer {
    pub fn get_time(&self) -> u64 {
        CLINT.get_mtime()
    }
}

impl Timer for ClintTimer {
    fn set_timer(&mut self, time_value: u64) {
        let this_mhartid = riscv::register::mhartid::read();
        CLINT.set_timer(this_mhartid, time_value);
        unsafe {
            use riscv::register::{mie, mip};
            mip::clear_stimer();
            mie::set_mtimer();
        }
    }
}

const PMP_R: usize = 0x01;
const PMP_W: usize = 0x02;
const PMP_X: usize = 0x04;
const PMP_A: usize = 0x18;
const PMP_L: usize = 0x80;

const PMP_TOR: usize = 0x08;
const PMP_NA4: usize = 0x10;
const PMP_NAPOT: usize = 0x18;

pub fn init_pmp() {
    //enable all pmp region
    unsafe {
        use riscv::register::{pmpaddr0, pmpcfg0};
        pmpaddr0::write(usize::MAX);
        pmpcfg0::write(PMP_NAPOT | PMP_R | PMP_W | PMP_X)
    }
}
