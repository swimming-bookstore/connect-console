use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/css");
    println!("cargo:rerun-if-changed=ui/Cargo.toml");
    println!("cargo:rerun-if-changed=package.json");
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let css = Command::new("npm")
        .arg("run")
        .arg("css")
        .current_dir(&manifest)
        .status()?;
    if !css.success() {
        return Err("ui css build failed (npm run css)".into());
    }
    let mut cmd = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(manifest.join("ui/Cargo.toml"))
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release")
        .env("CARGO_TARGET_DIR", manifest.join("ui/target"));
    for (k, _) in env::vars() {
        if k.starts_with("CARGO") && k != "CARGO" && k != "CARGO_HOME" && k != "CARGO_TERM_COLOR" {
            cmd.env_remove(&k);
        }
    }
    cmd.env_remove("RUSTC");
    cmd.env_remove("RUSTC_WRAPPER");
    cmd.env_remove("RUSTFLAGS");
    if !cmd.status()?.success() {
        return Err("ui wasm build failed".into());
    }
    let wasm = manifest.join("ui/target/wasm32-unknown-unknown/release/connect_console_ui.wasm");
    let out = PathBuf::from(env::var("OUT_DIR")?).join("webui");
    std::fs::create_dir_all(&out)?;
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(&wasm)
        .web(true)?
        .debug(false)
        .generate(&out)?;
    Ok(())
}
