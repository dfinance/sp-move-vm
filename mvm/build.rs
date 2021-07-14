use std::process::{Command, Stdio};

fn main() {
    #[cfg(feature = "assets")]
    {
        run("tests/assets", "sh", &["./build_assets.sh"]);
    }
}

pub fn run(path: &str, cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .current_dir(path)
        .args(args)
        .stdout(Stdio::piped())
        .status()
        .unwrap();
    if !status.success() {
        panic!("Failed to run {} {} {:?}", path, cmd, args);
    }
}
