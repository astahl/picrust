pub fn write_to(ptr: *mut u32, data: u32) {
    unsafe { core::ptr::write_volatile(ptr, data) };
}

pub fn read_from(ptr: *const u32) -> u32 {
    unsafe { core::ptr::read_volatile(ptr) }
}

pub struct MMIO<const BASE: usize, const OFFSET: usize>();
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
