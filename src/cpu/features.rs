use core::marker::PhantomData;

use raw_cpuid::*;

/// Marker is a zero sized type that prevents a struct from being constructed publicly
type Marker = PhantomData<()>;

macro_rules! make_flag {
    ($name:ident, $set:ident, $flag:ident) => {
    pub struct $name(Marker);
    impl $name {
        const fn new() -> Self {
            Self{0: PhantomData}
        }
        pub fn check() -> Option<Self> {
            CpuId::new().$set().and_then(|x| if x.$flag() { Some(Self::new()) } else { None } )
        }
    }}
}

make_flag!(LongMode, get_extended_function_info, has_64bit_mode);
make_flag!(FPU, get_feature_info, has_fpu);
make_flag!(TSC, get_feature_info, has_tsc);
make_flag!(MSR, get_feature_info, has_msr);
make_flag!(APIC, get_feature_info, has_apic);
make_flag!(PAT, get_feature_info, has_pat);

#[derive(Debug, Clone, Copy)]
pub enum Missing {
    LongMode,
    FPU,
    TSC,
    MSR,
    APIC,
    PAT,
}

pub struct Required(LongMode, FPU, TSC, MSR, APIC, PAT);

impl Required {
    const unsafe fn empty() -> Self {
        Required{
            0: LongMode::new(),
            1: FPU::new(),
            2: TSC::new(),
            3: MSR::new(),
            4: APIC::new(),
            5: PAT::new(),
        }
    }
    pub fn check() -> Result<Self, Missing> {
        let lm = LongMode::check().ok_or(Missing::LongMode)?;
        let fpu = FPU::check().ok_or(Missing::FPU)?;
        let tsc = TSC::check().ok_or(Missing::TSC)?;
        let msr = MSR::check().ok_or(Missing::MSR)?;
        let apic = APIC::check().ok_or(Missing::APIC)?;
        let pat = PAT::check().ok_or(Missing::PAT)?;
        Ok(Required {
            0: lm,
            1: fpu,
            2: tsc,
            3: msr,
            4: apic,
            5: pat,
        })
    }
}

pub struct Features {
    required: Required,
}

impl Features {
    pub const unsafe fn empty() -> Self {
        Features { required: Required::empty()}
    }
    pub fn check() -> Result<Self, Missing> {
        let required = Required::check()?;
        Ok(Self { required: required})
    }
}
