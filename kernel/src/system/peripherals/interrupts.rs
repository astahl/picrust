use mystd::bit_field;

use crate::system::arm_core::registers::aarch64::special_purpose;

use super::mmio::MMIO;

const IRQ_BASE: usize = 0xB000;

#[inline]
pub fn irq_enabled() -> bool {
    special_purpose::Daif::read_register().irq_masked().is_clear()
}

#[inline]
pub fn irq_enable() {
    special_purpose::Daif::read_register().irq_masked().clear().write_register();
}

#[inline]
pub fn irq_disabled() -> bool {
    special_purpose::Daif::read_register().irq_masked().is_set()
}

#[inline]
pub fn irq_disable() {
    special_purpose::Daif::read_register().irq_masked().set().write_register();
}

impl IrqPendingBase {

    const REG: MMIO<IRQ_BASE, 0x200> = MMIO();
    pub fn read_register() -> Self {
        Self::REG.read().into()
    }
}

impl GpuIrqs1 {
    const PENDING: MMIO<IRQ_BASE, 0x204> = MMIO();
    const ENABLE: MMIO<IRQ_BASE, 0x210> = MMIO();
    const DISABLE: MMIO<IRQ_BASE, 0x21c> = MMIO();
    
    pub fn read_pending() -> Self {
        Self::PENDING.read().into()
    }

    pub fn read_enable() -> Self {
        Self::ENABLE.read().into()
    }

    pub fn write_enable(&self) {
        Self::ENABLE.write(self.0)
    }

    pub fn read_disable() -> Self {
        Self::ENABLE.read().into()
    }

    pub fn write_disable(&self) {
        Self::ENABLE.write(self.0)
    }
}

impl GpuIrqs2 {
    const PENDING: MMIO<IRQ_BASE, 0x208> = MMIO();
    const ENABLE: MMIO<IRQ_BASE, 0x214> = MMIO();
    const DISABLE: MMIO<IRQ_BASE, 0x220> = MMIO();
    
    pub fn read_pending() -> Self {
        Self::PENDING.read().into()
    }

    pub fn read_enable() -> Self {
        Self::ENABLE.read().into()
    }

    pub fn write_enable(&self) {
        Self::ENABLE.write(self.0)
    }

    pub fn read_disable() -> Self {
        Self::ENABLE.read().into()
    }

    pub fn write_disable(&self) {
        Self::ENABLE.write(self.0)
    }
}

impl BasicIrqs {
    const PENDING: MMIO<IRQ_BASE, 0x200> = MMIO();
    const ENABLE: MMIO<IRQ_BASE, 0x218> = MMIO();
    const DISABLE: MMIO<IRQ_BASE, 0x224> = MMIO();
    
    pub fn read_pending() -> Self {
        Self::PENDING.read().into()
    }

    pub fn read_enable() -> Self {
        Self::ENABLE.read().into()
    }

    pub fn write_enable(&self) {
        Self::ENABLE.write(self.0)
    }

    pub fn read_disable() -> Self {
        Self::ENABLE.read().into()
    }

    pub fn write_disable(&self) {
        Self::ENABLE.write(self.0)
    }
}

impl Fiq {
    const REG: MMIO<IRQ_BASE, 0x20c> = MMIO();

    pub fn read_register() -> Self {
        Self::REG.read().into()
    }
    
    pub fn write_register(&self) {
        Self::REG.write(self.0)
    }
}


bit_field!(pub IrqPendingBase(u32) {
    /// GPUIRQ62
    20 => gpu_irq_62,
    /// GPUIRQ57
    19 => gpu_irq_57_uart,
    /// GPUIRQ56
    18 => gpu_irq_56,
    /// GPUIRQ55
    17 => gpu_irq_55_pcm,
    /// GPUIRQ54
    16 => gpu_irq_54_spi,
    /// GPUIRQ53
    15 => gpu_irq_53_i2c,
    /// GPUIRQ19
    14 => gpu_irq_19,
    /// GPUIRQ18
    13 => gpu_irq_18,
    /// GPUIRQ10
    12 => gpu_irq_10,
    /// GPUIRQ9
    11 => gpu_irq_9,
    /// GPUIRQ7
    10 => gpu_irq_7,
    /// One or more bits set in pending register 2 
    /// 
    /// These bits indicates if there are bits set in the pending 1 / 2 registers. The pending 1 / 2 registers hold ALL interrupts 0..63 from the GPU side. Some of these 64 interrupts are also connected to the basic pending register. Any bit set in pending register 1/2 which is NOT connected to the basic pending register causes bit 8 or 9 to set. Status bits 9 should be seen as "There are some interrupts pending which you don't know about. They are in pending register 2."
    9 => pend_reg_2,
    /// One or more bits set in pending register 1 
    /// 
    /// These bits indicates if there are bits set in the pending 1 / 2 registers. The pending 1 / 2 registers hold ALL interrupts 0..63 from the GPU side. Some of these 64 interrupts are also connected to the basic pending register. Any bit set in pending register 1/2 which is NOT connected to the basic pending register causes bit 8 or 9 to set. Status bits 8 should be seen as "There are some interrupts pending which you don't know about. They are in pending register 1."
    8 => pend_reg_1,
    /// Illegal access type 0 IRQ pending
    /// 
    /// This bit indicate that the address/access error line from the ARM processor has generated an interrupt. That signal is asserted when either an address bit 31 or 30 was high or when an access was seen on the ARM Peripheral bus. The status of that signal can be read from Error/HALT status register bit 2.
    7 => illegal_accesss_type_0,
    /// Illegal access type 1 IRQ pending 
    /// 
    /// This bit indicates that an address/access error is seen in the ARM control has generated an interrupt. That can either be an address bit 29..26 was high or when a burst access was seen on the GPU Peripheral bus. The status of that signal can be read from Error/HALT status register bits 0 and 1.
    6 => illegal_accesss_type_1,
    /// GPU1 halted IRQ pending
    /// 
    /// This bit indicate that the GPU-1 halted status bit has generated an interrupt. The status of that signal can be read from Error/HALT status register bits 4.
    5 => gpu1_halted,
    /// GPU0 halted IRQ pending (Or GPU1 halted if bit 10 of control register 1 is set)
    /// 
    /// This bit indicate that the GPU-0 halted status bit has generated an interrupt. The status of that signal can be read from Error/HALT status register bits 3.
    /// In order to allow a fast interrupt (FIQ) routine to cope with GPU 0 OR GPU-1 there is a bit in control register 1 which, if set will also route a GPU-1 halted status on this bit.
    4 => gpu0_halted,
    /// ARM Doorbell 1 IRQ pending 
    3 => arm_doorbell1,
    /// ARM Doorbell 0 IRQ pending 
    2 => arm_doorbell0,
    /// ARM Mailbox IRQ pending 
    1 => arm_mailbox,
    /// ARM Timer IRQ pending
    0 => arm_timer,
});

bit_field!(pub BasicIrqs(u32) {
    0 => arm_timer,
    1 => arm_mailbox,
    2 => arm_doorbell0,
    3 => arm_doorbell1,
    4 => gpu0_halted,
    5 => gpu1_halted,
    6 => illegal_accesss_type_1,
    7 => illegal_accesss_type_0
});

bit_field!(pub GpuIrqs1(u32) {
    29 => aux_int,
    3:0 => system_timers,
    3 => system_timer_3,
    2 => system_timer_2,
    1 => system_timer_1,
    0 => system_timer_0,
});

bit_field!(pub GpuIrqs2(u32) {
    25 => uart_int,
    23 => pcm_int,
    22 => spi_int,
    21 => i2c_int,
    20 => gpio_int3,
    19 => gpio_int2,
    18 => gpio_int1,
    17 => gpio_int0,
    16 => smi,
    14 => pwa1,
    13 => pwa0,
    11 => i2c_spi_slv_int,
});

bit_field!(pub Fiq(u32) {
    7 => fiq_enable,
    6:0 => fiq_source: enum FiqSource {
        GpuAuxInterrupt = 29,
        GpuI2cSpiSlvInterrupt = 43,
        GpuPwa0 = 45,
        GpuPwa1 = 46,
        GpuSmi = 48,
        GpioInt0 = 49,
        GpioInt1 = 50,
        GpioInt2 = 51,
        GpioInt3 = 52,
        I2cInt = 53,
        SpiInt = 54,
        PcmInt = 55,
        UartInt = 57,
        ArmTimerInterrupt = 64,
        ArmMailboxInterrupt = 65,
        ArmDoorbell0Interrupt = 66,
        ArmDoorbell1Interrupt = 67,
        Gpu0HaltedInterruptOrGpu1 = 68,
        Gpu1HaltedInterrupt = 69,
        IllegalAccessType1Interrupt = 70,
        IllegalAccessType0Interrupt = 71,
    } 
});