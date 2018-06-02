pub mod features;
mod pat;

pub use self::features::Features;

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

pub fn init() -> bool {
    print!(Trace, "Performing CPU initialization");
    pat::init();
    true
}
