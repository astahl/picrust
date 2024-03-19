use core::{arch::asm, ops::Index};

use mystd::bit_field;

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

bit_field!(pub MairEl1(u64) {
    63:56 => attr_7 : MemoryAttributes,
    55:48 => attr_6 : MemoryAttributes,
    47:40 => attr_5 : MemoryAttributes,
    39:32 => attr_4 : MemoryAttributes,
    31:24 => attr_3 : MemoryAttributes,
    23:16 => attr_2 : MemoryAttributes,
    15:8 => attr_1 : MemoryAttributes,
    7:0 => attr_0 : MemoryAttributes,
});

impl MairEl1 {
    #[must_use]
    pub fn set(self, index: u64, attr: MemoryAttributeDescriptor) -> Self {
        match index {
            0 => self.attr_0().set_value(attr.into()),
            1 => self.attr_1().set_value(attr.into()),
            2 => self.attr_2().set_value(attr.into()),
            3 => self.attr_3().set_value(attr.into()),
            4 => self.attr_4().set_value(attr.into()),
            5 => self.attr_5().set_value(attr.into()),
            6 => self.attr_6().set_value(attr.into()),
            7 => self.attr_7().set_value(attr.into()),
            _ => panic!("MairEl1: Attribute Index > 7")
        }
    }
}


bit_field!(pub MemoryAttributes(u64) {
    7:4 => outer: NormalMemoryAttributes,
    3:0 => inner: NormalMemoryAttributes,
    3:2 => device: enum DeviceMemoryType {
        NGnRnE = 0b00,
        NGnRE = 0b01,
        NGRE = 0b10,
        GRE = 0b11
    },
    1:0 => flags,
    #[cfg(feature = "FEAT_XS")]
    0 => device_xs
});


bit_field!(pub NormalMemoryAttributes(u64) {
    3 => cache_transience: enum CacheTransience {
        Transient,
        NonTransient
    },
    2 => write_policy: enum CacheWritePolicy {
        WriteThrough,
        WriteBack
    },
    1 => read_allocate: enum AllocatePolicy {
        NoAllocate,
        Allocate    
    },
    0 => write_allocate: AllocatePolicy,
});

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
    pub fn device(memory_type: DeviceMemoryType) -> Self {
        Self::Device { 
            memory_type,
            #[cfg(feature = "FEAT_XS")]
            xs_zero: false
        }
    }

    #[cfg(feature = "FEAT_XS")]
    pub fn device_xs(memory_type: DeviceMemoryType) -> Self {
        Self::Device { 
            memory_type,
            xs_zero: true
        }
    }

    pub fn normal(outer: NormalMemoryType, inner: NormalMemoryType) -> Self {
        Self::Normal { 
            outer, 
            inner, 
            #[cfg(feature = "FEAT_MTE2")]
            tagged: false 
        }
    }
}

impl From<MemoryAttributes> for MemoryAttributeDescriptor {
    fn from(value: MemoryAttributes) -> Self {
        let outer = value.outer().value().map(NormalMemoryType::try_from).unwrap();
        let inner = value.inner().value().map(NormalMemoryType::try_from).unwrap();
        match (inner, outer) {
            (Err(_), _) => {
            // either device or unpredictable
            let device_memory_type = value.device().value().unwrap();
            match value.flags().value() {
                0b00 => MemoryAttributeDescriptor::device(device_memory_type),
                #[cfg(feature = "FEAT_XS")]
                0b01 => MemoryAttributeDescriptor::device_xs(device_memory_type),
                _ => MemoryAttributeDescriptor::Unpredictable
            }
        },
        (_, Err(_)) => MemoryAttributeDescriptor::Unpredictable,
        (Ok(inner), Ok(outer)) => MemoryAttributeDescriptor::normal(outer, inner)
        }

    }
}

impl Into<MemoryAttributes> for MemoryAttributeDescriptor {
    fn into(self) -> MemoryAttributes {
        let mut result = MemoryAttributes::zero();
        match self {
            MemoryAttributeDescriptor::Device { memory_type, #[cfg(feature = "FEAT_XS")] xs_zero } => {
                let result = result.device().set_value(memory_type);
                #[cfg(feature = "FEAT_XS")]
                {
                    result.device_xs().set_value(xs_zero);
                }
                result
            },
            MemoryAttributeDescriptor::Normal { outer, inner, #[cfg(feature = "FEAT_XS")] xs_zero, #[cfg(feature = "FEAT_MTE2")] tagged } => {
                result.inner().set_value(inner.into())
                    .outer().set_value(outer.into())
            },
            MemoryAttributeDescriptor::Unpredictable => panic!("can't map an undefined / unpredictable value to MemoryAttributes"),
        }
    }
}


#[derive(Clone, Copy)]
pub struct NormalCacheType {
    pub write_policy: CacheWritePolicy,
    pub transience: CacheTransience,
    pub read_allocate_policy: AllocatePolicy,
    pub write_allocate_policy: AllocatePolicy,
}

#[derive(Clone, Copy)]
pub struct NormalMemoryType {
    pub caching: Option<NormalCacheType>
}


impl Into<NormalMemoryAttributes> for NormalMemoryType {
    fn into(self) -> NormalMemoryAttributes {
        match self.caching {
            Some(NormalCacheType { write_policy, transience, read_allocate_policy, write_allocate_policy }) => 
                NormalMemoryAttributes::zero()
                    .write_policy().set_value(write_policy)
                    .cache_transience().set_value(transience)
                    .read_allocate().set_value(read_allocate_policy)
                    .write_allocate().set_value(write_allocate_policy)
            ,
            None => 0b0100.into(),
        }
    }
}


impl TryFrom<NormalMemoryAttributes> for NormalMemoryType {
    
    type Error = ();
    
    fn try_from(value: NormalMemoryAttributes) -> Result<Self, Self::Error> {
        match value.to_underlying() {
            0 => Err(()),
            0b0100 => Ok(Self { caching: None }),
            _ => {
                let write_policy = value.write_policy().value();
                let transience = value.cache_transience().value();
                let read_allocate_policy = value.read_allocate().value();
                let write_allocate_policy = value.write_allocate().value();
                Ok(Self{ caching: Some(NormalCacheType { 
                    write_policy,
                    transience,
                    read_allocate_policy,
                    write_allocate_policy 
                })})
            }
        
        }
    }
}





