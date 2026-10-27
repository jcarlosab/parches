fn main() {
    println!("cargo:rustc-cdylib-link-arg={}/exports.def", std::env::var("CARGO_MANIFEST_DIR").unwrap());
    // Smackw32.dll cannot relocate from 0x10000000, the default GNU DLL base.
    println!("cargo:rustc-cdylib-link-arg=-Wl,--image-base,0x61000000");
    println!("cargo:rerun-if-changed=exports.def");
}
