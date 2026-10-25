//! Text helpers: language mapping, cleanup and [`Text`] construction from PokéAPI resources.

use rustemon::model::{resource::NamedApiResource, utility::Language};
use starrydex_model::{Lang, Text};

/// Maps a PokéAPI language resource to one of the bundled languages.
pub fn lang_of(language: &NamedApiResource<Language>) -> Option<Lang> {
    match language.name.to_ascii_lowercase().as_str() {
        "en" => Some(Lang::En),
        "es" => Some(Lang::Es),
        "fr" => Some(Lang::Fr),
        "de" => Some(Lang::De),
        "it" => Some(Lang::It),
        // "ja-hrkt" (kana) is what the games show; plain "ja" (kanji) is used as a fallback.
        "ja-hrkt" | "ja" => Some(Lang::Ja),
        "ko" => Some(Lang::Ko),
        "zh-hans" => Some(Lang::ZhHans),
        "zh-hant" => Some(Lang::ZhHant),
        "cs" => Some(Lang::Cs),
        "pt-br" => Some(Lang::PtBr),
        _ => None,
    }
}

/// Collapses whitespace/control characters and removes PokéAPI artifacts (soft hyphens,
/// form feeds, hard line breaks from the games' text boxes).
pub fn clean(raw: &str) -> String {
    raw.replace('\u{00ad}', "")
        .replace("-\n", "-")
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Builds a [`Text`] from any PokéAPI list of localized entries.
pub fn text_from<'a, T: 'a>(
    entries: impl IntoIterator<Item = &'a T>,
    language: impl Fn(&T) -> &NamedApiResource<Language>,
    value: impl Fn(&T) -> &str,
) -> Text {
    Text::from_pairs(
        entries
            .into_iter()
            .filter_map(|entry| lang_of(language(entry)).map(|lang| (lang, clean(value(entry))))),
    )
}

/// Like [`text_from`] but keeps the *last* entry for each language (PokéAPI lists flavor texts
/// chronologically, so the last one is the most recent game).
pub fn latest_text_from<'a, T: 'a>(
    entries: impl DoubleEndedIterator<Item = &'a T>,
    language: impl Fn(&T) -> &NamedApiResource<Language>,
    value: impl Fn(&T) -> &str,
) -> Text {
    text_from(entries.rev(), language, value)
}

/// Fallback display name built from an API identifier: `mr-mime` → `Mr Mime`.
pub fn title_case(identifier: &str) -> String {
    identifier
        .split('-')
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Uses `text` unless it is empty, in which case an English title-cased identifier is used.
pub fn or_title(text: Text, identifier: &str) -> Text {
    if text.is_empty() {
        Text::en(title_case(identifier))
    } else {
        text
    }
}

/// Joins several texts language by language (used for "Red / Blue" version group names).
pub fn join(texts: &[&Text], separator: &str) -> Text {
    Text::from_pairs(Lang::ALL.iter().map(|lang| {
        (
            *lang,
            texts
                .iter()
                .map(|t| t.get(*lang))
                .collect::<Vec<_>>()
                .join(separator),
        )
    }))
}

/// Applies `f` to the English text and every translation.
pub fn map_text(text: Text, f: impl Fn(&str) -> String) -> Text {
    Text {
        en: f(&text.en),
        translations: text
            .translations
            .into_iter()
            .map(|mut t| {
                t.text = f(&t.text);
                t
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_game_text() {
        assert_eq!(
            clean("A strange seed was\nplanted on its\u{c}back at birth."),
            "A strange seed was planted on its back at birth."
        );
        assert_eq!(clean("Poké\u{ad}mon"), "Pokémon");
    }

    #[test]
    fn title_cases_identifiers() {
        assert_eq!(title_case("mr-mime"), "Mr Mime");
        assert_eq!(title_case("kanto-route-1-area"), "Kanto Route 1 Area");
    }
}
