pub fn io_write32(ptr: &mut u32, data: u32) {
    unsafe {
        (ptr as *mut u32).write_volatile(data);
    }
}

pub fn io_read32(ptr: &u32) -> u32 {
    unsafe { (ptr as *const u32).read_volatile() }
}
