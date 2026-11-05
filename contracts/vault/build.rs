//! Compile the `counter` example to wasm so the vault can embed and deploy it
//! (`include_bytes!(concat!(env!("OUT_DIR"), "/counter.wasm"))`).
//!
//! A nested cargo build with its own target directory avoids the lock on the
//! outer one; the rustc flags of the outer build (e.g. the ones set by
//! `cargo massa build`) are inherited through `CARGO_ENCODED_RUSTFLAGS`.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let counter_dir = manifest_dir.join("../counter");
    println!("cargo:rerun-if-changed={}", counter_dir.join("src").display());
    println!("cargo:rerun-if-changed={}", counter_dir.join("Cargo.toml").display());

    let target_dir = out_dir.join("child-target");
    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["build", "--release", "--target", "wasm32v1-none", "--target-dir"])
        .arg(&target_dir)
        .arg("--manifest-path")
        .arg(counter_dir.join("Cargo.toml"))
        // Not a contract build of our own: drop the variables cargo sets for
        // build scripts that would confuse the nested invocation.
        .env_remove("RUSTC_WRAPPER")
        .env_remove("CARGO_TARGET_DIR")
        .status();

    let built = target_dir.join("wasm32v1-none/release/counter.wasm");
    let dest = out_dir.join("counter.wasm");
    match status {
        Ok(s) if s.success() && built.exists() => {
            let mut bytes = std::fs::read(&built).expect("read counter.wasm");
            // Optional post-optimization, like `cargo massa build` does.
            let opt = out_dir.join("counter.opt.wasm");
            let optimized = Command::new("wasm-opt")
                .args([
                    "-Os",
                    "--mvp-features",
                    "--enable-mutable-globals",
                    "--enable-sign-ext",
                    "--enable-nontrapping-float-to-int",
                    "--enable-bulk-memory",
                    "--strip-debug",
                    "--strip-producers",
                    "--strip-target-features",
                ])
                .arg(&built)
                .arg("-o")
                .arg(&opt)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if optimized {
                bytes = std::fs::read(&opt).expect("read optimized counter.wasm");
            }
            std::fs::write(&dest, bytes).expect("write counter.wasm");
        }
        _ => {
            if env::var("TARGET").map(|t| t.starts_with("wasm32")).unwrap_or(false) {
                panic!("failed to build the embedded `counter` contract (is the wasm32v1-none target installed?)");
            }
            // Native build (cargo test): an empty placeholder is enough.
            println!("cargo:warning=embedded counter.wasm not built (wasm32v1-none target missing?); using an empty placeholder");
            std::fs::write(&dest, []).expect("write placeholder");
        }
    }
}
