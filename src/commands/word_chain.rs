use crate::{Context, Data, Error};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{
    model, ChannelType, CreateEmbed, CreateMessage, CreateThread, Mentionable,
};
use poise::CreateReply;
use quick_xml::de::from_str;
use reqwest::Client;
use serde::Deserialize;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct Channel {
    item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    word: String,
    sense: Vec<Sense>,
}

#[derive(Debug, Deserialize)]
struct Sense {
    definition: String,
}

/// Play the Word Chain Game
#[poise::command(slash_command)]
pub async fn wordchain(
    ctx: Context<'_>,
    #[description = "People who want to play together"] user: model::id::UserId,
) -> Result<(), Error> {
    if *ctx.data().wordchain_playing.lock().await {
        let embed = CreateEmbed::default()
            .title("Sorry!!")
            .description("Other people are already playing");

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    } else if user == ctx.cache().current_user().id {
        let embed = CreateEmbed::default()
            .title("Sorry!!")
            .description("Bot cannot be invited");

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    }

    let channel = ctx.channel_id();

    if let Ok(channel) = channel.to_channel(&ctx.http()).await {
        match channel {
            serenity::Channel::Guild(channel) => {
                if channel.kind != ChannelType::Text {
                    let embed = CreateEmbed::default()
                        .title("Error!!")
                        .description("This command can only be used in a text channel");

                    ctx.send(CreateReply::default().embed(embed)).await?;

                    return Ok(());
                }
            }
            _ => {
                let embed = CreateEmbed::default()
                    .title("Error!!")
                    .description("This command can only be used in a text channel");

                ctx.send(CreateReply::default().embed(embed)).await?;

                return Ok(());
            }
        }
    } else {
        let embed = CreateEmbed::default()
            .title("Error!!")
            .description("This command can only be used in a text channel");

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title("Word Chain Game Start!!")
        .description(format!(
            "participant : {} {}\nThose who want to spectate, please join the thread",
            ctx.author().mention(),
            ctx.serenity_context().http.get_user(user).await?.mention()
        ));

    ctx.send(CreateReply::default().embed(embed)).await?;

    let thread = channel
        .create_thread(
            &ctx,
            CreateThread::new(format!("Word Chain Game")).kind(ChannelType::PublicThread),
        )
        .await?;

    thread.id.add_thread_member(&ctx, ctx.author().id).await?;
    thread.id.add_thread_member(&ctx, user).await?;

    thread
        .id
        .send_message(
            &ctx,
            CreateMessage::default().content(format!("You are first {}", ctx.author().mention())),
        )
        .await?;

    *ctx.data().wordchain_player.lock().await = (ctx.author().id, user);

    *ctx.data().wordchain_thread.lock().await = thread.id;

    *ctx.data().wordchain_playing.lock().await = true;

    loop {
        let mut wordchain_check = ctx.data().wordchain_check.lock().await;
        let wordchain_message = ctx.data().wordchain_message.lock().await;

        if !(*wordchain_check) {
            *wordchain_check = true;
            if wordchain_message.len() != 0 {
                if wordchain_message[wordchain_message.len() - 1]
                    .split_terminator("")
                    .skip(1)
                    .collect::<Vec<_>>()
                    .len()
                    <= 1
                {
                    let embed = CreateEmbed::default()
                        .title("Wrong!!")
                        .description("The word must be at least two letters long");

                    thread
                        .id
                        .send_message(&ctx, CreateMessage::default().embed(embed))
                        .await?;

                    break;
                }

                if wordchain_message.len() != 1 {
                    let len = wordchain_message[wordchain_message.len() - 2]
                        .split_terminator("")
                        .skip(1)
                        .collect::<Vec<_>>()
                        .len();
                    if wordchain_message[wordchain_message.len() - 1]
                        .split_terminator("")
                        .skip(1)
                        .collect::<Vec<_>>()[0]
                        != wordchain_message[wordchain_message.len() - 2]
                            .split_terminator("")
                            .skip(1)
                            .collect::<Vec<_>>()[len - 1]
                    {
                        let embed = CreateEmbed::default().title("Wrong!!").description(
                            "You must start with the last letter of the previous word",
                        );

                        thread
                            .id
                            .send_message(&ctx, CreateMessage::default().embed(embed))
                            .await?;

                        break;
                    }
                }
                let client = Client::new();
                let response = client
                    .get("https://krdict.korean.go.kr/api/search")
                    .query(&[
                        ("key", "1F97BE6A85F5E06B611AF77B70B0289D"),
                        (
                            "q",
                            &format!("{}", wordchain_message[wordchain_message.len() - 1]),
                        ),
                        ("translated", "y"),
                        ("sort", "popular"),
                    ])
                    .send()
                    .await?
                    .text()
                    .await?;

                let embed = CreateEmbed::default()
                    .title("Wrong!!")
                    .description("I cannot find the word");

                if let Ok(channel) = from_str::<Channel>(&response) {
                    if let Some(first_item) = channel.item.get(0) {
                        if format!("{}", wordchain_message[wordchain_message.len() - 1])
                            == format!("{}", first_item.word)
                        {
                            let embed = CreateEmbed::default()
                                .title(format!("Word : {}", first_item.word))
                                .description(format!(
                                    "Meaning : {}",
                                    &first_item.sense[0].definition
                                ));

                            thread
                                .id
                                .send_message(&ctx, CreateMessage::default().embed(embed))
                                .await?;
                        } else {
                            thread
                                .id
                                .send_message(&ctx, CreateMessage::default().embed(embed))
                                .await?;

                            break;
                        }
                    } else {
                        thread
                            .id
                            .send_message(&ctx, CreateMessage::default().embed(embed))
                            .await?;

                        break;
                    }
                } else {
                    thread
                        .id
                        .send_message(&ctx, CreateMessage::default().embed(embed))
                        .await?;

                    break;
                }

                let mention = if *ctx.data().wordchain_turn.lock().await {
                    ctx.serenity_context().http.get_user(user).await?.mention()
                } else {
                    ctx.author().mention()
                };

                thread
                    .id
                    .send_message(
                        &ctx,
                        CreateMessage::default().content(format!("It's your turn {}", mention)),
                    )
                    .await?;
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    let result = ctx.data().wordchain_message.lock().await.join(" -> ");

    let mention = if *ctx.data().wordchain_turn.lock().await {
        ctx.serenity_context().http.get_user(user).await?.mention()
    } else {
        ctx.author().mention()
    };

    let embed = CreateEmbed::default()
        .title("Game Over!!")
        .description(format!(
            "You won!! {}\n{}\nThis thread will be deleted in 30 seconds",
            mention, result
        ));

    thread
        .id
        .send_message(&ctx, CreateMessage::default().embed(embed.clone()))
        .await?;

    *ctx.data().wordchain_message.lock().await = vec![];

    *ctx.data().wordchain_playing.lock().await = false;

    *ctx.data().wordchain_turn.lock().await = false;

    sleep(Duration::from_secs(30)).await;

    thread.id.delete(ctx).await?;

    Ok(())
}

pub async fn wordchain_message_event(
    data: &Data,
    new_message: &serenity::model::channel::Message,
) -> Result<(), Error> {
    let is_playing = *data.wordchain_playing.lock().await;
    if is_playing {
        let thread_channel = *data.wordchain_thread.lock().await;
        if thread_channel == new_message.channel_id {
            let players = data.wordchain_player.lock().await;
            let turn = *data.wordchain_turn.lock().await;

            if (!turn && new_message.author.id == players.0)
                || (turn && new_message.author.id == players.1)
            {
                let check = *data.wordchain_check.lock().await;
                if check {
                    *data.wordchain_turn.lock().await = !turn;
                    *data.wordchain_check.lock().await = false;

                    data.wordchain_message
                        .lock()
                        .await
                        .push(format!("{}", new_message.content));
                }
            }
        }
    }
    Ok(())
}
