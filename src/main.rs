#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod commands;
mod config;
mod constants;
mod new_user;

use crate::config::*;
use ::config::{Config, File};
use chrono::{Local, NaiveTime, Timelike};
use commands::{cleaner, debug, verify};
use constants::*;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::LogPacker;
use poise::futures_util::StreamExt;
use poise::{async_trait, serenity_prelude as serenity};
use std::future::Future;
use std::io::prelude::*;
use std::path::Path;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedSender as Sender};
use tokio::sync::RwLock;
use tokio::time;

#[derive(Clone, Copy)]
pub enum TimerCommand {
    Reset,
}

pub struct Data {
    timer_tx: Sender<TimerCommand>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let admins: Vec<u64> = vec![];
        let mut settings = Config::default();
        settings
            .set_default("discord_token", "YOUR_TOKEN_HERE")
            .unwrap();
        settings.set_default("reset_time", "06:00:00").unwrap();
        settings.set_default("main_guild", 0).unwrap();
        settings.set_default("admins", admins).unwrap();
        settings.set_default("channels.console", 0).unwrap();
        settings.set_default("channels.log", 0).unwrap();
        settings.set_default("channels.private", 0).unwrap();
        settings.merge(File::with_name(CONFIG_FILE));
        settings
    });
}

struct Handler {}

#[async_trait]
impl serenity::EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: serenity::Context, new_member: serenity::Member) {
        let config: AppConfig = SETTINGS
            .read()
            .await
            .clone()
            .try_deserialize()
            .expect("Unable to load configuration");
        let console = serenity::ChannelId(config.channels.console);
        let guild = serenity::GuildId(config.main_guild);
        let deny_uuid = uuid::Uuid::new_v4();
        let allow_uuid = uuid::Uuid::new_v4();
        console
            .send_message(Arc::clone(&ctx.http), |m| {
                m.content(format!(
                    "{} joined to the server.",
                    new_member.display_name()
                ))
                .components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_button(|b| {
                            b.style(serenity::ButtonStyle::Success)
                                .label("Allow")
                                .custom_id(allow_uuid)
                        });
                        ar.create_button(|b| {
                            b.style(serenity::ButtonStyle::Danger)
                                .label("Deny")
                                .custom_id(deny_uuid)
                        })
                    })
                })
            })
            .await
            .unwrap();
        tokio::spawn(async move {
            while let Some(mci) = serenity::CollectComponentInteraction::new(ctx.clone())
                .channel_id(console)
                .filter(move |mci| {
                    mci.data.custom_id == allow_uuid.to_string()
                        || mci.data.custom_id == deny_uuid.to_string()
                })
                .await
            {
                if mci.data.custom_id == allow_uuid.to_string() {
                    if let Ok(mut mem) = guild
                        .member(Arc::clone(&ctx.http), new_member.clone())
                        .await
                    {
                        mem.add_role(
                            Arc::clone(&ctx.http),
                            serenity::RoleId(config.verified_role),
                        )
                        .await
                        .unwrap();
                        let mut msg = mci.message.clone();
                        msg.edit(ctx.clone(), |m| {
                            m.content(format!(
                                "Added verified role to {}.",
                                new_member.display_name()
                            ))
                        })
                        .await
                        .unwrap();
                        mci.create_interaction_response(ctx, |ir| {
                            ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
                        })
                        .await
                        .unwrap();
                    } else {
                        let mut msg = mci.message.clone();
                        msg.edit(ctx.clone(), |m| {
                            m.content(format!(
                                "Unable to find user {}.",
                                new_member.display_name()
                            ))
                        })
                        .await
                        .unwrap();
                        mci.create_interaction_response(ctx, |ir| {
                            ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
                        })
                        .await
                        .unwrap();
                    }
                    break;
                } else if mci.data.custom_id == deny_uuid.to_string() {
                    guild
                        .kick(Arc::clone(&ctx.http), new_member.clone())
                        .await
                        .unwrap();
                    let mut msg = mci.message.clone();
                    msg.edit(ctx.clone(), |m| {
                        m.content(format!("Kicked user {}.", new_member.display_name()))
                    })
                    .await
                    .unwrap();
                    mci.create_interaction_response(ctx, |ir| {
                        ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await
                    .unwrap();
                    break;
                }
            }
        });
    }
}

#[poise::command(category = "Main Commands", slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    info!("{} issued command ping()", ctx.author().id.0);
    ctx.say(format!(
        "Pong! (channel id: {}, user id: {})",
        ctx.channel_id(),
        ctx.author().id
    ))
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    fast_log::init(
        fast_log::Config::new()
            .level(log::LevelFilter::Warn)
            .file_split(LOG_FILE_PATH, LOG_FILE_SIZE, RollingType::All, LogPacker {}),
    )
    .unwrap();
    let config: AppConfig = SETTINGS
        .read()
        .await
        .clone()
        .try_deserialize()
        .expect("Unable to load configuration");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, _, _| {
                Box::pin(async move {
                    event.clone().dispatch(ctx.clone(), &Handler {}).await;
                    Ok(())
                })
            },
            commands: vec![
                ping(),
                cleaner::clean_now(),
                cleaner::set_clean(),
                debug::get_me(),
                verify::add_verified(),
            ],
            ..Default::default()
        })
        .token(config.discord_token)
        .intents(serenity::GatewayIntents::privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                ctx.set_activity(serenity::Activity::playing(format!("{} {} ({})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("COMMIT_HASH")))).await;
                let http = Arc::clone(&ctx.http);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let (tx, mut rx) = mpsc::unbounded_channel::<TimerCommand>();
                tokio::spawn(async move {
                    loop {
                        let new_config: AppConfig = SETTINGS
                            .read()
                            .await
                            .clone()
                            .try_deserialize()
                            .expect("Unable to load configuration");
                        let private = serenity::ChannelId(new_config.channels.private);
                        let mut is_cleaned = false;
                        loop {
                            let current_time = Local::now().time();
                            if current_time.hour() == new_config.reset_time.hour()
                                && current_time.minute() == new_config.reset_time.minute()
                            {
                                if is_cleaned {
                                    continue;
                                }
                                if let Err(e) = cleaner::clean(Arc::clone(&http), private).await {
                                    error!("Failed to delete messages: {:?}", e);
                                }
                                is_cleaned = true;
                            } else if is_cleaned {
                                break;
                            }
                            if let Ok(cmd) = rx.try_recv() {
                                match cmd {
                                    TimerCommand::Reset => {
                                        break;
                                    }
                                }
                            }
                            time::sleep(TIMER_INTERVAL).await;
                        }
                    }
                });
                Ok(Data { timer_tx: tx })
            })
        });
    framework.run().await.unwrap();
}
