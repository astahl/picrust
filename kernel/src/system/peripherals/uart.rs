use mystd::bit_field;
use mystd::fixed_point::FixedPoint;

use crate::peripherals::gpio;
use crate::system::hal::clocks::Clock;

use super::gpio::PinSet;
use super::mmio::PeripheralRegister;

#[derive(Clone, Copy)]
pub enum Uart {
    Pl011Uart(usize),
    MiniUart(usize),
}

pub const UART_BASE: usize = 0x201000;

pub const UART_0: Uart = Uart::Pl011Uart(UART_BASE);
// pub type Uart2 = Pl011Uart<0x201400>;
// pub type Uart3 = Pl011Uart<0x201600>;
// pub type Uart4 = Pl011Uart<0x201800>;
// pub type Uart5 = Pl011Uart<0x201a00>;

pub type UartDataReg = PeripheralRegister<0x00, UartData>;
pub type UartReceiveStatusErrorClearReg = PeripheralRegister<0x04, u32>;
pub type UartFlagReg = PeripheralRegister<0x18, UartFlags>;
pub type UartIlpReg = PeripheralRegister<0x20, u32>;
pub type UartIntegerBaudRateDivisorReg = PeripheralRegister<0x24, u32>;
pub type UartFractionalBaudRateDivisorReg = PeripheralRegister<0x28, u32>;
pub type UartLineControlReg = PeripheralRegister<0x2c, UartLineControl>;
pub type UartControlReg = PeripheralRegister<0x30, UartControl>;
pub type UartInterruptFIFOLevelSelectReg = PeripheralRegister<0x34, u32>;
pub type UartInterruptMaskSetClearReg = PeripheralRegister<0x38, UartInterrupts>;
pub type UartRawInterruptStatusReg = PeripheralRegister<0x3c, UartInterrupts>;
pub type UartMaskedInterruptStatusReg = PeripheralRegister<0x40, UartInterrupts>;
pub type UartInterruptClearReg = PeripheralRegister<0x44, UartInterrupts>;
pub type UartDMAControlReg = PeripheralRegister<0x48, u32>;
pub type UartTestControlReg = PeripheralRegister<0x80, u32>;
pub type UartIntegrationTestInputReg = PeripheralRegister<0x84, u32>;
pub type UartIntegrationTestOutputReg = PeripheralRegister<0x88, u32>;
pub type UartTestDataReg = PeripheralRegister<0x8c, u32>;


pub fn handle_interrupts() {
    use mystd::io::Write;
    let masked = UART_0.masked_interrupt_status_reg();
    let interrupts = masked.read();
    writeln!(UART_0, "{:#?}", interrupts).unwrap();
    UART_0.interrupt_clear_reg().write(interrupts);
    let interrupts = masked.read();
    writeln!(UART_0, "{:#?}", interrupts).unwrap();
    writeln!(UART_0, "MASK: {:#?}", UART_0.interrupt_mask_reg().read()).unwrap();
}

impl Uart {
    fn base_address(&self) -> usize {
        match self {
            Uart::Pl011Uart(a) => *a,
            Uart::MiniUart(a) => *a,
        }
    }
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
        let base_address = self.base_address();
        UartControlReg::at(base_address).write(UartControl::disabled());

        while UartFlagReg::at(base_address).read().busy().is_set() {
            core::hint::spin_loop();
        }

        // flush transmit fifo
        UartLineControlReg::at(base_address).update(|u| u.fifo_enabled().clear());

        // todo figure out how to select pins / functions for each uart on pi4
        let pins = PinSet::select(&[14, 15]);
        gpio::Gpio::set_functions(pins, gpio::PinFunction::Alt0);
        gpio::Gpio::set_pull_resistors(pins, gpio::Resistor::None);

        // Clear all pending UART interrupts
        UartInterruptClearReg::at(base_address).write(UartInterrupts::all_set());

        let clock_rate = Clock::UART.rate().unwrap_or(3_000_000);
        let (brd_int, brd_frac) = UartBitrate::Baud1200.to_int_frac(clock_rate);
        UartIntegerBaudRateDivisorReg::at(base_address).write(brd_int);
        UartFractionalBaudRateDivisorReg::at(base_address).write(brd_frac);

        UartLineControlReg::at(base_address).write(
            UartLineControl::zero()
                .word_length()
                .set_value(UartWordLength::Bit8)
                .fifo_enabled()
                .set(),
        );
        // mask (disable) all interrupts
        self.interrupt_mask_reg().write(UartInterrupts::zero());
        UartInterruptClearReg::at(base_address).write(UartInterrupts::all_set());

        // enable UART
        UartControlReg::at(base_address).write(UartControl::enabled());
    }

    pub fn put_byte(&self, data: u8) {
        while self.flags().txff().is_set() {
            core::hint::spin_loop();
        }
        UartDataReg::at(self.base_address()).write(UartData::new(data as u32));
    }

    pub fn get_byte(&self) -> Result<u8, UartStatus> {
        while self.flags().rxfe().is_set() {
            core::hint::spin_loop();
        }
        let read = UartDataReg::at(self.base_address()).read();
        let (status, data): (UartStatus, u8) =
            (read.status().into(), read.data().value().unwrap());
        if status.is_all_clear() {
            Ok(data)
        } else {
            Err(status)
        }
    }

    pub fn flags(&self) -> UartFlags {
        UartFlagReg::at(self.base_address()).read()
    }


    pub fn raw_interrupt_status_reg(&self) -> UartRawInterruptStatusReg {
        UartRawInterruptStatusReg::at(self.base_address())
    }

    pub fn masked_interrupt_status_reg(&self) -> UartMaskedInterruptStatusReg {
        UartMaskedInterruptStatusReg::at(self.base_address())
    }

    /// Write a one to clear the corresponding interrupt
    pub fn interrupt_clear_reg(&self) -> UartInterruptClearReg {
        UartInterruptClearReg::at(self.base_address())
    }

    /// If the mask bit is set, the interrupt is enabled, so it acts like logical AND to the raw interrupts.
    pub fn interrupt_mask_reg(&self) -> UartInterruptMaskSetClearReg {
        UartInterruptMaskSetClearReg::at(self.base_address())
    }
}

impl mystd::io::Write for Uart {
    fn write(&mut self, buf: &[u8]) -> mystd::io::Result<mystd::io::Size> {
        while self.flags().txff().is_set() {
            core::hint::spin_loop();
        }
        let reg_ptr = UartDataReg::at(self.base_address())
            .as_mut_ptr()
            .cast::<u32>();
        let mut count = 0;
        for b in buf {
            unsafe {
                reg_ptr.write_volatile(*b as u32);
            }
            count += 1;
        }
        Ok(mystd::io::Size::from_usize(count))
    }

    fn flush(&mut self) -> mystd::io::Result<()> {
        while !self.flags().txfe().is_set() {
            core::hint::spin_loop();
        }
        Ok(())
    }
}

impl mystd::io::Read for Uart {
    fn read(&mut self, buf: &mut [u8]) -> mystd::io::Result<mystd::io::Size> {
        while self.flags().rxfe().is_set() {
            core::hint::spin_loop();
        }

        let reg_ptr = UartDataReg::at(self.base_address()).as_ptr();
        let mut count = 0;
        while count < buf.len() && !self.flags().rxfe().is_set() {
            let received = unsafe { reg_ptr.read_volatile() };
            if received.break_error().is_set() {
                return Err(mystd::io::Error::Interrupted);
            }
            if received.parity_error().is_set() || received.framing_error().is_set() {
                return Err(mystd::io::Error::InvalidData);
            }
            buf[count] = received.data().value().unwrap();
            count += 1;
        }
        Ok(mystd::io::Size::from_usize(count))
    }
}

bit_field!(pub UartData(u32){
    /// Overrun error. This bit is set to 1 if data is received and the receive FIFO is already full.
    ///
    /// This is cleared to 0 once there is an empty space in the FIFO and a new character can be written to it.
    11 => overrun_error,
    /// Break error. This bit is set to 1 if a break condition was detected, indicating that the received data input was held LOW for longer than a full-word transmission time (defined as start, data, parity and stop bits).
    ///
    /// In FIFO mode, this error is associated with the character at the top of the FIFO. When a break occurs, only one 0 character is loaded into the FIFO. The next character is only enabled after the receive data input goes to a 1 (marking state), and the next valid start bit is received.
    10 => break_error,
    /// Parity error. When set to 1, it indicates that the parity of the received data character does not match the parity that the EPS and SPS bits in the Line Control Register, UART_LCRH select.
    ///
    /// In FIFO mode, this error is associated with the character at the top of the FIFO.
    9 => parity_error,
    /// Framing error. When set to 1, it indicates that the received character did not have a valid stop bit (a valid stop bit is 1).
    ///
    /// In FIFO mode, this error is associated with the character at the top of the FIFO.
    8 => framing_error,
    8:11 => status,
    0:7 => data: u8
});

bit_field!(pub UartStatus(u32){
    /// Overrun error. This bit is set to 1 if data is received and the receive FIFO is already full.
    ///
    /// This is cleared to 0 once there is an empty space in the FIFO and a new character can be written to it.
    3 => overrun_error,
    /// Break error. This bit is set to 1 if a break condition was detected, indicating that the received data input was held LOW for longer than a full-word transmission time (defined as start, data, parity and stop bits).
    ///
    /// In FIFO mode, this error is associated with the character at the top of the FIFO. When a break occurs, only one 0 character is loaded into the FIFO. The next character is only enabled after the receive data input goes to a 1 (marking state), and the next valid start bit is received.
    2 => break_error,
    /// Parity error. When set to 1, it indicates that the parity of the received data character does not match the parity that the EPS and SPS bits in the Line Control Register, UART_LCRH select.
    ///
    /// In FIFO mode, this error is associated with the character at the top of the FIFO.
    1 => parity_error,
    /// Framing error. When set to 1, it indicates that the received character did not have a valid stop bit (a valid stop bit is 1).
    ///
    /// In FIFO mode, this error is associated with the character at the top of the FIFO.
    0 => framing_error
});

bit_field!(pub UartFlags(u32){
    8 => _ri_unsupported,
    /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the Line Control Register, UART_LCRH.
    /// If the FIFO is disabled, this bit is set when the transmit holding register is empty.
    /// If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does not indicate if there is data in the transmit shift register.
    7 => txfe,
    /// Receive FIFO full. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
    /// If the FIFO is disabled, this bit is set when the receive holding register is full.
    /// If the FIFO is enabled, the RXFF bit is set when the receive FIFO is full.
    6 => rxff,
    /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
    /// If the FIFO is disabled, this bit is set when the transmit holding register is full.
    /// If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
    5 => txff,
    /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the UART_LCRH Register.
    /// If the FIFO is disabled, this bit is set when the receive holding register is empty.
    /// If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
    4 => rxfe,
    /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains set until the complete byte, including all the stop bits, has been sent from the shift register.
    /// This bit is set as soon as the transmit FIFO becomes non- empty, regardless of whether the UART is enabled or not.
    3 => busy,
    2 => _dcd_unsupported,
    1 => _dsr_unsupported,
    /// Clear to send. This bit is the complement of the UART clear to send, nUARTCTS, modem status input. That is, the bit is 1 when nUARTCTS is LOW.
    0 => cts
});

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum UartBitrate {
    Baud75 = 75,
    Baud110 = 110,
    Baud300 = 300,
    Baud1200 = 1200,
    Baud2400 = 2400,
    Baud4800 = 4800,
    Baud9600 = 9600,
    Baud19200 = 19200,
    Baud38400 = 38400,
    Baud57600 = 57600,
    Baud115200 = 115200,
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

bit_field!(pub UartLineControl(u32){
    7 => stick_parity,
    5:6 => word_length: enum UartWordLength {
        Bit5 = 0b00,
        Bit6 = 0b01,
        Bit7 = 0b10,
        Bit8 = 0b11,
    },
    4 => fifo_enabled,
    3 => two_stop_bits,
    2 => even_parity,
    1 => parity_enabled,
    0 => send_break
});

bit_field!(pub UartControl (u32){
    15 => cts_hardware_flow_control,
    14 => rts_hardware_flow_control,
    11 => request_to_send,
    9 => receive_enable,
    8 => transmit_enable,
    7 => loopback_enable,
    0 => uart_enable
});

impl UartControl {
    pub fn disabled() -> Self {
        Self::zero()
    }

    pub fn enabled() -> Self {
        Self::zero()
            .receive_enable()
            .set()
            .transmit_enable()
            .set()
            .uart_enable()
            .set()
    }
}

bit_field!(pub UartInterrupts(u32){
    10 => overrun_error,
    9 => break_error,
    8 => parity_error,
    7 => framing_error,
    6 => receive_timeout,
    5 => transmit,
    4 => receive,
    1 => n_uartcts_modem
});
