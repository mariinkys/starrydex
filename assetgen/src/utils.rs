/// Transforms a kebab-case string into a space-separated string where each word starts with an uppercase letter.
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

pub fn clean_flavor_text(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}
