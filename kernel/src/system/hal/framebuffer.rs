use core::fmt::DebugStruct;

pub mod color {
    pub const WHITE: u32 = 0xff_ff_ff_ff;
    pub const BLACK: u32 = 0xff_00_00_00;
    pub const RED: u32 = 0xff_00_00_ff;
    pub const GREEN: u32 = 0xff_00_ff_00;
    pub const BLUE: u32 = 0xff_ff_00_00;
}

mod tags {
    pub const FB_ALLOCATE_BUFFER: u32 = 0x00040001;
    pub const FB_RELEASE_BUFFER: u32 = 0x00048001;
    pub const FB_GET_PHYSICAL_DIMENSIONS: u32 = 0x00040003;
    pub const FB_TEST_PHYSICAL_DIMENSIONS: u32 = 0x00044003;
    pub const FB_SET_PHYSICAL_DIMENSIONS: u32 = 0x00048003;
    pub const FB_GET_VIRTUAL_DIMENSIONS: u32 = 0x00040004;
    pub const FB_TEST_VIRTUAL_DIMENSIONS: u32 = 0x00044004;
    pub const FB_SET_VIRTUAL_DIMENSIONS: u32 = 0x00048004;
    pub const FB_GET_DEPTH: u32 = 0x00040005;
    pub const FB_TEST_DEPTH: u32 = 0x00044005;
    pub const FB_SET_DEPTH: u32 = 0x00048005;
    pub const FB_GET_PIXEL_ORDER: u32 = 0x00040006;
    pub const FB_TEST_PIXEL_ORDER: u32 = 0x00044006;
    pub const FB_SET_PIXEL_ORDER: u32 = 0x00048006;
    pub const FB_GET_ALPHA_MODE: u32 = 0x00040007;
    pub const FB_TEST_ALPHA_MODE: u32 = 0x00044007;
    pub const FB_SET_ALPHA_MODE: u32 = 0x00048007;
    pub const FB_GET_PITCH: u32 = 0x00040008;
    pub const FB_GET_VIRTUAL_OFFSET: u32 = 0x00040009;
    pub const FB_TEST_VIRTUAL_OFFSET: u32 = 0x00044009;
    pub const FB_SET_VIRTUAL_OFFSET: u32 = 0x00048009;

    #[repr(u32)]
    #[derive(Copy, Clone)]
    pub enum PixelOrder {
        Bgr = 0,
        Rgb = 1,
    }

    #[repr(u32)]
    #[derive(Copy, Clone)]
    pub enum AlphaMode {
        Enabled0Opaque = 0,
        Enabled0Transparent = 1,
        Ignored,
    }

    pub struct FbDepth {
        pub bits_per_pixel: u32,
    }

    pub struct FbPitch {
        pub bytes_per_line: u32,
    }

    #[derive(Clone, Copy)]
    pub struct FbDimensions {
        pub width_px: u32,
        pub height_px: u32,
    }

    pub struct FbOffset {
        pub x_px: u32,
        pub y_px: u32,
    }

    pub struct FbAllocate {
        pub alignment_bytes: u32,
    }

    pub type Palette = [u32; 256];

    pub struct PaletteChange<const N: usize> {
        offset: u32,
        length: u32,
        values: [u32; N],
    }
}

pub struct Framebuffer {
    ptr: *mut u8,
    pub size_bytes: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub bits_per_pixel: u32,
    pub pitch_bytes: u32,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        use crate::peripherals::mailbox::*;
        use tags::*;
        let mut mailbox = Mailbox::<64>::new();
        let dimensions = FbDimensions {
            width_px: width,
            height_px: height,
        };
        *mailbox
            .push_request(tags::FB_SET_PHYSICAL_DIMENSIONS, 8)
            .ok()? = dimensions;

        *mailbox
            .push_request(tags::FB_SET_VIRTUAL_DIMENSIONS, 8)
            .ok()? = dimensions;
        mailbox
            .push_request_zeroed(tags::FB_SET_VIRTUAL_OFFSET, 8)
            .ok()?;

        *mailbox.push_request(tags::FB_SET_DEPTH, 4).ok()? = 32_u32;
        *mailbox.push_request(tags::FB_SET_PIXEL_ORDER, 4).ok()? = PixelOrder::Bgr;

        // alignment to page size
        *mailbox.push_request(tags::FB_ALLOCATE_BUFFER, 8).ok()? = 4096_u32;

        mailbox.push_request_empty(tags::FB_GET_PITCH, 4).ok()?;

        let mut responses = mailbox.submit_messages(8).ok()?;
        let (width_px, height_px) = *responses.next()?.ok()?.try_value_as()?;
        // FbSetVirtualDimensions {..},
        let _ = responses.next();
        // FbSetVirtualOffset { .. },
        let _ = responses.next();
        let bits_per_pixel: u32 = *responses.next()?.ok()?.try_value_as()?;
        // FbSetPixelOrder { .. },
        responses.next();

        let (base_address_bytes, size_bytes): (u32, u32) =
            *responses.next()?.ok()?.try_value_as()?;
        let pitch_bytes: u32 = *responses.next()?.ok()?.try_value_as()?;

        let ptr: *mut u8 = (0x3FFFFFFF & base_address_bytes) as *mut u8;
        Some(Self {
            width_px,
            height_px,
            ptr,
            size_bytes,
            bits_per_pixel,
            pitch_bytes,
        })
    }

    pub fn set_pixel_a8b8g8r8(&self, x: u32, y: u32, value: u32) {
        let bytes_per_pixel = self.bits_per_pixel >> 3;
        if x < self.width_px && y < self.height_px {
            unsafe {
                *self
                    .ptr
                    .add((self.pitch_bytes * y + x * bytes_per_pixel) as usize)
                    .cast::<u32>() = value;
            }
        }
    }

    pub fn clear(&self, abgr: u32) {
        self.as_mut_pixels().fill(abgr);
    }

    // pub fn as_pixels(&self) -> &[u32] {
    //     unsafe { core::slice::from_raw_parts(self.ptr.cast::<u32>(), self.size_bytes / 4) }
    // }

    fn as_mut_pixels(&self) -> &mut [u32] {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr.cast::<u32>(), self.size_bytes as usize >> 2)
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.size_bytes as usize) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.size_bytes as usize) }
    }

    pub fn write_text(&self, text: &[u8], font: &[u64], mapping: impl Fn(u8) -> u8) {
        let repeat = (2, 2);
        let offset = (40, 48);
        let size = (self.width_px - 2 * offset.0, self.height_px - 2 * offset.1);
        let columns = size.0 as usize / (repeat.0 * 8);
        for y in 0..size.1 {
            let yy = y as usize / repeat.1;
            for x in 0..size.0 {
                let xx = x as usize / repeat.0;
                let char_index = (xx / 8, yy / 8);
                let linear_index = char_index.1 * columns + char_index.0;
                let ch = text.get(linear_index).copied().unwrap_or_default();
                let char = font[mapping(ch) as usize % font.len()];
                let char_subpixel = (xx % 8, yy % 8);
                if (char << ((7 - char_subpixel.1) * 8 + char_subpixel.0)) & (1_u64 << 63) == 0 {
                    self.set_pixel_a8b8g8r8(x + offset.0, y + offset.1, 0xFF0000AA);
                } else {
                    self.set_pixel_a8b8g8r8(x + offset.0, y + offset.1, 0xFFFFFFFF);
                }
            }
        }
    }
}
