use core::fmt::DebugStruct;

pub mod color {
    pub const BLACK: u32 = 0xff_00_00_00;
    pub const RED: u32 = 0xff_00_00_ff;
    pub const GREEN: u32 = 0xff_00_ff_00;
    pub const BLUE: u32 = 0xff_ff_00_00;
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
        let mut mailbox = Mailbox::<256>::new();
        {
            use PropertyMessageRequest::*;
            mailbox.push_tag(FbSetPhysicalDimensions {
                width_px: width,
                height_px: height,
            });
            mailbox.push_tag(FbSetVirtualDimensions {
                width_px: width,
                height_px: height,
            });
            mailbox.push_tag(FbSetVirtualOffset { x_px: 0, y_px: 0 });
            mailbox.push_tag(FbSetDepth { bpp: 32 });
            mailbox.push_tag(FbSetPixelOrder {
                state: PixelOrder::Rgb,
            });
            mailbox.push_tag(FbAllocateBuffer {
                alignment_bytes: 4096,
            });
            mailbox.push_tag(FbGetPitch);
            mailbox.push_tag(Null);
        }

        if mailbox.submit_messages(8).is_ok() {
            let (width_px, height_px) = mailbox.pop_values();
            // FbSetVirtualDimensions {..},
            mailbox.skip_tag();
            // FbSetVirtualOffset { .. },
            mailbox.skip_tag();
            let bits_per_pixel: u32 = mailbox.pop_values();
            // FbSetPixelOrder { .. },
            mailbox.skip_tag();

            let (base_address_bytes, size_bytes): (u32, u32) = mailbox.pop_values();
            let pitch_bytes: u32 = mailbox.pop_values();

            let ptr: *mut u8 = (0x3FFFFFFF & base_address_bytes) as *mut u8;
            Some(Self {
                width_px,
                height_px,
                ptr,
                size_bytes,
                bits_per_pixel,
                pitch_bytes,
            })
        } else {
            return None;
        }
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

    pub fn as_mut_pixels(&self) -> &mut [u32] {
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
        let repeat = (1, 1);
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
