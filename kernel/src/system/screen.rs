use mystd::{byte_value::ByteValue, slice::slice2d::{self, traits::{MutSlice2dTrait, Slice2dTrait}, MutSlice2d}};

use super::hal::framebuffer::Framebuffer;

#[derive(Debug)]
pub enum ScreenError {
    NotEnoughMemory { required: ByteValue },
    ResolutionUnsupported { nearest_width: usize, nearest_height: usize },
    CouldNotCreateFramebuffer,
    CouldNotCreateVRam,
}

pub struct Screen<'a, T> where T: Copy {
    vram: MutSlice2d<'a, T>,
    framebuffer: MutSlice2d<'a, T>,
    framebuffer_vc_address: u32,
}

impl<'a, T> Screen<'a, T> where T: Copy {
    const BYTES_PER_PIXEL: usize = core::mem::size_of::<T>();

    pub fn try_create_in_slice(slice: &mut [u8], width: usize, height: usize) -> Result<Self, ScreenError> {
        let required_size_bytes = 2 * width * height * Self::BYTES_PER_PIXEL;
        let memory: &mut [T] = unsafe {
            slice.align_to_mut().1
        };
        if memory.len() < required_size_bytes {
            return Err(ScreenError::NotEnoughMemory { required: ByteValue::from_bytes(required_size_bytes as u64) })
        }
        let vram = slice2d::MutSlice2d::with_mut_slice(memory, width, width, height).ok_or(ScreenError::CouldNotCreateVRam)?.0;

        // todo create framebuffer
        let fb = Framebuffer::new(width as u32, height as u32, Self::BYTES_PER_PIXEL as u32 * 8).ok_or(ScreenError::CouldNotCreateFramebuffer)?;
        
        let framebuffer = unsafe {
            slice2d::MutSlice2d::from_raw_parts(fb.ptr.cast(), fb.width_px as usize, fb.pitch_bytes as usize / Self::BYTES_PER_PIXEL, fb.height_px as usize)
        };

        Ok(Screen { vram, framebuffer, framebuffer_vc_address: fb.base_address })
    }

    pub fn draw<F: Fn(&mut MutSlice2d<'a, T>)> (&mut self, f: F) {
        f(&mut self.vram)
    }

    pub fn present(&mut self) {
        crate::peripherals::dma::one_shot_copy2d(&self.vram.as_slice2d(), &mut self.framebuffer);
        //crate::peripherals::dma::one_shot_copy(self.vram.buf_slice(), self.framebuffer_vc_address as *mut T);
        //crate::peripherals::dma::one_shot_copy(self.vram.buf_slice(), self.framebuffer.as_mut_ptr());
        // swap buffers and copy the formerly current buffer to the framebuffer
        unsafe {
            //self.framebuffer.copy_buf_unchecked(&self.vram);
        }
    }
}

