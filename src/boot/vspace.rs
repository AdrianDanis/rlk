//! Definitions for boot time vspaces

use vspace::Window;
use util::units::GB;
use util::Empty;
use core::ops::Range;
use core::marker::PhantomData;
use core::borrow::Borrow;

pub struct Init;

/// Beginning of initial high memory window which is only 4gb
const INIT: [Range<usize>; 1] = [0xffffff8000000000..0xffffff8000000000 + 4*GB];

unsafe impl Window for Init {
    fn range_valid(&self, range: [Range<usize>; 1]) -> bool {
        INIT[0].contains(&range[0].start) && INIT[0].contains(&(range[0].end - 1))
    }
}

impl Init {
    /// Construct the Init vspace window
    pub fn make() -> Self {
        Init
    }
}
