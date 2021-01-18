use crate::htif::htif_write32;
pub struct HTIFPowerDown;
impl rustsbi::Reset for HTIFPowerDown {
    fn system_reset(&self, _reset_type: usize, reset_reason: usize) -> rustsbi::SbiRet {
        htif_write32(0, (reset_reason as u32) << 16 | 1);
        rustsbi::SbiRet { error: 0, value: 0 }
    }
}
