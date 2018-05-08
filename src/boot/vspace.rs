//! Definitions for boot time vspaces

use vspace::window::Window;
use util::units::GB;
use core::ops::Range;

pub struct Low;

/// Beginning of low 1-1 mapped memory
///
/// This is 1 and not 0 as we must avoid creationg objects in rust that have the 0 pointer
/// as this is UB.
const LOW: [Range<usize>; 1] = [1..4*GB];

unsafe impl<'a> Window<'a> for Low {
    fn range_valid(&self, range: [Range<usize>; 1]) -> bool {
        LOW.contains(&range[0])
    }
}

impl Low {
    /// Construct the low vspace window
    ///
    /// # Safety
    ///
    /// This must only be created if this *is* the active window and with a lifetime that
    /// ensures it is deleted at least before we switch away.
    pub unsafe fn make() -> Self {
        Low
    }
}
