use eframe::egui;
use std::collections::HashMap;
use std::process::Command;
use rand::Rng;
use std::time::{Instant, Duration};
use dark_light::Mode;

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
    Command::new(format!("/usr/share/discord/Discord2"))
        .output()
        .expect("Failed to execute command");

    app.alert_message = Some("Opening Discord...".to_string());
}


//* Smaller functions.

fn get_shop_price(val: &str, shop: HashMap<String, i32>) -> i32 {
    return *shop.get(&val.to_string()).unwrap()
}



struct MyApp {
    counter: i32,
    previous_counter: i32,
    unlocks: HashMap<String, bool>,
    shop_prices: HashMap<String, i32>,
    show_shop: bool,
    cpc: i32,
    alert_message: Option<String>,
    humanclicks:i32,
    total: i32,
    autoclickers: i32,
    prevt: Option<Instant>,
    system_theme: egui::Color32,
}

impl Default for MyApp {
    fn default() -> Self {
        let system_theme: egui::Color32;    // ! system_theme signifies the colour of text assoc. with the system theme.
                                            // ! eg. light theme = black text, dark theme = white text.
        let mode = dark_light::detect();
        match mode {
            Mode::Dark => system_theme = egui::Color32::from_rgb(255, 255, 255),
            Mode::Light => system_theme = egui::Color32::from_rgb(0, 0, 0),
            Mode::Default => system_theme = egui::Color32::from_rgb(0, 0, 0),
        }
        println!("Detected mode: {:?}", mode);

        let mut shop_prices = HashMap::new();
        shop_prices.insert("+cpc".to_string(), 25);
        shop_prices.insert("+auto".to_string(), 100);

        Self {
            
            unlocks: HashMap::new(),
            shop_prices: shop_prices,
            show_shop: false,

            // click related
            counter: 0,
            previous_counter: 0,
            cpc: 1,
            alert_message: None,
            humanclicks: 0,
            total: 0,
            autoclickers: 0,

            // GUI related
            system_theme: system_theme,

            // time related
            /*
                autoclickers are also part of time, everything to do with time is for autoclicking,
                as thread cant be used.
            */
            prevt: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("No More Discord for You :3");

            ui.label("Might as well start clicking!");

            let clicker = ui.add(
                egui::Button::new("Click")
                    .frame(false)
                    .fill(egui::Color32::from_rgb(60, 120, 255))
            );
            if clicker.clicked() {
                let previous_counter_u16: u16 = self.counter.try_into().unwrap_or(0);
                self.previous_counter = self.counter;
            
                self.counter += self.cpc;
                self.total += self.cpc;
                self.humanclicks += 1;
            
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

            //* Clicks until the target reached.
            //* if you reach the target, discord will always open.

            ui.label(egui::RichText::new(format!(
                "Clicks: {}/65535 ({}%)",
                self.counter,
                (self.counter as f32 / 65535.0 * 100.0) as f32
            ))
            .text_style(egui::TextStyle::Heading)
            .color(self.system_theme));

            //* [Next event]

            ui.label(egui::RichText::new(format!(
                "Clicks until next event: {}/50 ({}%)",
                self.counter % 50,
                ((self.counter as f32 % 50.0) / 50.0 * 100.0) as f32
            ))
            .text_style(egui::TextStyle::Heading)
            .color(self.system_theme));

            //* C/C, how many clicks you get per human click.

            ui.label(egui::RichText::new(format!(
                "Clicks per Human Click: {}",
                self.cpc
            ))
            .text_style(egui::TextStyle::Heading)
            .color(self.system_theme));

            //* Total human-performed clicks (excludes clicks gained from upgrades and shop items.)
            //* eg. having a cpc of 5 will only add one to self.humanclicks instead of five.

            ui.label(egui::RichText::new(format!(
                "Human Clicks (total): {}",
                self.humanclicks
            ))
            .text_style(egui::TextStyle::Heading)
            .color(self.system_theme));

            //* Total clicks, (includes autoclickers & click upgrades)

            ui.label(egui::RichText::new(format!(
                "Clicks (total): {}",
                self.total
            ))
            .text_style(egui::TextStyle::Heading)
            .color(self.system_theme));


            //* AC/s

            ui.label(egui::RichText::new(format!(
                "Autoclicks per second: {}",
                self.autoclickers
            ))
            .text_style(egui::TextStyle::Heading)
            .color(self.system_theme));

            ui.add_space(10.0);
        });

        if self.show_shop {
            egui::Window::new("Shop")
                .resizable(true)
                .collapsible(true)
                .default_pos(egui::pos2(100.0, 100.0))
                .show(ctx, |ui| {
                    ui.label("Welcome to the Shop!");

                    //* +1 Clicks, increase cpc by 1.
                    /*
                      ? Click Start Price: 25c
                      ? Click Price Equa.: C(1) (constant)
                    */

                    ui.horizontal(|ui| {
                        if ui.button("+1 Clicks per click").clicked() {
                            let click_price = get_shop_price("+cpc", self.shop_prices.clone());
                            if self.counter < click_price {
                                self.alert_message = Some(format!("Not enough clicks ({} required)", click_price).to_string());
                            } else {
                                self.counter -= click_price;
                                self.cpc += 1;
                            }
                        }
                        ui.label(format!("Price: {}", get_shop_price("+cpc", self.shop_prices.clone())));
                    });
                    //* +1 Autoclickers, increase autoclickers by 1.
                    /*
                      ? Click Start Price: 100c
                      ? Click Price Equa.: C(1.1) (python: x *= 1.1)
                    */
                    

                    ui.horizontal(|ui| {
                        if ui.button("+1 Autoclickers").clicked() {
                            let click_price = get_shop_price("+auto", self.shop_prices.clone());
                            if self.counter < click_price {
                                self.alert_message = Some(format!("Not enough clicks ({} required)", click_price).to_string());
                            } else {
                                self.counter -= click_price;
                                if let Some(price) = self.shop_prices.get_mut("+auto") {
                                    *price = (*price as f32 * 1.1) as i32;
                                }
                                self.autoclickers += 1;
                            }
                        }
                        ui.label(format!("Price: {}", get_shop_price("+auto", self.shop_prices.clone())));
                    });

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

        if self.autoclickers > 0 {
            if self.autoclickers == 1 && self.prevt.is_none() {
                self.prevt = Some(Instant::now());
                return;
            }

            
            if let Some(prevt) = self.prevt {
                if prevt.elapsed() >= Duration::from_secs(1) {
                    self.counter += self.autoclickers;
                    self.prevt = Some(Instant::now());
                }
            }
        }
        
    }
    
}
