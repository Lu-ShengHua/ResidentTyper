use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn find_resource_compiler() -> Option<PathBuf> {
    if let Ok(explicit) = env::var("RC_EXE") {
        let path = PathBuf::from(explicit);
        if path.exists() {
            return Some(path);
        }
    }

    let program_files = env::var("ProgramFiles(x86)").ok()?;
    let kits_bin = PathBuf::from(program_files)
        .join("Windows Kits")
        .join("10")
        .join("bin");
    let mut versions: Vec<PathBuf> = fs::read_dir(kits_bin)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    versions.sort();
    versions.reverse();
    versions
        .into_iter()
        .map(|version| version.join("x64").join("rc.exe"))
        .find(|path| path.exists())
}

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app_logo.ico");

    let Some(rc) = find_resource_compiler() else {
        panic!("Windows resource compiler rc.exe was not found");
    };
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is missing")).join("app.res");
    let status = Command::new(rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&output)
        .arg("assets/app.rc")
        .status()
        .expect("failed to run rc.exe");
    if !status.success() {
        panic!("rc.exe failed with status {status}");
    }
    println!("cargo:rustc-link-arg={}", output.display());
}
