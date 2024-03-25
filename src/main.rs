use std::{fs, thread};

use tokio::runtime;
use toml;
use serde::Deserialize;

mod app;
mod bot;

#[derive(Deserialize)]
struct Config {
    token: String,
    channel_id: u64,
    invite_link: String,
    bot_name: String,
}

fn main() {
    let config = fs::read_to_string("config.toml")
        .expect("Should have been able to read the file");
    let config: Config = toml::from_str(&config).unwrap();
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let bot_thread = thread::spawn(move || {
        let threaded_rt = runtime::Runtime::new().unwrap();
        threaded_rt.block_on(bot::main(config.token, config.channel_id, rx));
    });
    match app::main(config.bot_name, tx) {
        Ok(_) => {}
        Err(why) => {
            eprintln!("Error in app: {}", why);
        }
    }
    bot_thread.join().unwrap();
}