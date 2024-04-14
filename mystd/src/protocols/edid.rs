use core::fmt::Debug;

pub mod cta_rev3;
pub mod edid_ver14;

#[derive(Debug, PartialEq, Eq)]
pub enum EdidError {
    TryFromSliceError,
    ChecksumError
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union EdidBlock {
    edid: edid_ver14::EdidVer14,
    cta: cta_rev3::CtaExtensionRev3,
    raw_bytes: [u8;128], 
}

impl Debug for EdidBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(edid) = self.try_as_edid() {
            write!(f, "EDID: {:?}", edid)
        } else if let Some(cta) = self.try_as_cta_rev3() {
            write!(f, "CTA: {:?}", cta)
        } else {
            Ok(())
        }
    }
}

impl EdidBlock {
    pub fn try_with_bytes(bytes: &[u8]) -> Result<Self, EdidError> {
        let raw_bytes: [u8;128] = bytes.try_into().map_err(|_| EdidError::TryFromSliceError)?;
        if raw_bytes.into_iter().reduce(u8::wrapping_add) != Some(0) {
            Err(EdidError::ChecksumError)
        } else {
            Ok( Self { raw_bytes } )
        }
    }

    pub fn try_as_edid(&self) -> Option<&edid_ver14::EdidVer14> {
        unsafe {
            if self.edid.check_magic_number() {
                Some (&self.edid)
            } else {
                None
            }
        }
    }

    pub fn try_as_cta_rev3(&self) -> Option<&cta_rev3::CtaExtensionRev3> {
        unsafe {
            if self.cta.check_extension_tag() {
                Some (&self.cta)
            } else {
                None
            }
        }
    }
}

pub struct VideoMode {
    display_aspect_ratio: (u8, u8),
    pixel_aspect_ratio: (u8, u8),
    pixel_clock_hz: u64,
    vertical_clock_millihz: u64,
    horizontal_clock_hz: u64,
    horizontal_sync_width: u32,
    horizontal_sync_offset: u32,
    horizontal_active_pixels: u32,
    horizontal_blanking_pixels: u32, 
    vertical_active_lines: u32,
    vertical_blanking_lines: u32,
    interlaced: bool,
}

#[cfg(test)]
mod tests {
    use crate::collections::ring::RingArray;

    use super::*;

    const EDID_BYTES: [u8; 128] = [
        0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x05, 0xe3, 0x09, 0x19, 0x01, 0x01,
        0x01, 0x01, 0x00, 0x14, 0x01, 0x03, 0x80, 0x40, 0x24, 0x78, 0x0a, 0x5d, 0x95, 0xa3,
        0x59, 0x53, 0xa0, 0x27, 0x0f, 0x50, 0x54, 0xaf, 0xce, 0x00, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x1b, 0x21,
        0x50, 0xa0, 0x51, 0x00, 0x1e, 0x30, 0x48, 0x88, 0x35, 0x00, 0x80, 0x68, 0x21, 0x00,
        0x00, 0x18, 0x02, 0x3a, 0x80, 0xd0, 0x72, 0x38, 0x2d, 0x40, 0x10, 0x2c, 0x45, 0x80,
        0x80, 0x68, 0x21, 0x00, 0x00, 0x1e, 0x00, 0x00, 0x00, 0xfc, 0x00, 0x4c, 0x45, 0x31,
        0x39, 0x4b, 0x30, 0x39, 0x37, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00, 0xfd,
        0x00, 0x38, 0x4c, 0x1e, 0x53, 0x11, 0x00, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
        0x01, 0x3c,
    ];

    const CTA_BYTES: [u8; 128] = [
        0x02, 0x03, 0x26, 0x70, 0x4e, 0x13, 0x04, 0x1f, 0x10, 0x20, 0x21, 0x22, 0x14, 0x05,
        0x11, 0x02, 0x15, 0x06, 0x01, 0x26, 0x09, 0x07, 0x03, 0x15, 0x07, 0x50, 0x83, 0x01,
        0x00, 0x00, 0x67, 0x03, 0x0c, 0x00, 0x20, 0x00, 0xb8, 0x2d, 0x01, 0x1d, 0x80, 0x3e,
        0x73, 0x38, 0x2d, 0x40, 0x7e, 0x2c, 0x45, 0x80, 0x80, 0x68, 0x21, 0x00, 0x00, 0x1e,
        0x01, 0x1d, 0x80, 0xd0, 0x72, 0x1c, 0x16, 0x20, 0x10, 0x2c, 0x25, 0x80, 0x80, 0x68,
        0x21, 0x00, 0x00, 0x9e, 0x01, 0x1d, 0x00, 0xbc, 0x52, 0xd0, 0x1e, 0x20, 0xb8, 0x28,
        0x55, 0x40, 0x80, 0x68, 0x21, 0x00, 0x0, 0x1e, 0x8c, 0x0a, 0xd0, 0x90, 0x20, 0x40,
        0x31, 0x20, 0x0c, 0x40, 0x55, 0x00, 0x90, 0x2c, 0x11, 0x00, 0x00, 0x18, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x3b,
    ];

    #[test]
    fn test_try_with_bytes() {
        assert_eq!(EdidError::TryFromSliceError, EdidBlock::try_with_bytes(&[]).expect_err("should fail"));
        assert_eq!(EdidError::TryFromSliceError, EdidBlock::try_with_bytes(&[0,1,2,3]).expect_err("should fail"));
        let mut corrupted = EDID_BYTES;
        corrupted[20] = !corrupted[20];
        assert_eq!(EdidError::ChecksumError, EdidBlock::try_with_bytes(&corrupted).expect_err("should fail"));
        EdidBlock::try_with_bytes(&EDID_BYTES).expect("try_with_bytes should work for EDID");
        EdidBlock::try_with_bytes(&CTA_BYTES).expect("try_with_bytes should work for CTA");
    }

    #[test]
    fn test_edid_block() {
        let edid_block = EdidBlock::try_with_bytes(&EDID_BYTES).expect("try_with_bytes should work for EDID");
        assert!(edid_block.try_as_edid().is_some());
    }

    #[test]
    fn test_cta_block() {
        let edid_block = EdidBlock::try_with_bytes(&CTA_BYTES).expect("try_with_bytes should work for CTA");
        assert!(edid_block.try_as_cta_rev3().is_some());
    }

    #[test]
    fn test_edid_debug_fmt() {
        use core::fmt::Write;
        let mut buf = RingArray::<u8, 4096>::new();
        write!(&mut buf, "{:?}", EdidBlock::try_with_bytes(&EDID_BYTES).unwrap()).expect("should work");
        assert_eq!("", buf.to_str().unwrap());
    }
}