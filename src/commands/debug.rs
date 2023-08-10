use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use std::sync::Arc;

fn get_me_help() -> String {
    String::from("Get information from user")
}

#[poise::command(
    category = "Debug Commands",
    guild_only,
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES",
    help_text_fn = "get_me_help"
)]
pub async fn get_me(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(
        "`guild id: {:?}\nchannel id: {}\nuser id: {}\nuser roles: {:?}`",
        ctx.guild_id(),
        ctx.channel_id(),
        ctx.author().id,
        ctx.author_member()
            .await
            .unwrap()
            .roles(Arc::clone(&ctx.serenity_context().cache))
    ))
    .await?;
    Ok(())
}
