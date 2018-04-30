fn foo() {
    print!(Info, "Foo");
}

#[repr(C)]
pub struct FooType {
    pub fn_ref: fn () -> (),
}

#[link_section=".test"]
#[used]
#[linkage="external"]
static FOO: FooType = FooType {
    fn_ref: foo,
};
