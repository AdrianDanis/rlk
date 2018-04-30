use core;

pub struct CMDLine {
    pub option: &'static str,
    pub f: fn(&str) -> (),
}

pub enum Type {
    CMDLine(CMDLine),
    OtherCase,
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
    ($decltype:ident) => ($crate::link_decls::iter().filter_map(|x| if let $crate::link_decls::Type::$decltype(y) = x { Some(y) } else { None}))
}
