pub struct Framebuffer {
    ptr: *mut u8,
    pub size_bytes: usize,
    width_px: usize,
    height_px: usize,
    bits_per_pixel: usize,
    pitch_bytes: usize,
}

impl Framebuffer {
    pub fn new() -> Option<Self> {
        use crate::peripherals::mailbox::*;
        let mut mailbox = Mailbox::<256>::new();
        let mut result = Framebuffer {
            ptr: 0 as *mut u8,
            size_bytes: 0,
            width_px: 0,
            height_px: 0,
            bits_per_pixel: 0,
            pitch_bytes: 0,
        };
        use PropertyMessageRequest::*;
        let messages = [
            FbSetPhysicalDimensions {
                width_px: 1920,
                height_px: 1080,
            },
            FbSetVirtualDimensions {
                width_px: 1920,
                height_px: 1080,
            },
            FbSetDepth { bpp: 32 },
            Null,
        ];

        if let Ok(response) = mailbox.request(8, &messages) {
            use PropertyMessageResponse::*;
            match response {
                [FbSetPhysicalDimensions {..}, FbSetVirtualDimensions {
                    width_px,
                    height_px,
                }, FbSetDepth { bpp }, Null] => {
                    result.width_px = width_px as usize;
                    result.height_px = height_px as usize;
                    result.bits_per_pixel = bpp as usize;
                }
                _ => return None,
            }
        } else {
            return None;
        }

        let messages: [PropertyMessageRequest; 3] = [
            FbAllocateBuffer {
                alignment_bytes: 16,
            },
            FbGetPitch,
            Null,
        ];

        if let Ok(response) = mailbox.request(8, &messages) {
            use PropertyMessageResponse::*;
            match response {
                [FbAllocateBuffer {
                    base_address_bytes,
                    size_bytes,
                }, FbGetPitch { bytes_per_line }, Null] => {
                    result.ptr = base_address_bytes as *mut u8;
                    result.size_bytes = size_bytes as usize;
                    result.pitch_bytes = bytes_per_line as usize;
                }
                _ => return None,
            }
        } else {
            return None;
        }

        Some(result)
    }

    pub fn set_pixel_a8r8g8b8(&self, x: usize, y: usize, value: u32) {
        unsafe {
            *self.ptr.add(self.pitch_bytes * y + x * 4).cast::<u32>() = value;
        }
    }

    // pub fn as_pixels(&self) -> &[u32] {
    //     unsafe { core::slice::from_raw_parts(self.ptr.cast::<u32>(), self.size_bytes / 4) }
    // }

    // pub fn as_mut_pixels(&self) -> &mut [u32] {
    //     unsafe { core::slice::from_raw_parts_mut(self.ptr.cast::<u32>(), self.size_bytes / 4) }
    // }

    // pub fn as_slice(&self) -> &[u8] {
    //     unsafe { core::slice::from_raw_parts(self.ptr, self.size_bytes) }
    // }

    // pub fn as_mut_slice(&mut self) -> &mut [u8] {
    //     unsafe { core::slice::from_raw_parts_mut(self.ptr, self.size_bytes) }
    // }
}
