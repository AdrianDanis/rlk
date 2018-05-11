//! Modules for in place declarations
//!
//! This provides mechanism for registering handlers in place and, through placing information in
//! linker sections, have them get called from the appropriate place. With this you can declare
//! at the original implementation site that this needs to happen without needing to perform further
//! registration or querying.
//!
//! The use case is for things like cmdline options etc where any module in the kernel may wish to
//! be parameterized by a cmdline argument and just have it 'work' without introducing further logic

use core;

// Random bytes curtesy of random.org
pub const DECL_NONCE: u64 = 0x4ea4789985e1ad56;

/// Declares a function to run on a cmdline switch
///
/// cmdline options are processed *very* early, before there is even memory allocation
/// available, so these functions should do absolute minimal work to record that an
/// option took place.
pub struct CMDLine {
    pub option: &'static str,
    pub f: fn(&str) -> (),
}

pub struct SelfTest {
}

pub enum Type {
    CMDLine(CMDLine),
    SelfTest(SelfTest),
}

#[repr(align(64))]
pub struct RawDecl {
    pub nonce: u64,
    pub decl: &'static Type,
}

pub struct DeclIter<'t> {
    iter: core::slice::Iter<'t, RawDecl>,
}

impl <'t> Iterator for DeclIter<'t> {
    type Item = &'t Type;

    fn next(&mut self) -> Option<&'t Type> {
        self.iter.next().map(|x| x.decl)
    }
}

extern {
    static decls_section_begin: usize;
    static decls_section_end: usize;
}

pub fn iter() -> DeclIter<'static> {
    unsafe {
        let begin = &decls_section_begin as *const usize as *const RawDecl;
        let end = &decls_section_end as *const usize as *const RawDecl;
        DeclIter {iter: core::slice::from_raw_parts(begin, end.offset_from(begin) as usize).iter()}
    }
}

#[macro_export]
macro_rules! decls_iter {
    ($decltype:ident) => ($crate::decls::iter().filter_map(|x| if let $crate::decls::Type::$decltype(y) = x { Some(y) } else { None}))
}

#[macro_export]
macro_rules! make_decl {
    ($init:expr, $name:ident) => (interpolate_idents! {
        static [DECL_ $name]: $crate::decls::Type = $init;
        #[link_section=".decls"]
        #[used]
        #[linkage="external"]
        static [DECL_LINK_ $name]: $crate::decls::RawDecl = $crate::decls::RawDecl {
            nonce: $crate::decls::DECL_NONCE,
            decl: &[DECL_ $name],
        };
    })
}

#[macro_export]
macro_rules! make_cmdline_decl {
    ($option:expr, $function: expr, $name:ident) => {make_decl!(
        $crate::decls::Type::CMDLine($crate::decls::CMDLine{option:$option, f: $function}), $name
    );}
}
