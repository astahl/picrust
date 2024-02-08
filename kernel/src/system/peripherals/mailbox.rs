use core::ptr::read_volatile;

use crate::peripherals::mmio;
use crate::peripherals::mmio::MMIO;

use super::mmio::DynamicMmioField;

#[repr(align(16), C)]
pub struct Mailbox<const BUFFER_SIZE: usize> {
    write_offset: usize,
    read_offset: usize,
    size: u32,
    req_res_code: ReqResCode,
    buffer: [u8; BUFFER_SIZE],
}

#[repr(u32)]
pub enum RequestResponseStatus {
    Pending = 0,
    Success = 0x80000000,
    ErrorParsingRequestBuffer = 0x80000001
}

pub struct ReqResCode(DynamicMmioField<RequestResponseStatus>);

impl ReqResCode {
    pub const fn cleared() -> Self {
        Self(DynamicMmioField::init(RequestResponseStatus::Pending))
    }

    pub fn clear(&mut self) {
        self.0.write(RequestResponseStatus::Pending)
    }

    pub fn get(&self) -> RequestResponseStatus {
        self.0.read()
    }
}

pub enum MailboxError {
    Unknown,
    RequestResponseError(RequestResponseStatus)
}

pub struct MboxStatus(u32);

impl MboxStatus {
    pub const fn is_full(&self) -> bool {
        self.0 & 0x80000000 != 0
    }

    pub const fn is_empty(&self) -> bool {
        self.0 & 0x40000000 != 0
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum LedStatus {
    Off = 0,
    On = 1,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Led {
    Status = 42,
    Power = 130,
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


#[repr(u32)]
pub enum Tag {
    End = 0,
    VcGetFirmwareRevision = 0x00000001,
    HwGetBoardModel = 0x00010001,
    HwGetBoardRevision = 0x00010002,
    HwGetBoardMacAddress = 0x00010003,
    HwGetBoardSerial = 0x00010004,
    HwGetArmMemory = 0x00010005,
    HwGetVcMemory = 0x00010006,
    HwGetClocks = 0x00010007,
    GetEdidBlock = 0x00030020,
    GetOnboardLedStatus = 0x00030041,
    TestOnboardLedStatus = 0x00034041,
    SetOnboardLedStatus = 0x00038041,
    FbAllocateBuffer = 0x00040001,
    FbReleaseBuffer = 0x00048001,
    FbGetPhysicalDimensions = 0x00040003,
    FbTestPhysicalDimensions = 0x00044003,
    FbSetPhysicalDimensions = 0x00048003,
    FbGetVirtualDimensions = 0x00040004,
    FbTestVirtualDimensions = 0x00044004,
    FbSetVirtualDimensions = 0x00048004,
    FbGetDepth = 0x00040005,
    FbTestDepth = 0x00044005,
    FbSetDepth = 0x00048005,
    FbGetPixelOrder = 0x00040006,
    FbTestPixelOrder = 0x00044006,
    FbSetPixelOrder = 0x00048006,
    FbGetAlphaMode = 0x00040007,
    FbTestAlphaMode = 0x00044007,
    FbSetAlphaMode = 0x00048007,
    FbGetPitch = 0x00040008,
    FbGetVirtualOffset = 0x00040009,
    FbTestVirtualOffset = 0x00044009,
    FbSetVirtualOffset = 0x00048009,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum PropertyMessageRequest {
    Null = 0,
    VcGetFirmwareRevision = 0x00000001,
    HwGetBoardModel = 0x00010001,
    HwGetBoardRevision = 0x00010002,
    HwGetBoardMacAddress = 0x00010003,
    HwGetBoardSerial = 0x00010004,
    HwGetArmMemory = 0x00010005,
    HwGetVcMemory = 0x00010006,
    // HwGetClocks = 0x00010007,
    GetEdidBlock { block_number: u32 } = 0x00030020,
    GetOnboardLedStatus = 0x00030041,
    TestOnboardLedStatus { pin_number: Led, status: LedStatus } = 0x00034041,
    SetOnboardLedStatus { pin_number: Led, status: LedStatus } = 0x00038041,
    FbAllocateBuffer { alignment_bytes: u32 } = 0x00040001,
    FbReleaseBuffer = 0x00048001,
    FbGetPhysicalDimensions = 0x00040003,
    FbTestPhysicalDimensions { width_px: u32, height_px: u32 } = 0x00044003,
    FbSetPhysicalDimensions { width_px: u32, height_px: u32 } = 0x00048003,
    FbGetVirtualDimensions = 0x00040004,
    FbTestVirtualDimensions { width_px: u32, height_px: u32 } = 0x00044004,
    FbSetVirtualDimensions { width_px: u32, height_px: u32 } = 0x00048004,
    FbGetDepth = 0x00040005,
    FbTestDepth { bpp: u32 } = 0x00044005,
    FbSetDepth { bpp: u32 } = 0x00048005,
    FbGetPixelOrder = 0x00040006,
    FbTestPixelOrder { state: PixelOrder } = 0x00044006,
    FbSetPixelOrder { state: PixelOrder } = 0x00048006,
    FbGetAlphaMode = 0x00040007,
    FbTestAlphaMode { state: AlphaMode } = 0x00044007,
    FbSetAlphaMode { state: AlphaMode } = 0x00048007,
    FbGetPitch = 0x00040008,
    FbGetVirtualOffset = 0x00040009,
    FbTestVirtualOffset { x_px: u32, y_px: u32 } = 0x00044009,
    FbSetVirtualOffset { x_px: u32, y_px: u32 } = 0x00048009,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum PropertyMessageResponse {
    Null = 0,
    VcGetFirmwareRevision {
        firmware_revision: u32,
    } = 0x00000001,
    HwGetBoardModel {
        board_model: u32,
    } = 0x00010001,
    HwGetBoardRevision {
        board_revision: u32,
    } = 0x00010002,
    HwGetBoardMacAddress {
        board_mac_address: [u8; 6],
    } = 0x00010003,
    HwGetBoardSerial {
        board_serial: u64,
    } = 0x00010004,
    HwGetArmMemory {
        base_address: u32,
        size: u32,
    } = 0x00010005,
    HwGetVcMemory {
        base_address: u32,
        size: u32,
    } = 0x00010006,
    // HwGetClocks = 0x00010007,
    GetEdidBlock {
        block_number: u32,
        status: u32,
        data: [u8; 128],
    } = 0x00030020,
    GetOnboardLedStatus {
        pin_number: Led,
        status: LedStatus,
    } = 0x00030041,
    TestOnboardLedStatus {
        pin_number: Led,
        status: LedStatus,
    } = 0x00034041,
    SetOnboardLedStatus {
        pin_number: Led,
        status: LedStatus,
    } = 0x00038041,
    FbAllocateBuffer {
        base_address_bytes: u32,
        size_bytes: u32,
    } = 0x00040001,
    FbReleaseBuffer = 0x00048001,
    FbGetPhysicalDimensions {
        width_px: u32,
        height_px: u32,
    } = 0x00040003,
    FbTestPhysicalDimensions {
        width_px: u32,
        height_px: u32,
    } = 0x00044003,
    FbSetPhysicalDimensions {
        width_px: u32,
        height_px: u32,
    } = 0x00048003,
    FbGetVirtualDimensions {
        width_px: u32,
        height_px: u32,
    } = 0x00040004,
    FbTestVirtualDimensions {
        width_px: u32,
        height_px: u32,
    } = 0x00044004,
    FbSetVirtualDimensions {
        width_px: u32,
        height_px: u32,
    } = 0x00048004,
    FbGetDepth {
        bpp: u32,
    } = 0x00040005,
    FbTestDepth {
        bpp: u32,
    } = 0x00044005,
    FbSetDepth {
        bpp: u32,
    } = 0x00048005,
    FbGetPixelOrder {
        state: PixelOrder,
    } = 0x00040006,
    FbTestPixelOrder {
        state: PixelOrder,
    } = 0x00044006,
    FbSetPixelOrder {
        state: PixelOrder,
    } = 0x00048006,
    FbGetAlphaMode {
        state: AlphaMode,
    } = 0x00040007,
    FbTestAlphaMode {
        state: AlphaMode,
    } = 0x00044007,
    FbSetAlphaMode {
        state: AlphaMode,
    } = 0x00048007,
    FbGetPitch {
        bytes_per_line: u32,
    } = 0x00040008,
    FbGetVirtualOffset {
        x_px: u32,
        y_px: u32,
    } = 0x00040009,
    FbTestVirtualOffset {
        x_px: u32,
        y_px: u32,
    } = 0x00044009,
    FbSetVirtualOffset {
        x_px: u32,
        y_px: u32,
    } = 0x00048009,
}

impl PropertyMessageRequest {
    const fn value_buffer_len(&self) -> (u32, u32) {
        match self {
            Self::Null | Self::FbReleaseBuffer => (0, 0),

            Self::HwGetBoardModel
            | Self::HwGetBoardRevision
            | Self::FbGetPitch
            | Self::FbGetDepth
            | Self::VcGetFirmwareRevision
            | Self::FbGetPixelOrder
            | Self::FbGetAlphaMode => (0, 4),

            Self::FbTestDepth { .. }
            | Self::FbSetDepth { .. }
            | Self::FbTestPixelOrder { .. }
            | Self::FbSetPixelOrder { .. }
            | Self::FbTestAlphaMode { .. }
            | Self::FbSetAlphaMode { .. } => (4, 4),

            Self::HwGetBoardMacAddress => (0, 6),

            Self::HwGetBoardSerial
            | Self::HwGetArmMemory
            | Self::HwGetVcMemory
            | Self::FbGetPhysicalDimensions
            | Self::FbGetVirtualDimensions
            | Self::FbGetVirtualOffset
            | Self::GetOnboardLedStatus => (0, 8),
            Self::FbAllocateBuffer { .. } => (4, 8),
            Self::FbTestPhysicalDimensions { .. }
            | Self::FbSetPhysicalDimensions { .. }
            | Self::FbTestVirtualDimensions { .. }
            | Self::FbSetVirtualDimensions { .. }
            | Self::FbTestVirtualOffset { .. }
            | Self::FbSetVirtualOffset { .. }
            | Self::TestOnboardLedStatus { .. }
            | Self::SetOnboardLedStatus { .. } => (8, 8),

            Self::GetEdidBlock { .. } => (4, 136),
        }
    }

    pub fn copy_to(&self, buffer: &mut [u8]) -> usize {
        match self {
            Self::Null => unsafe {
                let ptr = buffer.as_ptr() as *mut u32;
                *ptr = 0;
                4
            },
            _ => {
                let (req_len, res_len) = self.value_buffer_len();
                let max_len = req_len.max(res_len);
                unsafe {
                    let src: *const u32 = <*const _>::from(self).cast();
                    let dst: *mut u32 = buffer.as_mut_ptr().cast();
                    // copy discriminant
                    core::ptr::write_volatile(dst, *src);
                    // write value buffer length
                    core::ptr::write_volatile(dst.add(1), max_len);
                    // write zero req / resp field
                    core::ptr::write_volatile(dst.add(2), 0);
                    // copy value buffer
                    let padded_length = (req_len as usize + 3) / 4;
                    for i in 0..padded_length {
                        core::ptr::write_volatile(dst.add(3 + i), *src.add(1 + i));
                    }
                }
                max_len as usize + 12
            }
        }
    }

    pub fn copied_len(&self) -> usize {
        match self {
            Self::Null => 4,
            _ => {
                let (req_len, res_len) = self.value_buffer_len();
                let max_len = req_len.max(res_len);
                ((max_len as usize + 3) & !0b11) + 12
            }
        }
    }
}

impl PropertyMessageResponse {
    pub fn fill_from(&mut self, buffer: &[u8]) -> usize {
        unsafe {
            let src: *const u32 = buffer.as_ptr().cast();
            let dst: *mut u32 = <*mut _>::from(self).cast();

            let discriminant = core::ptr::read_volatile(src);
            if discriminant == 0 {
                return 4;
            }
            *dst = discriminant;
            let value_buffer_len = core::ptr::read_volatile(src.add(1));
            let req_res_field = core::ptr::read_volatile(src.add(2));
            let value_len_bytes = req_res_field & 0x7fff_ffff;

            let src_bytes: *const u8 = src.add(3).cast();
            let dst_bytes: *mut u8 = dst.add(1).cast();

            for i in 0..(value_len_bytes as usize) {
                *dst_bytes.add(i) = core::ptr::read_volatile(src_bytes.add(i));
            }
            ((value_buffer_len as usize + 3) & !0b11) + 12
        }
    }

    pub fn peek_len(buffer: &[u8]) -> usize {
        let src: *const u32 = buffer.as_ptr().cast();

        unsafe {
            let discriminant = core::ptr::read_volatile(src);
            if discriminant == 0 {
                return 4;
            }
            let value_buffer_len = core::ptr::read_volatile(src.add(1));
            ((value_buffer_len as usize + 3) & !0b11) + 12
        }
    }
}

const MBOX_BASE: usize = 0xB880; //0x201000;
impl<const BUFFER_SIZE: usize> Mailbox<BUFFER_SIZE> {
    const MBOX_READ: MMIO<MBOX_BASE, 0x00> = MMIO();
    const MBOX_POLL: MMIO<MBOX_BASE, 0x10> = MMIO();
    const MBOX_SENDER: MMIO<MBOX_BASE, 0x14> = MMIO();
    const MBOX_STATUS: MMIO<MBOX_BASE, 0x18> = MMIO();
    const MBOX_CONFIG: MMIO<MBOX_BASE, 0x1C> = MMIO();
    const MBOX_WRITE: MMIO<MBOX_BASE, 0x20> = MMIO();

    pub const fn new() -> Self {
        Self {
            write_offset: 0,
            read_offset: 0,
            size: BUFFER_SIZE as u32 * 4 + 8,
            req_res_code: ReqResCode::cleared(),
            buffer: [0; BUFFER_SIZE],
        }
    }

    pub fn reset(&mut self) {
        self.write_offset = 0;
        self.read_offset = 0;
        self.req_res_code.clear();
        self.buffer.fill(0);
    }

    fn status() -> MboxStatus {
        MboxStatus(Self::MBOX_STATUS.read())
    }

    fn read(channel: u8) -> u32 {
        loop {
            while Self::status().is_empty() {
                core::hint::spin_loop();
            }
            let data = Self::MBOX_READ.read();
            let read_channel = (data & 0xF) as u8;

            if read_channel == channel {
                return data >> 4;
            }
        }
    }

    fn write(channel: u8, data: u32) {
        while Self::status().is_full() {
            core::hint::spin_loop();
        }

        Self::MBOX_WRITE.write(data << 4 | channel as u32);
    }

    fn call(&self, channel: u8) {
        let address = core::ptr::addr_of!(self.size) as usize >> 4;

        Self::write(channel, address as u32);
        let _read_address = Self::read(channel);
        //assert_eq!(address, read_address as usize);
    }

    pub fn push_tag(&mut self, request: PropertyMessageRequest) {
        let (_, dst) = self.buffer.split_at_mut(self.write_offset);
        self.write_offset += request.copy_to(dst);
    }

    pub fn pop_tag(&mut self, response: &mut PropertyMessageResponse) {
        let (_, src) = self.buffer.split_at_mut(self.read_offset);
        self.read_offset += response.fill_from(src);
    }

    pub fn pop_values<T: Copy>(&mut self) -> T {
        unsafe {
            let src: *const u32 = self.buffer.as_ptr().add(self.read_offset).cast();
            let value_buffer_len = core::ptr::read_volatile(src.add(1));
            let bytes_len = ((value_buffer_len as usize + 3) & !0b11) + 12;
            self.read_offset += bytes_len;
            let data: *const T = src.add(3).cast();
            core::ptr::read_volatile(data)
        }
    }

    pub fn skip_tag(&mut self) {
        let (_, src) = self.buffer.split_at_mut(self.read_offset);
        self.read_offset += PropertyMessageResponse::peek_len(src);
    }

    pub fn submit_messages(&mut self, channel: u8) -> Result<(), MailboxError> {
        // crate::peripherals::uart::Uart0::put_hex_bytes(&self.buffer);
        self.call(channel);

        // crate::peripherals::uart::Uart0::put_hex_bytes(&self.req_res_code.raw_value().to_ne_bytes());
        // crate::peripherals::uart::Uart0::putc(b'\n');
        // crate::peripherals::uart::Uart0::put_hex_bytes(&self.buffer);
        // crate::peripherals::uart::Uart0::putc(b'\n');
        // crate::peripherals::uart::Uart0::putc(b'\n');

        match self.req_res_code.get() {
            RequestResponseStatus::Success => Ok(()),
            e => Err(MailboxError::RequestResponseError(e)),
        }
    }
}
