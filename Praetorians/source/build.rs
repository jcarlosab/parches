use std::{env,path::PathBuf,process::Command};
fn main() {
    println!("cargo:rerun-if-changed=assets/praetorians.ico");
    println!("cargo:rerun-if-changed=assets/icon.res");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") { return; }
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("icon.o");
    let status = Command::new("windres").args(["-J", "res", "-i", "assets/icon.res", "-O", "coff", "-o"]).arg(&output).status().expect("windres no disponible");
    assert!(status.success(), "No se pudo integrar el icono");
    println!("cargo:rustc-link-arg={}",output.display());
}
