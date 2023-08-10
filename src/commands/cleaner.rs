use crate::config::AppConfig;
use crate::constants::*;
use crate::{Context, Error};
use crate::{TimerCommand, SETTINGS};
use ::config::{Config, File};
use chrono::NaiveTime;
use poise::futures_util::StreamExt;
use poise::serenity_prelude as serenity;
use std::str::FromStr;
use std::sync::Arc;
use toml_edit::{value, Document};

async fn is_private(ctx: Context<'_>) -> Result<bool, Error> {
    let config: AppConfig = SETTINGS
        .read()
        .await
        .clone()
        .try_deserialize()
        .expect("Unable to load configuration");
    let res = ctx.channel_id() == serenity::ChannelId(config.channels.private);
    if !res {
        ctx.say("This channel is not a private channel.").await?;
    }
    Ok(res)
}

#[poise::command(
    category = "Admin Commands",
    guild_only,
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES",
    owners_only
)]
pub async fn set_clean(
    ctx: Context<'_>,
    #[description = "Time to be deleted"] time: String,
) -> Result<(), Error> {
    if let Ok(ntime) = NaiveTime::from_str(&time) {
        let toml = std::fs::read_to_string(CONFIG_FILE).unwrap();
        let mut doc = toml.parse::<Document>().unwrap();
        doc["reset_time"] = value(ntime.to_string());
        std::fs::write(CONFIG_FILE, doc.to_string().as_bytes()).unwrap();
        SETTINGS.write().await.refresh().unwrap();
        ctx.data().timer_tx.send(TimerCommand::Reset).unwrap();
        ctx.say(format!("Okay. Set clean time to {}.", time))
            .await?;
    } else {
        ctx.say("Invalid time format. (ex. hh:mm:ss)").await?;
    }
    Ok(())
}

fn clean_now_help() -> String {
    String::from("Clean the private chat")
}

#[poise::command(
    category = "Main Commands",
    guild_only,
    slash_command,
    prefix_command,
    check = "is_private",
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | MANAGE_MESSAGES",
    help_text_fn = "clean_now_help"
)]
pub async fn clean_now(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Cleaning...").await?;
    clean(Arc::clone(&ctx.serenity_context().http), ctx.channel_id()).await?;
    Ok(())
}

pub async fn clean(http: Arc<serenity::Http>, channel: serenity::ChannelId) -> Result<(), Error> {
    let mut msg_ids = vec![];
    let mut messages = channel.messages_iter(Arc::clone(&http)).boxed();
    while let Some(message_result) = messages.next().await {
        match message_result {
            Ok(message) => msg_ids.push(message.id),
            Err(_) => break,
        }
    }
    channel.delete_messages(Arc::clone(&http), msg_ids).await?;
    channel
        .say(Arc::clone(&http), format!("🗑️ Cleaned!"))
        .await?;
    Ok(())
}
