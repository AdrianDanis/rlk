pub mod features;
mod pat;

pub use self::features::Features;
use state::CPU_FEATURES;
use x86::shared::control_regs::{cr4, cr4_write, CR4_ENABLE_GLOBAL_PAGES};

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
