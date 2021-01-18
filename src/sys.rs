extern crate alloc;

use alloc::boxed::Box;
use core::alloc::Layout;
use core::fmt;
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;
use spin::Mutex;

use crate::hal::{HTIFConsole, HTIFPowerDown};

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
        $crate::sys::_print(format_args!($fmt, $($arg)*));
    });
}

#[macro_export(local_inner_macros)]
macro_rules! println {
    ($fmt:expr) => ({
        $crate::sys::_print_str($fmt);
        $crate::sys::_print_str("\n");
    });
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
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
