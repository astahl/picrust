#![allow(dead_code)]

pub fn delay(mut count: usize) {
    while count > 0 {
        count -= 1;
        core::hint::spin_loop();
    }
}

pub fn led_on() {
    use mailbox::*;
    let mut mailbox = Mailbox::<256>::new();
    let _ = mailbox.request(8, &[
        PropertyMessageRequest::SetOnboardLedStatus { pin_number: Led::Status, status: LedStatus::On },
        PropertyMessageRequest::Null]);
}

pub fn led_off() {
    use mailbox::*;
    let mut mailbox = Mailbox::<256>::new();
    let _ = mailbox.request(8, &[
        PropertyMessageRequest::SetOnboardLedStatus { pin_number: Led::Status, status: LedStatus::Off },
        PropertyMessageRequest::Null]);
}

pub fn blink_led() {
    led_on();
    delay(100000);
    led_off();
    delay(100000);
}


pub struct BcmHost {
    pub peripheral_address: usize,
    pub peripheral_size: usize,
    pub sdram_address: usize
}


#[cfg(feature="raspi1")]
compile_error!("Can't compile for Raspberry Pi Model 1 / Zero.");

#[cfg(
    any(
        all(feature="raspi4", feature="raspi3b"), 
        all(feature="raspi2b", feature="raspi3b"),
        all(feature="raspi2b", feature="raspi4"), 
    ))]
compile_error!("Can't compile for multiple Raspberry Pi Models.");

#[cfg(feature="bcm2711")]
pub const BCM_HOST: BcmHost = BcmHost{
    peripheral_address: 0xFE000000,
    peripheral_size: 0x01800000,
    sdram_address: 0xC0000000,
};

#[cfg(any(feature="bcm2837", feature="bcm2836"))]
pub const BCM_HOST: BcmHost = BcmHost{
    peripheral_address: 0x3F000000,
    peripheral_size: 0x01000000,
    sdram_address: 0xC0000000,
};

#[cfg(feature="bcm2835")]
pub const BCM_HOST: BcmHost = BcmHost{
    peripheral_address: 0x20000000,
    peripheral_size: 0x01000000,
    sdram_address: 0x40000000,
};

mod mmio {

    pub fn write_to(ptr: *mut u32, data: u32) {
        unsafe { core::ptr::write_volatile(ptr, data) };
    }

    pub fn read_from(ptr: *const u32) -> u32 {
        unsafe { core::ptr::read_volatile(ptr) }
    }

    pub struct  MMIO<const BASE: usize, const OFFSET: usize>();
    impl<const BASE: usize, const OFFSET: usize> MMIO<BASE, OFFSET> {
        const ADDRESS: usize = crate::peripherals::BCM_HOST.peripheral_address + BASE + OFFSET;

        pub fn write(&self, data: u32) {
            write_to(Self::ADDRESS as *mut u32, data);
        }

        pub fn read(&self) -> u32 {
            read_from(Self::ADDRESS as *const u32)
        }

        pub fn update(&self, mask: u32, data: u32) -> u32 {
            let old_value = self.read();
            let new_value = (!mask & old_value) | (mask & data);
            self.write(new_value);
            old_value
        }
    }
}

mod gpio {

    use crate::peripherals::delay;
    use crate::peripherals::mmio::MMIO;

    pub struct Gpio ();

    const GPIO_BASE: usize = 0x200000;
    impl Gpio {
        const GPPUD: MMIO<GPIO_BASE, 0x94> = MMIO();
        const GPPUDCLK0: MMIO<GPIO_BASE, 0x98> = MMIO();
    
        pub fn init_uart0() {
            // select GPIO Pin Update Disable
            Self::GPPUD.write(0x00000000);
            delay(150);
    
            // select Pin 14 and 15
            Self::GPPUDCLK0.write((1 << 14) | (1 << 15));
            delay(150);
    
            // Commit Pin Update
            Self::GPPUDCLK0.write(0x00000000);
        }
    }
}

pub mod uart {
    use crate::peripherals::gpio;
    use crate::peripherals::mmio::MMIO;

    pub struct Pl011Uart<const UART_BASE: usize>();

    pub type Uart0 = Pl011Uart<0x201000>;
    pub type Uart2 = Pl011Uart<0x201400>;
    pub type Uart3 = Pl011Uart<0x201600>;
    pub type Uart4 = Pl011Uart<0x201800>;
    pub type Uart5 = Pl011Uart<0x201a00>;

    impl<const UART_BASE: usize> Pl011Uart<UART_BASE> {
        const DATA_REGISTER: MMIO<UART_BASE, 0x00> = MMIO();
        const RECV_STATUS_ERROR_CLEAR_REGISTER: MMIO<UART_BASE, 0x04> = MMIO();
        const FLAG_REGISTER: MMIO<UART_BASE, 0x18> = MMIO();
        const UNUSED_IRDA_REGISTER: MMIO<UART_BASE, 0x20> = MMIO();
        const INTEGER_BAUD_RATE_DIVISOR: MMIO<UART_BASE, 0x24> = MMIO();
        const FRACTIONAL_BAUD_RATE_DIVISOR: MMIO<UART_BASE, 0x28> = MMIO();
        // Line Control Register
        const LINE_CONTROL_REGISTER: MMIO<UART_BASE, 0x2C> = MMIO();
        // CR Control Register
        const CONTROL_REGISTER: MMIO<UART_BASE, 0x30> = MMIO();
        const INTERRUPT_FIFO_LEVEL_SELECT: MMIO<UART_BASE, 0x34> = MMIO();
        // Interrupt Mask Set-Clear
        const INTERRUPT_MASK_SET_CLEAR: MMIO<UART_BASE, 0x38> = MMIO();
        const RAW_INTERRUPT_STATUS: MMIO<UART_BASE, 0x3C> = MMIO();
        const MASKED_INTERRUPT_STATUS: MMIO<UART_BASE, 0x40> = MMIO();
        // ICR Interrupt Clear Register
        const INTERRUPT_CLEAR_REGISTER: MMIO<UART_BASE, 0x44> = MMIO();
        const DMA_CONTROL_REGISTER: MMIO<UART_BASE, 0x48> = MMIO();
        const INTEGRATION_TEST_CONTROL_REGISTER: MMIO<UART_BASE, 0x80> = MMIO();
        const INTEGRATION_TEST_ITIP: MMIO<UART_BASE, 0x84> = MMIO();
        const INTEGRATION_TEST_ITOP: MMIO<UART_BASE, 0x88> = MMIO();
        const TEST_DATA_REGISTER: MMIO<UART_BASE, 0x8C> = MMIO();
    
        pub fn init() {
            // disable UART
            Self::CONTROL_REGISTER.write(0x00000000);
    
            gpio::Gpio::init_uart0();
    
            // Clear all pending UART interrupts
            Self::INTERRUPT_CLEAR_REGISTER.write(0x7FF);
    
            // Set UART Baud Rate to 115200 (look at docs for formula for values)
            Self::INTEGER_BAUD_RATE_DIVISOR.write(1);
            Self::FRACTIONAL_BAUD_RATE_DIVISOR.write(40);
    
            // Set Line Control Register to 01110000
            //                                 ^ use 8 item FIFO
            //                               ^^ 8 bit words
            Self::LINE_CONTROL_REGISTER.write((1 << 4) | (1 << 5) | (1 << 6));
    
            // Set Interrupt Mask Set-Clear Register to 11111110010, disabling all UART interrupts
            Self::INTERRUPT_MASK_SET_CLEAR.write(
                (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) | (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10),
            );
    
            // enable UART 1100000001
            //                      ^ enable Hardware
            //              ^ enable receive
            //             ^ enable transmit
            Self::CONTROL_REGISTER.write((1 << 0) | (1 << 8) | (1 << 9));
        }

        pub fn putc(c: u8) {
            while Self::flags().transmit_fifo_full() {
                core::hint::spin_loop();
            }
            Self::DATA_REGISTER.write(c as u32);
        }

        pub fn put_hex(byte: u8) {
            let upper = (byte >> 4) & 0xF;
            let lower = byte & 0xF;
            match upper {
                0..=9 => Self::putc(b'0' + upper),
                _ => Self::putc(b'A' + (upper - 10)),
            }
            match lower {
                0..=9 => Self::putc(b'0' + lower),
                _ => Self::putc(b'A' + (lower - 10)),
            }
        }


        pub fn put_hex_bytes(buffer: &[u8]) {
            for chunk in buffer.chunks(16) {
                for chunk in chunk.chunks(4) {
                    for byte in chunk {
                        Self::put_hex(*byte);
                        Self::putc(b' ');
                    }
                    Self::putc(b' ');
                }
                Self::putc(b'\n');
            }
        }

        pub fn put_uint(mut value: u64) {
            let mut power_of_ten = 1;
            let mut next_power_of_ten = 10;
            while next_power_of_ten < value {
                power_of_ten = next_power_of_ten;
                next_power_of_ten *= 10;
            }
            while power_of_ten > 0 {
                let quotient = (value / power_of_ten) as u8;
                value %= power_of_ten;
                Self::putc(b'0' + quotient);
                power_of_ten /= 10;
            }
        }

        pub fn get_byte() -> Result<u8, UartStatus> {
            while Self::flags().receive_fifo_empty() {
                core::hint::spin_loop();
            }
            let read = Self::DATA_REGISTER.read();
            let status = UartStatus(read >> 8);
            if status.is_clear() {
                Ok(read as u8)
            } else {
                Err(status)
            }
        }

        pub fn get_bytes(buffer: &mut [u8]) -> Result<usize, UartStatus> {
            while Self::flags().receive_fifo_empty() {
                core::hint::spin_loop();
            }
            let mut count: usize = 0;
            while !Self::flags().receive_fifo_empty() {
                let read = Self::DATA_REGISTER.read();
                let status = UartStatus(read >> 8);
                if !status.is_clear() {
                    return Err(status);
                }
                if count < buffer.len() {
                    unsafe { *buffer.get_unchecked_mut(count) = read as u8 };
                } else {
                    return Ok(count);
                }
                count += 1;
            }
            Ok(count)
        }

        pub fn puts(string: &str) {
            for b in string.bytes() {
                Self::putc(b);
            }
            Self::putc(0);
        }

        pub fn flags() -> UartFlags {
            UartFlags(Self::FLAG_REGISTER.read())
        }
    }


    pub struct UartStatus(u32);
    impl UartStatus {
        const OE: u32 = 1 << 3;
        const BE: u32 = 1 << 2;
        const PE: u32 = 1 << 1;
        const FE: u32 = 1;

        pub const fn overrun_error(&self) -> bool {
            self.0 & Self::OE != 0
        }

        pub const fn break_error(&self) -> bool {
            self.0 & Self::BE != 0
        }

        pub const fn parity_error(&self) -> bool {
            self.0 & Self::PE != 0
        }

        pub const fn framing_error(&self) -> bool {
            self.0 & Self::FE != 0
        }

        pub const fn is_clear(&self) -> bool {
            self.0 == 0
        }
    }

    pub struct UartFlags(u32);
    impl UartFlags {
        const TXFE: u32 = 1 << 7;
        const RXFF: u32 = 1 << 6;
        const TXFF: u32 = 1 << 5;
        const RXFE: u32 = 1 << 4;
        const BUSY: u32 = 1 << 3;
        const CTS: u32 = 1;

        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the Line Control Register, UART_LCRH.
        /// If the FIFO is disabled, this bit is set when the transmit holding register is empty.
        /// If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does not indicate if there is data in the transmit shift register.
        pub const fn transmit_fifo_empty(&self) -> bool {
            self.0 & Self::TXFE != 0
        }

        // Receive FIFO full. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
        // If the FIFO is disabled, this bit is set when the receive holding register is full.
        // If the FIFO is enabled, the RXFF bit is set when the receive FIFO is full.
        pub const fn receive_fifo_full(&self) -> bool {
            self.0 & Self::RXFF != 0
        }

        // Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
        // If the FIFO is disabled, this bit is set when the transmit holding register is full.
        // If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        pub const fn transmit_fifo_full(&self) -> bool {
            self.0 & Self::TXFF != 0
        }

        // Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
        // If the FIFO is disabled, this bit is set when the receive holding register is empty.
        // If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        pub const fn receive_fifo_empty(&self) -> bool {
            self.0 & Self::RXFE != 0
        }

        // UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains set until the complete byte, including all the stop bits, has been sent from the shift register.
        // This bit is set as soon as the transmit FIFO becomes non- empty, regardless of whether the UART is enabled or not.
        pub const fn busy(&self) -> bool {
            self.0 & Self::BUSY != 0
        }

        // Clear to send. This bit is the complement of the UART clear to send, nUARTCTS, modem status input. That is, the bit is 1 when nUARTCTS is LOW.
        pub const fn clear_to_send(&self) -> bool {
            self.0 & Self::CTS != 0
        }
    }
}

pub mod mailbox {
    use crate::peripherals::mmio;
    use crate::peripherals::mmio::MMIO;


    #[repr(align(16), C)]
    pub struct Mailbox<const BUFFER_SIZE: usize> {
        size: u32,
        req_res_code: ReqResCode,
        buffer: [u8; BUFFER_SIZE],
    }
    pub struct ReqResCode(u32);

    impl ReqResCode {
        pub const fn new() -> Self {
            Self(0)
        }

        pub fn clear(&mut self) {
            mmio::write_to(&mut self.0 as *mut u32, 0);
        }

        pub fn is_pending(&self) -> bool {
            mmio::read_from(&self.0 as *const u32) == 0
        }

        pub fn is_success(&self) -> bool {
            mmio::read_from(&self.0 as *const u32) == 0x80000000
        }

        pub fn is_error_parsing_request_buffer(&self) -> bool {
            mmio::read_from(&self.0 as *const u32) == 0x80000001
        }

        pub fn raw_value(&self) -> u32 {
            mmio::read_from(&self.0 as *const u32)
        }
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
        On = 1
    }

    #[repr(u32)]
    #[derive(Copy, Clone)]
    pub enum Led {
        Status = 42,
        Power = 130
    }

    #[repr(u32)]
    #[derive(Copy, Clone)]
    pub enum PixelOrder {
        Bgr = 0,
        Rgb = 1
    }

    #[repr(u32)]
    #[derive(Copy, Clone)]
    pub enum AlphaMode {
        Enabled0Opaque = 0,
        Enabled0Transparent = 1,
        Ignored
    }

    #[repr(u32)]
    #[derive(Copy, Clone)]
    pub enum PropertyMessageRequest {
        Null = 0,
        VcGetFirmwareRevision = 0x00000001,
        HwGetBoardModel = 0x00010001,
        GetEdidBlock {
            block_number: u32
        } = 0x00030020,
        GetOnboardLedStatus  = 0x00030041,
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
        GetEdidBlock {
            block_number: u32,
            status: u32,
            data: [u8; 128]
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
        FbGetPixelOrder { state: PixelOrder } = 0x00040006,
        FbTestPixelOrder { state: PixelOrder } = 0x00044006,
        FbSetPixelOrder { state: PixelOrder } = 0x00048006,
        FbGetAlphaMode { state: AlphaMode } = 0x00040007,
        FbTestAlphaMode { state: AlphaMode } = 0x00044007,
        FbSetAlphaMode { state: AlphaMode } = 0x00048007,
        FbGetPitch {
            bytes_per_line: u32,
        } = 0x00040008,
        FbGetVirtualOffset { x_px: u32, y_px: u32 } = 0x00040009,
        FbTestVirtualOffset { x_px: u32, y_px: u32 } = 0x00044009,
        FbSetVirtualOffset { x_px: u32, y_px: u32 } = 0x00048009,
    }

    impl PropertyMessageRequest {
        const fn value_buffer_len(&self) -> u32 {
            match self {
                Self::Null | Self::FbReleaseBuffer => 0,

                Self::FbGetPitch
                | Self::FbGetDepth
                | Self::FbTestDepth { .. }
                | Self::FbSetDepth { .. } 
                | Self::VcGetFirmwareRevision
                | Self::HwGetBoardModel
                | Self::FbGetPixelOrder 
                | Self::FbTestPixelOrder { .. } 
                | Self::FbSetPixelOrder { .. } 
                | Self::FbGetAlphaMode
                | Self::FbTestAlphaMode { .. }
                | Self::FbSetAlphaMode { .. }
                => 4,

                Self::FbGetPhysicalDimensions
                | Self::FbGetVirtualDimensions
                | Self::FbAllocateBuffer { .. }
                | Self::FbTestPhysicalDimensions { .. }
                | Self::FbSetPhysicalDimensions { .. }
                | Self::FbTestVirtualDimensions { .. }
                | Self::FbSetVirtualDimensions { .. } 
                | Self::FbGetVirtualOffset
                | Self::FbTestVirtualOffset { .. }
                | Self::FbSetVirtualOffset { .. }
                | Self::GetOnboardLedStatus
                | Self::TestOnboardLedStatus { .. }
                | Self::SetOnboardLedStatus { .. } 
                => 8,

                Self::GetEdidBlock { .. } => 136
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
                    let len = self.value_buffer_len();
                    unsafe {
                        let mut ptr = buffer.as_ptr() as *mut u32;
                        let this = <*const _>::from(self).cast::<u32>();
                        core::ptr::copy_nonoverlapping(this, ptr, 1);
                        ptr = ptr.add(1);
                        *ptr = len;
                        ptr = ptr.add(1);
                        *ptr = 0x0;
                        ptr = ptr.add(1);
                        let this = this.add(1);
                        core::ptr::copy_nonoverlapping(this, ptr, (len as usize + 3) / 4);
                    }
                    len as usize + 12
                }
            }
        }

        pub const fn copied_len(&self) -> usize {
            match self {
                Self::Null => 4,
                _ => ((self.value_buffer_len() as usize + 3) & !0b11) + 12,
            }
        }
    }

    impl PropertyMessageResponse {
        pub fn fill_from(&mut self, buffer: &[u8]) {
            unsafe {
                let ptr = buffer.as_ptr() as *const u32;
                let mut this = <*mut _>::from(self).cast::<u32>();
                core::ptr::copy_nonoverlapping(ptr, this, 1);
                if *ptr == 0 {
                    return;
                }
                let ptr = ptr.add(2);
                let value_buffer_len = *ptr & 0x7fffffff;
                let ptr = ptr.add(1);
                this = this.add(1);
                let value_ptr = this.cast::<u8>();
                core::ptr::copy_nonoverlapping(
                    ptr.cast::<u8>(),
                    value_ptr,
                    value_buffer_len as usize,
                );
            }
        }

        pub fn peek_len(buffer: &[u8]) -> usize {
            unsafe {
                let ptr = buffer.as_ptr() as *const u32;
                if *ptr == 0 {
                    return 4;
                }
                let ptr = ptr.add(2);
                let value_buffer_len = *ptr & 0x7fffffff;
                ((value_buffer_len as usize + 3) & !0b11) + 12
            }
        }

        pub const fn tag_name(&self) -> &str {
            match self {
                Self::Null => "null",
                _ => "somethign",
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
                size: BUFFER_SIZE as u32 * 4 + 8,
                req_res_code: ReqResCode::new(),
                buffer: [0; BUFFER_SIZE],
            }
        }

        fn status() -> MboxStatus {
            MboxStatus(Self::MBOX_STATUS.read())
        }

        fn call(&self, channel: u8) {
            let this = <*const _>::from(self);
            let address = (this as u32 & !0xF) | (channel as u32 & 0xF);

            while Self::status().is_full() {
                core::hint::spin_loop();
            }
            Self::MBOX_WRITE.write(address);

            loop {
                while Self::status().is_empty() {
                    core::hint::spin_loop();
                }
                let read = Self::MBOX_READ.read();
                if read == address {
                    break;
                }
            }
        }

        pub fn request<const TAG_COUNT: usize>(
            &mut self,
            channel: u8,
            request: &[PropertyMessageRequest; TAG_COUNT],
        ) -> Result<[PropertyMessageResponse; TAG_COUNT], u32> {
            self.buffer.fill(0x00);

            let mut buffer = self.buffer.as_mut_slice();
            for req in request {
                let length = req.copied_len();
                let (head, rest) = buffer.split_at_mut(length);
                req.copy_to(head);
                buffer = rest;
            }
            self.req_res_code.clear();
            // crate::peripherals::uart::Uart0::put_hex_bytes(&self.buffer);
            self.call(channel);

            // crate::peripherals::uart::Uart0::put_hex_bytes(&self.req_res_code.raw_value().to_ne_bytes());
            // crate::peripherals::uart::Uart0::putc(b'\n');
            // crate::peripherals::uart::Uart0::put_hex_bytes(&self.buffer);

            if !self.req_res_code.is_success() {
                return Err(self.req_res_code.raw_value());
            }

            let mut buffer = self.buffer.as_slice();

            let mut response = [PropertyMessageResponse::Null; TAG_COUNT];

            for res in response.as_mut_slice() {
                let length = PropertyMessageResponse::peek_len(buffer);
                // crate::peripherals::uart::put_uint(length as u64);
                // crate::peripherals::uart::putc(b'\n');
                let (head, rest) = buffer.split_at(length);
                res.fill_from(head);
                buffer = rest;
            }
            Ok(response)
        }
    }
}
