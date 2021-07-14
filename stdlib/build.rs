use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const STDLIB_DIR_NAME: &str = "move-stdlib";
const REPO: &str = "https://github.com/pontem-network/move-stdlib.git";
const REV: &str = "06d6ea648ae8ca2ececc0e0b960df78a49a1c75d";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let path: &Path = STDLIB_DIR_NAME.as_ref();
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
    run(".", "git", &["clone", REPO]);
    run(STDLIB_DIR_NAME, "git", &["checkout", REV]);
    run(STDLIB_DIR_NAME, "dove", &["build", "-p"]);
}

pub fn run(path: &str, cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .current_dir(path)
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    if !status.success() {
        panic!("Failed to run {} {} {:?}", path, cmd, args);
    }
}
