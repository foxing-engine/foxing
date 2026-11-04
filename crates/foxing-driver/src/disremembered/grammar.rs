//! A story is told in sentences. Colophon struck four hidden words with lore salads
//! of a hundred proper nouns and no verbs; the word count and the distinct-word count
//! both passed them. This reads a telling the way a listener would, by cheap marks:
//! are there small words holding the big ones together, is it in sentences, is it
//! mostly names. It is not a parser; it refuses the salad and lets the plain tale by.

/// The small words a sentence hangs on. A story of a dozen words without one of
/// these is a list.
const FUNCTION_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "so", "of", "to", "in", "on", "at", "by", "for", "with",
    "from", "into", "out", "up", "down", "over", "under", "through", "as", "than", "it", "its",
    "is", "was", "were", "be", "been", "are", "am", "had", "has", "have", "did", "do", "does",
    "not", "no", "he", "she", "they", "we", "i", "you", "me", "him", "her", "them", "us", "my",
    "his", "our", "your", "their", "this", "that", "these", "those", "there", "here", "who",
    "what", "which", "when", "where", "how", "why", "then", "now", "once", "if", "because",
    "while", "until", "before", "after", "said", "like", "one", "all", "some", "any", "none",
    "never", "always", "still", "again", "would", "could", "should", "will", "can", "may", "might",
    "must", "went", "came", "told", "kept",
];

/// The longest a sentence may run before it is a wall.
const SENTENCE_MAX: usize = 60;
/// The share of words that must be small ones, for a story of a dozen or more.
const FUNCTION_SHARE: f64 = 0.18;
/// The share of words that may be capitalized mid-sentence before it is a name salad.
const CAPITAL_SHARE: f64 = 0.3;
/// Letters in a word before it is two words run together.
const WORD_MAX: usize = 24;

/// Why a listener could not follow, or nothing: the story reads as told.
pub fn why_unfollowable(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 12 {
        return None;
    }
    let clean = |w: &str| -> String {
        w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'')
            .to_lowercase()
    };
    if let Some(long) = words
        .iter()
        .map(|w| clean(w))
        .find(|w| w.chars().filter(|c| c.is_alphabetic()).count() > WORD_MAX)
    {
        return Some(format!("\"{long}\" is not a word anyone says"));
    }
    let total = words.len() as f64;
    let small = words
        .iter()
        .filter(|w| FUNCTION_WORDS.contains(&clean(w).as_str()))
        .count() as f64;
    if small / total < FUNCTION_SHARE {
        return Some(
            "there are no small words in it (the, and, of, it, was) to hold the rest together; it is a list, not a telling".to_string(),
        );
    }
    // Sentences: split on terminal punctuation; a wall of sixty words is not one.
    let sentences: Vec<Vec<&str>> = text
        .split(['.', '!', '?', ';'])
        .map(|s| s.split_whitespace().collect::<Vec<_>>())
        .filter(|s| !s.is_empty())
        .collect();
    if let Some(wall) = sentences.iter().find(|s| s.len() > SENTENCE_MAX) {
        return Some(format!(
            "one breath of {} words; a story stops for a period now and then",
            wall.len()
        ));
    }
    // Names: a word capitalized that is not the first of its sentence.
    let mut capitals = 0usize;
    let mut counted = 0usize;
    for s in &sentences {
        for (i, w) in s.iter().enumerate() {
            let w = w.trim_matches(|c: char| !c.is_alphanumeric());
            if w.is_empty() || i == 0 {
                continue;
            }
            counted += 1;
            if w.chars().next().is_some_and(char::is_uppercase) && w != "I" {
                capitals += 1;
            }
        }
    }
    if counted >= 10 && capitals as f64 / counted as f64 > CAPITAL_SHARE {
        return Some("most of its words are names; a story is what the names did".to_string());
    }
    // Every sentence of five words or more carries a small word.
    if let Some(s) = sentences.iter().filter(|s| s.len() >= 6).find(|s| {
        !s.iter()
            .any(|w| FUNCTION_WORDS.contains(&clean(w).as_str()))
    }) {
        return Some(format!(
            "\"{}…\" is a list; put a verb and a the in it",
            s.iter().take(5).copied().collect::<Vec<_>>().join(" ")
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::why_unfollowable;

    #[test]
    fn a_plain_tale_passes_and_a_salad_does_not() {
        assert_eq!(
            why_unfollowable("There was a lantern on the stair that would not go out, however hard the wind came up the shaft. Hollis kept it lit for a reader who never came back."),
            None
        );
        assert_eq!(why_unfollowable("a short one"), None);
        let salad = "Fogbank Threshold Namery Bindery Clocktower Ossuary Lantern Row Undertow Orchard Sandbar Skiff Exergue Conclave Register Ledger Docent Hollis Quire Purse Warden";
        assert!(
            why_unfollowable(salad).is_some_and(|r| r.contains("small words")),
            "{:?}",
            why_unfollowable(salad)
        );
        let names = "The Fogbank and the Threshold and the Namery and the Bindery and the Clocktower and the Ossuary and the Undertow and the Orchard and the Sandbar and the Skiff and the Exergue and the Conclave";
        assert!(
            why_unfollowable(names).is_some_and(|r| r.contains("names")),
            "{:?}",
            why_unfollowable(names)
        );
        let wall = (0..70)
            .map(|i| if i % 3 == 0 { "the" } else { "fog" })
            .collect::<Vec<_>>()
            .join(" ");
        assert!(why_unfollowable(&wall).is_some_and(|r| r.contains("breath")));
        assert!(why_unfollowable("It was the night the water came and the reader stayed, and the lanternlanternlanternlanternlantern burned.").is_some());
    }
}
