use crate::{Context, Error};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use sha2::{Digest, Sha256};

/// Make sha-256 hash value
#[poise::command(slash_command)]
pub async fn sha256(
    ctx: Context<'_>,
    #[description = "Value to change"] input: String,
) -> Result<(), Error> {
    let mut hasher = Sha256::new();
    hasher.update(&input);
    let hash = hasher.finalize();

    let embed = CreateEmbed::default()
        .title("Generated!!")
        .description(format!("input: {}\nhash: {:x}", input, hash));

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Make argon2 hash value
#[poise::command(slash_command)]
pub async fn argon2(
    ctx: Context<'_>,
    #[description = "Value to change"] input: String,
) -> Result<(), Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(input.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let embed = CreateEmbed::default()
        .title("Generated!!")
        .description(format!("input: {}\nhash: {}", input, hash));

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Verify argon2 hash value
#[poise::command(slash_command)]
pub async fn verify(
    ctx: Context<'_>,
    #[description = "Verify input"] input: String,
    #[description = "Verify hash"] hash: String,
) -> Result<(), Error> {
    let parsed_hash = PasswordHash::new(&hash).unwrap();
    if Argon2::default()
        .verify_password(input.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let embed = CreateEmbed::default()
            .title("Same!!")
            .description(format!("input: {}\nhash: {}", input, hash));

        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = CreateEmbed::default()
            .title("Different!!")
            .description(format!("input: {}\nhash: {}", input, hash));

        ctx.send(CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}
