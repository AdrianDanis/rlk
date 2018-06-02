use core::marker::PhantomData;

use raw_cpuid::*;

/// Marker is a zero sized type that prevents a struct from being constructed publicly
type Marker = PhantomData<()>;

macro_rules! make_flag {
    ($name:ident, $set:ident, $flag:ident) => {
    #[derive(Debug, Clone, Copy)]
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
make_flag!(Page1GB, get_extended_function_info, has_1gib_pages);

#[derive(Debug, Clone, Copy)]
pub enum Missing {
    LongMode,
    FPU,
    TSC,
    MSR,
    APIC,
    PAT,
}

#[derive(Debug, Clone, Copy)]
pub struct Required {
    long: LongMode,
    fpu: FPU,
    tsc: TSC,
    msr: MSR,
    apic: APIC,
    pat: PAT,
}

impl Required {
    const unsafe fn empty() -> Self {
        Required{
            long: LongMode::new(),
            fpu: FPU::new(),
            tsc: TSC::new(),
            msr: MSR::new(),
            apic: APIC::new(),
            pat: PAT::new(),
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
            long: lm,
            fpu: fpu,
            tsc: tsc,
            msr: msr,
            apic: apic,
            pat: pat,
        })
    }
    pub fn get_pat(&self) -> PAT {
        self.pat
    }
    pub fn get_msr(&self) -> MSR {
        self.msr
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Features {
    required: Required,
    page1gb: Option<Page1GB>,
}

impl Features {
    pub const unsafe fn empty() -> Self {
        Features {
            required: Required::empty(),
            page1gb: None,
        }
    }
    pub fn check() -> Result<Self, Missing> {
        let required = Required::check()?;
        Ok(Self {
            required: required,
            page1gb: Page1GB::check(),
        })
    }
    pub fn required(&self) -> Required {
        self.required
    }
    pub fn page1gb(&self) -> Option<Page1GB> {
        self.page1gb
    }
}
