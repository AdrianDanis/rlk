pub mod features;
mod pat;

pub use self::features::Features;
use state::CPU_FEATURES;
use x86::shared::control_regs::{cr4, cr4_write, CR4_ENABLE_GLOBAL_PAGES, cr3_write};
use x86::bits64::paging::{PDPTEntry, PDPT_PWT, PDPT_PCD, PDPT_PAT};

/// x86 Memory Types
#[derive(Debug, Clone, Copy)]
pub enum MemoryType {
    /// Uncacheable
    StrongUC,
    /// Uncacheable
    ///
    /// Unlike `StrongUC`, this can be overridden by the MTRRs
    UC,
    /// Write Combing
    WC,
    /// Write Through
    WT,
    /// Write Protect
    WP,
    /// Write Back
    WB,
}

impl MemoryType {
    /// Array of all the possible memory types
    fn all<'a>() -> &'a[MemoryType] {
        &[MemoryType::StrongUC, MemoryType::UC, MemoryType::WC, MemoryType::WT, MemoryType::WP, MemoryType::WB]
    }
}

impl From<MemoryType> for PDPTEntry {
    fn from(mt: MemoryType) -> PDPTEntry {
        let mut entry = PDPTEntry::empty();
        let index = pat::Entry::from(mt).index();
        if index.pwt() {
            entry.insert(PDPT_PWT);
        }
        if index.pcd() {
            entry.insert(PDPT_PCD);
        }
        if index.pat() {
            entry.insert(PDPT_PAT);
        }
        entry
    }
}

/// Read MSR wrapper
///
/// This wrapper allows for handling faults to deal with MSRs that may or may not exist
pub unsafe fn maybe_rdmsr(_msr: u32) -> Option<u64> {
    let _msr = CPU_FEATURES.get_required().get_msr();
    unimplemented!()
}

/// Write MSR wrapper
///
/// This wrapper allows for handling faults to deal with MSRs that may or may not exist
pub unsafe fn maybe_wrmsr(_msr: u32, _value: u64) -> bool {
    let _msr = CPU_FEATURES.get_required().get_msr();
    unimplemented!()
}

/// Make a new address space
///
/// TODO: needs some more sensible types at some point?
pub unsafe fn load_cr3(paddr: usize, _pcid: u16, preserve_translation: bool) {
    if preserve_translation {
        panic!("not supported yet");
    }
    // put the pcid in the bin and ignore it
    cr3_write(paddr as u64);
}

pub fn init() -> bool {
    print!(Info, "Checking CPU for required and optional feature");
    match Features::check() {
        Err(e) => panic!("Failed to find required CPU features: {:?}", e),
        Ok(features) => unsafe { CPU_FEATURES = features; },
    }
    print!(Info, "CPU has minimal supported features");
    // TODO: printout feature list
    print!(Info, "Initializing CPU");
    pat::init();
    // Enable global pages if supported
    unsafe {
        if CPU_FEATURES.get_pge().is_some() {
            cr4_write(cr4() | CR4_ENABLE_GLOBAL_PAGES);
        }
    }
    true
}
