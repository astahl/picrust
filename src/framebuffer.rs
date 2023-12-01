pub struct Framebuffer {
    ptr: *mut u8,
    pub size_bytes: u32,
    width_px: u32,
    height_px: u32,
    bits_per_pixel: u32,
    pitch_bytes: u32,
}

impl Framebuffer {
    pub fn new() -> Option<Self> {
        use crate::peripherals::mailbox::*;
        let mut mailbox = Mailbox::<256>::new();

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
            FbSetVirtualOffset { x_px: 0, y_px: 0 },
            FbSetDepth { bpp: 32 },
            FbSetPixelOrder { state: PixelOrder::Rgb },
            FbAllocateBuffer {
                alignment_bytes: 4096,
            },
            FbGetPitch,
            //SetOnboardLedStatus { pin_number: Led::Status, status: LedStatus::Off },
            Null,
        ];

        if let Ok(response) = mailbox.request(8, &messages) {
            use PropertyMessageResponse::*;
            return match response {
                [
                    FbSetPhysicalDimensions { width_px, height_px}, 
                    FbSetVirtualDimensions {..}, 
                    FbSetVirtualOffset { .. },
                    FbSetDepth { bpp: bits_per_pixel }, 
                    FbSetPixelOrder { .. },
                    FbAllocateBuffer {
                        base_address_bytes,
                        size_bytes,
                    },
                    FbGetPitch { bytes_per_line: pitch_bytes },
                    //SetOnboardLedStatus { .. },
                    Null] => {
                    let ptr: *mut u8 = (0x3FFFFFFF & base_address_bytes) as *mut u8; // Convert GPU address to ARM address
                    Some(Self {
                        width_px, height_px, ptr, size_bytes, bits_per_pixel, pitch_bytes
                    })
                }
                _ => return None,
            }
        } else {
            return None;
        }
    }

    pub fn set_pixel_a8r8g8b8(&self, x: usize, y: usize, value: u32) {
        unsafe {
            *self.ptr.add(self.pitch_bytes as usize * y + x * 4).cast::<u32>() = value;
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
