#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CtaExtensionRev3 {
    extension_tag: u8,
    revision: u8,
}

impl CtaExtensionRev3 {
    pub const fn check_extension_tag(&self) -> bool {
        self.extension_tag == 0x02
    } 
}
