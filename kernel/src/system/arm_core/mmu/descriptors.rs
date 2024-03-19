use core::ops::Add;

use mystd::{bit_field, bit_field_type_definition, bitfield2::BitFieldError};

use crate::system::arm_core::features::FEAT_LPA;

bit_field!(pub TableDescriptor(u64) {
    /// # NSTable \[63]
    /// 
    /// Table descriptor bit\[63] is one of the following:
    /// 
    /// * For stage 1 translations in Secure state, the __NSTable__ field which determines the IPA or PA space used for translation tables in subsequent lookup levels. For more information, see Hierarchical control of Secure or Non-secure memory accesses on page D8-5867.
    /// * For stage 1 translations in Non-secure state, this bit is RES0.
    /// * For stage 1 translations in Realm state, this bit is RES0.
    /// * For stage 1 translations in Root state, this bit is RES0.
    /// * For stage 2 translations, this bit is RES0.
    /// 
    /// For stage 1 translations in the EL3 translation regime, the removal of NSTable in Root state is a change from the behavior of EL3 in Secure state.
    /// 
    /// ## Hierarchical control of Secure or Non-secure memory accesses
    /// 
    /// For a Secure translation regime, when a Table descriptor is accessed from a stage 1 translation table in Secure IPA or PA space, the NSTable field determines all of the following:
    /// * If NSTable is 0, the next-level translation table address in the Table descriptor is in Secure IPA or PA space.
    /// * If NSTable is 1, the next-level translation table address in the Table descriptor is in Non-secure IPA or PA space.
    /// 
    /// If the next-level translation table address in the Table descriptor is in Non-secure IPA or PA space, then the address specified by a descriptor in the next lookup-level translation table is in Non-secure IPA or PA space.
    /// 
    /// If stage 2 translation is enabled, the VSTCR_EL2.SA, VTCR_EL2.NSA, VSTCR_EL2.SW, and VTCR_EL2.NSW fields can map an IPA space to a PA space not matching the Security of the IPA space.
    /// 
    /// If all of the following apply, then a stage 1 translation is treated as non-global, meaning the Effective value of nG is 1, regardless of the actual value of the Block descriptor or Page descriptor nG bit:
    /// * The stage 1 translation supports two privilege levels.
    /// * The PE is in Secure state.
    /// * NSTable is 1 at any level of the translation table walk.
    /// 
    /// For more information, see Global and process-specific translation table entries on page D8-5930.
    /// 
    /// The descriptor NSTable field affects all subsequent lookup levels and the translation IPA or PA space. When an NSTable field is changed, software is required to use a break-before-make sequence, including TLB maintenance for all lookup levels for the VA range translated by the descriptor.
    /// 
    /// For more information, see TLB maintenance on page D8-5933 and Using break-before-make when updating translation table entries on page D8-5934.
    63 => ns,

    /// # APTable \[62:61]
    /// 
    /// Access Permission Limit
    /// 
    /// Table descriptor bits\[62:61] are one of the following:
    /// 
    /// * For stage 1 translations, the __APTable__\[1:0] field which determines the access permissions limit for subsequent lookup levels.
    /// * For stage 2 translations, these bits are RES0.
    /// 
    /// ## Hierarchical control of data access permissions
    /// 
    /// Translation table entries at a given lookup level can limit data access permissions at subsequent lookup levels.
    /// 
    /// For a stage 1 translation, the Table descriptor APTable\[1:0] field limits the data access permission of subsequent stage 1 translation lookup levels, regardless of the permissions in subsequent lookup levels, as shown in the following table:
    /// 
    /// APTable\[1:0] Effect at subsequent lookup levels
    /// 
    /// * 00 No effect on permissions.
    /// * 01 Unprivileged access not permitted.
    /// * 10 Write access not permitted.
    /// * 11 Write access not permitted.
    ///     Unprivileged read access not permitted.
    /// 
    /// For a Permission fault, the level of the Block descriptor or Page descriptor is reported regardless of whether the lack of permission was caused by configuration of the APTable or AP fields.
    /// 
    /// For translation regimes that support one Exception level, APTable\[0] is RES0.
    /// 
    /// The APTable\[1:0] settings are combined with the descriptor access permissions in subsequent lookup levels. They do not change the values entered in those descriptors, nor restrict what values can be entered.
    /// 
    /// For the translation regime controlled by a TCR_ELx, one or more of the following can be used to disable the Table descriptor APTable\[1:0] field so that it is IGNORED by the PE and the behavior is as if the value is 0:
    /// * If the Effective value of TCR_ELx.HPD{0} is 1, hierarchical data access permission control is disabled in the translation tables pointed to by TTBR0_ELx.
    /// * If the Effective value of TCR_ELx.HPD1 is 1, hierarchical data access permission control is disabled in the translation tables pointed to by TTBR1_ELx.
    /// 
    /// The descriptor APTable field affects all subsequent lookup levels. When an APTable field is changed, software is required to use a break-before-make sequence, including TLB maintenance for all lookup levels for the VA range translated by the descriptor.
    /// 
    /// For EL1&0 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then APTable\[0] is treated as 0 regardless of the actual value.
    /// 
    /// For more information, see Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    62:61 => stage1_ap: enum APTable {
        NoEffectOnPermissions = 0b00,
        UnpriviledgedAccessNotPermitted = 0b01,
        WriteAccessNotPermitted = 0b10,
        WriteAndUnpriviligedReadAccessNotPermitted = 0b11,
    },

    /// # XNTable / UXNTable [60]
    /// 
    /// Execute Never / Unpriviledged Execute Never
    /// 
    /// Table descriptor bit\[60] is one of the following:
    /// 
    /// * For stage 1 translations that support __one__ privilege level, the __XNTable__ field which determines the execute-never limit for subsequent lookup levels.
    /// * For stage 1 translations that support __two__ privilege levels, the __UXNTable__ field which determines the unprivileged execute-never limit for subsequent lookup levels at EL0.
    /// * For EL1&0 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then the PXNTable field. For more information, see Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// * For stage 2 translations, this bit is RES0.
    /// 
    /// ## Hierarchical control of instruction execution permissions
    /// 
    /// Stage 1 translation table entries at a given lookup level can limit instruction execution permissions at subsequent lookup levels.
    /// 
    /// ### XN 
    /// 
    /// For a stage 1 translation, the value of the XNTable Table descriptor field has one of the following effects:
    /// * If the Effective value of the XNTable field is 0, then the field has no effect.
    /// * If the Effective value of the XNTable field is 1, then all of the following apply:
    ///     - The XNTable field is treated as 1 in all Table descriptors in subsequent lookup levels, regardless of the actual value of XNTable.
    ///     - The XN field in Block descriptors and Page descriptors is treated as 1 in subsequent lookup levels, regardless of the actual value of XN.
    ///     - The value and interpretation of the XNTable and XN fields in all subsequent lookup levels are otherwise unaffected.
    /// 
    /// ### UXN
    /// 
    /// For a stage 1 translation, the value of the UXNTable Table descriptor field has one of the following effects:
    /// * If the Effective value of the UXNTable field is 0, then the field has no effect.
    /// * If the Effective value of the UXNTable field is 1, then all of the following apply:
    ///     - The UXNTable field is treated as 1 in all Table descriptors in subsequent lookup levels, regardless of the actual value of UXNTable.
    ///     - The UXN field in Block descriptors and Page descriptors is treated as 1 in subsequent lookup levels, regardless of the actual value of UXN.
    ///     - The value and interpretation of the UXNTable and UXN fields in all subsequent lookup levels are otherwise unaffected.
    60 => stage1_xn_uxn,

    /// # PXNTable\[59]
    /// 
    /// Priviledged Execute Never
    /// 
    /// Table descriptor bit\[59] is one of the following:
    /// 
    /// * For stage 1 translations that support one privilege level, RES0.
    /// * For stage 1 translations that support two privilege levels, the PXNTable field which determines the privileged execute-never limit for subsequent lookup levels.
    /// * For EL1&0 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then RES0. For more information, see Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// * For stage 2 translations, this bit is RES0.
    /// 
    /// For a stage 1 translation, the value of the PXNTable Table descriptor field has one of the following effects:
    /// * If the Effective value of the PXNTable field is 0, then the field has no effect.
    /// * If the Effective value of the PXNTable field is 1, then all of the following apply:
    ///     - The PXNTable field is treated as 1 in all Table descriptors in subsequent lookup levels, regardless of the actual value of PXNTable.
    ///     - The PXN field in Block descriptors and Page descriptors is treated as 1 in subsequent lookup levels, regardless of the actual value of PXN.
    ///     - The value and interpretation of the PXNTable and PXN fields all subsequent lookup levels are otherwise unaffected.
    59 => stage1_pxn,

    // 58:51 => ignore,
    // 50 => RES0,
    
    // Next level table addresses
    // 
    // these fields are too messy to expose directly, use the accessor methods
    // 49:2 => _next_level_table_address,

    49:12 => _address_48bit_gran4,
    49:14 => _address_48bit_gran16,
    49:16 => _address_48bit_gran64,


    49:12 => _address_52bit_gran4_lsb,
    11:10 => _address_52bit_gran4_msb,
    49:14 => _address_52bit_gran16_lsb,
    11:10 => _address_52bit_gran16_msb,
    47:16 => _address_52bit_gran64_lsb,
    15:12 => _address_52bit_gran64_msb,
    
    /// 
    1:0 => format: enum TableOrBlockDescriptorFormat {
        Invalid00 = 0b00,
        Invalid10 = 0b10,
        Block = 0b01,
        Table = 0b11,
    } = TableOrBlockDescriptorFormat::Table,
});

// Models the "Effective Value of TCR_ELx.DS". In reality it's a bit more complicated, 
// but until we support chips that support FEAT_LPA2 totally irrelevant
const TCR_ELX_DS: bool = crate::system::arm_core::features::FEAT_LPA2;


pub enum AddressingMode {
    Gran4KBAddr48bit,
    #[cfg(feature = "FEAT_LPA2")]
    Gran4KBAddr52bit,
    #[cfg(not(any(feature = "cortex_a72", feature = "cortex_a53")))]
    Gran16KBAddr48bit,
    #[cfg(feature = "FEAT_LPA2")]
    Gran16KBAddr52bit,
    Gran64KBPAddr48BitVAddr48Bit,
    #[cfg(feature = "FEAT_LPA")]
    Gran64KBPAddr52BitVAddr48Bit,
    #[cfg(feature = "FEAT_LVA")]
    Gran64KBPAddr48BitVAddr52Bit,
    #[cfg(all(feature = "FEAT_LVA", feature = "FEAT_LPA"))]
    Gran64KBPAddr52BitVAddr25Bit,
}


impl TableDescriptor {

    pub const fn invalid() -> Self {
        Self::zero()
    }

    pub fn with_next_level_table_at(self, next_level_address: u64, mode: AddressingMode) -> Self {
        match mode {
            AddressingMode::Gran4KBAddr48bit => {
                let field = self._address_48bit_gran4();
                field.set_value(next_level_address >> field.lsb())
            },
            AddressingMode::Gran64KBPAddr48BitVAddr48Bit => {
                let field = self._address_48bit_gran64();
                field.set_value(next_level_address >> field.lsb())
            },
            _ => unimplemented!("Currently only max 48 bit addressing supported, at 4 and 64 KB Granule"),
        }
    }

    pub fn next_level_table_address(self, mode: AddressingMode) -> u64 {
        match mode {
            AddressingMode::Gran4KBAddr48bit => {
                let field = self._address_48bit_gran4();
                field.value() << field.lsb()
            },
            AddressingMode::Gran64KBPAddr48BitVAddr48Bit => {
                let field = self._address_48bit_gran64();
                field.value() << field.lsb()
            },
            _ => unimplemented!("Currently only max 48 bit addressing supported, at 4 and 64 KB Granule"),
        }
    }
}

bit_field_type_definition!(0;1;pub enum DeviceAttribute<u64>{
    NGnRnE = 0b00,
    NGnRE = 0b01,
    NGRE = 0b10,
    Gre = 0b11,
});

bit_field_type_definition!(0;1;pub enum Cacheability<u64> {
    NotApplicable = 0b00,
    NonCacheable = 0b01,
    WriteThroughCacheable = 0b10,
    WriteBackCacheable = 0b11,
});

#[derive(Debug, Clone, Copy)]
pub enum Stage2MemoryAttr {
    Device(DeviceAttribute),
    Normal{
        outer: Cacheability,
        inner: Cacheability,
    }
}

impl Into<u64> for Stage2MemoryAttr {
    fn into(self) -> u64 {
        match self {
            Stage2MemoryAttr::Device(device_flags) => 
                device_flags as u64,
            Stage2MemoryAttr::Normal{outer, inner} =>
                ((outer as u64) << 2) | inner as u64,
        }
    }
}

impl TryFrom<u64> for Stage2MemoryAttr {
    type Error = BitFieldError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > 0b1111 {
            Err(BitFieldError::ValueTooLargeForField)
        } else {
            match (value >> 2, value & 0b11) {
                (0b00, device_flags) => Ok(Self::Device(device_flags.try_into().unwrap())),
                (normal_outer, normal_inner) => Ok(Self::Normal {
                    outer: normal_outer.try_into().unwrap(), 
                    inner: normal_inner.try_into().unwrap()
                })
            }
        }
    }
}

pub enum BlockLevel {
    Level0,
    Level1,
    Level2
}

bit_field!(pub BlockDescriptor(u64) {
    // upper attributes

    /// # AMEC \[63]
    /// Block descriptor and Page descriptor bit\[63] is one of the following:
    /// * For stage 1 translations using the Non-secure translation regime, Secure translation regime, or Realm EL1&0 translation regime, this bit is IGNORED.
    /// * For stage 1 translations using the Realm EL2 or EL2&0 translation regimes, this bit is the AMEC field.
    /// * For stage 2 translations using the Non-secure translation regime or Secure translation regime, this bit is RES0. • For stage 2 translations using the Realm translation regime, this bit is the AMEC field.
    63 => amec,

    /// # PBHA \[62:59]
    /// If FEAT_HPDS2 is not implemented, then Block descriptor and Page descriptor bits\[62:59] are one of the following:
    /// * For stage 1 translations, the bits are IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - Bits\[62:60] are reserved for use by a System MMU.
    ///     - Bit\[59] is IGNORED.
    /// 
    /// If FEAT_HPDS2 is implemented, then Block descriptor and Page descriptor bits\[62:59] can be individually enabled as the Page-Based Hardware Attributes bits (PBHA\[3:0]).
    /// For more information, see Page Based Hardware attributes on page D8-5891.
    /// 
    /// If FEAT_HPDS2 is implemented and a PBHA bit is not enabled by TCR_ELx control bits, then the corresponding
    /// descriptor and Page descriptor bit is one of the following: 
    /// * For stage 1 translations, the bit is IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - For descriptor bits\[62:60], the bit is reserved for use by a System MMU.
    ///     - For descriptor bit\[59], the bit is IGNORED.
    62:59 => pbha,

    // 58:56 => ignore,

    /// # NS (Realm Security) \[55]
    /// Block descriptor and Page descriptor bit\[55] is one of the following: 
    /// * For stage 1 translations, IGNORED.
    /// * For stage 2 translations, one of the following:
    ///     - For the Realm Security state, the NS field.
    ///     - For all Security states other than Realm Security state, IGNORED.
    55 => ns_realm,

    /// # XN / UXN / PXN Flags \[54:53]
    /// 
    /// Block descriptor and Page descriptor bits\[54:53] are one of the following: 
    /// 
    /// * For stage 1 translations, bit\[54] is one of the following:
    ///     - If the translation regime supports only one privilege level, then the Execute-never field (XN).
    ///     - If the translation regime can support two privilege levels, then Unprivileged execute-never field (UXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then the Privileged execute-never field (PXN).
    /// * For stage 1 translations, bit\[53] is one of the following:
    ///     - If the translation regime supports only one privilege level, then RES0.
    ///     - If the translation regime can support two privilege levels, then the Privileged execute-never field (PXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then RES0. 
    /// * For stage 2 translations, bits\[54:53] are one of the following:
    ///     - If FEAT_XNX is not implemented, then bit\[54] is the Execute-never field (XN) and bit\[53] is RES0.
    ///     - If FEAT_XNX is implemented, then bits\[54:53] are the Execute-never field (XN\[1:0]).
    /// 
    /// For more information, see Instruction execution permissions on page D8-5870 and Additional behavior when
    /// HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    54:53 => xn_uxn_pxn,

    /// # Contiguous bit \[52]
    /// 
    /// Block descriptor and Page descriptor bit\[52] is the Contiguous bit.
    /// 
    /// The Contiguous bit identifies a descriptor as belonging to a group of adjacent translation table entries that point to a contiguous OA range.
    /// 
    /// For more information, see The Contiguous bit on page D8-5890.
    52 => contiguous,

    /// # Dirty Bit Modifier (DBM) \[51]
    /// 
    /// Block descriptor and Page descriptor bit\[51] is the Dirty Bit Modifier (DBM).
    /// 
    /// The dirty state is used to indicate a memory block or page has been modified. When hardware update of the dirty state is enabled, the descriptor DBM field indicates whether the descriptor is a candidate for hardware updates of the dirty state.
    /// 
    /// For more information, see Hardware management of the dirty state on page D8-5877.
    51 => dbm,

    /// # Guarded Page (GP) \[50]
    /// 
    /// Block descriptor and Page descriptor bit\[50] is one of the following:
    /// * If FEAT_BTI is not implemented, then Block descriptor and Page descriptor bit\[50] is RES0. 
    /// * If FEAT_BTI is implemented, then one of the following:
    ///     - For stage 1 translations, the Block descriptor and Page descriptor bit\[50] is Guarded Page field (GP).
    ///     - For stage 2 translations, the Block descriptor and Page descriptor bit\[50] is RES0.
    50 => gp,

    // output address
    // too messy, use accessors

    47:30 => _address_48bit_gran4_level1,
    47:21 => _address_48bit_gran4_level2,

    47:25 => _address_48bit_gran16_level2,
    47:29 => _address_48bit_gran64_level2,

    49:39 => _address_52bit_gran4_lsb_level0,
    49:30 => _address_52bit_gran4_lsb_level1,
    49:21 => _address_52bit_gran4_lsb_level2,
    9:8   => _address_52bit_gran4_msb,

    49:36 => _address_52bit_gran16_lsb_level1,
    49:25 => _address_52bit_gran16_lsb_level2,
    9:8   => _address_52bit_gran16_msb,

    47:42 => _address_52bit_gran64_lsb_level1,
    47:29 => _address_52bit_gran64_lsb_level2,
    15:12 => _address_52bit_gran64_msb,


    // lower attributes

    /// # nT \[16]
    /// __Block__ descriptor bit\[16] is one of the following:
    /// * If FEAT_BBM is not implemented, then Block descriptor bit\[16] is RES0.
    /// * If FEAT_BBM is not implemented, then Block descriptor bit\[16] is the nT field. For more information, see Block translation entry on page D8-5892.
    16 => n_t,

    /// # NSE / nG /FnXS \[11]
    /// 
    /// Block descriptor and Page descriptor bit\[11] is one of the following:
    /// * For stage 1 translations that support a single privilege level and are not in Root state, Block descriptor and Page descriptor bit\[11] is RES0.
    /// * For stage 1 translations that are in Root state, Block descriptor and Page descriptor bit\[11] is the NSE field.
    /// * For stage 1 translations that support two privilege levels, Block descriptor and Page descriptor bit\[11] is the not global bit (nG). 
    ///   The nG bit in a Block descriptor and Page descriptor indicates one of the following:
    ///     - If the value is 0, the translation is global and the TLB entry applies to all ASID values.
    ///     - If the value is 1, the translation is non-global and the TLB entry applies to only the current ASID value.
    ///     - For more information, see Global and process-specific translation table entries on page D8-5930.
    /// * For stage 2 translations, one of the following:
    ///     - If FEAT_XS is not implemented, then Block descriptor and Page descriptor bit\[11] is RES0.
    ///     - If FEAT_XS is implemented, then Block descriptor and Page descriptor bit\[11] is FnXS. 
    ///     For a stage 2 translation, the FnXS bit in a Block descriptor and Page descriptor has all of the following properties:
    ///         + If the FnXS bit is 0, then the XS attribute of the resultant memory translation is not modified.
    ///         + If the FnXS bit is 1, then the XS attribute of the resultant memory translation is set to 0.
    ///         + For more information, see XS attribute modifier on page D8-5892.
    11 => nse_ng_fnxs,

    /// # Access Flag (AF) \[10]
    /// The AF in a Block descriptor and Page descriptor indicates one of the following:
    /// * If the value is 0, then the memory region has not been accessed since the value of AF was last set to 0.
    /// * If the value is 1, then the memory region has been accessed since the value of AF was last set to 0.
    /// * For more information, see The Access flag on page D8-5875.
    10 => af,

    /// # Shareability (SH\[1:0]) \[9:8]
    /// 
    /// Block descriptor and Page descriptor bits\[9:8] are one of the following:
    /// 
    /// * For the 4KB and 16KB granules, one of the following:
    ///     - If the Effective value of TCR_ELx.DS is 0, then the Shareability field (SH\[1:0]).
    ///     - If the Effective value of TCR_ELx.DS is 1, then bits\[51:50] of the OA. 
    /// * For the 64KB translation granule, the Shareability field (SH\[1:0]).
    /// 
    /// For more information, see Stage 1 Shareability attributes on page D8-5884 and Stage 2 Shareability attributes on page D8-5888.
    9:8 => sh: enum Shareability {
        NonShareable = 0b00,
        Reserved = 0b01,
        OuterShareable = 0b10,
        InnerShareable = 0b11
    },

    /// # Access Permissions (AP \[2:1]) or Stage 2 Access Permissions (S2AP\[1:0]) \[7:6]
    /// 
    /// Block descriptor and Page descriptor bits\[7:6] are one of the following:
    /// * For stage 1 translations that support a single privilege level, all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is RES1.
    /// * For EL1 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is treated as 0 regardless of the actual value.
    /// * For stage 1 translations that support two privilege levels, the data Access Permissions bits (AP\[2:1]). 
    /// * For stage 2 translations, the stage 2 data Access Permissions field (S2AP\[1:0]).
    /// 
    /// For more information, see Data access permissions for stage 1 translations on page D8-5867, Data access permissions for stage 2 translations on page D8-5868, and Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// 
    /// The Armv8 Block descriptor and Page descriptor format defines the data Access Permissions bits, AP\[2:1], and does not define an AP\[0] bit.
    7:6 => ap_s2ap,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1 Secure State: Non-Secure Flag NS \[5]
    /// * For stage 1 translations bit\[5] is one of the following:
    ///     - When the access is from Secure state, from Realm state using the EL2 or EL2&0 translation regimes, or from Root state, the Non-secure bit (NS). For more information, see Controlling memory access Security state on page D8-5866.
    ///     - When the access is from Non-secure state, or from Realm state using the EL1&0 translation regime, the bit is RES0.
    5 => stage_1_ns_secure,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1: Memory Attributes Index (AttrIndex\[2:0]) \[4:2]
    /// * For stage 1 translations, bits\[4:2] are the stage 1 memory attributes index field for the MAIR_ELx (AttrIndx\[2:0]). For more information, see Stage 1 memory type and Cacheability attributes on page D8-5883.    
    4:2 => stage_1_mem_attr_indx,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[3:0]) \[5:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 0, then bits\[5:2] are the stage 2 memory attributes (MemAttr\[3:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is disabled on page D8-5885.
    5:2 => stage_2_mem_attr: Stage2MemoryAttr,

    /// # !!Not Implemented on A72!!
    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[2:0]) \[4:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 1, then all of the following:
    ///     - Bit\[5] is RES0.
    ///     - Bits\[4:2] are the stage 2 memory attributes (MemAttr\[2:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is enabled on page D8-5887.
    4:2 => stage_2_fwb_mem_attr,

    // descriptor formats
    1:0 => format: TableOrBlockDescriptorFormat = TableOrBlockDescriptorFormat::Block,
});

impl BlockDescriptor {

    pub const fn invalid() -> Self {
        Self::zero()
    }

    pub fn with_output_address(self, output_address: u64, mode: AddressingMode, level: BlockLevel) -> Self {
        match (mode, level) {
            (AddressingMode::Gran4KBAddr48bit, BlockLevel::Level0) => panic!("4KB Granule has no level 0 blocks"),
            (AddressingMode::Gran4KBAddr48bit, BlockLevel::Level1) => {
                let field = self._address_48bit_gran4_level1();
                field.set_value(output_address >> field.lsb())
            },
            (AddressingMode::Gran4KBAddr48bit, BlockLevel::Level2) => {
                let field = self._address_48bit_gran4_level2();
                field.set_value(output_address >> field.lsb())
            },
            (AddressingMode::Gran64KBPAddr48BitVAddr48Bit, BlockLevel::Level0) => panic!("64KB Granule has no level 0 blocks"),
            (AddressingMode::Gran64KBPAddr48BitVAddr48Bit, BlockLevel::Level1) => panic!("64KB Granule has no level 1 blocks"),
            (AddressingMode::Gran64KBPAddr48BitVAddr48Bit, BlockLevel::Level2) => {
                let field = self._address_48bit_gran64_level2();
                field.set_value(output_address >> field.lsb())
            },
           
            _ => unimplemented!("Currently only max 48 bit addressing supported, at 4 and 64 KB Granule"),
        }
    }

    pub fn output_address(self, mode: AddressingMode, level: BlockLevel) -> u64 {
        match (mode, level) {
            (AddressingMode::Gran4KBAddr48bit, BlockLevel::Level0) => panic!("4KB Granule has no level 0 blocks"),
            (AddressingMode::Gran4KBAddr48bit, BlockLevel::Level1) => {
                let field = self._address_48bit_gran4_level1();
                field.value() << field.lsb()
            },
            (AddressingMode::Gran4KBAddr48bit, BlockLevel::Level2) => {
                let field = self._address_48bit_gran4_level2();
                field.value() << field.lsb()
            },
            (AddressingMode::Gran64KBPAddr48BitVAddr48Bit, BlockLevel::Level0) => panic!("64KB Granule has no level 0 blocks"),
            (AddressingMode::Gran64KBPAddr48BitVAddr48Bit, BlockLevel::Level1) => panic!("64KB Granule has no level 1 blocks"),
            (AddressingMode::Gran64KBPAddr48BitVAddr48Bit, BlockLevel::Level2) => {
                let field = self._address_48bit_gran64_level2();
                field.value() << field.lsb()
            },
            _ => unimplemented!("Currently only max 48 bit addressing supported, at 4 and 64 KB Granule"),
        }
    }
}


bit_field!(pub PageDescriptor(u64) {
    // upper attributes

    /// # AMEC \[63]
    /// Block descriptor and Page descriptor bit\[63] is one of the following:
    /// * For stage 1 translations using the Non-secure translation regime, Secure translation regime, or Realm EL1&0 translation regime, this bit is IGNORED.
    /// * For stage 1 translations using the Realm EL2 or EL2&0 translation regimes, this bit is the AMEC field.
    /// * For stage 2 translations using the Non-secure translation regime or Secure translation regime, this bit is RES0. • For stage 2 translations using the Realm translation regime, this bit is the AMEC field.
    63 => amec,

    /// # PBHA \[62:59]
    /// If FEAT_HPDS2 is not implemented, then Block descriptor and Page descriptor bits\[62:59] are one of the following:
    /// * For stage 1 translations, the bits are IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - Bits\[62:60] are reserved for use by a System MMU.
    ///     - Bit\[59] is IGNORED.
    /// 
    /// If FEAT_HPDS2 is implemented, then Block descriptor and Page descriptor bits\[62:59] can be individually enabled as the Page-Based Hardware Attributes bits (PBHA\[3:0]).
    /// For more information, see Page Based Hardware attributes on page D8-5891.
    /// 
    /// If FEAT_HPDS2 is implemented and a PBHA bit is not enabled by TCR_ELx control bits, then the corresponding
    /// descriptor and Page descriptor bit is one of the following: 
    /// * For stage 1 translations, the bit is IGNORED.
    /// * For stage 2 translations, all of the following:
    ///     - For descriptor bits\[62:60], the bit is reserved for use by a System MMU.
    ///     - For descriptor bit\[59], the bit is IGNORED.
    62:59 => pbha,

    // 58:56 => ignore,

    /// # NS (Realm Security) \[55]
    /// Block descriptor and Page descriptor bit\[55] is one of the following: 
    /// * For stage 1 translations, IGNORED.
    /// * For stage 2 translations, one of the following:
    ///     - For the Realm Security state, the NS field.
    ///     - For all Security states other than Realm Security state, IGNORED.
    55 => ns_realm,

    /// # XN / UXN / PXN Flags \[54:53]
    /// 
    /// Block descriptor and Page descriptor bits\[54:53] are one of the following: 
    /// 
    /// * For stage 1 translations, bit\[54] is one of the following:
    ///     - If the translation regime supports only one privilege level, then the Execute-never field (XN).
    ///     - If the translation regime can support two privilege levels, then Unprivileged execute-never field (UXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then the Privileged execute-never field (PXN).
    /// * For stage 1 translations, bit\[53] is one of the following:
    ///     - If the translation regime supports only one privilege level, then RES0.
    ///     - If the translation regime can support two privilege levels, then the Privileged execute-never field (PXN).
    ///     - For EL1&0 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then RES0. 
    /// * For stage 2 translations, bits\[54:53] are one of the following:
    ///     - If FEAT_XNX is not implemented, then bit\[54] is the Execute-never field (XN) and bit\[53] is RES0.
    ///     - If FEAT_XNX is implemented, then bits\[54:53] are the Execute-never field (XN\[1:0]).
    /// 
    /// For more information, see Instruction execution permissions on page D8-5870 and Additional behavior when
    /// HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    54:53 => xn_uxn_pxn,

    /// # Contiguous bit \[52]
    /// 
    /// Block descriptor and Page descriptor bit\[52] is the Contiguous bit.
    /// 
    /// The Contiguous bit identifies a descriptor as belonging to a group of adjacent translation table entries that point to a contiguous OA range.
    /// 
    /// For more information, see The Contiguous bit on page D8-5890.
    52 => contiguous,

    /// # Dirty Bit Modifier (DBM) \[51]
    /// 
    /// Block descriptor and Page descriptor bit\[51] is the Dirty Bit Modifier (DBM).
    /// 
    /// The dirty state is used to indicate a memory block or page has been modified. When hardware update of the dirty state is enabled, the descriptor DBM field indicates whether the descriptor is a candidate for hardware updates of the dirty state.
    /// 
    /// For more information, see Hardware management of the dirty state on page D8-5877.
    51 => dbm,

    /// # Guarded Page (GP) \[50]
    /// 
    /// Block descriptor and Page descriptor bit\[50] is one of the following:
    /// * If FEAT_BTI is not implemented, then Block descriptor and Page descriptor bit\[50] is RES0. 
    /// * If FEAT_BTI is implemented, then one of the following:
    ///     - For stage 1 translations, the Block descriptor and Page descriptor bit\[50] is Guarded Page field (GP).
    ///     - For stage 2 translations, the Block descriptor and Page descriptor bit\[50] is RES0.
    50 => gp,

    // output address
    // too messy, use accessors

    47:12 => _address_48bit_gran4,
    47:14 => _address_48bit_gran16,
    47:16 => _address_48bit_gran64,

    49:12 => _address_52bit_gran4_lsb,
    9:8   => _address_52bit_gran4_msb,

    49:14 => _address_52bit_gran16_lsb,
    9:8   => _address_52bit_gran16_msb,

    47:16 => _address_52bit_gran64_lsb,
    15:12 => _address_52bit_gran64_msb,


    // lower attributes

    /// # NSE / nG /FnXS \[11]
    /// 
    /// Block descriptor and Page descriptor bit\[11] is one of the following:
    /// * For stage 1 translations that support a single privilege level and are not in Root state, Block descriptor and Page descriptor bit\[11] is RES0.
    /// * For stage 1 translations that are in Root state, Block descriptor and Page descriptor bit\[11] is the NSE field.
    /// * For stage 1 translations that support two privilege levels, Block descriptor and Page descriptor bit\[11] is the not global bit (nG). 
    ///   The nG bit in a Block descriptor and Page descriptor indicates one of the following:
    ///     - If the value is 0, the translation is global and the TLB entry applies to all ASID values.
    ///     - If the value is 1, the translation is non-global and the TLB entry applies to only the current ASID value.
    ///     - For more information, see Global and process-specific translation table entries on page D8-5930.
    /// * For stage 2 translations, one of the following:
    ///     - If FEAT_XS is not implemented, then Block descriptor and Page descriptor bit\[11] is RES0.
    ///     - If FEAT_XS is implemented, then Block descriptor and Page descriptor bit\[11] is FnXS. 
    ///     For a stage 2 translation, the FnXS bit in a Block descriptor and Page descriptor has all of the following properties:
    ///         + If the FnXS bit is 0, then the XS attribute of the resultant memory translation is not modified.
    ///         + If the FnXS bit is 1, then the XS attribute of the resultant memory translation is set to 0.
    ///         + For more information, see XS attribute modifier on page D8-5892.
    11 => nse_ng_fnxs,

    /// # Access Flag (AF) \[10]
    /// The AF in a Block descriptor and Page descriptor indicates one of the following:
    /// * If the value is 0, then the memory region has not been accessed since the value of AF was last set to 0.
    /// * If the value is 1, then the memory region has been accessed since the value of AF was last set to 0.
    /// * For more information, see The Access flag on page D8-5875.
    10 => af,

    /// # Shareability (SH\[1:0]) \[9:8]
    /// 
    /// Block descriptor and Page descriptor bits\[9:8] are one of the following:
    /// 
    /// * For the 4KB and 16KB granules, one of the following:
    ///     - If the Effective value of TCR_ELx.DS is 0, then the Shareability field (SH\[1:0]).
    ///     - If the Effective value of TCR_ELx.DS is 1, then bits\[51:50] of the OA. 
    /// * For the 64KB translation granule, the Shareability field (SH\[1:0]).
    /// 
    /// For more information, see Stage 1 Shareability attributes on page D8-5884 and Stage 2 Shareability attributes on page D8-5888.
    9:8 => sh: Shareability,

    /// # Access Permissions (AP \[2:1]) or Stage 2 Access Permissions (S2AP\[1:0]) \[7:6]
    /// 
    /// Block descriptor and Page descriptor bits\[7:6] are one of the following:
    /// * For stage 1 translations that support a single privilege level, all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is RES1.
    /// * For EL1 stage 1 translations, if the Effective value of HCR_EL2.{NV, NV1} is {1, 1}, then all of the following:
    ///     - Bit\[7] is the data Access Permissions bit (AP\[2]).
    ///     - Bit\[6] is treated as 0 regardless of the actual value.
    /// * For stage 1 translations that support two privilege levels, the data Access Permissions bits (AP\[2:1]). 
    /// * For stage 2 translations, the stage 2 data Access Permissions field (S2AP\[1:0]).
    /// 
    /// For more information, see Data access permissions for stage 1 translations on page D8-5867, Data access permissions for stage 2 translations on page D8-5868, and Additional behavior when HCR_EL2.NV is 1 and HCR_EL2.NV1 is 1 on page D8-5909.
    /// 
    /// The Armv8 Block descriptor and Page descriptor format defines the data Access Permissions bits, AP\[2:1], and does not define an AP\[0] bit.
    7:6 => ap_s2ap,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1 Secure State: Non-Secure Flag NS \[5]
    /// * For stage 1 translations bit\[5] is one of the following:
    ///     - When the access is from Secure state, from Realm state using the EL2 or EL2&0 translation regimes, or from Root state, the Non-secure bit (NS). For more information, see Controlling memory access Security state on page D8-5866.
    ///     - When the access is from Non-secure state, or from Realm state using the EL1&0 translation regime, the bit is RES0.
    5 => stage_1_ns_secure,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 1: Memory Attributes Index (AttrIndex\[2:0]) \[4:2]
    /// * For stage 1 translations, bits\[4:2] are the stage 1 memory attributes index field for the MAIR_ELx (AttrIndx\[2:0]). For more information, see Stage 1 memory type and Cacheability attributes on page D8-5883.    
    4:2 => stage_1_mem_attr_indx,

    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[3:0]) \[5:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 0, then bits\[5:2] are the stage 2 memory attributes (MemAttr\[3:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is disabled on page D8-5885.
    5:2 => stage_2_mem_attr: Stage2MemoryAttr,

    /// # !!Not Implemented on A72!!
    /// Block descriptor and Page descriptor bits\[5:2] are one of the following: 
    /// # Stage 2: Memory Attributes (MemAttr\[2:0]) \[4:2]
    /// * For stage 2 translations, if the Effective value of HCR_EL2.FWB is 1, then all of the following:
    ///     - Bit\[5] is RES0.
    ///     - Bits\[4:2] are the stage 2 memory attributes (MemAttr\[2:0]). For more information, see Stage 2 memory type and Cacheability attributes when FEAT_S2FWB is enabled on page D8-5887.
    4:2 => stage_2_fwb_mem_attr,

    // descriptor formats
    1:0 => format: enum PageDescriptorFormat {
        Invalid00 = 0b00,
        Invalid01 = 0b01,
        Invalid10 = 0b10,
        Page = 0b11,
    } = PageDescriptorFormat::Page
});


impl PageDescriptor {

    pub const fn invalid() -> Self {
        Self::zero()
    }

    pub fn with_output_address(self, output_address: u64, mode: AddressingMode) -> Self {
        match mode {
            AddressingMode::Gran4KBAddr48bit => {
                let field = self._address_48bit_gran4();
                field.set_value(output_address >> field.lsb())
            },
            AddressingMode::Gran64KBPAddr48BitVAddr48Bit => {
                let field = self._address_48bit_gran64();
                field.set_value(output_address >> field.lsb())
            },
            
            _ => unimplemented!("Currently only max 48 bit addressing supported, at 4 and 64 KB Granule"),
        }
    }

    pub fn output_address(self, mode: AddressingMode) -> u64 {
        match mode {
            AddressingMode::Gran4KBAddr48bit => {
                let field = self._address_48bit_gran4();
                field.value() << field.lsb()
            },
            AddressingMode::Gran64KBPAddr48BitVAddr48Bit => {
                let field = self._address_48bit_gran64();
                field.value() << field.lsb()
            },
            _ => unimplemented!("Currently only max 48 bit addressing supported, at 4 and 64 KB Granule"),
        }
    }
}