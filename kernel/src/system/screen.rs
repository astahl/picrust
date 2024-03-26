use core::usize;

use mystd::{byte_value::ByteValue, slice::slice2d::{self, traits::{MutSlice2dTrait, Slice2dTrait}, MutSlice2d}};

use super::hal::framebuffer::{self, Framebuffer};

#[derive(Debug)]
pub enum ScreenError {
    NotEnoughMemory { required: ByteValue },
    ResolutionUnsupported { nearest_width: usize, nearest_height: usize },
    CouldNotCreateFramebuffer,
    CouldNotCreateVRam,
}

pub enum SwapStrategy<T> {
    SwapAndClear(T),
    SwapAndCopy,
    Swap
}

pub enum PresentStrategy {
    Memcopy,
    Dma,
    Dma2d,
}


pub struct Screen<'a, T> where T: Copy {
    front: MutSlice2d<'a, T>,
    back: MutSlice2d<'a, T>,
    framebuffer: MutSlice2d<'a, T>
    //framebuffer_vc: MutSlice2d<'a, T>,
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
        let vram = slice2d::MutSlice2d::with_mut_slice(memory, width, width, height * 2).ok_or(ScreenError::CouldNotCreateVRam)?.0;

        // todo create framebuffer
        let fb = Framebuffer::new(width as u32, height as u32, Self::BYTES_PER_PIXEL as u32 * 8).ok_or(ScreenError::CouldNotCreateFramebuffer)?;
        
        let framebuffer = unsafe {
            slice2d::MutSlice2d::from_raw_parts(fb.ptr.cast(), fb.width_px as usize, fb.pitch_bytes as usize / Self::BYTES_PER_PIXEL, fb.height_px as usize)
        };
        // let framebuffer_vc = unsafe {
        //     slice2d::MutSlice2d::from_raw_parts(fb.base_address as *mut T, fb.width_px as usize, fb.pitch_bytes as usize / Self::BYTES_PER_PIXEL, fb.height_px as usize)
        // };
        let (front, back) = vram.split_at_line_mut(height);
        Ok(Screen { front, back, framebuffer,
            // framebuffer_vc 
            })
    }

    pub fn draw<F: Fn(&mut MutSlice2d<'a, T>)> (&mut self, f: F) {
        f(&mut self.back)
    }

    pub fn present(&mut self, swap: SwapStrategy<T>, present: PresentStrategy) {
        // swap buffers and copy the formerly back buffer to the framebuffer
        unsafe { self.front.swap_with_slice2d_unchecked(&mut self.back); }
        match present {
            PresentStrategy::Memcopy => unsafe { self.framebuffer.copy_buf_unchecked(&self.front); },
            PresentStrategy::Dma2d => crate::peripherals::dma::dma_copy_slice2d(&self.front.as_slice2d(), &mut self.framebuffer),
            PresentStrategy::Dma => crate::peripherals::dma::dma_copy_slice(self.front.buf_slice(), self.framebuffer.buf_mut_slice()),
        }
        match swap {            
            SwapStrategy::SwapAndClear(value) => self.back.fill(value),
            SwapStrategy::SwapAndCopy => self.back.copy_from_slice2d(&self.front),
            _ => {}
        }
        
    }
}

impl<'a> Screen<'a, u8> {
    pub const PALETTE_CGA: [u32; 16] = Self::palette_cga();

    pub fn set_palette<const N: usize>(&mut self, colors: &[u32; N]) {
        assert!(Framebuffer::set_palette(0, colors).expect("Palette update should work"));
    }

    pub fn get_palette(&self) -> [u32;256] {
        Framebuffer::get_palette()
    }

    const fn palette_cga() -> [u32; 16] {
        [
            0xff_00_00_00,
            0xff_00_00_aa,
            0xff_00_aa_00,
            0xff_00_aa_aa,
            0xff_aa_00_00,
            0xff_aa_00_aa,
            0xff_aa_55_00,
            0xff_aa_aa_aa,
            0xff_55_55_55,
            0xff_55_55_ff,
            0xff_55_ff_55,
            0xff_55_ff_ff,
            0xff_ff_55_55,
            0xff_ff_55_ff,
            0xff_ff_ff_55,
            0xff_ff_ff_ff,
        ]
    }
}

