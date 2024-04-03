use core::marker::PhantomData;

pub struct Mmio<const BASE: usize, const OFFSET: usize>();
impl<const BASE: usize, const OFFSET: usize> Mmio<BASE, OFFSET> {
    const ADDRESS: usize = crate::system::peripherals::BCM_HOST.peripheral_address + BASE + OFFSET;
    pub fn address(&self) -> usize {
        Self::ADDRESS
    }
    
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

pub struct TypedMMIO<T, const BASE: usize, const OFFSET: usize>(PhantomData<T>);
impl<T, const BASE: usize, const OFFSET: usize> TypedMMIO<T, BASE, OFFSET> {
    const ADDRESS: usize = crate::system::peripherals::BCM_HOST.peripheral_address + BASE + OFFSET;

    pub fn write(data: T) {
        unsafe { (Self::ADDRESS as *mut T).write_volatile(data) };
    }

    pub fn read() -> T {
        unsafe { (Self::ADDRESS as *const T).read_volatile() }
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

#[derive(Clone, Copy)]
pub struct Register<const BASE: usize, const OFFSET: usize, T>(usize, core::marker::PhantomData<T>);
impl<const BASE: usize, const OFFSET: usize, T> Register<BASE, OFFSET, T> {
    const ADDRESS: usize = BASE + OFFSET;

    pub const fn no_offset() -> Self {
        Self(0, core::marker::PhantomData {})
    }

    pub const fn at(address: usize) -> Self {
        Self(address, core::marker::PhantomData {})
    }

    pub fn read(self) -> T {
        unsafe { self.as_ptr().read_volatile() }
    }

    pub fn write(self, value: T) {
        unsafe { self.as_mut_ptr().write_volatile(value) }
    }

    pub fn update<F: Fn(T) -> T>(self, f: F) {
        let ptr = self.as_mut_ptr();
        unsafe {
            let value = ptr.read_volatile();
            ptr.write_volatile(f(value));
        }
    }

    pub const fn as_ptr(&self) -> *const T {
        (Self::ADDRESS + self.0) as *const T
    }

    pub const fn as_mut_ptr(&self) -> *mut T {
        (Self::ADDRESS + self.0) as *mut T
    }
}

pub type PeripheralRegister<const OFFSET: usize, T> =
    Register<{ super::BCM_HOST.peripheral_address }, OFFSET, T>;
