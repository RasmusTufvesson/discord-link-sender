use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::model::gateway::{Ready};
use serenity::model::id::{GuildId};
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;

struct Handler {
    loop_running: AtomicBool,
    loop_handler: LoopHandler,
}

#[derive(Clone)]
struct LoopHandler {
    channel_id: Arc<ChannelId>,
    to_send_mutex: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built successfully!");

        if !self.loop_running.load(Ordering::Relaxed) {
            let handler = self.loop_handler.clone();
            let channel = Arc::clone(&handler.channel_id);
            tokio::spawn(async move {
                loop {
                    let mut to_send = (*handler.to_send_mutex).lock().await;
                    if to_send.len() != 0 {
                        let message = to_send.pop().unwrap();
                        send_message(&ctx, *channel, &message).await;
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            self.loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

async fn main(token: String, send_channel_id: u64, to_send_mutex: Arc<Mutex<Vec<String>>>) {
    let channel_id = Arc::new(ChannelId::from(send_channel_id));
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            loop_running: AtomicBool::new(false),
            loop_handler: LoopHandler { channel_id: channel_id, to_send_mutex: to_send_mutex }
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}

async fn send_message(ctx: &Context, channel_id: ChannelId, message: &str) {
    if let Err(why) = channel_id.say(&ctx.http, message).await {
        eprintln!("Error sending message: {:?}", why);
    }
}