use crate::tools::explain::words::{WordId, word_number, word_text};
use patterns::{because, source};

pub struct WordPositions;
source!(WordPositions, "a building block of the engine for long sentences by people: a condition reads the words of a whole sentence by their places in it, whether a word stands in it and whether one word comes after another, in place of cutting the sentence at every place where open places could part it; a reader of a sentence of people looks for its words and their order, not for one fixed form of it, so that one rule reads a sentence wherever its words stand");

pub(super) fn held_words(sentence: &[WordId], word: Option<WordId>) -> Vec<WordId> {
    let mut found: Vec<WordId> = Vec::new();
    for told in sentence {
        if word.is_none_or(|asked| asked == *told) && !found.contains(told) {
            found.push(*told);
        }
    }
    found
}
because!(held_words, WordPositions, "the different words of a sentence in the order in which they first stand, or the one asked word when the sentence holds it, and none when it does not");

pub(super) fn first_letter(word: WordId) -> Option<WordId> {
    word_text(word).chars().next().filter(|letter| letter.is_alphanumeric()).map(|letter| word_number(&letter.to_string()))
}
because!(first_letter, WordPositions, "the first letter of a word as a word of its own, as a name is compared by its first letter (a rule about names that start with the same letter), and nothing for a mark");

const FIRST_LETTER_PLACE: usize = 1;
because!(FIRST_LETTER_PLACE, WordPositions, "the place of the first letter of a word, as people count the letters of a word, so that the place of the last letter is the length of the word");

pub(super) fn placed_letters(word: WordId) -> Vec<(WordId, WordId)> {
    word_text(word).chars().enumerate().map(|(at, sign)| (word_number(&(at + FIRST_LETTER_PLACE).to_string()), word_number(&sign.to_string()))).collect()
}
because!(placed_letters, WordPositions, "each sign of a word with its place, counted as people count, as words of their own (a mark such as an apostrophe counts as a sign): the letters of a name or the digits of a number, so that a rule can read a word letter by letter (a common subsequence of two strings, the digits of a year)");

pub(super) fn later_pairs(sentence: &[WordId], earlier: Option<WordId>, later: Option<WordId>) -> Vec<(WordId, WordId)> {
    let mut found: Vec<(WordId, WordId)> = Vec::new();
    for (at, first) in sentence.iter().enumerate() {
        if earlier.is_some_and(|asked| asked != *first) {
            continue;
        }
        for second in &sentence[at + 1..] {
            if later.is_none_or(|asked| asked == *second) && !found.contains(&(*first, *second)) {
                found.push((*first, *second));
            }
        }
    }
    found
}
because!(later_pairs, WordPositions, "the different pairs of words of a sentence where the second stands somewhere after the first, each pair once, in the order of the sentence; a filled word of the pair must be that word");
