// SPDX-License-Identifier: GPL-3.0-only

use rkyv::{Archive, Deserialize, Serialize};

use crate::Lang;

/// A piece of user-facing text with its English version plus any translations PokéAPI has.
///
/// Translations identical to the English text are not stored.
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct Text {
    pub en: String,
    pub translations: Vec<Translation>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct Translation {
    pub lang: Lang,
    pub text: String,
}

impl Text {
    /// Plain English-only text.
    pub fn en(text: impl Into<String>) -> Self {
        Self {
            en: text.into(),
            translations: Vec::new(),
        }
    }

    /// Builds a [`Text`] out of `(language, text)` pairs. Unknown languages must already be
    /// filtered out by the caller. If there is no English entry, the first entry is promoted.
    pub fn from_pairs(pairs: impl IntoIterator<Item = (Lang, String)>) -> Self {
        let mut en: Option<String> = None;
        let mut translations: Vec<Translation> = Vec::new();
        for (lang, text) in pairs {
            let text = text.trim().to_string();
            if text.is_empty() {
                continue;
            }
            if lang == Lang::En {
                if en.is_none() {
                    en = Some(text);
                }
            } else if !translations.iter().any(|t| t.lang == lang) {
                translations.push(Translation { lang, text });
            }
        }
        let en = match en {
            Some(en) => en,
            None if !translations.is_empty() => translations[0].text.clone(),
            None => String::new(),
        };
        translations.retain(|t| t.text != en);
        translations.sort_by_key(|t| t.lang);
        Self { en, translations }
    }

    pub fn is_empty(&self) -> bool {
        self.en.is_empty()
    }

    pub fn get(&self, lang: Lang) -> &str {
        if lang != Lang::En
            && let Some(t) = self.translations.iter().find(|t| t.lang == lang)
        {
            return &t.text;
        }
        &self.en
    }
}

impl ArchivedText {
    /// Text in the requested language, falling back to English.
    pub fn get(&self, lang: Lang) -> &str {
        if lang != Lang::En
            && let Some(t) = self.translations.iter().find(|t| t.lang == lang)
        {
            return t.text.as_str();
        }
        self.en.as_str()
    }

    /// `true` if a translation (or the English original) exists for `lang`.
    pub fn has(&self, lang: Lang) -> bool {
        lang == Lang::En || self.translations.iter().any(|t| t.lang == lang)
    }

    pub fn is_empty(&self) -> bool {
        self.en.is_empty()
    }
}
