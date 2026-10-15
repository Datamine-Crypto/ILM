use crate::know::signs::is_word_sign;
use patterns::{because, source};

pub struct WholeWords;
source!(WholeWords, "a value is found in a text as whole words without regard to the case of its letters: no word sign may stand before or after the place, so that a short value is not found inside a longer word");

const WRITE_PAIRED_SIGNS: &[(char, char)] = &[('i', '\u{131}'), ('s', '\u{17f}'), ('\u{b5}', '\u{3bc}')];
because!(WRITE_PAIRED_SIGNS, WholeWords, "small letters that are one letter when the case of letters does not count, beside the pairs that the small form of a letter gives: the letter i and the same letter with no dot, the letter s and its long form, the micro sign and the Greek letter that it comes from; the pairs of other writing systems are left out");

fn first_sign_of(mut form: impl Iterator<Item = char>, sign: char) -> char {
    form.next().unwrap_or(sign)
}
because!(first_sign_of, WholeWords, "another form of one sign as one sign: the first sign where that form is more than one sign, so that a compare can go sign by sign");

fn small_sign(sign: char) -> char {
    first_sign_of(sign.to_lowercase(), sign)
}
because!(small_sign, WholeWords, "the small form of one sign as one sign, for a compare without regard to case");

fn paired_signs(one: char, other: char) -> bool {
    WRITE_PAIRED_SIGNS.iter().any(|(a, b)| (*a == one && *b == other) || (*a == other && *b == one))
}
because!(paired_signs, WholeWords, "are two small letters one letter for a search without regard to case, beyond what their small forms say");

#[derive(Clone, Debug)]
pub struct WantedValue {
    pub(crate) signs: Vec<(char, bool, char)>,
    pub(crate) low: String,
}
because!(WantedValue, WholeWords, "a value to be found in texts as whole words without regard to case: each of its signs with the mark that it has more than one case and with its small form. A sign with one case must stand as it is; a sign with more cases stands when the small forms are the same or are a pair. No word sign may stand before the place or after it; an empty value stands at every place with no word sign on either side. found gives the first place from a start on, places gives every place from left to right with none over another, replaced gives the text with another text in every place, and the test for many texts asks first that the small text holds the small value");

impl WantedValue {
    pub fn new(value: &str) -> WantedValue {
        let signs = value.chars().map(|sign| (sign, sign != small_sign(sign) || sign != first_sign_of(sign.to_uppercase(), sign), small_sign(sign))).collect();
        WantedValue { signs, low: value.to_lowercase() }
    }

    fn ends_at(&self, text: &str, start: usize) -> Option<usize> {
        if text[..start].chars().next_back().is_some_and(is_word_sign) {
            return None;
        }
        let mut rest = text[start..].chars();
        let mut end = start;
        for (sign, cased, low) in &self.signs {
            let here = rest.next()?;
            let same = if *cased { small_sign(here) == *low || paired_signs(small_sign(here), *low) } else { here == *sign };
            if !same {
                return None;
            }
            end += here.len_utf8();
        }
        if rest.next().is_some_and(is_word_sign) {
            return None;
        }
        Some(end)
    }

    pub fn found(&self, text: &str, from: usize) -> Option<(usize, usize)> {
        text[from..].char_indices().map(|(at, _)| from + at).chain(std::iter::once(text.len())).find_map(|start| self.ends_at(text, start).map(|end| (start, end)))
    }

    pub fn places(&self, text: &str) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = Vec::new();
        let mut from = 0;
        while from <= text.len() {
            let Some((start, end)) = self.found(text, from) else {
                break;
            };
            out.push((start, end));
            from = if end > start { end } else { start + text[start..].chars().next().map_or(1, char::len_utf8) };
        }
        out
    }

    pub fn replaced(&self, text: &str, with: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut from = 0;
        for (start, end) in self.places(text) {
            out.push_str(&text[from..start]);
            out.push_str(with);
            from = end;
        }
        out.push_str(&text[from..]);
        out
    }

    pub fn stands(&self, text: &str) -> bool {
        self.found(text, 0).is_some()
    }

    pub fn stands_in_low(&self, text: &str, low_text: &str) -> bool {
        low_text.contains(self.low.as_str()) && self.stands(text)
    }
}
