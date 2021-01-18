use crate::io_access::{io_read32, io_write32};
use core::convert::Infallible;
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

pub fn htif_write32(cmd: u32, data: u32) {
    unsafe {
        io_write32(&mut HTIF_TO_HOST.to_host_cmd, 0x100 << 16);
        io_write32(&mut HTIF_TO_HOST.to_host_cmd, cmd);
        io_write32(&mut HTIF_TO_HOST.to_host_data, data);
    }
}

pub fn htif_wait32() -> nb::Result<(), Infallible> {
    let busy = unsafe { io_read32(&mut HTIF_TO_HOST.to_host_cmd) != 0 };
    if !busy {
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}
