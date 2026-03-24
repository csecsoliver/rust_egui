use std::{ fmt::format, sync::{OnceLock, mpsc::{self, Receiver, Sender}}, thread, time::{self, SystemTime}};

use eframe::egui::{self, Button, Image, ImageSource, Label, Ui, include_image};

static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

fn get_client() -> reqwest::blocking::Client {
    CLIENT.get_or_init(||reqwest::blocking::Client::builder().build().unwrap()).clone()
}

struct ClickerApp {
    url: String,
    new_url_text: String,
    upgrades: u32,
    cats: u32,
    local_clicks: u64,
    global_clicks: String,
    last_update: SystemTime,
    last_cat_trigger: SystemTime,
    rx: Receiver<String>,
    tx: Sender<String>,
    
}

impl ClickerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            url: ("https://hip.temp.olio.ovh/").to_string(),
            new_url_text: String::new(),
            upgrades: 0,
            cats: 0,
            local_clicks: 0,
            global_clicks: ("Loading...").to_string(),
            last_update: time::SystemTime::now(),
            last_cat_trigger: SystemTime::now(),
            rx,
            tx,
        }
    }
    fn click(&mut self) {
        let mut mult = 1.0;
        for n in 0..self.upgrades {
            mult += math::round::ceil((n as f64 + 1.0) / 2.0, 0);
        }
        let url = self.url.clone();
        let tx = self.tx.clone();
        self.local_clicks += mult as u64;
        thread::spawn(move|| {
            
            let client = get_client();
            let mut response = ("").to_string();
            for n in 0..mult as u64 {
                let resp_res = client.post(format!("{url}click")).send();
                response = match (match resp_res {
                    Ok(resp) => resp.text(),
                    Err(e) => {
                        println!("Error sending reqwest No.{n}: {e}");
                        return;
                    }
                }){
                    Ok(t) => t,
                    Err(e) => format!("Error idk: {e}")
                };
            }
            println!("sent {mult} clicks");
            _ = tx.send(response);
        });
    }
    fn update_counter(&mut self) {
        self.last_update = SystemTime::now();
        let url = self.url.clone();
        let tx = self.tx.clone();
        thread::spawn(move || {
            let client = get_client();
            let clicks = match (match client.post(format!("{url}clicks")).send() {
                Ok(r) => r.text(),
                Err(e) => Err(e)
            }) {
                Ok(t) => t,
                Err(e) => format!("Unable to load clicks: {e}")
            };
            _ = tx.send(clicks);
        });
        
    }
}

impl eframe::App for ClickerApp {
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.last_update.elapsed().unwrap().as_secs() >= 1 {
                self.update_counter();
                ui.ctx().request_repaint_after_secs(0.3);
                
            }
            if self.last_cat_trigger.elapsed().unwrap().as_secs() >= 1 {
                for _ in 0..self.cats {
                    self.click();
                }
                self.last_cat_trigger = SystemTime::now();
            }
            
            
            
            ui.heading("MMO clicker lol");
            ui.add_space(10.0);
            
            let gc = match self.rx.try_recv(){
                Ok(r) => r,
                Err(_) => self.global_clicks.clone()
            };
            ui.ctx().request_repaint_after_secs(0.5);
            self.global_clicks = gc.clone();
            ui.label(format!("Global clicks: {gc}"));
            ui.add_space(5.0);
            let lc = self.local_clicks;
            ui.label(format!("Local clicks: {lc}"));
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            if ui.button("Click").clicked()
            {
                self.click();
                self.update_counter();
            }
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            ui.horizontal(|ui|{
                ui.label(format!("You own {} upgrades", self.upgrades));
                ui.add_space(3.0);
                let upcost = 50 * (2 as u32).pow(self.upgrades);
                if ui.button(format!("Buy upgrade for {upcost} coins")).clicked() && upcost as u64 <= self.local_clicks {
                    self.local_clicks -= upcost as u64;
                    self.upgrades += 1;
                }
            });
            ui.horizontal(|ui| {
                ui.label(format!("You own {} cats", self.cats));
                ui.add_space(3.0);
                let catcost = 10 * (3 as u32).pow(self.cats);
                if ui.button(format!("Buy cat for {catcost} coins")).clicked() && catcost as u64 <= self.local_clicks {
                    self.local_clicks -= catcost as u64;
                    self.cats += 1;
                }
            });
        });
    }
}
fn main() -> eframe::Result {
    println!("Hello, world!");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MMO Clicker",
        native_options,
        Box::new(|cc| Ok(Box::new(ClickerApp::new(cc)))),
    )
}
