use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::prelude::{ChannelId, Ready};
use serenity::prelude::*;
use tokio::sync::mpsc::{Receiver, error::TryRecvError};

#[derive(Debug)]
pub enum Packet {
    Send(String, usize),
    SendAndQuit(Vec<String>),
}

struct Handler {
    loop_running: AtomicBool,
    loop_handler: LoopHandler,
    to_send_recv: Arc<Mutex<Receiver<Packet>>>,
}

#[derive(Clone)]
struct LoopHandler {
    channel_ids: Arc<Vec<(u64, Option<u64>)>>,
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
            let channels = Arc::clone(&handler.channel_ids);
            let to_send_recv = Arc::clone(&self.to_send_recv);
            for (send_channel, recv_channel) in channels.iter() {
                if let Some(recv_channel) = recv_channel {
                    let recv_channel = ChannelId(*recv_channel);
                    let send_channel = ChannelId(*send_channel);
                    let mut messages = vec![];
                    for msg in recv_channel.messages(&ctx.http, |retriever| retriever).await.unwrap() {
                        for line in msg.content.split("\n") {
                            if line != "" {
                                messages.push(line.to_string());
                            }
                        }
                        msg.delete(&ctx.http).await.unwrap();
                    }
                    for chunk in messages.chunks(5) {
                        let chunk_string = chunk.join("\n");
                        send_message(&ctx, send_channel.clone(), &chunk_string).await;
                    }
                }
            }
            tokio::spawn(async move {
                loop {
                    let result = to_send_recv.lock().await.try_recv();
                    match result {
                        Ok(packet) => {
                            match packet {
                                Packet::Send(message, channel) => {
                                    send_message(&ctx, channels[channel].0.into(), &message).await;
                                }
                                Packet::SendAndQuit(messages) => {
                                    for (channel, message) in messages.iter().enumerate() {
                                        if message.len() != 0 {
                                            send_message(&ctx, channels[channel].0.into(), message).await;
                                        }
                                    }
                                    let data = ctx.data.read().await;
                                    let shard_manager = match data.get::<ShardManagerContainer>() {
                                        Some(v) => v,
                                        None => {
                                            panic!("couldnt get shard manager")
                                        },
                                    };
                                    let mut manager = shard_manager.lock().await;
                                    manager.shutdown_all().await;
                                    break;
                                }
                            }
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
                            break;
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

pub async fn main(token: String, send_channel_ids: Vec<(u64, Option<u64>)>, to_send_recv: Receiver<Packet>) {
    let channel_ids = Arc::new(send_channel_ids);
    let to_send_recv = Arc::new(Mutex::new(to_send_recv));
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            loop_running: AtomicBool::new(false),
            loop_handler: LoopHandler { channel_ids: channel_ids },
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