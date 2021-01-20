use super::io_access::{io_read64, io_write32, io_write64};

const MTIME_OFF: usize = 0xbff8;
const TIMER_OFF: usize = 0x4000;

pub struct Clint {
    base: usize,
}

impl Clint {
    pub fn new(base: usize) -> Clint {
        Clint { base: base }
    }

    pub fn get_mtime(&self) -> u64 {
        io_read64(&((self.base + MTIME_OFF) as u64))
    }

    pub fn set_timer(&self, hart_id: usize, instant: u64) {
        io_write64(
            &mut ((self.base + TIMER_OFF + (hart_id << 3)) as u64),
            instant,
        )
    }

    pub fn send_soft(&self, hart_id: usize) {
        io_write32(&mut ((self.base + (hart_id << 2)) as u32), 1)
    }

    pub fn clear_soft(&self, hart_id: usize) {
        io_write32(&mut ((self.base + (hart_id << 2)) as u32), 0)
    }
}
