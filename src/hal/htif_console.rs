use crate::htif::{htif_wait32, htif_write32};
use core::convert::Infallible;
use embedded_hal::serial::{Read, Write};

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
