use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::prelude::{ChannelId, Ready};
use serenity::prelude::*;
use tokio::sync::mpsc::{Receiver, error::TryRecvError};

struct Handler {
    loop_running: AtomicBool,
    loop_handler: LoopHandler,
    to_send_recv: Arc<Mutex<Receiver<String>>>,
}

#[derive(Clone)]
struct LoopHandler {
    channel_id: Arc<ChannelId>,
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        
        if !self.loop_running.load(Ordering::Relaxed) {
            let handler = self.loop_handler.clone();
            let channel = Arc::clone(&handler.channel_id);
            let to_send_recv = Arc::clone(&self.to_send_recv);
            tokio::spawn(async move {
                loop {
                    let result = to_send_recv.lock().await.try_recv();
                    match result {
                        Ok(message) => {
                            send_message(&ctx, *channel, &message).await;
                        },
                        Err(TryRecvError::Disconnected) => {
                            let data = ctx.data.read().await;
                            let shard_manager = match data.get::<ShardManagerContainer>() {
                                Some(v) => v,
                                None => {
                                    panic!("couldnt get shard manager")
                                },
                            };
                            let mut manager = shard_manager.lock().await;
                            manager.shutdown_all().await;
                            panic!("channel closed");
                        },
                        _ => {}
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            self.loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

pub async fn main(token: String, send_channel_id: u64, to_send_recv: Receiver<String>) {
    let channel_id = Arc::new(ChannelId::from(send_channel_id));
    let to_send_recv = Arc::new(Mutex::new(to_send_recv));
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            loop_running: AtomicBool::new(false),
            loop_handler: LoopHandler { channel_id: channel_id },
            to_send_recv,
        })
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}

async fn send_message(ctx: &Context, channel_id: ChannelId, message: &str) {
    if let Err(why) = channel_id.say(&ctx.http, message).await {
        eprintln!("Error sending message: {:?}", why);
    }
}