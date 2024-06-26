use core::{cell::RefCell, usize};

use mystd::{byte_value::ByteValue, slice::slice2d::{self, traits::{MutSlice2dTrait, Slice2dTrait}, MutSlice2d}, sync::mutex::{Mutex, MutexGuard}};

use super::hal::framebuffer::{self, FbDepth, Framebuffer, FramebufferDescriptor, PixelOrder};


#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Offset {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Overscan {
    top: usize,
    bottom: usize,
    left: usize,
    right: usize
}

#[derive(Clone, Copy, Debug)]
pub struct ScreenGeometry {
    pub physical_size: Size,
    pub virtual_size: Size,
    pub virtual_offset: Offset,
    pub overscan: Overscan
}

impl From<ScreenGeometry> for FramebufferDescriptor {
    fn from(value: ScreenGeometry) -> Self {
        FramebufferDescriptor {
            physical_display: framebuffer::FbDimensions { 
                width_px: value.physical_size.width as u32, 
                height_px: value.physical_size.height as u32 
            },
            virtual_buffer: framebuffer::FbDimensions { 
                width_px: value.virtual_size.width as u32, 
                height_px: value.virtual_size.height as u32 
            },
            virtual_buffer_offset: framebuffer::FbOffset { 
                x_px: value.virtual_offset.x as u32, 
                y_px: value.virtual_offset.y as u32 
            },
            overscan: framebuffer::FbOverscan { 
                top_px: value.overscan.top as u32, 
                bottom_px: value.overscan.bottom as u32, 
                left_px: value.overscan.left as u32, 
                right_px: value.overscan.right as u32 
            },
            depth: FbDepth { bits_per_pixel: 0 },
            pixel_order: framebuffer::PixelOrder::Bgr,
            alignment: 4096,
        }
    }
}

impl ScreenGeometry {
    pub const fn with_size(size: Size) -> Self {
        if size.width == 0 || size.height == 0 {
            panic!("Can't create screen for empty size")
        }
        Self {
            physical_size: size,
            virtual_size: size,
            virtual_offset: Offset { x: 0, y: 0 },
            overscan: Overscan { top: 0, bottom: 0, left: 0, right: 0 },
        }
    }

    pub const fn pixel_count(&self) -> usize {
        self.physical_size.width * self.physical_size.height
    }
}

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

type ScreenType<'a> = Screen<'a, u8>;
static SCREEN: Mutex<RefCell<Option<Screen<u8>>>> = Mutex::new(RefCell::new(None));


pub fn create_screen(base_ptr: *mut u8) {
    let screen_geometry = ScreenGeometry::with_size(Size{ width: 640, height: 480 });
    let bytes_required = ScreenType::required_size_bytes(&screen_geometry);
    let slice = unsafe { core::slice::from_raw_parts_mut(base_ptr, bytes_required) };
    if let Some(screen_lock) = SCREEN.try_lock() {
        screen_lock.replace(Screen::try_create_in_raw_slice(slice, screen_geometry).ok());
        Palette::vga().make_current();
    }
}

pub struct ScreenLock<'a> {
    inner: MutexGuard<'a, RefCell<Option<Screen<'a, u8>>>>,
}

impl<'a> ScreenLock<'a> {
    pub fn with_screen_mut<F: Fn(&mut ScreenType) -> R, R>(&mut self, f: F) -> Option<R> {
        Some(f(self.inner.get_mut().as_mut()?))
    }
}

pub struct SharedScreen {
    inner: &'static Mutex<RefCell<Option<Screen<'static, u8>>>>
}

pub fn shared() -> SharedScreen {
    SharedScreen { inner: &SCREEN }
}

impl SharedScreen {
    pub fn lock(&self) -> ScreenLock<'static> {
        ScreenLock { inner: unsafe { self.inner.lock() } }
    }
}

pub struct Screen<'a, T> where T: Copy {
    front: MutSlice2d<'a, T>,
    back: MutSlice2d<'a, T>,
    framebuffer: MutSlice2d<'a, T>
}

impl<'a, T> Screen<'a, T> where T: Copy + 'a {
    pub const BYTES_PER_PIXEL: usize = core::mem::size_of::<T>();
    pub const BITS_PER_PIXEL: usize = core::mem::size_of::<T>() * 8;

    pub fn width(&self) -> usize {
        self.back.width()
    }

    pub fn height(&self) -> usize {
        self.back.height()
    }

    pub fn required_size_bytes(geom: &ScreenGeometry) -> usize {
        2 * geom.pixel_count() * Self::BYTES_PER_PIXEL
    }

    pub fn try_create_in_raw_slice(slice: &mut [u8], geom: ScreenGeometry) -> Result<Self, ScreenError> {
        let required_size_bytes = Self::required_size_bytes(&geom);
        let memory: &mut [T] = unsafe {
            slice.align_to_mut().1
        };
        if memory.len() < required_size_bytes {
            return Err(ScreenError::NotEnoughMemory { required: ByteValue::from_bytes(required_size_bytes as u64) })
        }
        let width = geom.physical_size.width;
        let height = geom.physical_size.height;
        
        // try to create the framebuffer
        let mut fbdesc: FramebufferDescriptor = geom.into();
        fbdesc.depth.bits_per_pixel = Self::BITS_PER_PIXEL as u32;
        fbdesc.pixel_order = PixelOrder::Rgb;
        let fb = Framebuffer::new(fbdesc).ok_or(ScreenError::CouldNotCreateFramebuffer)?;
        
        let framebuffer = unsafe {
            slice2d::MutSlice2d::from_raw_parts(fb.raw_slice.as_mut_ptr().cast(), fb.width_px as usize, fb.pitch_bytes as usize / Self::BYTES_PER_PIXEL, fb.height_px as usize)
        };

        // allocate front and back buffer
        let (front, remaining_memory) = slice2d::MutSlice2d::with_mut_slice(memory, width, width, height).ok_or(ScreenError::CouldNotCreateVRam)?;
        let (back, _) = slice2d::MutSlice2d::with_mut_slice(remaining_memory, width, width, height).ok_or(ScreenError::CouldNotCreateVRam)?;
        Ok(Screen { front, back, framebuffer })
    }

    pub fn draw<F: Fn(&mut MutSlice2d<'a, T>)> (&mut self, f: F) {
        f(&mut self.back)
    }

    pub fn present(&mut self, swap: SwapStrategy<T>, present: PresentStrategy) {
        // swap buffers and copy the formerly back buffer to the framebuffer
        unsafe { self.front.swap_with_slice2d_unchecked(&mut self.back); }
        match present {
            PresentStrategy::Memcopy => unsafe { self.framebuffer.copy_buf_unchecked(self.front.as_ptr()); },
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

pub enum Palette {
    Bgra([mystd::drawing::color::Bgra;256]),
    Rgba([mystd::drawing::color::Rgba;256]),
}


impl Palette {

    pub const fn rgba_from_u32(values: [u32;256]) -> Palette {
        type Color = mystd::drawing::color::Rgba;
        let mut rgba_values: [Color;256] = [Color::zero(); 256];
        let mut i = 0;
        loop {
            rgba_values[i] = Color::from_u32(values[i]);
            i += 1;
            if i == 256 {
                break;
            }
        }
        Palette::Rgba(rgba_values)
    }

    pub const fn bgra_from_u32(values: [u32;256]) -> Palette {
        type Color = mystd::drawing::color::Bgra;
        let mut bgra_values: [Color;256] = [Color::zero(); 256];
        let mut i = 0;
        loop {
            bgra_values[i] = Color::from_u32(values[i]);
            i += 1;
            if i == 256 {
                break;
            }
        }
        Palette::Bgra(bgra_values)
    }

    pub const fn from_u32_and_pixel_order(values: [u32;256], pixel_order: PixelOrder) -> Palette {
        match pixel_order {
            framebuffer::PixelOrder::Bgr => Self::bgra_from_u32(values),
            framebuffer::PixelOrder::Rgb => Self::rgba_from_u32(values),
        }
    }

    pub const fn into_u32_with_pixel_order(self, pixel_order: PixelOrder) -> [u32; 256] {
        let mut result = [0; 256];
        let mut i = 0;
        match (self, pixel_order) {
            (Palette::Bgra(bgra), PixelOrder::Bgr) => {
                loop {
                    result[i] = bgra[i].into_u32();
                    i += 1; if i == 256 { break; }
                }
            },
            (Palette::Bgra(bgra), PixelOrder::Rgb) => {
                loop {
                    result[i] = bgra[i].into_rgba().into_u32();
                    i += 1; if i == 256 { break; }
                }
            },
            (Palette::Rgba(rgba), PixelOrder::Bgr) => {
                loop {
                    result[i] = rgba[i].into_bgra().into_u32();
                    i += 1; if i == 256 { break; }
                }
            },
            (Palette::Rgba(rgba), PixelOrder::Rgb) => {
                loop {
                    result[i] = rgba[i].into_u32();
                    i += 1; if i == 256 { break; }
                }
            }
        };
        result
    }


    pub fn current() -> Self {
        let values = Framebuffer::get_palette();
        let pixel_order = Framebuffer::get_pixel_order();
        Self::from_u32_and_pixel_order(values, pixel_order)
    }

    pub fn make_current(self)  {
        let pixel_order = Framebuffer::get_pixel_order();
        let values = self.into_u32_with_pixel_order(pixel_order);
        assert!(Framebuffer::set_palette(0, &values).expect("Palette update should work"));
    }

    pub const fn cga() -> Self {
        let mut values: [u32; 256] = [0; 256];
        
        values[0] = 0xff_00_00_00;
        values[1] = 0xff_00_00_aa;
        values[2] = 0xff_00_aa_00;
        values[3] = 0xff_00_aa_aa;
        values[4] = 0xff_aa_00_00;
        values[5] = 0xff_aa_00_aa;
        values[6] = 0xff_aa_55_00;
        values[7] = 0xff_aa_aa_aa;
        values[8] = 0xff_55_55_55;
        values[9] = 0xff_55_55_ff;
        values[10] = 0xff_55_ff_55;
        values[11] = 0xff_55_ff_ff;
        values[12] = 0xff_ff_55_55;
        values[13] = 0xff_ff_55_ff;
        values[14] = 0xff_ff_ff_55;
        values[15] = 0xff_ff_ff_ff;
    
        Self::rgba_from_u32(values)
    }

    pub fn vga() -> Self {
        use mystd::drawing::color;
        let mut values: [u32; 256] = [0; 256];
        
        // start with CGA palette 
        values[0] = 0xff_00_00_00;
        values[1] = 0xff_00_00_aa;
        values[2] = 0xff_00_aa_00;
        values[3] = 0xff_00_aa_aa;
        values[4] = 0xff_aa_00_00;
        values[5] = 0xff_aa_00_aa;
        values[6] = 0xff_aa_55_00;
        values[7] = 0xff_aa_aa_aa;
        values[8] = 0xff_55_55_55;
        values[9] = 0xff_55_55_ff;
        values[10] = 0xff_55_ff_55;
        values[11] = 0xff_55_ff_ff;
        values[12] = 0xff_ff_55_55;
        values[13] = 0xff_ff_55_ff;
        values[14] = 0xff_ff_ff_55;
        values[15] = 0xff_ff_ff_ff;

        // 16 grayscale colors black to white
        let mut i = 16;
        for color in color::HsvF::BLACK.lerp(color::HsvF::WHITE, 16) {
            let rgbf: color::RgbF = color.into();
            let rgba: color::Rgba = rgbf.into();
            values[i] = rgba.into();
            i += 1;
        }


        // 3 x 3 x 24 colors with decreasing saturation and brightness
        for brt in [1.0, 0.44, 0.29] {
            for sat in [1.0, 0.52, 0.29] {
                
                for color in color::HsvF::BLUE.lerp(color::HsvF::BLUE_2, 24) {
                    let rgbf: color::RgbF = color.lifted_by(sat).dimmed_by(brt).into();
                    let rgba: color::Rgba = rgbf.into();
                    values[i] = rgba.into();
                    i += 1;
                }
            }
        }

        // fill the rest with black
        while i < 256 {
            values[i] = 0xff_00_00_00;
            i += 1;
        }
    
        Self::rgba_from_u32(values)
    }
}

