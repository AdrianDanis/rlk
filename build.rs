extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/boot/head_32.S")
        .compile("head_asm");
}
