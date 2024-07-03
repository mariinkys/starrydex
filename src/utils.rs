pub fn capitalize_string(input: &str) -> String {
    let words: Vec<&str> = input.split('-').collect();

    let capitalized_words: Vec<String> = words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            if let Some(first_char) = chars.next() {
                first_char.to_uppercase().collect::<String>() + chars.as_str()
            } else {
                String::new()
            }
        })
        .collect();

    capitalized_words.join(" ")
}

pub fn scale_numbers(num: i64) -> f64 {
    (num as f64) / 10.0
}

pub async fn download_image(
    image_url: String,
    pokemon_name: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let image_filename = format!("{}_front.png", pokemon_name);
    let image_path = format!("resources/sprites/{}/{}", pokemon_name, image_filename);

    // file already downloaded?
    if tokio::fs::metadata(&image_path).await.is_ok() {
        return Ok(image_path);
    }

    let response = reqwest::get(&image_url).await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;

        let path = std::path::PathBuf::from(&image_path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        save_image(&image_path, &bytes).await?;
        Ok(image_path)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download image. Status: {}", response.status()),
        )))
    }
}

async fn save_image(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = tokio::fs::File::create(path).await?;
    tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
    Ok(())
}
