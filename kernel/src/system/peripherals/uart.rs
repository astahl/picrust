use mystd::bitfield::BitField;
use mystd::fixed_point::FixedPoint;
use mystd::io::Write;

use crate::peripherals::gpio;
use crate::system::hal::clocks::Clock;

use super::gpio::PinSet;
use super::mmio::PeripheralRegister;

#[derive(Clone, Copy)]
pub struct Pl011Uart(usize);

pub const UART_0: Pl011Uart = Pl011Uart(0x201000);
// pub type Uart2 = Pl011Uart<0x201400>;
// pub type Uart3 = Pl011Uart<0x201600>;
// pub type Uart4 = Pl011Uart<0x201800>;
// pub type Uart5 = Pl011Uart<0x201a00>;

type UartDataReg = PeripheralRegister<0x00, UartData>;
type UartReceiveStatusErrorClearReg = PeripheralRegister<0x04, u32>;
type UartFlagReg = PeripheralRegister<0x18, UartFlags>;
type UartIlpReg = PeripheralRegister<0x20, u32>;
type UartIntegerBaudRateDivisorReg = PeripheralRegister<0x24, u32>;
type UartFractionalBaudRateDivisorReg = PeripheralRegister<0x28, u32>;
type UartLineControlReg = PeripheralRegister<0x2c, UartLineControl>;
type UartControlReg = PeripheralRegister<0x30, UartControl>;
type UartInterruptFIFOLevelSelectReg = PeripheralRegister<0x34, u32>;
type UartInterruptMaskSetClearReg = PeripheralRegister<0x38, UartInterrupts>;
type UartRawInterruptStatusReg = PeripheralRegister<0x3c, UartInterrupts>;
type UartMaskedInterruptStatusReg = PeripheralRegister<0x40, UartInterrupts>;
type UartInterruptClearReg = PeripheralRegister<0x44, UartInterrupts>;
type UartDMAControlReg = PeripheralRegister<0x48, u32>;
type UartTestControlReg = PeripheralRegister<0x80, u32>;
type UartIntegrationTestInputReg = PeripheralRegister<0x84, u32>;
type UartIntegrationTestOutputReg = PeripheralRegister<0x88, u32>;
type UartTestDataReg = PeripheralRegister<0x8c, u32>;

impl Pl011Uart {

    pub fn init(&self) {
        // NOTE: The UART_LCRH, UART_IBRD, and UART_FBRD registers must not be changed:
        // when the UART is enabled
        // when completing a transmission or a reception when it has been programmed to become disabled.
        // NOTE: Program the control registers as follows:
        // 1. Disable the UART.
        // 2. Wait for the end of transmission or reception of the current character.
        // 3. Flush the transmit FIFO by setting the FEN bit to 0 in the Line Control Register, UART_LCRH. 
        // 4. Reprogram the Control Register, UART_CR.
        // 5. Enable the UART.
        
        // disable UART
        UartControlReg::at(self.0).write(UartControl::disabled());

        while UartFlagReg::at(self.0).read().is_busy() {
            core::hint::spin_loop();
        }

        // flush transmit fifo
        UartLineControlReg::at(self.0).update(UartLineControl::with_fifo_disabled);

        // todo figure out how to select pins / functions for each uart on pi4
        let pins = PinSet::select(&[14, 15]);
        gpio::Gpio::set_functions(pins, gpio::PinFunction::Alt0);
        gpio::Gpio::set_pull_resistors(pins, gpio::Resistor::None);

        // Clear all pending UART interrupts
        UartInterruptClearReg::at(self.0).write(UartInterrupts::all_set());
        
        let clock_rate = Clock::UART.rate().unwrap_or(3_000_000);
        let (brd_int, brd_frac) = UartBitrate::_1200Baud.to_int_frac(clock_rate);
        UartIntegerBaudRateDivisorReg::at(self.0).write(brd_int);
        UartFractionalBaudRateDivisorReg::at(self.0).write(brd_frac);
        
        UartLineControlReg::at(self.0).write(UartLineControl::new().with_word_length(UartWordLength::_8Bits).with_fifo_enabled());
        // mask (disable) all interrupts
        UartInterruptMaskSetClearReg::at(self.0).write(UartInterrupts::all_set());
        
        // enable UART
        UartControlReg::at(self.0).write(UartControl::enabled());
    }

    pub fn put_byte(&self, data: u8) {
        while self.flags().is_transmit_fifo_full() {
            core::hint::spin_loop();
        }
        UartDataReg::at(self.0).write(UartData::send(data));
    }

    pub fn get_byte(&self) -> Result<u8, UartStatus> {
        while self.flags().is_receive_fifo_empty() {
            core::hint::spin_loop();
        }
        let read = UartDataReg::at(self.0).read();
        let (status, data) = read.recv_split();
        if status.is_clear() {
            Ok(data)
        } else {
            Err(status)
        }
    }

    pub fn flags(&self) -> UartFlags {
        UartFlagReg::at(self.0).read()
    }
}

impl mystd::io::Write for Pl011Uart {
    fn write(&mut self, buf: &[u8]) -> mystd::io::Result<usize> {
        while self.flags().is_transmit_fifo_full() {
            core::hint::spin_loop();
        }
        let reg_ptr = UartDataReg::at(self.0).as_mut_ptr().cast::<u32>();
        let mut count = 0;
        for b in buf {
            unsafe { reg_ptr.write_volatile(*b as u32); }
            count += 1;
        }
        Ok(count)
    }

    fn flush(&mut self) -> mystd::io::Result<()> {
        while !self.flags().is_transmit_fifo_empty() {
            core::hint::spin_loop();
        }
        Ok(())
    }
}

impl mystd::io::Read for Pl011Uart {
    fn read(&mut self, buf: &mut [u8]) -> mystd::io::Result<usize> {
        while self.flags().is_receive_fifo_empty() {
            core::hint::spin_loop();
        }

        let reg_ptr = UartDataReg::at(self.0).as_ptr();
        let mut count = 0;
        while count < buf.len() && !self.flags().is_receive_fifo_empty() {
            let received = unsafe { reg_ptr.read_volatile() };
            if received.is_break_error() {
                return Err(mystd::io::Error::Interrupted)
            }
            if received.is_parity_error() || received.is_framing_error() {
                return Err(mystd::io::Error::InvalidData)
            }
            buf[count] = received.data();
            count += 1;
        }
        Ok(count)
    }
}

impl core::fmt::Write for Pl011Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}


struct UartData(BitField<u32>);

impl UartData {
    pub const fn zero() -> Self {
        Self(BitField::zero())
    }

    pub const fn send(value: u8) -> Self {
        Self(BitField::new(value as u32))
    }

    /// Overrun error. This bit is set to 1 if data is received and the receive FIFO is already full.
    /// 
    /// This is cleared to 0 once there is an empty space in the FIFO and a new character can be written to it. 
    pub fn is_overrun_error(&self) -> bool {
        self.0.bit_test(11)
    }

    /// Break error. This bit is set to 1 if a break condition was detected, indicating that the received data input was held LOW for longer than a full-word transmission time (defined as start, data, parity and stop bits).
    /// 
    /// In FIFO mode, this error is associated with the character at the top of the FIFO. When a break occurs, only one 0 character is loaded into the FIFO. The next character is only enabled after the receive data input goes to a 1 (marking state), and the next valid start bit is received.
    pub fn is_break_error(&self) -> bool {
        self.0.bit_test(10)
    }

    /// Parity error. When set to 1, it indicates that the parity of the received data character does not match the parity that the EPS and SPS bits in the Line Control Register, UART_LCRH select.
    /// 
    /// In FIFO mode, this error is associated with the character at the top of the FIFO.
    pub fn is_parity_error(&self) -> bool {
        self.0.bit_test(9)
    }

    /// Framing error. When set to 1, it indicates that the received character did not have a valid stop bit (a valid stop bit is 1). 
    /// 
    /// In FIFO mode, this error is associated with the character at the top of the FIFO.
    pub fn is_framing_error(&self) -> bool {
        self.0.bit_test(8)
    }

    pub fn status(&self) -> UartStatus {
        UartStatus(BitField(self.0.field(8, 4)))
    }

    pub fn recv_split(&self) -> (UartStatus, u8) {
        (self.status(), self.data())
    }

    pub fn data(&self) -> u8 {
        self.0.0 as u8
    }

    #[must_use]
    pub fn with_data(&self, data: u8) -> Self {
        Self(self.0.with_field_set(0, 8, data as u32))
    }
}

pub struct UartStatus(BitField<u32>);
impl UartStatus {
    const OE: usize = 3;
    const BE: usize = 2;
    const PE: usize = 1;
    const FE: usize = 0;

    pub fn is_overrun_error(&self) -> bool {
        self.0.bit_test(Self::OE)
    }

    pub fn is_break_error(&self) -> bool {
        self.0.bit_test(Self::BE)
    }

    pub fn is_parity_error(&self) -> bool {
        self.0.bit_test(Self::PE)
    }

    pub fn is_framing_error(&self) -> bool {
        self.0.bit_test(Self::FE)
    }

    pub fn is_clear(&self) -> bool {
        self.0.0 == 0
    }
}

pub struct UartFlags(BitField<u32>);
impl UartFlags {
    const RI_UNSUPPORTED: usize = 8;
    const TXFE: usize = 7;
    const RXFF: usize = 6;
    const TXFF: usize = 5;
    const RXFE: usize = 4;
    const BUSY: usize = 3;
    const DCD_UNSUPPORTED: usize = 2;
    const DSR_UNSUPPORTED: usize = 1;
    const CTS: usize = 0;

    /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the Line Control Register, UART_LCRH.
    /// If the FIFO is disabled, this bit is set when the transmit holding register is empty.
    /// If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does not indicate if there is data in the transmit shift register.
    pub fn is_transmit_fifo_empty(&self) -> bool {
        self.0.bit_test(Self::TXFE)
    }

    /// Receive FIFO full. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
    /// If the FIFO is disabled, this bit is set when the receive holding register is full.
    /// If the FIFO is enabled, the RXFF bit is set when the receive FIFO is full.
    pub fn is_receive_fifo_full(&self) -> bool {
        self.0.bit_test(Self::RXFF)
    }

    /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
    /// If the FIFO is disabled, this bit is set when the transmit holding register is full.
    /// If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
    pub fn is_transmit_fifo_full(&self) -> bool {
        self.0.bit_test(Self::TXFF)
    }

    /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
    /// If the FIFO is disabled, this bit is set when the receive holding register is empty.
    /// If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
    pub fn is_receive_fifo_empty(&self) -> bool {
        self.0.bit_test(Self::RXFE)
    }

    /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains set until the complete byte, including all the stop bits, has been sent from the shift register.
    /// This bit is set as soon as the transmit FIFO becomes non- empty, regardless of whether the UART is enabled or not.
    pub fn is_busy(&self) -> bool {
        self.0.bit_test(Self::BUSY)
    }

    /// Clear to send. This bit is the complement of the UART clear to send, nUARTCTS, modem status input. That is, the bit is 1 when nUARTCTS is LOW.
    pub fn is_clear_to_send(&self) -> bool {
        self.0.bit_test(Self::CTS)
    }
}

#[repr(u32)]
pub enum UartWordLength {
    _5Bits = 0b00,
    _6Bits = 0b01,
    _7Bits = 0b10,
    _8Bits = 0b11,
}

#[repr(u32)]
pub enum UartBitrate {
    _75Baud = 75,
    _110Baud = 110,
    _300Baud = 300,
    _1200Baud = 1200,
    _2400Baud = 2400,
    _4800Baud = 4800,
    _9600Baud = 9600,
    _19200Baud = 19200,
    _38400Baud = 38400,
    _57600Baud = 57600,
    _115200Baud = 115200
}

impl UartBitrate {
    /// Set UART Baud Rate to 115200 (look at docs for formula for values)
    /// ```
    /// BAUDDIV = FUARTCLK / (16 * BaudRate)
    /// FUARTCLK = 3,000,000 Hz
    /// = 3,000,000 / (16 * 115,200)
    /// = 3,000,000 / 1,843,200
    /// = 1.62760
    /// = (1 + 40 * 2^-6)
    /// ````
    pub fn to_int_frac(self, uart_clock_rate: u32) -> (u32, u32) {
        let f_uart_clk = FixedPoint::<6, u32>::from_int(uart_clock_rate);
        let baud_rate: u32 = self as u32;
        let baud_rate_divisor = f_uart_clk / (16 * baud_rate);
        baud_rate_divisor.split_int_frac()
    }
}

pub struct UartLineControl(BitField<u32>);
impl UartLineControl {
    pub fn new() -> Self {
        Self(BitField::zero())
    }

    pub fn is_stick_parity_selected(self) -> bool {
        self.0.bit_test(7)
    }

    pub fn with_stick_parity_select_set(self) -> Self {
        Self(self.0.with_bit_set(7))
    }

    pub fn with_stick_parity_select_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(7))
    }

    pub fn word_length(self) -> UartWordLength {
        unsafe { core::mem::transmute(self.0.field(5, 2) & 0b11) }
    }

    pub fn with_word_length(self, word_length: UartWordLength) -> Self {
        Self(self.0.with_field_set(5, 2, word_length as u32))
    }

    pub fn is_fifo_enabled(self) -> bool {
        self.0.bit_test(4)
    }

    pub fn with_fifo_enabled(self) -> Self {
        Self(self.0.with_bit_set(4))
    }

    pub fn with_fifo_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(4))
    }

    pub fn is_two_stop_bits_selected(self) -> bool {
        self.0.bit_test(3)
    }

    pub fn with_two_stop_bits_selected(self) -> Self {
        Self(self.0.with_bit_set(3))
    }

    pub fn with_one_stop_bit_selected(self) -> Self {
        Self(self.0.with_bit_cleared(3))
    }

    pub fn is_even_parity_selected(self) -> bool {
        self.0.bit_test(2)
    }

    pub fn with_even_parity_selected(self) -> Self {
        Self(self.0.with_bit_set(2))
    }

    pub fn with_odd_parity_selected(self) -> Self {
        Self(self.0.with_bit_cleared(2))
    }

    pub fn is_parity_enabled(self) -> bool {
        self.0.bit_test(1)
    }

    pub fn with_parity_enabled(self) -> Self {
        Self(self.0.with_bit_set(1))
    }

    pub fn with_parity_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(1))
    }

    pub fn is_send_break_enabled(self) -> bool {
        self.0.bit_test(0)
    }

    pub fn with_send_break_enabled(self) -> Self {
        Self(self.0.with_bit_set(0))
    }

    pub fn with_send_break_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(0))
    }
}

pub struct UartControl (BitField<u32>);

impl UartControl {
    pub fn disabled() -> Self {
        Self(BitField::zero())
    }

    pub fn enabled() -> Self {
        Self(BitField::zero())
            .with_receive_enabled()
            .with_transmit_enabled()
            .with_uart_enabled()
    }

    pub fn is_cts_hardware_flow_control_enabled(self) -> bool {
        self.0.bit_test(15)
    }

    pub fn with_cts_hardware_flow_control_enabled(self) -> Self {
        Self(self.0.with_bit_set(15))
    }

    pub fn with_cts_hardware_flow_control_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(15))
    }

    pub fn is_rts_hardware_flow_control_enabled(self) -> bool {
        self.0.bit_test(14)
    }

    pub fn with_rts_hardware_flow_control_enabled(self) -> Self {
        Self(self.0.with_bit_set(14))
    }

    pub fn with_rts_hardware_flow_control_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(14))
    }

    pub fn is_request_to_send_set(self) -> bool {
        self.0.bit_test(11)
    }

    pub fn with_request_to_send_set(self) -> Self {
        Self(self.0.with_bit_set(11))
    }

    pub fn with_request_to_send_cleared(self) -> Self {
        Self(self.0.with_bit_cleared(11))
    }

    pub fn is_receive_enabled(self) -> bool {
        self.0.bit_test(9)
    }

    pub fn with_receive_enabled(self) -> Self {
        Self(self.0.with_bit_set(9))
    }

    pub fn with_receive_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(9))
    }

    pub fn is_transmit_enabled(self) -> bool {
        self.0.bit_test(8)
    }

    pub fn with_transmit_enabled(self) -> Self {
        Self(self.0.with_bit_set(8))
    }

    pub fn with_transmit_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(8))
    }

    pub fn is_loopback_enabled(self) -> bool {
        self.0.bit_test(7)
    }

    pub fn with_loopback_enabled(self) -> Self {
        Self(self.0.with_bit_set(7))
    }

    pub fn with_loopback_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(7))
    }

    pub fn is_uart_enabled(self) -> bool {
        self.0.bit_test(0)
    }

    pub fn with_uart_enabled(self) -> Self {
        Self(self.0.with_bit_set(0))
    }

    pub fn with_uart_disabled(self) -> Self {
        Self(self.0.with_bit_cleared(0))
    }
}


pub struct UartInterrupts(BitField<u32>);

impl UartInterrupts {
    pub fn all_clear() -> Self {
        Self(BitField::zero())
    }

    pub fn all_set() -> Self {
        Self(BitField::new(0x7f2))
    }

    pub fn is_overrun_error_set(&self) -> bool {
        self.0.bit_test(10)
    }

    pub fn with_overrun_error_set(&self) -> Self {
        Self(self.0.with_bit_set(10))
    }

    pub fn with_overrun_error_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(10))
    }

    pub fn is_break_error_set(&self) -> bool {
        self.0.bit_test(9)
    }

    pub fn with_break_error_set(&self) -> Self {
        Self(self.0.with_bit_set(9))
    }

    pub fn with_break_error_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(9))
    }

    pub fn is_parity_error_set(&self) -> bool {
        self.0.bit_test(8)
    }

    pub fn with_parity_error_set(&self) -> Self {
        Self(self.0.with_bit_set(8))
    }

    pub fn with_parity_error_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(8))
    }

    pub fn is_framing_error_set(&self) -> bool {
        self.0.bit_test(7)
    }

    pub fn with_framing_error_set(&self) -> Self {
        Self(self.0.with_bit_set(7))
    }

    pub fn with_framing_error_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(7))
    }

    pub fn is_receive_timeout_set(&self) -> bool {
        self.0.bit_test(6)
    }

    pub fn with_receive_timeout_set(&self) -> Self {
        Self(self.0.with_bit_set(6))
    }

    pub fn with_receive_timeout_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(6))
    }

    pub fn is_transmit_set(&self) -> bool {
        self.0.bit_test(5)
    }

    pub fn with_transmit_set(&self) -> Self {
        Self(self.0.with_bit_set(5))
    }

    pub fn with_transmit_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(5))
    }

    pub fn is_receive_set(&self) -> bool {
        self.0.bit_test(4)
    }

    pub fn with_receive_set(&self) -> Self {
        Self(self.0.with_bit_set(4))
    }

    pub fn with_receive_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(4))
    }

    pub fn is_n_uartcts_modem_set(&self) -> bool {
        self.0.bit_test(1)
    }

    pub fn with_n_uartcts_modem_set(&self) -> Self {
        Self(self.0.with_bit_set(1))
    }

    pub fn with_n_uartcts_modem_cleared(&self) -> Self {
        Self(self.0.with_bit_cleared(1))
    }
}