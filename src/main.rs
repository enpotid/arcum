mod commands;

use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::model::id::{ChannelId, UserId};
use std::env;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    wordchain_message: Arc<Mutex<Vec<String>>>,
    wordchain_player: Arc<Mutex<(UserId, UserId)>>,
    wordchain_turn: Arc<Mutex<bool>>,
    wordchain_check: Arc<Mutex<bool>>,
    wordchain_thread: Arc<Mutex<ChannelId>>,
    wordchain_playing: Arc<Mutex<bool>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // There are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::help(),
            commands::ping(),
            commands::badapple(),
            commands::tts(),
            commands::matrix(),
            commands::alarm(),
            commands::wordchain(),
            commands::maze(),
            commands::exit(),
            commands::ptf(),
            commands::bwimg(),
            commands::rgbimg(),
            commands::hexcode(),
            commands::sha256(),
            commands::argon2(),
            commands::verify(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    wordchain_message: Arc::new(Mutex::new(vec![])),
                    wordchain_player: Arc::new(Mutex::new((
                        UserId::from(NonZeroU64::new(1).unwrap()),
                        UserId::from(NonZeroU64::new(1).unwrap()),
                    ))),
                    wordchain_turn: Arc::new(Mutex::new(false)),
                    wordchain_check: Arc::new(Mutex::new(false)),
                    wordchain_thread: Arc::new(Mutex::new(ChannelId::from(
                        NonZeroU64::new(1).unwrap(),
                    ))),
                    wordchain_playing: Arc::new(Mutex::new(false)),
                })
            })
        })
        .options(options)
        .build();

    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            commands::wordchain_message_event(data, new_message)
                .await
                .unwrap();
        }
        _ => {}
    }
    Ok(())
}
