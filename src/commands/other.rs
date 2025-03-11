use crate::{Context, Error};
use poise::serenity_prelude::{CreateAttachment, CreateEmbed, Mentionable};
use poise::CreateReply;
use rand::{prelude::IteratorRandom, rngs::StdRng, seq::SliceRandom, SeedableRng};
use reqwest::Client;
use serde_json;
use tokio::time::{sleep, Duration};

/// Show this help menu
#[poise::command(track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Make a maze
#[poise::command(slash_command)]
pub async fn maze(
    ctx: Context<'_>,
    #[description = "Enter the width of the maze"] width: u8,
    #[description = "Enter the height of the maze"] height: u8,
) -> Result<(), Error> {
    ctx.defer().await?;

    let width = width as usize;
    let height = height as usize;

    if width == 1 || height == 1 || width > 19 || height > 19 || width % 2 == 0 || height % 2 == 0 {
        let embed = CreateEmbed::default().title("Error!!").description(
            "The width and height must be odd numbers and between 3 and 20, inclusive",
        );

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    }

    let mut maze = vec![vec![1; width]; height];

    let mut rng = StdRng::from_entropy();

    let start_x = 1;
    let start_y = 1;
    maze[start_y][start_x] = 0;

    let mut stack = vec![(start_x, start_y)];

    let directions = [(0, -2), (0, 2), (-2, 0), (2, 0)];

    while let Some((x, y)) = stack.pop() {
        let mut shuffled_directions = directions.clone();
        shuffled_directions.shuffle(&mut rng);

        for (dx, dy) in shuffled_directions {
            let nx = (x as isize + dx as isize) as usize;
            let ny = (y as isize + dy as isize) as usize;

            if ny > 0 && ny < height && nx > 0 && nx < width && maze[ny][nx] == 1 {
                maze[(y as isize + dy as isize / 2) as usize]
                    [(x as isize + dx as isize / 2) as usize] = 0;
                maze[ny][nx] = 0;
                stack.push((nx, ny));
            }
        }
    }

    let entrance_x = (0..width).filter(|&x| x % 2 == 1).choose(&mut rng).unwrap();
    maze[0][entrance_x] = 0;

    let exit_x = (0..width).filter(|&x| x % 2 == 1).choose(&mut rng).unwrap();
    maze[height - 1][exit_x] = 0;

    let mut result: Vec<Vec<String>> = vec![];

    for y in maze {
        let mut memo: Vec<String> = vec![];
        for x in y {
            if x == 1 {
                memo.push("⬜️".to_string());
            } else {
                memo.push("⬛".to_string());
            }
        }
        result.push(memo);
    }

    let maze_str = result
        .iter()
        .map(|row| row.join(" "))
        .collect::<Vec<String>>()
        .join("\n");

    ctx.send(CreateReply::default().content(maze_str)).await?;

    Ok(())
}

/// It will notify you when the time is up
#[poise::command(slash_command)]
pub async fn alarm(
    ctx: Context<'_>,
    #[description = "Enter the desired number of seconds for the alarm"] seconds: u32,
) -> Result<(), Error> {
    if seconds > 2592000 {
        let embed = CreateEmbed::default()
            .title("Time is too long!!")
            .description("Enter a value within 30 days");

        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = CreateEmbed::default()
            .title("Set!!")
            .description(format!("The alarm will go off in {} seconds", seconds));

        ctx.send(CreateReply::default().embed(embed)).await?;

        sleep(Duration::from_secs(seconds as u64)).await;

        let embed = CreateEmbed::default()
            .title("Alarm!!")
            .description(format!("{} seconds timer is up!!", seconds));

        ctx.send(
            CreateReply::default()
                .content(format!("{}", ctx.author().mention()))
                .embed(embed),
        )
        .await?;
    }

    Ok(())
}

/// Matrix multiplication
#[poise::command(slash_command)]
pub async fn matrix(
    ctx: Context<'_>,
    #[description = "Enter two matrices"] matrix: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let matrices: Result<Vec<Vec<Vec<f64>>>, _> = serde_json::from_str(&matrix);
    let example = "\n\nExample:\n```[\n    [\n        [1.0, 2.0], \n        [3.0, 4.0], \n        [5.0, 6.0]\n    ], \n    [\n        [1.0, 0.0, 0.0, 1.0], \n        [0.0, 0.0, 0.0, 0.0]\n    ]\n]```";

    match matrices {
        Ok(matrices) => {
            let a = &matrices[0];
            let b = &matrices[1];

            let a_rows = a.len();
            let a_cols = a[0].len();
            let b_rows = b.len();
            let b_cols = b[0].len();

            let mut ok = true;

            if a_cols != b_rows {
                ok = false;
            } else {
                for i in 0..a_rows {
                    if a[i].len() != a_cols {
                        ok = false;
                    }
                }
                for i in 0..b_rows {
                    if b[i].len() != b_cols {
                        ok = false;
                    }
                }
            }

            if ok {
                let mut result = vec![vec![0.0; b_cols]; a_rows];

                for i in 0..a_rows {
                    for j in 0..b_cols {
                        for k in 0..a_cols {
                            result[i][j] += a[i][k] * b[k][j];
                        }
                    }
                }

                let mut resultmatrix = String::new();
                let mut col_widths = vec![0; b_cols];

                for row in &result {
                    for (j, &num) in row.iter().enumerate() {
                        if num.fract() == 0.0 {
                            col_widths[j] = col_widths[j].max(format!("{}", num as i64).len());
                        } else {
                            col_widths[j] = col_widths[j].max(format!("{:.2}", num).len());
                        }
                    }
                }

                for row in &result {
                    let row_str: String = row
                        .iter()
                        .enumerate()
                        .map(|(j, &x)| {
                            if x.fract() == 0.0 {
                                format!("{:width$}", x as i64, width = col_widths[j])
                            } else {
                                format!("{:width$.2}", x, width = col_widths[j])
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" , ");
                    resultmatrix.push_str(&format!("```\n[{}]\n```", row_str));
                }

                let embed = CreateEmbed::default()
                    .title("Calculated!!")
                    .description(resultmatrix);

                ctx.send(CreateReply::default().embed(embed)).await?;
            } else {
                let embed = CreateEmbed::default()
                    .title("Error!!")
                    .description(format!("The number of columns in the first matrix must match the number of rows in the second matrix.{}", example));

                ctx.send(CreateReply::default().embed(embed)).await?;
            }
        }

        Err(_) => {
            let embed = CreateEmbed::default().title("Error!!").description(format!(
                "Invalid input format. Please provide two matrices in JSON format.{}",
                example
            ));

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}

/// TTS Japanese
#[poise::command(slash_command)]
pub async fn tts(
    ctx: Context<'_>,
    #[description = "Enter the sentence to be generated"] text: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let voicevox_url = "http://localhost:50021";

    let client = Client::new();
    let audio_query_url = format!("{}/audio_query", voicevox_url);
    let response = client
        .post(&audio_query_url)
        .query(&[("text", &text), ("speaker", &"14".to_string())])
        .send()
        .await?;

    let audio_query: serde_json::Value = response.json().await?;

    let synthesis_url = format!("{}/synthesis", voicevox_url);
    let synthesis_response = client
        .post(&synthesis_url)
        .query(&[("speaker", &"14".to_string())])
        .json(&audio_query)
        .send()
        .await?;

    let audio_data = synthesis_response.bytes().await?;

    let embed = CreateEmbed::default()
        .title("TTS Generated!!")
        .description(text);

    let attachment = CreateAttachment::bytes(audio_data, "output.wav");

    ctx.send(CreateReply::default().embed(embed).attachment(attachment))
        .await?;

    Ok(())
}

/// Send a ping command to measure latency
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let ping_before = std::time::SystemTime::now();

    let embed = CreateEmbed::default()
        .title("pong!!")
        .description("Loading...");

    let ping_msg = ctx.send(poise::CreateReply::default().embed(embed)).await?;

    let msg = CreateEmbed::default()
        .title("pong!!")
        .description(format!("Latency: {}ms", ping_before.elapsed()?.as_millis()));

    ping_msg
        .edit(ctx, poise::CreateReply::default().embed(msg))
        .await?;

    Ok(())
}

/// Play Bad Apple!!
#[poise::command(slash_command)]
pub async fn badapple(
    ctx: Context<'_>,
    #[description = "Number of frames to skip"] frame_count: Option<usize>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let frame_count = frame_count.unwrap_or(1);

    let mut frames = Vec::new();
    for i in 0..=6571 / frame_count {
        let path = format!("output/frame{}.txt", i * frame_count);
        if let Ok(content) = std::fs::read_to_string(path) {
            frames.push(content);
        }
        println!("{}", i);
    }

    let message = ctx
        .say(frames.get(0).unwrap_or(&"".to_string()).clone())
        .await?;

    for frame in frames.iter().skip(41) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        message
            .edit(ctx, CreateReply::default().content(frame.clone()))
            .await?;
    }

    Ok(())
}
