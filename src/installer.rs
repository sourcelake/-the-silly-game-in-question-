use std::fs;
use std::io::Write;
use std::process::Command;

const EMBEDDED_BINARY: &[u8] = include_bytes!("../target/release/main");

fn main() {
    let discord_dir = "/usr/share/discord";

    if directory_exists(discord_dir) {
        let discord_bin = format!("{}/Discord", discord_dir);
        let discord_new = format!("{}/Discord2", discord_dir);

        if let Err(e) = run_command("sudo", &["mv", &discord_bin, &discord_new]) {
            eprintln!("Failed!\n{}",e);
            return;
        }

        if let Err(e) = write_binary(&discord_bin) {
            eprintln!("Failed to install binary!\n{}", e);
            println!("Reverting changes...");
            if let Err(_e) = run_command("sudo", &["mv", &discord_bin, &discord_new]) {
                println!("Failed to revert changes, the following apps WILL be affected:\n/usr/share/discord/");
                return;
            }
        }
    }
}

fn directory_exists(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

fn write_binary(output_path: &str) -> std::io::Result<()> {
    let mut child = Command::new("sudo")
        .arg("tee")
        .arg(output_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("Critical Error.");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(EMBEDDED_BINARY)?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to unpack binary!",
        ));
    }

    let chmod_status = Command::new("sudo")
        .arg("chmod")
        .arg("755")
        .arg(output_path)
        .status()?;

    if !chmod_status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to change file permissions! (cannot execute)",
        ));
    }

    Ok(())
}

fn run_command(command: &str, args: &[&str]) -> std::io::Result<()> {
    let status = Command::new(command)
        .args(args)
        .status()
        .expect("Failed to execute command");

    if !status.success() {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{:?} failed with status: {}", args, status),
        ))
    } else {
        Ok(())
    }
}
