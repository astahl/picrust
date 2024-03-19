use core::arch::asm;

impl MairEl1 {
    pub fn load_register() -> Self {
        let value: u64;
        unsafe { asm!("mrs {0}, mair_el1", out(reg) value) };
        value.into()
    }
    
    pub fn write_register(self) {
        let val: u64 = self.into();
        unsafe { asm!("msr mair_el1, {}", in(reg) val) };
    }
}

#[derive(Clone, Copy)]
pub struct MairEl1([u8;8]);

impl MairEl1 {
    pub fn zero() -> Self {
        MairEl1([0;8])
    }

    pub fn attr_n(&self, index: usize) -> MemoryAttributeDescriptor {
        MemoryAttributeDescriptor::from(self.0[index])
    }

    pub fn set_attr_n(&mut self, index: usize, desc: MemoryAttributeDescriptor) {
        self.0[index] = desc.into()
    }
}

impl From<u64> for MairEl1 {
    fn from(value: u64) -> Self {
        MairEl1(value.to_le_bytes())
    }
}

impl Into<u64> for MairEl1 {
    fn into(self) -> u64 {
        u64::from_le_bytes(self.0)
    }
}

#[derive(Clone, Copy)]
pub enum MemoryAttributeDescriptor {
    Device{
        memory_type: DeviceMemoryType,
        #[cfg(feature = "FEAT_XS")]
        xs_zero: bool
    },
    Normal{
        outer: NormalMemoryType,
        inner: NormalMemoryType,
        #[cfg(feature = "FEAT_XS")]
        xs_zero: bool,
        #[cfg(feature = "FEAT_MTE2")]
        tagged: bool,
    },
    Unpredictable
}

impl MemoryAttributeDescriptor {
    pub fn device(device_flags: u8) -> Self {
        Self::Device { 
            memory_type: device_flags.into(),
            #[cfg(feature = "FEAT_XS")]
            xs_zero: false
        }
    }

    #[cfg(feature = "FEAT_XS")]
    pub fn device_xs(device_flags: u8) -> Self {
        Self::Device { 
            memory_type: device_flags.into(),
            xs_zero: true
        }
    }

    pub fn normal(outer_flags: u8, inner_flags: u8) -> Self {
        assert!(inner_flags > 0 && inner_flags < 0x10, "Inner Flags must be non-zero four bit");
        Self::Normal { 
            outer: outer_flags.into(), 
            inner: inner_flags.into(), 
            #[cfg(feature = "FEAT_MTE2")]
            tagged: false 
        }
    }
}

impl From<u8> for MemoryAttributeDescriptor {
    fn from(value: u8) -> Self {
        const OUTER_MASK: u8 = 0b11110000;
        const INNER_MASK: u8 = 0b00001111;
        let outer = value & OUTER_MASK;
        let inner = value & INNER_MASK;
        match (outer, inner) {
            (0, inner) => {
                let device = inner >> 2;
                let xs = inner & 0b0011;
                match (device, xs) {
                    (device_flags, 0) => MemoryAttributeDescriptor::device(device_flags),
                    #[cfg(feature = "FEAT_XS")]
                    (device_flags, 0b01) => MemoryAttributeDescriptor::device_xs(device_flags),
                    (_, _) =>  MemoryAttributeDescriptor::Unpredictable,
                }
            }
            #[cfg(feature = "FEAT_XS")]
            (0b0100, 0) => unimplemented!("XS Feature support not implemented"),
            #[cfg(feature = "FEAT_XS")]
            (0b1010, 0) => unimplemented!("XS Feature support not implemented"),
            #[cfg(feature = "FEAT_MTE2")]
            (0b1111, 0) => unimplemented!("MTE2 Feature support not implemented"),
            (_, 0) => MemoryAttributeDescriptor::Unpredictable,
            (outer, inner) => MemoryAttributeDescriptor::normal(outer , inner)
        }

    }
}

impl Into<u8> for MemoryAttributeDescriptor {
    fn into(self) -> u8 {
        match self {
            MemoryAttributeDescriptor::Device { memory_type, #[cfg(feature = "FEAT_XS")] xs_zero } => {
                let mut result: u8 = 0;
                result |= (memory_type as u8) << 2;
                #[cfg(feature = "FEAT_XS")]
                {
                    result |= if xs_zero { 1 } else { 0 };
                }
                result
            },
            MemoryAttributeDescriptor::Normal { outer, inner, #[cfg(feature = "FEAT_XS")] xs_zero, #[cfg(feature = "FEAT_MTE2")] tagged } => {
                let mut result: u8 = <NormalMemoryType as Into<u8>>::into(inner);
                result |= <NormalMemoryType as Into<u8>>::into(outer) << 4;
                result
            },
            MemoryAttributeDescriptor::Unpredictable => panic!("can't map an undefined / unpredictable value to u8"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum AllocatePolicy {
    NoAllocate = 0,
    Allocate = 1
}

#[derive(Clone, Copy)]
pub enum CacheWritePolicy {
    WriteThrough = 0,
    WriteBack = 1
}

#[derive(Clone, Copy)]
pub enum CacheTransistence {
    Transient = 0,
    NonTransient = 1
}

#[derive(Clone, Copy)]
pub struct NormalCacheType {
    pub write_policy: CacheWritePolicy,
    pub transistence: CacheTransistence,
    pub read_allocate_policy: AllocatePolicy,
    pub write_allocate_policy: AllocatePolicy,
}

#[derive(Clone, Copy)]
pub struct NormalMemoryType {
    pub caching: Option<NormalCacheType>
}


impl Into<u8> for NormalMemoryType {
    fn into(self) -> u8 {
        match self.caching {
            Some(cache) => (cache.write_policy as u8) << 3 |
                (cache.transistence as u8) << 2 |
                (cache.read_allocate_policy as u8) << 1 |
                cache.write_allocate_policy as u8,
            None => 0b0100,
        }
    }
}


impl From<u8> for NormalMemoryType {
    fn from(value: u8) -> Self {
        assert!(value > 0 && value < 0x10, "Attribute Flags must be non-zero four bit");
        let c = value >> 2;
        let w = value & 1;
        let r = (value >> 1) & 1;
        Self { caching: 
            match (c, r, w) {
                (0b00, 0, 0) => unreachable!("Asserted away"),
                (0b01, 0, 0) => None,
                (0b00, r, w) => Some(NormalCacheType {
                    write_policy: CacheWritePolicy::WriteThrough,
                    transistence: CacheTransistence::Transient,
                    read_allocate_policy: r.into(),
                    write_allocate_policy: w.into(),
                }),
                (0b01, r, w) => Some(NormalCacheType {
                    write_policy: CacheWritePolicy::WriteBack,
                    transistence: CacheTransistence::Transient,
                    read_allocate_policy: r.into(),
                    write_allocate_policy: w.into(),
                }),
                (0b10, r, w) => Some(NormalCacheType {
                    write_policy: CacheWritePolicy::WriteThrough,
                    transistence: CacheTransistence::NonTransient,
                    read_allocate_policy: r.into(),
                    write_allocate_policy: w.into(),
                }),
                (0b11, r, w) => Some(NormalCacheType {
                    write_policy: CacheWritePolicy::WriteBack,
                    transistence: CacheTransistence::NonTransient,
                    read_allocate_policy: r.into(),
                    write_allocate_policy: w.into(),
                }),
                _ => unreachable!("Asserted away all other cases")
            }
        }
    }
}



#[derive(Clone, Copy)]
pub enum DeviceMemoryType {
    NGnRnE = 0b00,
    NGnRE = 0b01,
    NGRE = 0b10,
    GRE = 0b11
}

impl From<u8> for DeviceMemoryType {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => DeviceMemoryType::NGnRnE,
            1 => DeviceMemoryType::NGnRE,
            2 => DeviceMemoryType::NGRE,
            3 => DeviceMemoryType::GRE,
            _ => unreachable!("AND-masked away all other cases")
        }
    }
}

impl From<u8> for AllocatePolicy {
    fn from(value: u8) -> Self {
        match value & 0b1 {
            0 => AllocatePolicy::NoAllocate,
            1 => AllocatePolicy::Allocate,
            _ => unreachable!("AND-masked away all other cases")
        }
    }
}




