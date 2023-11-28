#![allow(dead_code)]

fn delay(mut count: usize) {
    while count > 0 {
        count -= 1;
        core::hint::spin_loop();
    }
}

mod mmio {
    const PERIPHERAL_BASE: usize = 0x3F000000;

    pub fn write_to(ptr: *mut u32, data: u32) {
        unsafe { core::ptr::write_volatile(ptr, data) };
    }

    pub fn read_from(ptr: *const u32) -> u32 {
        unsafe { core::ptr::read_volatile(ptr) }
    }

    pub struct  MMIO<const BASE: usize, const OFFSET: usize>();
    impl<const BASE: usize, const OFFSET: usize> MMIO<BASE, OFFSET> {
        const ADDRESS: usize = PERIPHERAL_BASE + BASE + OFFSET;

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

    pub struct Uart0();

    const UART0_BASE: usize = 0x201000;
    impl Uart0 {
        const UART0_DR: MMIO<UART0_BASE, 0x00> = MMIO();
        const UART0_RSRECR: MMIO<UART0_BASE, 0x04> = MMIO();
        const UART0_FR: MMIO<UART0_BASE, 0x18> = MMIO();
        const UART0_ILPR: MMIO<UART0_BASE, 0x20> = MMIO();
        const UART0_IBRD: MMIO<UART0_BASE, 0x24> = MMIO();
        const UART0_FBRD: MMIO<UART0_BASE, 0x28> = MMIO();
        // Line Control Register
        const UART0_LCRH: MMIO<UART0_BASE, 0x2C> = MMIO();
        // CR Control Register
        const UART0_CR: MMIO<UART0_BASE, 0x30> = MMIO();
        const UART0_IFLS: MMIO<UART0_BASE, 0x34> = MMIO();
        // Interrupt Mask Set-Clear
        const UART0_IMSC: MMIO<UART0_BASE, 0x38> = MMIO();
        const UART0_RIS: MMIO<UART0_BASE, 0x3C> = MMIO();
        const UART0_MIS: MMIO<UART0_BASE, 0x40> = MMIO();
        // ICR Interrupt Clear Register
        const UART0_ICR: MMIO<UART0_BASE, 0x44> = MMIO();
        const UART0_DMACR: MMIO<UART0_BASE, 0x48> = MMIO();
        const UART0_ITCR: MMIO<UART0_BASE, 0x80> = MMIO();
        const UART0_ITIP: MMIO<UART0_BASE, 0x84> = MMIO();
        const UART0_ITOP: MMIO<UART0_BASE, 0x88> = MMIO();
        const UART0_TDR: MMIO<UART0_BASE, 0x8C> = MMIO();
    
        pub fn init() {
            // disable UART
            Self::UART0_CR.write(0x00000000);
    
            gpio::Gpio::init_uart0();
    
            // Clear all pending UART interrupts
            Self::UART0_ICR.write(0x7FF);
    
            // Set UART Baud Rate to 115200 (look at docs for formula for values)
            Self::UART0_IBRD.write(1);
            Self::UART0_FBRD.write(40);
    
            // Set Line Control Register to 01110000
            //                                 ^ use 8 item FIFO
            //                               ^^ 8 bit words
            Self::UART0_LCRH.write((1 << 4) | (1 << 5) | (1 << 6));
    
            // Set Interrupt Mask Set-Clear Register to 11111110010, disabling all UART interrupts
            Self::UART0_IMSC.write(
                (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) | (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10),
            );
    
            // enable UART 1100000001
            //                      ^ enable Hardware
            //              ^ enable receive
            //             ^ enable transmit
            Self::UART0_CR.write((1 << 0) | (1 << 8) | (1 << 9));
        }

        pub fn putc(c: u8) {
            while Self::flags().transmit_fifo_full() {
                core::hint::spin_loop();
            }
            Self::UART0_DR.write(c as u32);
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
            let read = Self::UART0_DR.read();
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
                let read = Self::UART0_DR.read();
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
            UartFlags(Self::UART0_FR.read())
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
    pub enum PropertyMessageRequest {
        Null = 0,
        VcGetFirmwareRevision = 0x00000001,
        HwGetBoardModel = 0x00010001,
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
        FbGetPitch = 0x00040008,
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
        FbGetPitch {
            bytes_per_line: u32,
        } = 0x00040008,
    }

    impl PropertyMessageRequest {
        const fn value_buffer_len(&self) -> u32 {
            match self {
                Self::Null | Self::FbReleaseBuffer => 0,

                Self::FbTestDepth { .. }
                | Self::FbGetPitch
                | Self::FbGetDepth
                | Self::VcGetFirmwareRevision
                | Self::HwGetBoardModel
                | Self::FbSetDepth { .. } => 4,

                Self::FbGetPhysicalDimensions
                | Self::FbGetVirtualDimensions
                | Self::FbAllocateBuffer { .. }
                | Self::FbTestPhysicalDimensions { .. }
                | Self::FbSetPhysicalDimensions { .. }
                | Self::FbTestVirtualDimensions { .. }
                | Self::FbSetVirtualDimensions { .. } => 8,
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
            // crate::peripherals::uart::put_hex_bytes(&self.buffer);
            self.call(channel);

            // crate::peripherals::uart::put_hex_bytes(&self.req_res_code.raw_value().to_ne_bytes());
            // crate::peripherals::uart::putc(b'\n');
            // crate::peripherals::uart::put_hex_bytes(&self.buffer);

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
