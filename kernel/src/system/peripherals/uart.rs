use mystd::bit_field;
use mystd::fixed_point::FxU32;
use crate::peripherals::gpio;
use crate::system::hal::clocks::Clock;
use crate::system::hal::counter::PointInTime;

use super::gpio::PinSet;
use super::mmio::PeripheralRegister;

#[derive(Clone, Copy)]
pub enum Uart {
    Pl011Uart{
        address: usize,
        behavior: UartBehavior
    },
    MiniUart{
        address: usize,
        behavior: UartBehavior
    }
}

pub enum UartReadError {
    ReceiveFifoEmpty,
    Status(UartStatus)
}

pub enum UartWriteError {
    TransmitFifoFull
}

#[derive(Clone, Copy)]
pub enum Blocking {
    Never,
    TimeoutAfter(core::time::Duration),
    Indefinetly,
}

#[derive(Clone, Copy)]
pub struct UartBehavior {
    read_blocking: Blocking,
    write_blocking: Blocking,
}

impl UartBehavior {
    pub const fn default() -> Self {
        Self {
            read_blocking: Blocking::Indefinetly,
            write_blocking: Blocking::Indefinetly,
        }
    } 
}

pub const UART_BASE: usize = 0x201000;

pub const UART_0: Uart = Uart::Pl011Uart{ address: UART_BASE, behavior: UartBehavior::default() };
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
    const fn base_address(&self) -> usize {
        match self {
            Uart::Pl011Uart{ address: a, ..} => *a,
            Uart::MiniUart{ address: a, ..} => *a,
        }
    }

    const fn read_blocking(&self) -> Blocking {
        match self {
            Uart::Pl011Uart{ behavior: b, ..} => b.read_blocking,
            Uart::MiniUart{ behavior: b, ..} => b.read_blocking,
        }
    }

    const fn write_blocking(&self) -> Blocking {
        match self {
            Uart::Pl011Uart{ behavior: b, ..} => b.write_blocking,
            Uart::MiniUart{ behavior: b, ..} => b.write_blocking,
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

        let clock_rate = Clock::Uart.rate().unwrap_or(3_000_000);
        let (brd_int, brd_frac) = UartBitrate::Baud115200.to_int_frac(clock_rate);
        UartIntegerBaudRateDivisorReg::at(base_address).write(brd_int);
        UartFractionalBaudRateDivisorReg::at(base_address).write(brd_frac);

        UartLineControlReg::at(base_address).write(
            UartLineControl::zero()
                .word_length()
                    .set_value(UartWordLength::Bit8)
                // .parity_enabled()
                //     .clear()
                // .stick_parity_select()
                //     .clear()
                // .even_parity_select()
                //     .clear()
                // .send_break()
                //     .set()
                // .two_stop_bits()
                //     .clear()
                .fifo_enabled()
                    .set(),
        );
        // mask (disable) all interrupts
        self.interrupt_mask_reg().write(UartInterrupts::zero());
        UartInterruptClearReg::at(base_address).write(UartInterrupts::all_set());

        // enable UART
        UartControlReg::at(base_address).write(UartControl::enabled());
    }

    pub fn try_put_byte(&self, data: u8) -> Result<(), UartWriteError> {
        if self.flags().txff().is_set() {
            Err(UartWriteError::TransmitFifoFull)
        } else {
            UartDataReg::at(self.base_address()).write(UartData::new(data as u32));
            Ok(())
        }
    }

    pub fn put_byte(&self, data: u8) -> Result<(), mystd::io::Error> {
        match self.write_blocking() {
            Blocking::Never => {
                if let Err(UartWriteError::TransmitFifoFull) = self.try_put_byte(data) {
                    return Err(mystd::io::Error::WouldBlock)
                }
            },
            Blocking::TimeoutAfter(timeout_duration) => {
                let timeout = PointInTime::now() + timeout_duration;
                while let Err(UartWriteError::TransmitFifoFull) = self.try_put_byte(data) {
                    if !timeout.is_in_the_future() {
                        return Err(mystd::io::Error::TimedOut)
                    }
                    core::hint::spin_loop();
                }
            }
            Blocking::Indefinetly => {
                while let Err(UartWriteError::TransmitFifoFull) = self.try_put_byte(data) {
                    core::hint::spin_loop();
                }
            },
        } 
        
        Ok(())
    }

    pub fn try_get_byte(&self) -> Result<u8, UartReadError> {
        if self.flags().rxfe().is_set() {
            Err(UartReadError::ReceiveFifoEmpty)
        } else {
            let read = UartDataReg::at(self.base_address()).read();
            let (status, data): (UartStatus, u8) =
                (read.status().into(), read.data().value().unwrap());
            if status.overrun_error().clear().is_all_clear() { 
                // don't propagate overrun errors, because reading clears the flag.
                Ok(data)
            } else {
                Err(UartReadError::Status(status))
            }
        }
    }

    pub fn get_byte(&self) -> Result<u8, mystd::io::Error> {
        let err_status = match self.read_blocking() {
            Blocking::Never => {
                match self.try_get_byte() {
                    Ok(byte) => return Ok(byte),
                    Err(UartReadError::ReceiveFifoEmpty) => return Err(mystd::io::Error::WouldBlock),
                    Err(UartReadError::Status(error_status)) => error_status,
                }
            }
            Blocking::TimeoutAfter(timeout_duration) => {
                let timeout = PointInTime::now() + timeout_duration;
                loop {
                    match self.try_get_byte() {
                        Ok(byte) => return Ok(byte),
                        Err(UartReadError::ReceiveFifoEmpty) => {
                            if !timeout.is_in_the_future() {
                                return Err(mystd::io::Error::TimedOut);
                            } else {
                                core::hint::spin_loop();
                            }
                        },
                        Err(UartReadError::Status(error_status)) => break error_status,
                    }
                }    
            }
            Blocking::Indefinetly => {
                loop {
                    match self.try_get_byte() {
                        Ok(byte) => return Ok(byte),
                        Err(UartReadError::ReceiveFifoEmpty) => {
                            core::hint::spin_loop();    
                        },
                        Err(UartReadError::Status(error_status)) => break error_status,
                    }
                }    
            }
        };
            
        if err_status.parity_error().is_set() || err_status.framing_error().is_set(){
            Err(mystd::io::Error::InvalidData)
        } else if err_status.break_error().is_set() {
            Err(mystd::io::Error::UnexpectedEof)
        } else {
            Err(mystd::io::Error::Unknown { err_code: err_status.to_underlying() as i32 })
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
        let mut count = 0;
        for b in buf {
            match self.put_byte(*b) {
                Ok(_) => count += 1,
                Err(e) => {
                    if count == 0 {
                        return Err(e)
                    } else {
                        break;
                    }
                },
            }
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
        let mut count = 0;
        while count < buf.len() {

            match self.get_byte() {
                Ok(byte) => {
                    buf[count] = byte;
                    count += 1;
                },
                Err(e) => if count == 0 {
                    return Err(e)
                } else {
                    break;
                },
            }
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
        let f_uart_clk: FxU32<6> = uart_clock_rate.into();
        let baud_rate: u32 = self as u32;
        let baud_rate_divisor = f_uart_clk / (16 * baud_rate);
        baud_rate_divisor.split_int_frac()
    }
}

bit_field!(
    /// The UART_LCRH Register is the line control register.
    /// 
    /// NOTE: The UART_LCRH, UART_IBRD, and UART_FBRD registers must not be changed:
    /// * when the UART is enabled
    /// * when completing a transmission or a reception when it has been programmed to become disabled.
    /// 
    /// The parity bit behavior is determined by three flags:
    /// 
    /// | PEN | EPS | SPS | Parity bit (transmitted or checked) |
    /// |-----|-----|-----|-------------------------------------|
    /// | 0   | x   | x   | Not transmitted or checked |
    /// | 1   | 1   | 0   | Even parity |
    /// | 1   | 0   | 0   | Odd parity |
    /// | 1   | 0   | 1   | 1 |
    /// | 1   | 1   | 1   | 0 |
    pub UartLineControl(u32){
    /// # SPS 
    /// Stick parity select.
    /// * 0b0
    ///     - stick parity is disabled
    /// * 0b1
    ///     - either:
    ///         + if the EPS bit is 0 then the parity bit is transmitted and checked as a 1
    ///         + if the EPS bit is 1 then the parity bit is transmitted and checked as a 0. See the table in the type documentation.
    /// 
    /// Resets to 0.
    7 => stick_parity_select,
    /// Word length. These bits indicate the number of data bits transmitted or received in a frame as follows:
    /// * b11 = 8 bits
    /// * b10 = 7 bits
    /// * b01 = 6 bits 
    /// * b00 = 5 bits.
    /// 
    /// Resets to 0.
    5:6 => word_length: enum UartWordLength {
        Bit5 = 0b00,
        Bit6 = 0b01,
        Bit7 = 0b10,
        Bit8 = 0b11,
    },
    /// Enable FIFOs:
    /// * 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding registers
    /// * 1 = transmit and receive FIFO buffers are enabled (FIFO mode).
    /// 
    /// Resets to 0.
    4 => fifo_enabled,
    /// Two stop bits select. If this bit is set to 1, two stop bits are transmitted at the end of the frame. 
    /// The receive logic does not check for two stop bits being received.
    /// 
    /// Resets to 0.
    3 => two_stop_bits,
    /// # EPS 
    /// Even parity select. Controls the type of parity the UART uses during transmission and reception:
    /// * 0 = odd parity. The UART generates or checks for an odd number of 1s in the data and parity bits.
    /// * 1 = even parity. The UART generates or checks for an even number of 1s in the data and parity bits.
    /// 
    /// This bit has no effect when the PEN bit disables parity checking and generation. See the table in the type documentation.
    /// 
    /// Resets to 0.
    2 => even_parity_select,
    /// # PEN 
    /// Parity enable. See the table in the type documentation.
    /// * 0 = parity is disabled and no parity bit added to the data frame
    /// * 1 = parity checking and generation is enabled.
    ///
    /// Resets to 0.
    1 => parity_enabled,
    /// Send break. If this bit is set to 1, a low-level is continually output on the TXD output, after completing transmission of the current character.
    /// 
    /// Resets to 0.
    0 => send_break
});

bit_field!(pub UartControl (u32){
    /// CTS hardware flow control enable. If this bit is set to 1, CTS hardware flow control is enabled. Data is only transmitted when the nUARTCTS signal is asserted.
    15 => cts_hardware_flow_control,
    /// RTS hardware flow control enable. If this bit is set to 1, RTS hardware flow control is enabled. Data is only requested when there is space in the receive FIFO for it to be received.
    14 => rts_hardware_flow_control,
    /// Request to send. This bit is the complement of the UART request to send, nUARTRTS, modem status output. That is, when the bit is programmed to a 1 then nUARTRTS is LOW.
    11 => request_to_send,
    /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled. Data reception occurs for UART signals. When the UART is disabled in the middle of reception, it completes the current character before stopping.
    9 => receive_enable,
    /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled. Data transmission occurs for UART signals. When the UART is disabled in the middle of transmission, it completes the current character before stopping.
    8 => transmit_enable,
    /// Loopback enable. If this bit is set to 1, the UARTTXD path is fed through to the UARTRXD path. In UART mode, when this bit is set, the modem outputs are also fed through to the modem inputs. This bit is cleared to 0 on reset, to disable loopback.
    7 => loopback_enable,
    /// UART enable:
    /// * 0b0 
    ///     - UART is disabled. If the UART is disabled in the middle of transmission or reception, it completes the current character before stopping.
    /// * 0b1 
    ///     - the UART is enabled.
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
