use std::fs;

const APP_ID: &'static str = "dev.mariinkys.StarryDex";

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
    client: &reqwest::Client,
    image_url: String,
    pokemon_name: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resources_path = dirs::data_dir()
        .unwrap()
        .join(APP_ID)
        .join("resources")
        .join("sprites");

    if !resources_path.exists() {
        fs::create_dir_all(&resources_path).expect("Failed to create the resources path");
    }

    let image_filename = format!("{}_front.png", pokemon_name);
    let image_path = resources_path.join(&pokemon_name).join(&image_filename);

    // Check if file already exists
    if tokio::fs::metadata(&image_path).await.is_ok() {
        return Ok(());
    }

    let response = client.get(&image_url).send().await?;
    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let path = std::path::PathBuf::from(&image_path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&image_path, &bytes).await?;
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download image. Status: {}", response.status()),
        )))
    }
}
