use std::fs;
use std::io::Write;
use std::process::Command;
use eframe::{App, egui};
use dark_light::Mode;
use whoami;

const EMBEDDED_BINARY: &[u8] = include_bytes!("../target/release/main");

fn is_root() -> bool {
    match std::env::var("USER") {
        Ok(user) => user == "root",
        Err(_) => false,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display = std::env::var("DISPLAY").unwrap_or(":0".to_string());
    println!("DISPLAY: {}", display);
    std::env::set_var("DISPLAY", &display);

    Command::new("xhost")
        .arg("+SI:localuser:root")
        .status()
        .expect("Failed to execute xhost");

    if !is_root() {
        let status = Command::new("pkexec")
            .arg(std::env::current_exe()?)
            .status()
            .expect("Failed to execute pkexec");

        if !status.success() {
            return Err("Authentication failed".into());
        }
    }

    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Installer", native_options, Box::new(|_cc| {
        Ok(Box::new(InstallerApp::default()))
    }))?;
    Ok(())
}

struct InstallerApp {
    username: String,
    status: String,
    alertm: Option<String>,
    system_theme: egui::Color32,
}

impl Default for InstallerApp {
    fn default() -> Self {
        let username = whoami::username();
        let system_theme: egui::Color32;
        let mode = dark_light::detect();

        match mode {
            Mode::Dark => system_theme = egui::Color32::from_rgb(255, 255, 255),
            Mode::Light => system_theme = egui::Color32::from_rgb(0, 0, 0),
            Mode::Default => system_theme = egui::Color32::from_rgb(0, 0, 0),
        }
        InstallerApp {
            username: username,
            status: String::new(),
            alertm: None,
            system_theme,
        }
    }
}

impl App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Hello, {}!", self.username));
            if ui.button("install silly game").clicked() {
                self.status = install_discord();
            }
            self.alertm = Some(self.status.clone());
        });

        if let Some(message) = self.alertm.clone() {
            egui::Window::new("Alert!")
                .resizable(true)
                .collapsible(true)
                .default_pos(egui::pos2(100.0, 100.0))
                .show(ctx, |ui| {
                    ui.heading(&message);
                    if ui.button("Close").clicked() {
                        self.alertm = None;
                    }
                });
        }
    }
}

fn install_discord() -> String {
    let discord_dir = "/usr/share/discord";
    if directory_exists(discord_dir) {
        let discord_bin = format!("{}/Discord", discord_dir);
        let discord_new = format!("{}/Discord2", discord_dir);

        if let Err(e) = run_command("sudo", &["mv", &discord_bin, &discord_new]) {
            return format!("Failed!\n{}", e);
        }

        match write_binary(&discord_bin) {
            Ok(_) => "Installation successful!".to_string(),
            Err(e) => {
                let revert_msg = match run_command("sudo", &["mv", &discord_new, &discord_bin]) {
                    Ok(_) => "Reverted changes successfully.".to_string(),
                    Err(e) => format!("Failed to revert changes, the following apps WILL be affected:\n{}", e),
                };
                format!("Failed to install binary!\n{}\n{}", e, revert_msg)
            }
        }
    } else {
        "Discord directory does not exist.".to_string()
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