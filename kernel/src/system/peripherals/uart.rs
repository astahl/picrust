use crate::peripherals::gpio;
use crate::peripherals::mmio::MMIO;

use super::gpio::PinSet;

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

        gpio::Gpio::set_pull_resistors(PinSet::select(&[14, 15]), gpio::Resistor::None);

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
        const SYMBOLS: &[u8; 16] = b"0123456789abcdef";
        let upper = (byte >> 4) & 0xF;
        let lower = byte & 0xF;
        Self::putc(SYMBOLS[upper as usize]);
        Self::putc(SYMBOLS[lower as usize]);
    }

    pub fn put_hex_bytes(buffer: &[u8]) {
        for (line, chunk) in buffer.chunks(16).enumerate() {
            if line != 0 {
                Self::putc(b'\n');
            }
            for chunk in chunk.chunks(4) {
                for byte in chunk {
                    Self::put_hex(*byte);
                    Self::putc(b' ');
                }
                Self::putc(b' ');
            }
        }
    }

    pub fn put_hex_usize(value: usize) {
        Self::putc(b'0');
        Self::putc(b'x');
        if value == 0 {
            Self::putc(b'0');
        } else {
            for b in value.to_be_bytes().into_iter().skip_while(|b| *b == 0) {
                Self::put_hex(b);
            }
        }
    }

    pub fn put_memory(ptr: *const u8, len: usize) {
        Self::put_hex_usize(ptr as usize);
        Self::putc(b':');
        for i in 0..len {
            Self::putc(b' ');
            let position = ptr.wrapping_add(i);
            let value = unsafe { position.read_volatile() };
            Self::put_hex(value);
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

pub struct Uart0Formatter();

impl core::fmt::Write for Uart0Formatter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Uart0::puts(s);
        Ok(())
    }
}
