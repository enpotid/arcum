use crate::{Context, Error};
use bytes::Bytes;
use image::{Rgb, RgbImage};
use ndarray::Array2;
use num_complex::Complex;
use poise::serenity_prelude::{CreateAttachment, CreateEmbed};
use poise::CreateReply;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::io::Cursor;
use tokio::task;

// 테트레이션 발산 계산 함수
fn compute_tetration_divergence(
    nx: usize,          // x축의 크기
    ny: usize,          // y축의 크기
    x0: f64,            // x축 시작 값
    y0: f64,            // y축 시작 값
    eps: f64,           // x축에 대한 간격
    eps_y: f64,         // y축에 대한 간격
    max_iter: usize,    // 최대 반복 횟수
    escape_radius: f64, // 탈출 반경
) -> Array2<bool> {
    // 결과는 발산 여부를 나타내는 배열
    let mut divergence_map = Array2::from_elem((nx, ny), false); // 초기값은 모두 false

    // x, y 값들 미리 계산
    let x_vals: Vec<f64> = (0..nx)
        .map(|i| x0 - eps + 2.0 * eps * (i as f64) / (nx as f64))
        .collect(); // x값 계산
    let y_vals: Vec<f64> = (0..ny)
        .map(|j| y0 - eps_y + 2.0 * eps_y * (j as f64) / (ny as f64))
        .collect(); // y값 계산

    // 각 행에 대해 병렬 처리
    divergence_map
        .axis_iter_mut(ndarray::Axis(0)) // `par_axis_iter_mut`을 사용하여 병렬 처리
        .enumerate()
        .for_each(|(i, mut row)| {
            for (j, cell) in row.iter_mut().enumerate() {
                let c = Complex::new(x_vals[i], y_vals[j]); // c 값을 계산
                let mut z = c;

                // 반복문을 통해 발산 여부 확인
                for _ in 0..max_iter {
                    z = c.powc(z); // z = c^z 계산
                    if z.norm() > escape_radius {
                        // 탈출 반경을 벗어나면 발산
                        *cell = true;
                        break;
                    }
                }
            }
        });

    divergence_map // 계산된 발산 맵 반환
}

/// Make a Power Tower Fractal image
#[poise::command(slash_command)]
pub async fn ptf(
    ctx: Context<'_>,
    #[description = "x0 coordinate"] x0: f64,
    #[description = "y0 coordinate"] y0: f64,
    #[description = "epsilon for spacing"] eps: f64,
    #[description = "number of x points"] nx: usize,
    #[description = "number of y points"] ny: usize,
) -> Result<(), Error> {
    if nx > 3840 || ny > 3840 || nx < 1 || ny < 1 {
        let embed = CreateEmbed::default()
            .title("Error!!")
            .description("The values of nx and ny must be between 1 and 3840 inclusive.");

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title("Processing!!")
        .description("Generating the Power Tower Fractal image");

    ctx.send(CreateReply::default().embed(embed)).await?;

    let eps_y = eps * (ny as f64 / nx as f64); // y축 간격을 nx와 ny의 비율에 맞게 설정
    let max_iter = 500; // 최대 반복 횟수
    let escape_radius = 1e10; // 탈출 반경

    // 테트레이션 발산 맵 계산 (비동기 처리)
    let divergence_map = task::spawn_blocking(move || {
        compute_tetration_divergence(nx, ny, x0, y0, eps, eps_y, max_iter, escape_radius)
    })
    .await
    .unwrap();

    // 이미지 생성 (메모리에 직접 생성)
    let mut img = RgbImage::new(nx as u32, ny as u32);

    // 발산 여부에 따른 픽셀 색상 설정
    for ((x, y), &is_divergent) in divergence_map.indexed_iter() {
        let color = if is_divergent {
            image::Rgb([255, 255, 255])
        } else {
            image::Rgb([0, 0, 0])
        }; // 흰색, 검은색 설정
        img.put_pixel(x as u32, y as u32, color);
    }

    // 이미지 데이터를 메모리에서 생성된 상태로 Discord에 첨부
    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;

    // 메모리에서 이미지를 Discord로 보내기
    let attachment = CreateAttachment::bytes(&*buffer, "tetration.png");

    let embed = CreateEmbed::default()
        .title("Generated!!")
        .description(format!(
            "x0 : {}\ny0 : {}\neps : {}\nnx : {}\nny : {}",
            x0, y0, eps, nx, ny
        ));

    ctx.send(CreateReply::default().embed(embed).attachment(attachment))
        .await?;

    Ok(())
}

/// Make a random black-and-white image
#[poise::command(slash_command)]
pub async fn bwimg(
    ctx: Context<'_>,
    #[description = "the width of the image"] width: usize,
    #[description = "the height of the image"] height: usize,
    #[description = "the seed for the random generator"] seed: Option<u64>,
) -> Result<(), Error> {
    if width > 3840 || height > 3840 || width < 1 || height < 1 {
        let embed = CreateEmbed::default()
            .title("Error!!")
            .description("The width and height values must be between 1 and 3840 inclusive");

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title("Processing!!")
        .description("Generating the black-and-white image");

    ctx.send(CreateReply::default().embed(embed)).await?;

    // 백그라운드 스레드에서 이미지 생성
    let img = task::spawn_blocking(move || {
        let mut rng = match seed {
            Some(s) => {
                let mut seed_array = [0u8; 32];
                let s_bytes = s.to_le_bytes();
                seed_array[0..8].copy_from_slice(&s_bytes);
                ChaCha8Rng::from_seed(seed_array)
            }
            None => ChaCha8Rng::from_entropy(),
        };

        let mut img = RgbImage::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                let color = if rng.gen_bool(0.5) {
                    Rgb([255, 255, 255])
                } else {
                    Rgb([0, 0, 0])
                };
                img.put_pixel(x as u32, y as u32, color);
            }
        }

        img
    })
    .await
    .unwrap();

    // 이미지 데이터를 메모리에서 생성된 상태로 Discord에 첨부
    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;

    let attachment = CreateAttachment::bytes(&*buffer, "random_image.png");

    let mut description = format!("width : {}\nheight : {}", width, height);

    if let Some(seed_value) = seed {
        description.push_str(&format!("\nseed: {}", seed_value));
    }

    let embed = CreateEmbed::default()
        .title("Generated!!")
        .description(description);

    ctx.send(CreateReply::default().embed(embed).attachment(attachment))
        .await?;

    Ok(())
}

/// Make a random rgb image
#[poise::command(slash_command)]
pub async fn rgbimg(
    ctx: Context<'_>,
    #[description = "the width of the image"] width: usize,
    #[description = "the height of the image"] height: usize,
    #[description = "the seed for the random generator"] seed: Option<u64>,
) -> Result<(), Error> {
    if width > 1000 || height > 1000 || width < 1 || height < 1 {
        let embed = CreateEmbed::default()
            .title("Error!!")
            .description("The width and height values must be between 1 and 1000 inclusive");

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title("Processing!!")
        .description("Generating the RGB image");

    ctx.send(CreateReply::default().embed(embed)).await?;

    // 백그라운드 스레드에서 이미지 생성
    let img = task::spawn_blocking(move || {
        let mut rng = match seed {
            Some(s) => {
                let mut seed_array = [0u8; 32];
                let s_bytes = s.to_le_bytes();
                seed_array[0..8].copy_from_slice(&s_bytes);
                ChaCha8Rng::from_seed(seed_array)
            }
            None => ChaCha8Rng::from_entropy(),
        };

        let mut img = RgbImage::new(width as u32, height as u32);

        for x in 0..width {
            for y in 0..height {
                let color = Rgb([
                    rng.gen_range(0..=255),
                    rng.gen_range(0..=255),
                    rng.gen_range(0..=255),
                ]);
                img.put_pixel(x as u32, y as u32, color);
            }
        }

        img
    })
    .await
    .unwrap();

    // 이미지 데이터를 메모리에서 생성된 상태로 Discord에 첨부
    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;

    let attachment = CreateAttachment::bytes(&*buffer, "random_rgb_image.png");

    let mut description = format!("width : {}\nheight : {}", width, height);

    if let Some(seed_value) = seed {
        description.push_str(&format!("\nseed: {}", seed_value));
    }

    let embed = CreateEmbed::default()
        .title("Generated!!")
        .description(description);

    ctx.send(CreateReply::default().embed(embed).attachment(attachment))
        .await?;

    Ok(())
}

/// Display the color of the hex code
#[poise::command(slash_command)]
pub async fn hexcode(
    ctx: Context<'_>,
    #[description = "Enter the hex code"] hex_code: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let example = "\nExample : #FFBF98\nCase does not matter";

    if hex_code.len() != 6 {
        let embed = CreateEmbed::default().title("Error!!").description(format!(
            "The hex code must be exactly 6 characters{example}"
        ));

        ctx.send(CreateReply::default().embed(embed)).await?;

        return Ok(());
    }

    let hex_code = hex_code.to_lowercase();

    let chunks: Vec<String> = hex_code
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect();
    let hex1 = &chunks[0];
    let hex2 = &chunks[1];
    let hex3 = &chunks[2];

    let parse_hex = |hex: &str| u8::from_str_radix(hex, 16);
    match (parse_hex(hex1), parse_hex(hex2), parse_hex(hex3)) {
        (Ok(val1), Ok(val2), Ok(val3)) => {
            let img = RgbImage::new(100, 100);
            let mut img = img.clone();

            for pixel in img.pixels_mut() {
                *pixel = Rgb([val1, val2, val3]);
            }

            let mut buffer = Vec::new();
            img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
                .expect("Failed to write image");

            let attachment = CreateAttachment::bytes(Bytes::from(buffer), "color.png");

            let embed = CreateEmbed::default()
                .title("Generated Color!!")
                .description(format!("#{}{}{}", hex1, hex2, hex3));

            ctx.send(CreateReply::default().embed(embed).attachment(attachment))
                .await?;
        }
        _ => {
            let embed = CreateEmbed::default()
                .title("Error!!")
                .description(format!("The hex code must be hexadecimal{example}"));

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}
