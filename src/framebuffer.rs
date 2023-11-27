
use crate::peripherals::mailbox;

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
        use mailbox::*;
        let mut mailbox = mailbox::Mailbox::<256>::new();
        let mut result = Framebuffer{
            ptr: 0 as *mut u8,
            size_bytes: 0,
            width_px: 0,
            height_px: 0,
            bits_per_pixel: 0,
            pitch_bytes: 0,
        };
        let messages = [
            PropertyMessageRequest::FbSetPhysicalDimensions{width_px: 1280, height_px: 720},
            PropertyMessageRequest::FbSetVirtualDimensions{width_px: 1280, height_px: 720},
            PropertyMessageRequest::FbSetDepth{bpp: 32},
            PropertyMessageRequest::Null
        ];
    
        if let Ok(response) = mailbox.request(8, &messages) 
        {
            use PropertyMessageResponse;
            match response {
                [
                    PropertyMessageResponse::FbSetPhysicalDimensions{width_px: ph_w, height_px: ph_h},
                    PropertyMessageResponse::FbSetVirtualDimensions{width_px: v_w, height_px: v_h},
                    PropertyMessageResponse::FbSetDepth{bpp},
                    PropertyMessageResponse::Null
                ] => {
                    result.width_px = v_w as usize;
                    result.height_px = v_h as usize;
                    result.bits_per_pixel = bpp as usize;
                }, 
                _ => return None
            }
        }
        else 
        {
            return None;
        }
    
        let messages: [PropertyMessageRequest; 3] = [
            PropertyMessageRequest::FbAllocateBuffer {
                alignment_bytes: 16
            },
            PropertyMessageRequest::FbGetPitch,
            PropertyMessageRequest::Null
        ];
    
        if let Ok(response) = mailbox.request(8, &messages) 
        {
            use PropertyMessageResponse;
            match response {
                [
                    PropertyMessageResponse::FbAllocateBuffer{
                        base_address_bytes,
                        size_bytes,
                    },
                    PropertyMessageResponse::FbGetPitch{bytes_per_line},
                    PropertyMessageResponse::Null
                ] => {
                    result.ptr = base_address_bytes as *mut u8;
                    result.size_bytes = size_bytes as usize;
                    result.pitch_bytes = bytes_per_line as usize;
                }, 
                _ => return None
            }
        }
        else 
        {
            return None;
        }

        Some(result)
    }

    pub fn set_pixel(&self, x: usize, y: usize, value: u32) {
        unsafe { *self.ptr.add(self.pitch_bytes * y + x*4).cast::<u32>() = value; }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.size_bytes) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.size_bytes) }
    }
}
