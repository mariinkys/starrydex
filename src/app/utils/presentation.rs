// SPDX-License-Identifier: GPL-3.0-only

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

/// Helper to scale some data from PokeApi such as weight...
/// scales a number down by dividing it by 10, converting it to a floating-point
pub fn scale_numbers(num: i64) -> f64 {
    (num as f64) / 10.0
}
