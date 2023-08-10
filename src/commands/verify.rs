use crate::config::AppConfig;
use crate::constants::*;
use crate::{Context, Error, SETTINGS};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CacheHttp;
use std::sync::Arc;

fn add_verified_help() -> String {
    String::from("Add verified role to user")
}

#[poise::command(
    category = "Admin Commands",
    guild_only,
    slash_command,
    prefix_command,
    help_text_fn = "add_verified_help"
)]
pub async fn add_verified(
    ctx: Context<'_>,
    #[description = "Selected user"] user: serenity::User,
) -> Result<(), Error> {
    let config: AppConfig = SETTINGS
        .read()
        .await
        .clone()
        .try_deserialize()
        .expect("Unable to load configuration");
    let guild = serenity::GuildId(config.main_guild);
    if let Ok(mut mem) = guild
        .member(Arc::clone(&ctx.serenity_context().http), user.id)
        .await
    {
        mem.add_role(
            Arc::clone(&ctx.serenity_context().http),
            serenity::RoleId(config.verified_role),
        )
        .await?;
        ctx.say("Added verified role.").await?;
    } else {
        ctx.say("Unable to find user.").await?;
    }
    Ok(())
}
