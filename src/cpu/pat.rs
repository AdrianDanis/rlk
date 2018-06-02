use state::CPU_FEATURES;
use cpu;
use super::MemoryType;
use x86::shared::msr::{IA32_PAT, rdmsr, wrmsr};
use core::mem;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Value {
    StrongUC = 0,
    UC = 7,
    WC = 1,
    WT = 4,
    WP = 5,
    WB = 6,
}

#[derive(Debug, Clone, Copy)]
pub struct Index(u32);

impl Index {
    fn index(&self) -> usize {
        self.0 as usize
    }
    pub fn pwt(&self) -> bool {
        (self.0 % 2) == 1
    }
    pub fn pcd(&self) -> bool {
        ((self.0 / 2) % 2) == 0
    }
    pub fn pat(&self) -> bool {
        self.0 / 4 == 1
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Entry(MemoryType);

impl From<MemoryType> for Entry {
    fn from(mt: MemoryType) -> Entry {
        Entry{0: mt}
    }
}

impl Entry {
    // TODO: index should be a struct of its own to allow conversions to bits in paging structures
    pub fn index(&self) -> Index {
        // Keep the first 4 entries the default as specified in Intel manual
        // so our current page tables keep working
        match self.0 {
            MemoryType::WB => Index{0:0},
            MemoryType::WT => Index{0:1},
            MemoryType::UC => Index{0:2},
            MemoryType::StrongUC => Index{0:3},
            MemoryType::WC => Index{0:4},
            MemoryType::WP => Index{0:5},
        }
    }
    fn value(&self) -> Value {
        match self.0 {
            MemoryType::WB => Value::WB,
            MemoryType::WT => Value::WT,
            MemoryType::UC => Value::UC,
            MemoryType::StrongUC => Value::StrongUC,
            MemoryType::WC => Value::WC,
            MemoryType::WP => Value::WP,
        }
    }
}

type PAT = [Value; 8];

pub fn init() {
    let _pat: cpu::features::PAT = unsafe{CPU_FEATURES}.get_required().get_pat();
    let _msr: cpu::features::MSR = unsafe{CPU_FEATURES}.get_required().get_msr();
    // read the current PAT
    let mut pat: PAT = unsafe{mem::transmute(rdmsr(IA32_PAT))};
    for mt in MemoryType::all().iter() {
        let e = Entry::from(*mt);
        pat[e.index().index()] = e.value();
    }
    unsafe{wrmsr(IA32_PAT, mem::transmute(pat))};
    print!(Trace, "Initialized PAT");
}
