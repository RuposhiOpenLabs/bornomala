use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let res_path = format!("{}/bornomala_res.o", out_dir);
        let rc_path = "src/win32/bornomala.rc";
        let status = Command::new("x86_64-w64-mingw32-windres")
            .args(&[rc_path, "-O", "coff", "-o", &res_path])
            .status();
        if let Ok(s) = status {
            if s.success() {
                println!("cargo:rustc-link-arg={}", res_path);
            }
        }
        println!("cargo:rerun-if-changed={}", rc_path);
        println!("cargo:rerun-if-changed=src/win32/assets/tray_en.ico");
        println!("cargo:rerun-if-changed=src/win32/assets/tray_uni.ico");
        println!("cargo:rerun-if-changed=src/win32/assets/tray_ansi.ico");
        println!("cargo:rerun-if-changed=src/win32/assets/tray_pho.ico");
        println!("cargo:rerun-if-changed=../../installer/bornomala.ico");
    }
}
