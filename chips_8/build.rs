use gl_generator::{Api, DebugStructGenerator, Fallbacks, Profile, Registry, StructGenerator};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=build.rs");

    let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();
    let registry = Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, []);

    if env::var("CARGO_FEATURE_GL_DEBUG").is_ok() {
        registry.write_bindings(DebugStructGenerator, &mut file).unwrap();
    } else {
        registry.write_bindings(StructGenerator, &mut file).unwrap();
    }
}
