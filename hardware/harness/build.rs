use std::{env, fs, path::Path};

fn main() {
    // re-run if TEST_BIN changes
    println!("cargo:rerun-if-env-changed=TEST_BIN");

    let bin = env::var("TEST_BIN")
        .expect("set TEST_BIN to path/to/your/test/binary.bin");

    let out = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out).join("embedded_stub.rs");
    let src = format!(
        "/// Auto-generated stub\n\
         pub const TEST_STUB: &'static [u8] = include_bytes!(r\"{}\");",
        bin
    );
    fs::write(dest, src).unwrap();
}