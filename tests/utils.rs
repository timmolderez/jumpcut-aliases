use assert_cmd::prelude::*;
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use std::env;
use std::io;
use std::panic;
use std::process::Output;

/// Return a Jumpcut Command process
pub fn jc_cmd() -> Command {
    return Command::cargo_bin("jumpcut").unwrap();
}

/// Runs a Jumpcut test with setup/teardown phases
/// 
/// Based on: https://medium.com/@ericdreichert/test-setup-and-teardown-in-rust-without-a-framework-ba32d97aa5ab
pub fn run_test<T>(test: T) -> () where T: FnOnce() -> () + panic::UnwindSafe {
    // Setup
    remove_all_aliases().unwrap();
    
    // Test body
    let result = panic::catch_unwind(|| {
        test()
    });

    // Teardown
    assert!(result.is_ok())
}

/// Copy of alias_path() in src/utils.rs
fn alias_path() -> PathBuf {
    let pwd = env::current_dir().unwrap_or_default();
    return pwd.join(".jumpcut_test");
}

/// Does a given alias exist?
pub fn alias_exists(al: &str) -> bool {
    let path = alias_path().join(al);
    return path.exists();
}

/// Removes everything from the test alias directory
fn remove_all_aliases() -> io::Result<()> {
    for entry in fs::read_dir(alias_path())? {
        let entry = entry?.path();
        fs::remove_file(entry)?;
    }
    return Ok(());
}

/// Returns the stdout of a process as a String
pub fn out_to_str(out: io::Result<Output>) -> String {
    return String::from_utf8_lossy(&out.unwrap().stdout).to_string();
}