use std::{env::var, path::PathBuf};

use bindgen::{Builder, CargoCallbacks};

fn main() {
    let out_path = PathBuf::from(var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-lib=cgos");
    let bindings = Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(CargoCallbacks))
        .clang_args(vec!["-I."])
        .generate()
        .expect("failed to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
}
