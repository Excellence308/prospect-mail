use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=data/public_suffix_list.dat");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    let profile = out.ancestors().nth(3).expect("Cargo profile directory");
    fs::copy(
        "data/public_suffix_list.dat",
        profile.join("public_suffix_list.dat"),
    )
    .expect("Copy bundled public suffix list next to helper");
}
