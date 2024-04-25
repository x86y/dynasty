use bindgen::*;use std::env;use std::path::PathBuf;
fn main(){let lp=PathBuf::from("src/l").canonicalize().expect("canon"); let hp=lp.join("k.h");let hp=hp.to_str().expect("tostr");
    println!("cargo:rustc-link-search={}",lp.to_str().unwrap());
    println!("cargo:rustc-link-lib=k");
    println!("cargo:rerun-if-changed={}",hp);
    Builder::default().header(hp).parse_callbacks(Box::new(CargoCallbacks)).generate().expect("binds")
        .write_to_file( PathBuf::from(env::var("OUT_DIR").unwrap()).join("B.rs") ).expect("write");}
