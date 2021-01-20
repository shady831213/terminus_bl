use super::io_access::{io_read32, io_write32};
use core::convert::Infallible;
use embedded_hal::serial::{Read, Write};

#[repr(C)]
struct HtifToHost {
    to_host_data: u32,
    to_host_cmd: u32,
}

#[link_section = ".tohost"]
static mut HTIF_TO_HOST: HtifToHost = HtifToHost {
    to_host_data: 0,
    to_host_cmd: 0,
};

fn htif_write32(cmd: u32, data: u32) {
    unsafe {
        io_write32(&mut HTIF_TO_HOST.to_host_cmd, 0x100 << 16);
        io_write32(&mut HTIF_TO_HOST.to_host_cmd, cmd);
        io_write32(&mut HTIF_TO_HOST.to_host_data, data);
    }
}

fn htif_wait32() -> nb::Result<(), Infallible> {
    let busy = unsafe { io_read32(&mut HTIF_TO_HOST.to_host_cmd) != 0 };
    if !busy {
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

pub struct HTIFConsole;

impl Read<u8> for HTIFConsole {
    type Error = Infallible;

    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        Ok(0)
    }
}

impl Write<u8> for HTIFConsole {
    type Error = Infallible;

    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        htif_write32(0x101 << 16, word as u32);
        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        htif_wait32()
    }
}

pub struct HTIFPowerDown;
impl rustsbi::Reset for HTIFPowerDown {
    fn system_reset(&self, _reset_type: usize, reset_reason: usize) -> rustsbi::SbiRet {
        htif_write32(0, (reset_reason as u32) << 16 | 1);
        rustsbi::SbiRet { error: 0, value: 0 }
    }
}
