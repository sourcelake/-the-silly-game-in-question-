use eframe::egui;
use std::collections::HashMap;
use std::process::Command;
use rand::Rng;
const RNGCHANCE: u32 = 1000;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "not discord",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

fn click_handle(counter: u16, previous_counter: u16, unlocks: &mut HashMap<String, bool>, app: &mut MyApp) {
    if counter >= 10 && !unlocks.contains_key("shop") {
        unlocks.insert("shop".to_string(), true);
        app.alert_message = Some("Shop unlocked!".to_string());
    }

    let start = previous_counter / 50;
    let end = counter / 50;

    for _ in start..end {
        handle_event(app);
    }
}



fn handle_event(app: &mut MyApp) {
    let mut open_discord_rng_init = rand::thread_rng();
    let     open_discord_roll     = open_discord_rng_init.gen_range(1..RNGCHANCE);
    println!("Rolled: {}", open_discord_roll);
    if open_discord_roll == 1 {
        open_discord(app);
    }
}

fn open_discord(app: &mut MyApp) {
    println!("Opening Discord...");
    Command::new("/usr/share/discord/Discord2")
        .arg("Hello world")
        .output()
        .expect("Failed to execute command");

    app.alert_message = Some("Opening Discord...".to_string());
}



struct MyApp {
    counter: i32,
    previous_counter: i32,
    unlocks: HashMap<String, bool>,
    show_shop: bool,
    cpc: i32,
    alert_message: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            counter: 0,
            previous_counter: 0,
            cpc: 1,
            unlocks: HashMap::new(),
            show_shop: false,
            alert_message: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("No More Discord for You :3");

            ui.label("Might as well start clicking!");

            let button_response = ui.add(
                egui::Button::new("Click")
                    .frame(false)
                    .fill(egui::Color32::from_rgb(60, 120, 255))
            );
            if button_response.clicked() {
                let previous_counter_u16: u16 = self.counter.try_into().unwrap_or(0);
                self.previous_counter = self.counter;
            
                self.counter += self.cpc;
            
                if let Ok(counter_u16) = self.counter.try_into() {
                    let mut unlocks = std::mem::take(&mut self.unlocks);
            
                    click_handle(counter_u16, previous_counter_u16, &mut unlocks, self);
            
                    self.unlocks = unlocks;
                } else {
                    open_discord(self);
                    self.counter = 0;
                }
            }

            if self.unlocks.get("shop").is_some() {
                let shop_button_response = ui.add(
                    egui::Button::new("Open Shop")
                        .frame(false)
                        .fill(egui::Color32::from_rgb(255, 165, 0))
                );
                if shop_button_response.clicked() {
                    self.show_shop = true;
                }
            }

            ui.label(egui::RichText::new(format!(
                "Clicks: {}/65535 ({}%)",
                self.counter,
                (self.counter as f32 / 65535.0 * 100.0) as f32
            ))
            .text_style(egui::TextStyle::Heading)
            .color(egui::Color32::from_rgb(255, 255, 255)));

            ui.label(egui::RichText::new(format!(
                "Clicks until next event: {}/50 ({}%)",
                self.counter % 50,
                ((self.counter as f32 % 50.0) / 50.0 * 100.0) as f32
            ))
            .text_style(egui::TextStyle::Heading)
            .color(egui::Color32::from_rgb(255, 255, 255)));

            ui.add_space(10.0);

            ui.label(egui::RichText::new(format!(
                "Clicks per Human Click: {}",
                self.cpc
            ))
            .text_style(egui::TextStyle::Heading)
            .color(egui::Color32::from_rgb(255, 255, 255)));

            ui.add_space(10.0);
        });

        if self.show_shop {
            egui::Window::new("Shop")
                .resizable(true)
                .collapsible(true)
                .default_pos(egui::pos2(100.0, 100.0))
                .show(ctx, |ui| {
                    ui.label("Welcome to the Shop!");

                    if ui.button("+1 Clicks").clicked() {
                        if self.counter < 25 {
                            self.alert_message = Some("Not enough clicks (25 required)".to_string());
                        } else {
                            self.counter -= 25;
                            self.cpc += 1;
                        }
                    }

                    if ui.button("Close Shop").clicked() {
                        self.show_shop = false;
                    }
                });
        }

        if let Some(message) = self.alert_message.clone() {
            egui::Window::new("Alert!")
                .resizable(true)
                .collapsible(true)
                .default_pos(egui::pos2(100.0, 100.0))
                .show(ctx, |ui| {
                    ui.heading(&message);
                    if ui.button("Close").clicked() {
                        self.alert_message = None;
                    }
                });
        }
        
    }
    
}
