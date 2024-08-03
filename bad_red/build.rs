use itertools::Itertools;
use std::{env, fs, path::Path};

fn main() {
    println!("cargo::rerun-if-changed=bad_red_lib/");

    let out_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("consts_defs.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lua_lib_dir = Path::new(&manifest_dir).join("lua_lib");

    let files_iter = fs::read_dir(lua_lib_dir).unwrap();
    let full_content = files_iter
        .map(|file| fs::read_to_string(file.unwrap().path()).unwrap())
        .join("\n");

    let rust_content_string = format!("r#\"{}\"#", full_content);
    let preload = format!("pub const PRELOAD: &str = {};", rust_content_string);

    let defaults_dir = Path::new(&manifest_dir).join("default_files");
    let default_files_iter = fs::read_dir(defaults_dir).unwrap();
    let defaults_defs = default_files_iter
        .map(|entry| {
            let entry = entry.unwrap().path();
            let entry_contents = fs::read_to_string(entry.clone()).unwrap();

            format!(
                "pub const {}: &str = r#\"{}\"#;",
                entry
                    .file_stem()
                    .unwrap()
                    .to_ascii_uppercase()
                    .to_str()
                    .unwrap(),
                entry_contents
            )
        })
        .join("\n");

    let rust_mod = format!(
        r#"
mod generated {{
    {}
    {}
}}"#,
        preload, defaults_defs
    );
    fs::write(out_path, rust_mod).unwrap();
}
