pub fn io_write32(ptr: *mut u32, data: u32) {
    unsafe { ptr.write_volatile(data) }
}

pub fn io_read32(ptr: *const u32) -> u32 {
    unsafe { ptr.read_volatile() }
}

pub fn io_write64(ptr: *mut u64, data: u64) {
    unsafe { ptr.write_volatile(data) }
}

pub fn io_read64(ptr: *const u64) -> u64 {
    unsafe { ptr.read_volatile() }
}
