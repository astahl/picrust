use core::ptr::read_volatile;

use crate::exception::ExceptionSyndrome;
use crate::peripherals::mmio;
use crate::peripherals::mmio::MMIO;

use super::mmio::DynamicMmioField;

#[repr(align(16), C)]
pub struct Mailbox<const BUFFER_SIZE: usize> {
    size: u32,
    req_res_code: ReqResCode,
    buffer: [u32; BUFFER_SIZE],
}


#[derive(Debug)]
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

#[derive(Debug)]
pub enum MailboxError {
    Unknown,
    RequestResponseError(RequestResponseStatus),
    BufferOverflow,
    BufferSizeMismatch,
    ResponseIterationError,
    ResponseReinterpretationError,
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

pub struct FbDepth { bits_per_pixel: u32 }

pub struct FbPitch { bytes_per_line: u32 }

pub struct FbDimensions { width_px: u32, height_px: u32 }

pub struct FbOffset { x_px: u32, y_px: u32 }

pub struct EdidBlock {
    block_number: u32,
    status: u32,
    data: [u8; 128],
}

pub struct MemoryBlock {
    address: u32,
    size: u32
}

pub type Palette = [u32; 256];

pub struct PaletteChange<const N: usize> {
    offset: u32,
    length: u32,
    values: [u32; N]
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


const MBOX_BASE: usize = 0xB880; //0x201000;
impl<const BUFFER_SIZE: usize> Mailbox<BUFFER_SIZE> {
    const MBOX_READ: MMIO<MBOX_BASE, 0x00> = MMIO();
    const MBOX_POLL: MMIO<MBOX_BASE, 0x10> = MMIO();
    const MBOX_SENDER: MMIO<MBOX_BASE, 0x14> = MMIO();
    const MBOX_STATUS: MMIO<MBOX_BASE, 0x18> = MMIO();
    const MBOX_CONFIG: MMIO<MBOX_BASE, 0x1C> = MMIO();
    const MBOX_WRITE: MMIO<MBOX_BASE, 0x20> = MMIO();

    pub const fn new() -> Self {
        assert!(BUFFER_SIZE > 0);
        Self {
            size: 12, // header + null tag
            req_res_code: ReqResCode::cleared(),
            buffer: [0; BUFFER_SIZE],
        }
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

    const fn buffer_end_index(&self) -> usize {
        (self.size as usize - 12) >> 2
    }

    pub fn push_request_raw(&mut self, tag: u32, value_buffer_byte_size: u32) -> Result<&mut [u32], MailboxError> {
        let message_u32_size = ((core::mem::size_of::<MessageHeader>() + value_buffer_byte_size as usize + 3) >> 2) as usize;
        let message_byte_size = message_u32_size << 2;
        let message_start = self.buffer_end_index();
        let message_end = message_start + message_u32_size;
        if message_end > BUFFER_SIZE - 1 { // leave room for the null tag
            return Err(MailboxError::BufferOverflow);
        }
    
        self.size += message_byte_size as u32;
        self.buffer[message_start] = tag;
        self.buffer[message_start + 1] = value_buffer_byte_size;
        self.buffer[message_start + 2] = 0;
        let value_buffer = self.buffer.get_mut((message_start + 3)..message_end);
        value_buffer.ok_or(MailboxError::BufferOverflow)
    }

    pub fn submit_messages<'a>(&'a mut self, channel: u8) -> Result<ResponseIterator<'a>, MailboxError> {
        // for (i, v) in self.buffer.iter().enumerate() {
        //     crate::peripherals::uart::Uart0::put_uint(i as u64);
        //     crate::peripherals::uart::Uart0::putc(b'>');
        //     crate::peripherals::uart::Uart0::put_uint(*v as u64);
        //     crate::peripherals::uart::Uart0::putc(b'\n');
        // }
        self.call(channel);

        // crate::peripherals::uart::Uart0::put_hex_bytes(&self.req_res_code.raw_value().to_ne_bytes());
        // crate::peripherals::uart::Uart0::putc(b'\n');
        // for (i, v) in self.buffer.iter().enumerate() {
        //     crate::peripherals::uart::Uart0::put_uint(i as u64);
        //     crate::peripherals::uart::Uart0::putc(b'<');
        //     crate::peripherals::uart::Uart0::put_uint(*v as u64);
        //     crate::peripherals::uart::Uart0::putc(b'\n');
        // }
        // crate::peripherals::uart::Uart0::putc(b'\n');

        match self.req_res_code.get() {
            RequestResponseStatus::Success => Ok(ResponseIterator{buffer: &self.buffer}),
            e => Err(MailboxError::RequestResponseError(e)),
        }
    }

}

pub fn simple_single_call<Q, R: Clone>(tag: u32, request_value: Q) -> Result<R, MailboxError> {
    let byte_size = core::mem::size_of::<Q>().max(core::mem::size_of::<R>());
    let mut mbox = Mailbox::<64>::new();
    let mut buffer = mbox.push_request_raw(tag, byte_size as u32)?;
    unsafe {
        *buffer.as_mut_ptr().cast::<Q>() = request_value;
    };
    let mut responses = mbox.submit_messages(8)?;
    responses.nth(0).ok_or(MailboxError::ResponseIterationError)??.try_value_as().ok_or(MailboxError::ResponseReinterpretationError).cloned()
}

pub struct ResponseIterator<'a> {
    buffer: &'a [u32]
}

pub struct MessageHeader {
    pub tag: u32,
    value_length: u32,
    response_code_length: u32,
}

pub struct Response<'a> {
    pub header: MessageHeader,
    pub value_buffer: &'a [u32]
}

impl<'a>  Response<'a> {
    pub unsafe fn value_as_unchecked<T>(&self) -> &'a T {
        &*self.value_buffer.as_ptr().cast::<T>()
    } 

    pub fn try_value_as<T>(&self) -> Option<&'a T> {
        debug_assert!(self.value_buffer.as_ptr().align_offset(core::mem::align_of::<T>()) == 0);
        debug_assert!(self.value_buffer.len() >= (core::mem::size_of::<T>() + 3) >> 2);
        if self.value_buffer.as_ptr().align_offset(core::mem::align_of::<T>()) == 0
        && self.value_buffer.len() >= (core::mem::size_of::<T>() + 3) >> 2 {
            Some(unsafe { self.value_as_unchecked() })
        } else {
            None
        }
    }
}

impl<'a> Iterator for ResponseIterator<'a> {
    type Item = Result<Response<'a>, MailboxError>;

    fn next(&mut self) -> Option<Self::Item> {
        const HEADER_BYTE_SIZE: usize = core::mem::size_of::<MessageHeader>();
        const HEADER_U32_SIZE: usize = core::mem::size_of::<MessageHeader>() >> 2;
        if self.buffer.len() < 1 {
            return None;
        }
        let tag_ptr: *const u32 = self.buffer.as_ptr();
        let tag = unsafe { tag_ptr.read_volatile() };
        if tag == 0 {
            None
        } else if self.buffer.len() < HEADER_U32_SIZE {
            Some(Err(MailboxError::BufferSizeMismatch)) 
        } else {
            let header_ptr: *const MessageHeader = self.buffer.as_ptr().cast();
            let header = unsafe { header_ptr.read_volatile() };
            
            let end_index = ((HEADER_BYTE_SIZE + header.value_length as usize) + 3) >> 2;
            let (head, tail) = self.buffer.split_at(end_index);
            self.buffer = tail;
            Some(head.get(HEADER_U32_SIZE..)
                .ok_or(MailboxError::BufferSizeMismatch)
                .map(|value_buffer| Response{
                header,
                value_buffer
            }))
        }
    }
}
