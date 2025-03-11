use crate::{Context, Error};
use poise::serenity_prelude::{model, CreateEmbed};
use poise::CreateReply;

async fn is_owner(ctx: &Context<'_>) -> bool {
    let owner_id: model::id::UserId = 1065814795883921518.into();
    ctx.author().id == owner_id
}

/// If you are the owner exit the bot
#[poise::command(slash_command)]
pub async fn exit(ctx: Context<'_>) -> Result<(), Error> {
    let embed;
    if is_owner(&ctx).await {
        embed = CreateEmbed::default().title("Exit!!").description("~~");

        ctx.send(CreateReply::default().embed(embed)).await?;

        std::process::exit(0);
    } else {
        embed = CreateEmbed::default()
            .title("Error!!")
            .description("You are not the owner");

        ctx.send(CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}
