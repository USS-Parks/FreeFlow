use natural::phonetics::soundex;
use once_cell::sync::Lazy;
use regex::Regex;
use strsim::levenshtein;

/// Builds an n-gram string by cleaning and concatenating words
///
/// Strips punctuation from each word, lowercases, and joins without spaces.
/// This allows matching "Charge B" against "ChargeBee".
fn build_ngram(words: &[&str]) -> String {
    words
        .iter()
        .map(|w| build_match_key(w))
        .collect::<Vec<_>>()
        .concat()
}

fn build_match_key(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

struct CustomWordMatchKey {
    word_index: usize,
    key: String,
}

fn build_custom_word_match_keys(word: &str, word_index: usize) -> Vec<CustomWordMatchKey> {
    let primary_key = build_match_key(word);
    let mut keys = Vec::with_capacity(2);

    if !primary_key.is_empty() {
        keys.push(CustomWordMatchKey {
            word_index,
            key: primary_key.clone(),
        });
    }

    if word.contains('&') {
        let expanded_key = build_match_key(&word.replace('&', " and "));
        if !expanded_key.is_empty() && expanded_key != primary_key {
            keys.push(CustomWordMatchKey {
                word_index,
                key: expanded_key,
            });
        }
    }

    keys
}

/// Finds the best matching custom word for a candidate string
///
/// Uses Levenshtein distance and Soundex phonetic matching to find
/// the best match above the given threshold.
///
/// # Arguments
/// * `candidate` - The cleaned/lowercased candidate string to match
/// * `custom_words` - Original custom words (for returning the replacement)
/// * `custom_word_match_keys` - Normalized custom-word keys for comparison
/// * `threshold` - Maximum similarity score to accept
///
/// # Returns
/// The best matching custom word and its score, if any match was found
fn find_best_match<'a>(
    candidate: &str,
    custom_words: &'a [String],
    custom_word_match_keys: &[CustomWordMatchKey],
    threshold: f64,
) -> Option<(&'a String, f64)> {
    if candidate.is_empty() || candidate.len() > 50 {
        return None;
    }

    let mut best_match: Option<&String> = None;
    let mut best_score = f64::MAX;

    for custom_word_key in custom_word_match_keys {
        // Skip if lengths are too different (optimization + prevents over-matching)
        // Use percentage-based check: max 25% length difference (prevents n-grams from
        // matching significantly shorter custom words, e.g., "openaigpt" vs "openai")
        let len_diff = (candidate.len() as i32 - custom_word_key.key.len() as i32).abs() as f64;
        let max_len = candidate.len().max(custom_word_key.key.len()) as f64;
        let max_allowed_diff = (max_len * 0.25).max(2.0); // At least 2 chars difference allowed
        if len_diff > max_allowed_diff {
            continue;
        }

        // Calculate Levenshtein distance (normalized by length)
        let levenshtein_dist = levenshtein(candidate, &custom_word_key.key);
        let max_len = candidate.len().max(custom_word_key.key.len()) as f64;
        let levenshtein_score = if max_len > 0.0 {
            levenshtein_dist as f64 / max_len
        } else {
            1.0
        };

        // Calculate phonetic similarity using Soundex
        let phonetic_match = soundex(candidate, &custom_word_key.key);

        // Combine scores: favor phonetic matches, but also consider string similarity
        let combined_score = if phonetic_match {
            levenshtein_score * 0.3 // Give significant boost to phonetic matches
        } else {
            levenshtein_score
        };

        // Accept if the score is good enough (configurable threshold)
        if combined_score < threshold && combined_score < best_score {
            best_match = Some(&custom_words[custom_word_key.word_index]);
            best_score = combined_score;
        }
    }

    best_match.map(|m| (m, best_score))
}

/// Applies custom word corrections to transcribed text using fuzzy matching
///
/// This function corrects words in the input text by finding the best matches
/// from a list of custom words using a combination of:
/// - Levenshtein distance for string similarity
/// - Soundex phonetic matching for pronunciation similarity
/// - N-gram matching for multi-word speech artifacts (e.g., "Charge B" -> "ChargeBee")
///
/// # Arguments
/// * `text` - The input text to correct
/// * `custom_words` - List of custom words to match against
/// * `threshold` - Maximum similarity score to accept (0.0 = exact match, 1.0 = any match)
///
/// # Returns
/// The corrected text with custom words applied
pub fn apply_custom_words(text: &str, custom_words: &[String], threshold: f64) -> String {
    if custom_words.is_empty() {
        return text.to_string();
    }

    // Pre-compute normalized comparison keys to avoid repeated allocations.
    let custom_word_match_keys: Vec<CustomWordMatchKey> = custom_words
        .iter()
        .enumerate()
        .flat_map(|(index, word)| build_custom_word_match_keys(word, index))
        .collect();

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < words.len() {
        let mut matched = false;

        // Try n-grams from longest (3) to shortest (1) - greedy matching
        for n in (1..=3).rev() {
            if i + n > words.len() {
                continue;
            }

            let ngram_words = &words[i..i + n];
            let ngram = build_ngram(ngram_words);

            if let Some((replacement, _score)) =
                find_best_match(&ngram, custom_words, &custom_word_match_keys, threshold)
            {
                // Extract punctuation from first and last words of the n-gram
                let (prefix, _) = extract_punctuation(ngram_words[0]);
                let (_, suffix) = extract_punctuation(ngram_words[n - 1]);

                // Preserve case from first word
                let corrected = preserve_case_pattern(ngram_words[0], replacement);

                result.push(format!("{}{}{}", prefix, corrected, suffix));
                i += n;
                matched = true;
                break;
            }
        }

        if !matched {
            result.push(words[i].to_string());
            i += 1;
        }
    }

    result.join(" ")
}

/// Preserves the case pattern of the original word when applying a replacement
fn preserve_case_pattern(original: &str, replacement: &str) -> String {
    if original.chars().all(|c| c.is_uppercase()) {
        replacement.to_uppercase()
    } else if original.chars().next().is_some_and(|c| c.is_uppercase()) {
        let mut chars: Vec<char> = replacement.chars().collect();
        if let Some(first_char) = chars.get_mut(0) {
            *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
        }
        chars.into_iter().collect()
    } else {
        replacement.to_string()
    }
}

/// Extracts punctuation prefix and suffix from a word
fn extract_punctuation(word: &str) -> (&str, &str) {
    let prefix_end = word.chars().take_while(|c| !c.is_alphanumeric()).count();
    let suffix_start = word
        .char_indices()
        .rev()
        .take_while(|(_, c)| !c.is_alphanumeric())
        .count();

    let prefix = if prefix_end > 0 {
        &word[..prefix_end]
    } else {
        ""
    };

    let suffix = if suffix_start > 0 {
        &word[word.len() - suffix_start..]
    } else {
        ""
    };

    (prefix, suffix)
}

/// Returns filler words appropriate for the given language code.
///
/// Some words like "um" and "ha" are real words in certain languages
/// (e.g., Portuguese "um" = "a/an", Spanish "ha" = "has"), so we only
/// include them as fillers for languages where they are truly fillers.
fn get_filler_words_for_language(lang: &str) -> &'static [&'static str] {
    let base_lang = lang.split(&['-', '_'][..]).next().unwrap_or(lang);

    match base_lang {
        "en" => &[
            "uh", "um", "uhm", "umm", "uhh", "uhhh", "ah", "hmm", "hm", "mmm", "mm", "mh", "eh",
            "ehh", "ha",
        ],
        "es" => &["ehm", "mmm", "hmm", "hm"],
        "pt" => &["ahm", "hmm", "mmm", "hm"],
        "fr" => &["euh", "hmm", "hm", "mmm"],
        "de" => &["äh", "ähm", "hmm", "hm", "mmm"],
        "it" => &["ehm", "hmm", "mmm", "hm"],
        "cs" => &["ehm", "hmm", "mmm", "hm"],
        "pl" => &["hmm", "mmm", "hm"],
        "tr" => &["hmm", "mmm", "hm"],
        "ru" => &["хм", "ммм", "hmm", "mmm"],
        "uk" => &["хм", "ммм", "hmm", "mmm"],
        "ar" => &["hmm", "mmm"],
        "ja" => &["hmm", "mmm"],
        "ko" => &["hmm", "mmm"],
        "vi" => &["hmm", "mmm", "hm"],
        "zh" => &["hmm", "mmm"],
        // Conservative universal fallback (no "um", "eh", "ha")
        _ => &[
            "uh", "uhm", "umm", "uhh", "uhhh", "ah", "hmm", "hm", "mmm", "mm", "mh", "ehh",
        ],
    }
}

static MULTI_SPACE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t]{2,}").unwrap());

/// Collapses repeated words (3+ repetitions) to a single instance.
/// E.g., "wh wh wh wh" -> "wh", "I I I I" -> "I"
fn collapse_stutters(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return text.to_string();
    }

    let mut result: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < words.len() {
        let word = words[i];
        let word_lower = word.to_lowercase();

        if word_lower.chars().all(|c| c.is_alphabetic()) {
            // Count consecutive repetitions (case-insensitive)
            let mut count = 1;
            while i + count < words.len() && words[i + count].to_lowercase() == word_lower {
                count += 1;
            }

            // If 3+ repetitions, collapse to single instance
            if count >= 3 {
                result.push(word);
                i += count;
            } else {
                result.push(word);
                i += 1;
            }
        } else {
            result.push(word);
            i += 1;
        }
    }

    result.join(" ")
}

/// Filters transcription output by removing filler words and stutter artifacts.
///
/// This function cleans up raw transcription text by:
/// 1. Removing filler words based on the app language (or custom list)
/// 2. Collapsing repeated word stutters (e.g., "wh wh wh" -> "wh")
/// 3. Cleaning up excess whitespace
///
/// # Arguments
/// * `text` - The raw transcription text to filter
/// * `lang` - The app language code (e.g., "en", "pt-BR") used to select filler words
/// * `custom_filler_words` - Optional user-provided filler word list. `Some(vec)` overrides
///   language defaults; `Some(empty vec)` disables filtering; `None` uses language defaults.
///
/// # Returns
/// The filtered text with filler words and stutters removed
pub fn filter_transcription_output(
    text: &str,
    lang: &str,
    custom_filler_words: &Option<Vec<String>>,
) -> String {
    let mut filtered = text.to_string();

    // Build filler patterns from custom list or language defaults
    let patterns: Vec<Regex> = match custom_filler_words {
        Some(words) => words
            .iter()
            .filter_map(|word| Regex::new(&format!(r"(?i)\b{}\b[,.]?", regex::escape(word))).ok())
            .collect(),
        None => get_filler_words_for_language(lang)
            .iter()
            .map(|word| Regex::new(&format!(r"(?i)\b{}\b[,.]?", regex::escape(word))).unwrap())
            .collect(),
    };

    // Remove filler words
    for pattern in &patterns {
        filtered = pattern.replace_all(&filtered, "").to_string();
    }

    // Collapse repeated 1-2 letter words (stutter artifacts like "wh wh wh wh")
    filtered = collapse_stutters(&filtered);

    // Clean up multiple spaces to single space
    filtered = MULTI_SPACE_PATTERN.replace_all(&filtered, " ").to_string();

    // Trim leading/trailing whitespace
    filtered.trim().to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceControlOutput {
    pub text: String,
    pub submit_requested: bool,
}

const PERIOD_MARKER: char = '\u{e000}';
const COMMA_MARKER: char = '\u{e001}';
const QUESTION_MARKER: char = '\u{e002}';
const EXCLAMATION_MARKER: char = '\u{e003}';
const COLON_MARKER: char = '\u{e004}';
const SEMICOLON_MARKER: char = '\u{e005}';
const LINE_MARKER: char = '\u{e006}';
const PARAGRAPH_MARKER: char = '\u{e007}';
const BULLET_MARKER: char = '\u{e008}';
const NUMBERED_MARKER: char = '\u{e009}';

/// Apply only explicit, locally deterministic formatting and voice controls.
/// Ambiguous prose is left untouched. Callers retain the raw transcript.
pub fn apply_voice_controls(
    text: &str,
    language: &str,
    press_enter_enabled: bool,
) -> VoiceControlOutput {
    let (text, submit_requested) = parse_press_enter(text, press_enter_enabled);
    let text = apply_backtrack(&text, language);
    let text = normalize_explicit_numbers(&text, language);
    let text = replace_spoken_commands(&text, language);
    VoiceControlOutput {
        text: capitalize_sentence_starts(render_control_markers(&text).trim()),
        submit_requested,
    }
}

fn parse_press_enter(text: &str, enabled: bool) -> (String, bool) {
    if !enabled {
        return (text.to_string(), false);
    }
    let pattern = Regex::new(r"(?i)(?:^|[\s,]+)press[ \t]+enter[.!?]*[ \t]*$").unwrap();
    let Some(command) = pattern.find(text) else {
        return (text.to_string(), false);
    };
    let prefix = text[..command.start()].trim_end();
    if prefix
        .to_lowercase()
        .strip_suffix("literal")
        .is_some_and(|before| before.is_empty() || before.ends_with(char::is_whitespace))
    {
        let literal_prefix = prefix[..prefix.len() - "literal".len()].trim_end();
        let literal = if literal_prefix.is_empty() {
            "press enter".to_string()
        } else {
            format!("{literal_prefix} press enter")
        };
        return (literal, false);
    }
    (prefix.to_string(), true)
}

fn apply_backtrack(text: &str, language: &str) -> String {
    let base = language
        .split(['-', '_'])
        .next()
        .unwrap_or(language)
        .to_ascii_lowercase();
    let phrases: &[&str] = match base.as_str() {
        "en" | "auto" => &["scratch that", "delete that", "forget that"],
        "fr" => &["annule ça", "efface ça"],
        "es" => &["borra eso", "olvida eso"],
        _ => &[],
    };
    let mut output = text.to_string();
    for phrase in phrases {
        let pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(phrase))).unwrap();
        while let Some(command) = pattern.find(&output) {
            let before = output[..command.start()]
                .trim_end_matches(|character: char| character.is_whitespace() || character == ',');
            let after = output[command.end()..].trim_start_matches(|character: char| {
                character.is_whitespace() || character == ','
            });
            let segment_start = before
                .char_indices()
                .rev()
                .find(|(_, character)| matches!(character, '.' | '!' | '?' | '\n' | '\r'))
                .map_or(0, |(index, character)| index + character.len_utf8());
            let prefix = before[..segment_start].trim_end();
            output = match (prefix.is_empty(), after.is_empty()) {
                (true, true) => String::new(),
                (true, false) => after.to_string(),
                (false, true) => prefix.to_string(),
                (false, false) => format!("{prefix} {after}"),
            };
        }
    }
    output
}

fn replace_spoken_commands(text: &str, language: &str) -> String {
    let base = language
        .split(['-', '_'])
        .next()
        .unwrap_or(language)
        .to_ascii_lowercase();
    let commands: &[(&str, char)] = match base.as_str() {
        "en" | "auto" => &[
            ("new paragraph", PARAGRAPH_MARKER),
            ("question mark", QUESTION_MARKER),
            ("exclamation point", EXCLAMATION_MARKER),
            ("exclamation mark", EXCLAMATION_MARKER),
            ("numbered item", NUMBERED_MARKER),
            ("bullet point", BULLET_MARKER),
            ("next bullet", BULLET_MARKER),
            ("next item", NUMBERED_MARKER),
            ("new line", LINE_MARKER),
            ("line break", LINE_MARKER),
            ("full stop", PERIOD_MARKER),
            ("semicolon", SEMICOLON_MARKER),
            ("semi colon", SEMICOLON_MARKER),
            ("period", PERIOD_MARKER),
            ("comma", COMMA_MARKER),
            ("colon", COLON_MARKER),
        ],
        "fr" => &[
            ("nouveau paragraphe", PARAGRAPH_MARKER),
            ("nouvelle ligne", LINE_MARKER),
            ("point d'interrogation", QUESTION_MARKER),
            ("point d’exclamation", EXCLAMATION_MARKER),
            ("point d'exclamation", EXCLAMATION_MARKER),
            ("point virgule", SEMICOLON_MARKER),
            ("deux points", COLON_MARKER),
            ("virgule", COMMA_MARKER),
            ("point", PERIOD_MARKER),
        ],
        "es" => &[
            ("nuevo párrafo", PARAGRAPH_MARKER),
            ("nueva línea", LINE_MARKER),
            ("signo de interrogación", QUESTION_MARKER),
            ("signo de exclamación", EXCLAMATION_MARKER),
            ("punto y coma", SEMICOLON_MARKER),
            ("dos puntos", COLON_MARKER),
            ("coma", COMMA_MARKER),
            ("punto", PERIOD_MARKER),
        ],
        _ => &[],
    };
    let mut output = text.to_string();
    for (phrase, marker) in commands {
        let pattern = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(phrase))).unwrap();
        output = pattern
            .replace_all(&output, marker.to_string().as_str())
            .into_owned();
    }
    output
}

fn render_control_markers(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut needs_space = false;
    let mut numbered_item = 0_u32;
    for character in text.chars() {
        match character {
            PERIOD_MARKER | COMMA_MARKER | QUESTION_MARKER | EXCLAMATION_MARKER | COLON_MARKER
            | SEMICOLON_MARKER => {
                trim_horizontal_space(&mut output);
                output.push(match character {
                    PERIOD_MARKER => '.',
                    COMMA_MARKER => ',',
                    QUESTION_MARKER => '?',
                    EXCLAMATION_MARKER => '!',
                    COLON_MARKER => ':',
                    _ => ';',
                });
                needs_space = true;
            }
            LINE_MARKER | PARAGRAPH_MARKER => {
                trim_horizontal_space(&mut output);
                let required = if character == PARAGRAPH_MARKER { 2 } else { 1 };
                while output.ends_with('\n')
                    && output.chars().rev().take_while(|c| *c == '\n').count() > required
                {
                    output.pop();
                }
                let existing = output.chars().rev().take_while(|c| *c == '\n').count();
                for _ in existing..required {
                    output.push('\n');
                }
                needs_space = false;
            }
            BULLET_MARKER | NUMBERED_MARKER => {
                trim_horizontal_space(&mut output);
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                if character == BULLET_MARKER {
                    output.push_str("- ");
                } else {
                    numbered_item += 1;
                    output.push_str(&format!("{numbered_item}. "));
                }
                needs_space = false;
            }
            '\r' => {}
            '\n' => {
                trim_horizontal_space(&mut output);
                output.push('\n');
                needs_space = false;
            }
            character if character.is_whitespace() => {
                if !output.is_empty() && !output.ends_with([' ', '\n']) {
                    output.push(' ');
                }
            }
            character => {
                if needs_space
                    && !matches!(character, ',' | '.' | ';' | ':' | '!' | '?')
                    && !output.ends_with([' ', '\n'])
                {
                    output.push(' ');
                }
                needs_space = false;
                output.push(character);
            }
        }
    }
    trim_horizontal_space(&mut output);
    output
}

fn trim_horizontal_space(output: &mut String) {
    while output.ends_with([' ', '\t']) {
        output.pop();
    }
}

fn normalize_explicit_numbers(text: &str, language: &str) -> String {
    let base = language
        .split(['-', '_'])
        .next()
        .unwrap_or(language)
        .to_ascii_lowercase();
    if !matches!(base.as_str(), "en" | "auto") {
        return text.to_string();
    }
    let word = r"zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand|and";
    let pattern = Regex::new(&format!(
        r"(?i)\bnumber[ \t]+((?:{word})(?:[ \t-]+(?:{word}))*)\b"
    ))
    .unwrap();
    pattern
        .replace_all(text, |captures: &regex::Captures<'_>| {
            parse_english_number(&captures[1])
                .map_or_else(|| captures[0].to_string(), |number| number.to_string())
        })
        .into_owned()
}

fn parse_english_number(words: &str) -> Option<u64> {
    let normalized: Vec<String> = words
        .split(|character: char| character.is_whitespace() || character == '-')
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    if normalized.is_empty() {
        return None;
    }
    if normalized.iter().all(|word| {
        matches!(
            word.as_str(),
            "zero" | "one" | "two" | "three" | "four" | "five" | "six" | "seven" | "eight" | "nine"
        )
    }) {
        let digits = normalized
            .iter()
            .map(|word| match word.as_str() {
                "zero" => '0',
                "one" => '1',
                "two" => '2',
                "three" => '3',
                "four" => '4',
                "five" => '5',
                "six" => '6',
                "seven" => '7',
                "eight" => '8',
                _ => '9',
            })
            .collect::<String>();
        return digits.parse().ok();
    }
    let mut total = 0_u64;
    let mut current = 0_u64;
    for word in normalized {
        match word.as_str() {
            "zero" | "and" => {}
            "one" => current += 1,
            "two" => current += 2,
            "three" => current += 3,
            "four" => current += 4,
            "five" => current += 5,
            "six" => current += 6,
            "seven" => current += 7,
            "eight" => current += 8,
            "nine" => current += 9,
            "ten" => current += 10,
            "eleven" => current += 11,
            "twelve" => current += 12,
            "thirteen" => current += 13,
            "fourteen" => current += 14,
            "fifteen" => current += 15,
            "sixteen" => current += 16,
            "seventeen" => current += 17,
            "eighteen" => current += 18,
            "nineteen" => current += 19,
            "twenty" => current += 20,
            "thirty" => current += 30,
            "forty" => current += 40,
            "fifty" => current += 50,
            "sixty" => current += 60,
            "seventy" => current += 70,
            "eighty" => current += 80,
            "ninety" => current += 90,
            "hundred" if current > 0 => current *= 100,
            "thousand" if current > 0 => {
                total += current * 1_000;
                current = 0;
            }
            _ => return None,
        }
    }
    Some(total + current)
}

fn capitalize_sentence_starts(text: &str) -> String {
    let mut characters: Vec<char> = text.chars().collect();
    let mut sentence_start = true;
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if sentence_start && character.is_alphabetic() {
            let end = (index..characters.len())
                .find(|position| {
                    characters[*position].is_whitespace()
                        || matches!(characters[*position], ',' | '.' | ';' | ':' | '!' | '?')
                })
                .unwrap_or(characters.len());
            let token = &characters[index..end];
            let identifier_like = token.iter().skip(1).any(|value| value.is_uppercase())
                || token
                    .iter()
                    .any(|value| *value == '_' || value.is_numeric());
            if !identifier_like && character.is_lowercase() {
                let uppercase: Vec<char> = character.to_uppercase().collect();
                characters.splice(index..=index, uppercase.iter().copied());
            }
            sentence_start = false;
        }
        if matches!(character, '.' | '!' | '?' | '\n') {
            sentence_start = true;
        }
        index += 1;
    }
    characters.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_custom_words_exact_match() {
        let text = "hello world";
        let custom_words = vec!["Hello".to_string(), "World".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_apply_custom_words_fuzzy_match() {
        let text = "helo wrold";
        let custom_words = vec!["hello".to_string(), "world".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_preserve_case_pattern() {
        assert_eq!(preserve_case_pattern("HELLO", "world"), "WORLD");
        assert_eq!(preserve_case_pattern("Hello", "world"), "World");
        assert_eq!(preserve_case_pattern("hello", "WORLD"), "WORLD");
    }

    #[test]
    fn test_extract_punctuation() {
        assert_eq!(extract_punctuation("hello"), ("", ""));
        assert_eq!(extract_punctuation("!hello?"), ("!", "?"));
        assert_eq!(extract_punctuation("...hello..."), ("...", "..."));
    }

    #[test]
    fn test_empty_custom_words() {
        let text = "hello world";
        let custom_words = vec![];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_filter_filler_words() {
        let text = "So uhm I was thinking uh about this";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "So I was thinking about this");
    }

    #[test]
    fn test_filter_filler_words_case_insensitive() {
        let text = "UHM this is UH a test";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "this is a test");
    }

    #[test]
    fn test_filter_filler_words_with_punctuation() {
        let text = "Well, uhm, I think, uh. that's right";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "Well, I think, that's right");
    }

    #[test]
    fn test_filter_cleans_whitespace() {
        let text = "Hello    world   test";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "Hello world test");
    }

    #[test]
    fn test_filter_trims() {
        let text = "  Hello world  ";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_filter_combined() {
        let text = "  Uhm, so I was, uh, thinking about this  ";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "so I was, thinking about this");
    }

    #[test]
    fn test_filter_preserves_valid_text() {
        let text = "This is a completely normal sentence.";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "This is a completely normal sentence.");
    }

    #[test]
    fn test_filter_stutter_collapse() {
        let text = "w wh wh wh wh wh wh wh wh wh why";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "w wh why");
    }

    #[test]
    fn test_filter_stutter_short_words() {
        let text = "I I I I think so so so so";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "I think so");
    }

    #[test]
    fn test_filter_stutter_longer_words() {
        let text = "Check data doc doc doc doc documentation.";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "Check data doc documentation.");
    }

    #[test]
    fn test_filter_stutter_mixed_case() {
        let text = "No NO no NO no";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "No");
    }

    #[test]
    fn test_filter_stutter_preserves_two_repetitions() {
        let text = "no no is fine";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "no no is fine");
    }

    #[test]
    fn test_filter_english_removes_um() {
        let text = "um I think um this is good";
        let result = filter_transcription_output(text, "en", &None);
        assert_eq!(result, "I think this is good");
    }

    #[test]
    fn test_filter_portuguese_preserves_um() {
        // "um" means "a/an" in Portuguese
        let text = "um gato bonito";
        let result = filter_transcription_output(text, "pt", &None);
        assert_eq!(result, "um gato bonito");
    }

    #[test]
    fn test_filter_spanish_preserves_ha() {
        // "ha" means "has" in Spanish
        let text = "ha sido un buen día";
        let result = filter_transcription_output(text, "es", &None);
        assert_eq!(result, "ha sido un buen día");
    }

    #[test]
    fn test_filter_language_code_with_region() {
        // "pt-BR" should normalize to "pt"
        let text = "um gato bonito";
        let result = filter_transcription_output(text, "pt-BR", &None);
        assert_eq!(result, "um gato bonito");
    }

    #[test]
    fn test_filter_custom_filler_words_override() {
        let custom = Some(vec!["okay".to_string(), "right".to_string()]);
        let text = "okay so I think right this works";
        let result = filter_transcription_output(text, "en", &custom);
        assert_eq!(result, "so I think this works");
    }

    #[test]
    fn test_filter_custom_filler_words_empty_disables() {
        let custom = Some(vec![]);
        let text = "So uhm I was thinking uh about this";
        let result = filter_transcription_output(text, "en", &custom);
        // No filler words removed since custom list is empty
        assert_eq!(result, "So uhm I was thinking uh about this");
    }

    #[test]
    fn test_filter_unknown_language_uses_fallback() {
        let text = "uh I think uhm this works";
        let result = filter_transcription_output(text, "xx", &None);
        assert_eq!(result, "I think this works");
    }

    #[test]
    fn test_filter_fallback_does_not_remove_um() {
        // Fallback (unknown language) should not remove "um" since it's a real word in some languages
        let text = "um I think this works";
        let result = filter_transcription_output(text, "xx", &None);
        assert_eq!(result, "um I think this works");
    }

    #[test]
    fn voice_controls_format_punctuation_paragraphs_and_lists() {
        let result = apply_voice_controls(
            "hello comma world period new paragraph bullet point first next bullet second",
            "en",
            false,
        );
        assert_eq!(result.text, "Hello, world.\n\n- First\n- Second");
        assert!(!result.submit_requested);

        let numbered = apply_voice_controls(
            "tasks colon numbered item alpha next item beta",
            "en",
            false,
        );
        assert_eq!(numbered.text, "Tasks:\n1. Alpha\n2. Beta");
    }

    #[test]
    fn voice_controls_apply_explicit_numbers_and_backtrack_conservatively() {
        let filtered = filter_transcription_output(
            "uh send codeName number one two three scratch that number forty two period",
            "en",
            &None,
        );
        let result = apply_voice_controls(&filtered, "en", false);
        assert_eq!(result.text, "42.");
        assert!(!result.text.contains("codeName"));
    }

    #[test]
    fn ambiguous_numbers_names_and_identifiers_remain_literal() {
        let result = apply_voice_controls(
            "one Direction uses codeName and snake_case period",
            "en",
            false,
        );
        assert_eq!(result.text, "One Direction uses codeName and snake_case.");
    }

    #[test]
    fn multilingual_commands_are_scoped_to_the_effective_language() {
        assert_eq!(
            apply_voice_controls(
                "bonjour virgule monde point nouveau paragraphe merci",
                "fr-FR",
                false,
            )
            .text,
            "Bonjour, monde.\n\nMerci"
        );
        assert_eq!(
            apply_voice_controls("bonjour virgule monde", "de", false).text,
            "Bonjour virgule monde"
        );
    }

    #[test]
    fn press_enter_requires_enablement_and_a_valid_utterance_end() {
        let disabled = apply_voice_controls("send this press enter", "en", false);
        assert_eq!(disabled.text, "Send this press enter");
        assert!(!disabled.submit_requested);

        let enabled = apply_voice_controls("send this, press enter.", "en", true);
        assert_eq!(enabled.text, "Send this");
        assert!(enabled.submit_requested);

        let command_only = apply_voice_controls("PRESS ENTER", "en", true);
        assert!(command_only.text.is_empty());
        assert!(command_only.submit_requested);

        let middle = apply_voice_controls("say press enter literally here", "en", true);
        assert_eq!(middle.text, "Say press enter literally here");
        assert!(!middle.submit_requested);

        let escaped = apply_voice_controls("literal press enter", "en", true);
        assert_eq!(escaped.text, "Press enter");
        assert!(!escaped.submit_requested);
    }

    #[test]
    fn test_apply_custom_words_ngram_two_words() {
        let text = "il cui nome è Charge B, che permette";
        let custom_words = vec!["ChargeBee".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert!(result.contains("ChargeBee,"));
        assert!(!result.contains("Charge B"));
    }

    #[test]
    fn test_apply_custom_words_ngram_three_words() {
        let text = "use Chat G P T for this";
        let custom_words = vec!["ChatGPT".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert!(result.contains("ChatGPT"));
    }

    #[test]
    fn test_apply_custom_words_prefers_longer_ngram() {
        let text = "Open AI GPT model";
        let custom_words = vec!["OpenAI".to_string(), "GPT".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert_eq!(result, "OpenAI GPT model");
    }

    #[test]
    fn test_apply_custom_words_ngram_preserves_case() {
        let text = "CHARGE B is great";
        let custom_words = vec!["ChargeBee".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert!(result.contains("CHARGEBEE"));
    }

    #[test]
    fn test_apply_custom_words_ngram_with_spaces_in_custom() {
        // Custom word with space should also match against split words
        let text = "using Mac Book Pro";
        let custom_words = vec!["MacBook Pro".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert!(result.contains("MacBook"));
    }

    #[test]
    fn test_apply_custom_words_trailing_number_not_doubled() {
        // Verify that trailing non-alpha chars (like numbers) aren't double-counted
        // between build_ngram stripping them and extract_punctuation capturing them
        let text = "use GPT4 for this";
        let custom_words = vec!["GPT-4".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        // Should NOT produce "GPT-44" (double-counting the trailing 4)
        assert!(
            !result.contains("GPT-44"),
            "got double-counted result: {}",
            result
        );
    }

    #[test]
    fn test_apply_custom_words_matches_ampersand_word() {
        let text = "send it to RD for review";
        let custom_words = vec!["R&D".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.18);
        assert_eq!(result, "send it to R&D for review");
    }

    #[test]
    fn test_apply_custom_words_matches_spoken_ampersand_word() {
        let text = "send it to R and D for review";
        let custom_words = vec!["R&D".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.18);
        assert_eq!(result, "send it to R&D for review");
    }

    #[test]
    fn test_apply_custom_words_preserves_ampersand_word() {
        let text = "send it to R&D for review";
        let custom_words = vec!["R&D".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.18);
        assert_eq!(result, "send it to R&D for review");
    }
}
