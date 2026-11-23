use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=RADIAL_UIACCESS");
    println!("cargo:rerun-if-changed=manifest.tpl");
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.ico");
    let uia = env::var("RADIAL_UIACCESS").map(|v| v == "1").unwrap_or(false);
    let tpl = fs::read_to_string("manifest.tpl").unwrap();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("app.manifest"), tpl.replace("@UIA@", if uia { "true" } else { "false" })).unwrap();
    fs::copy("app.ico", out.join("app.ico")).unwrap();
    fs::copy("app.rc", out.join("app.rc")).unwrap();
    embed_resource::compile(out.join("app.rc"), embed_resource::NONE).manifest_required().unwrap();
}
