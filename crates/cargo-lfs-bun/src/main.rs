use std::{env, path::PathBuf, process::{Command, ExitCode}};

fn main() -> ExitCode {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let entry = root.join("src/cli.ts");
    let bun = env::var_os("YOLO_OS_BUN").unwrap_or_else(|| "bun".into());
    let mut command = Command::new(bun);
    command.arg("run").arg(entry).args(env::args_os().skip(1)).current_dir(root);
    match command.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1).clamp(0, 255) as u8),
        Err(error) => {
            eprintln!("cargo-lfs-bun: could not start Bun: {error}; install Bun or set YOLO_OS_BUN");
            ExitCode::FAILURE
        }
    }
}
