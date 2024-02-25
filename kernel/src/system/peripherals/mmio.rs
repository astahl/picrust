pub struct MMIO<const BASE: usize, const OFFSET: usize>();
impl<const BASE: usize, const OFFSET: usize> MMIO<BASE, OFFSET> {
    const ADDRESS: usize = crate::system::peripherals::BCM_HOST.peripheral_address + BASE + OFFSET;

    pub fn write(&self, data: u32) {
        unsafe { (Self::ADDRESS as *mut u32).write_volatile(data) };
    }

    pub fn read(&self) -> u32 {
        unsafe { (Self::ADDRESS as *const u32).read_volatile() }
    }

    pub fn update(&self, mask: u32, data: u32) -> u32 {
        let old_value = self.read();
        let new_value = (!mask & old_value) | (mask & data);
        self.write(new_value);
        old_value
    }
}

#[repr(transparent)]
pub struct DynamicMmioField<T>(core::mem::MaybeUninit<T>);

impl<T> DynamicMmioField<T> {
    pub const fn init(value: T) -> Self {
        Self(core::mem::MaybeUninit::new(value))
    }

    pub fn read(&self) -> T {
        unsafe { core::ptr::addr_of!(self.0).read_volatile().assume_init() }
    }

    pub fn write(&mut self, value: T) {
        unsafe {
            core::ptr::addr_of_mut!(self.0).write_volatile(core::mem::MaybeUninit::new(value))
        }
    }
}
