use core::fmt::DebugStruct;
use crate::{peripherals::mailbox, system::peripherals::mailbox::MailboxError};


pub mod color {
    pub const WHITE: u32 = 0xff_ff_ff_ff;
    pub const BLACK: u32 = 0xff_00_00_00;
    pub const RED: u32 = 0xff_00_00_ff;
    pub const GREEN: u32 = 0xff_00_ff_00;
    pub const BLUE: u32 = 0xff_ff_00_00;
}

mod tags {

    /// # [Allocate buffer](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface#allocate-buffer)
    /// * Tag: 0x00040001
    /// * Request:
    ///     * Length: 4
    ///     * Value:
    ///         * u32: alignment in bytes
    /// * Response:
    ///     * Length: 8
    ///     * Value:
    ///         * u32: frame buffer base address in bytes
    ///         * u32: frame buffer size in bytes
    /// 
    /// If the requested alignment is unsupported then the current base and size 
    /// (which may be 0 if not allocated) is returned and no change occurs.
    pub const FB_ALLOCATE_BUFFER: u32 = 0x00040001;

    /// # [Release buffer](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface#release-buffer)
    /// * Tag: 0x00048001
    /// * Request:
    ///     * Length: 0
    /// * Response:
    ///     * Length: 0
    /// 
    /// Releases and disables the frame buffer.
    pub const FB_RELEASE_BUFFER: u32 = 0x00048001;
    
    /// # [Blank screen](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface#blank-screen)
    /// * Tag: 0x00040002
    /// * Request:
    ///     * Length: 4
    /// * Value:
    ///     * u32: state
    ///         * State:
    ///             * Bit 0: 0=off, 1=on
    ///             * Bits 1-31: reserved for future use (set to 0)
    /// * Response:
    ///     * Length: 4
    /// * Value:
    ///     * u32: state
    ///         * State:
    ///             * Bit 0: 0=off, 1=on
    ///             * Bits 1-31: reserved for future use
    pub const FB_BLANK_SCREEN: u32 = 0x00040002;

    /// # [Get physical (display) width/height](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface#get-physical-display-widthheight)
    /// * Tag: 0x00040003
    /// * Request:
    ///     * Length: 0
    /// * Response:
    ///     * Length: 8
    ///     * Value:
    ///         * u32: width in pixels
    ///         * u32: height in pixels
    pub const FB_GET_PHYSICAL_DIMENSIONS: u32 = 0x00040003;

    /// # Test physical (display) width/height
    /// * Tag: 0x00044003
    /// * Request:
    ///     * Length: 8
    ///     * Value:
    ///         * u32: width in pixels
    ///         * u32: height in pixels
    /// * Response:
    ///     * Length: 8
    ///     * Value:
    ///         * u32: width in pixels
    ///         * u32: height in pixels
    /// 
    /// Response is the same as the request (or modified), to indicate if this configuration is supported (in combination with all the other settings). Does not modify the current hardware or frame buffer state.
    pub const FB_TEST_PHYSICAL_DIMENSIONS: u32 = 0x00044003;
    
    /// # Set physical (display) width/height
    /// * Tag: 0x00048003
    /// * Request:
    ///     * Length: 8
    ///     * Value:
    ///         * u32: width in pixels
    ///         * u32: height in pixels
    /// * Response:
    ///     * Length: 8
    ///     * Value:
    ///         * u32: width in pixels
    ///         * u32: height in pixels
    /// 
    /// The response may not be the same as the request so it must be checked. May be the previous width/height or 0 for unsupported.
    pub const FB_SET_PHYSICAL_DIMENSIONS: u32 = 0x00048003;
    pub const FB_GET_VIRTUAL_DIMENSIONS: u32 = 0x00040004;
    pub const FB_TEST_VIRTUAL_DIMENSIONS: u32 = 0x00044004;
    pub const FB_SET_VIRTUAL_DIMENSIONS: u32 = 0x00048004;
    pub const FB_GET_DEPTH: u32 = 0x00040005;
    pub const FB_TEST_DEPTH: u32 = 0x00044005;
    pub const FB_SET_DEPTH: u32 = 0x00048005;

    /// Get pixel order
    /// * Tag: 0x00040006
    /// * Request:
    ///     * Length: 0
    /// * Response:
    ///     * Length: 4
    ///     * Value:
    ///     * u32: state
    ///         * State:
    ///             * 0x0: BGR
    ///             * 0x1: RGB
    pub const FB_GET_PIXEL_ORDER: u32 = 0x00040006;
    
    /// # Test pixel order
    /// * Tag: 0x00044006
    /// * Request:
    ///     * Length: 4
    ///     * Value:
    ///         * u32: state 
    ///             * State:
    ///                 * 0x0: BGR
    ///                 * 0x1: RGB
    /// * Response:
    ///     * Length: 4
    ///     * Value:
    ///         * u32: state 
    ///             * State:
    ///                 * 0x0: BGR
    ///                 * 0x1: RGB
    /// 
    /// Response is the same as the request (or modified), to indicate if this configuration is supported (in combination with all the other settings). Does not modify the current hardware or frame buffer state.
    pub const FB_TEST_PIXEL_ORDER: u32 = 0x00044006;
    
    /// # Set pixel order
    /// * Tag: 0x00048006
    /// * Request:
    ///     * Length: 4
    ///     * Value:
    ///         * u32: state 
    ///             * State:
    ///                 * 0x0: BGR
    ///                 * 0x1: RGB
    /// * Response:
    ///     * Length: 4
    ///     * Value:
    ///         * u32: state 
    ///             * State:
    ///                 * 0x0: BGR
    ///                 * 0x1: RGB
    /// 
    /// The response may not be the same as the request so it must be checked.
    pub const FB_SET_PIXEL_ORDER: u32 = 0x00048006;
    pub const FB_GET_ALPHA_MODE: u32 = 0x00040007;
    pub const FB_TEST_ALPHA_MODE: u32 = 0x00044007;
    pub const FB_SET_ALPHA_MODE: u32 = 0x00048007;
    pub const FB_GET_PITCH: u32 = 0x00040008;
    pub const FB_GET_VIRTUAL_OFFSET: u32 = 0x00040009;
    pub const FB_TEST_VIRTUAL_OFFSET: u32 = 0x00044009;
    pub const FB_SET_VIRTUAL_OFFSET: u32 = 0x00048009;

    /// # Get overscan
    /// * Tag: 0x0004000a
    /// * Request:
    /// * Length: 0
    /// * Response:
    /// * Length: 16
    /// * Value:
    /// * u32: top in pixels
    /// * u32: bottom in pixels
    /// * u32: left in pixels
    /// * u32: right in pixels
    pub const FB_GET_OVERSCAN: u32 = 0x0004000a;

    /// # Test overscan
    /// * Tag: 0x0004400a
    /// * Request:
    ///     * Length: 16
    ///     * Value:
    ///         * u32: top in pixels
    ///         * u32: bottom in pixels
    ///         * u32: left in pixels
    ///         * u32: right in pixels
    /// * Response:
    ///     * Length: 16
    ///     * Value:
    ///         * u32: top in pixels
    ///         * u32: bottom in pixels
    ///         * u32: left in pixels
    ///         * u32: right in pixels
    /// 
    /// Response is the same as the request (or modified), to indicate if this configuration is supported (in combination with all the other settings). Does not modify the current hardware or frame buffer state.
    pub const FB_TEST_OVERSCAN: u32 = 0x0004400a;

    /// # Set overscan
    /// * Tag: 0x0004800a
    /// * Request:
    ///     * Length: 16
    ///     * Value:
    ///         * u32: top in pixels
    ///         * u32: bottom in pixels
    ///         * u32: left in pixels
    ///         * u32: right in pixels
    /// * Response:
    ///     * Length: 16
    ///     * Value:
    ///         * u32: top in pixels
    ///         * u32: bottom in pixels
    ///         * u32: left in pixels
    ///         * u32: right in pixels
    /// 
    /// The response may not be the same as the request so it must be checked. May be the previous overscan or 0 for unsupported.
    pub const FB_SET_OVERSCAN: u32 = 0x0004800a;

    /// # [Get palette](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface#get-palette)
    /// * Tag: 0x0004000b
    /// * Request:
    ///     * Length: 0
    /// * Response:
    ///     * Length: 1024
    ///     * Value:
    ///         * u32...: RGBA palette values (index 0 to 255)
    pub const FB_GET_PALETTE: u32 = 0x0004000b;

    /// # [Test palette](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface#test-palette)
    /// * Tag: 0x0004400b
    /// * Request:
    ///     * Length: 24..1032
    ///     * Value:
    ///         * u32: offset: first palette index to set (0-255)
    ///         * u32: length: number of palette entries to set (1-256)
    ///         * u32...: RGBA palette values (offset to offset+length-1)
    /// * Response:
    ///     * Length: 4
    ///     * Value:
    ///         * u32: 0=valid, 1=invalid
    /// 
    /// Response is the same as the request (or modified), to indicate if this 
    /// configuration is supported (in combination with all the other settings). 
    /// Does not modify the current hardware or frame buffer state.
    pub const FB_TEST_PALETTE: u32 = 0x0004400b;

    /// # [Set palette](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface#set-palette)
    /// * Tag: 0x0004800b
    /// * Request:
    ///     * Length: 24..1032
    ///     * Value:
    ///         * u32: offset: first palette index to set (0-255)
    ///         * u32: length: number of palette entries to set (1-256)
    ///         * u32...: RGBA palette values (offset to offset+length-1)
    /// * Response:
    ///     * Length: 4
    ///     * Value:
    ///         * u32: 0=valid, 1=invalid
    /// 
    /// The response may not be the same as the request so it must be checked. 
    /// Palette changes should not be partially applied.
    pub const FB_SET_PALETTE: u32 = 0x0004800b;

    
}

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

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FbDepth {
    pub bits_per_pixel: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FbPitch {
    pub bytes_per_line: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FbDimensions {
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FbOffset {
    pub x_px: u32,
    pub y_px: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FbOverscan {
    pub top_px: u32,
    pub bottom_px: u32,
    pub left_px: u32,
    pub right_px: u32,
}

#[repr(C)]
pub struct FbAllocate {
    pub alignment_bytes: u32,
}

pub type Palette = [u32; 256];


#[derive(Debug)]
#[repr(C)]
pub struct PaletteChange<const N: usize> {
    pub offset: u32,
    pub length: u32,
    pub values: [u32; N],
}

pub struct Framebuffer {
    pub raw_slice: &'static mut [u8],
    pub base_address: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub bits_per_pixel: u32,
    pub pitch_bytes: u32,
}

pub struct FramebufferDescriptor {
    pub physical_display: FbDimensions,
    pub virtual_buffer: FbDimensions,
    pub virtual_buffer_offset: FbOffset,
    pub overscan: FbOverscan,
    pub depth: FbDepth,
    pub pixel_order: PixelOrder,
    pub alignment: u32,
}


impl Framebuffer {
    pub fn new(desc: FramebufferDescriptor) -> Option<Self> {
        
        use tags::*;
        let mut mailbox = mailbox::Mailbox::<128>::new();
        *mailbox
            .push_request(tags::FB_SET_PHYSICAL_DIMENSIONS, 8)
            .ok()? = desc.physical_display;

        *mailbox
            .push_request(tags::FB_SET_VIRTUAL_DIMENSIONS, 8)
            .ok()? = desc.virtual_buffer;
        *mailbox
            .push_request(tags::FB_SET_VIRTUAL_OFFSET, 8)
            .ok()? = desc.virtual_buffer_offset;

        *mailbox
            .push_request(tags::FB_SET_OVERSCAN, 16)
            .ok()? = desc.overscan;

        *mailbox.push_request(tags::FB_SET_DEPTH, 4).ok()? = desc.depth;
        *mailbox.push_request(tags::FB_SET_PIXEL_ORDER, 4).ok()? = desc.pixel_order;
        *mailbox.push_request(tags::FB_ALLOCATE_BUFFER, 8).ok()? = desc.alignment;

        mailbox.push_request_empty(tags::FB_GET_PITCH, 4).ok()?;

        let mut responses = mailbox.submit_messages(mailbox::CHANNEL_PROPERTIES).ok()?;
        let (width_px, height_px) = *responses.next()?.ok()?.try_value_as()?;
        // FbSetVirtualDimensions {..},
        let _ = responses.next();
        // FbSetVirtualOffset { .. },
        let _ = responses.next();
        // FB_SET_OVERSCAN
        let _ = responses.next();
        let bits_per_pixel: u32 = *responses.next()?.ok()?.try_value_as()?;
        // FbSetPixelOrder { .. },
        responses.next();

        let (base_address, size_bytes): (u32, u32) =
            *responses.next()?.ok()?.try_value_as()?;
        let pitch_bytes: u32 = *responses.next()?.ok()?.try_value_as()?;

        let ptr: *mut u8 = (0x3FFFFFFF & base_address) as *mut u8;
        Some(Self {
            raw_slice: unsafe { core::slice::from_raw_parts_mut(ptr, size_bytes as usize) },
            width_px,
            height_px,
            base_address,
            bits_per_pixel,
            pitch_bytes,
        })
    }

    ///
    /// returns true if palette update was valid
    pub fn set_palette<const N: usize>(offset: u8, colors: &[u32;N]) -> Result<bool, MailboxError> {
        let mut mailbox = mailbox::Mailbox::<280>::new();
        let request = PaletteChange::<N>{
            offset: offset as u32,
            length: N as u32,
            values: *colors,
        };
        *mailbox
            .push_request(tags::FB_SET_PALETTE, core::mem::size_of::<PaletteChange::<N>>() as u32)? = request;
        if let Some(response) = mailbox.submit_messages(mailbox::CHANNEL_PROPERTIES)?.next() {
            let res: u32 = *response?.try_value_as().expect("Response should contain a single u32 value");
            Ok(res == 0)
        } else {
            Err(MailboxError::Unknown)
        }
    }

    pub fn get_palette() -> [u32; 256] {
        let mut mailbox = mailbox::Mailbox::<280>::new();
        mailbox.push_request_empty(tags::FB_GET_PALETTE, 256 * 4).expect("Mailbox should work");
        let response = mailbox
            .submit_messages(mailbox::CHANNEL_PROPERTIES)
            .expect("Mailbox should work")
            .next()
            .expect("Should have at least one response")
            .expect("response should not be an error");
        *response.try_value_as().expect("The palette should fit exactly into array of 256 u32")
    }

    pub fn get_pixel_order() -> PixelOrder {
        let mut mailbox = mailbox::Mailbox::<5>::new();
        mailbox.push_request_empty(tags::FB_GET_PIXEL_ORDER, 4).expect("Mailbox should work");
        let response = mailbox
            .submit_messages(mailbox::CHANNEL_PROPERTIES)
            .expect("Mailbox should work")
            .next()
            .expect("Should have at least one response")
            .expect("response should not be an error");
        *response.try_value_as().expect("PixelOrder should be the only response")
    }

    pub fn as_pixels(&self) -> &[u32] {
        unsafe {
            self.raw_slice.align_to().1
        }
    }

    fn as_mut_pixels(&mut self) -> &mut [u32] {
        unsafe {
            self.raw_slice.align_to_mut().1
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.raw_slice
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.raw_slice
    }

    // pub fn write_text(&self, text: &[u8], font: &[u64], mapping: impl Fn(u8) -> u8) {
    //     let repeat = (2, 2);
    //     let offset = (40, 48);
    //     let size = (self.width_px - 2 * offset.0, self.height_px - 2 * offset.1);
    //     let columns = size.0 as usize / (repeat.0 * 8);
    //     for y in 0..size.1 {
    //         let yy = y as usize / repeat.1;
    //         for x in 0..size.0 {
    //             let xx = x as usize / repeat.0;
    //             let char_index = (xx / 8, yy / 8);
    //             let linear_index = char_index.1 * columns + char_index.0;
    //             let ch = text.get(linear_index).copied().unwrap_or_default();
    //             let char = font[mapping(ch) as usize % font.len()];
    //             let char_subpixel = (xx % 8, yy % 8);
    //             if (char << ((7 - char_subpixel.1) * 8 + char_subpixel.0)) & (1_u64 << 63) == 0 {
    //                 self.set_pixel_a8b8g8r8(x + offset.0, y + offset.1, 0xFF0000AA);
    //             } else {
    //                 self.set_pixel_a8b8g8r8(x + offset.0, y + offset.1, 0xFFFFFFFF);
    //             }
    //         }
    //     }
    // }
}
