use crate::base::exact::Amount;
use crate::base::first_sight::NumberBag;
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::names::WordTexts;
use patterns::{because, source};

pub struct ProblemMarks;
source!(ProblemMarks, "what a tree may ask of a problem names a place and never a word of the problem: the count of numbers; for a word that many lessons hold, whether it stands in the question, in the sentence of a number, or one or two tokens before or after the number; the unit of a number; whether a number stands in the question; how the sentence of a number starts; and for two numbers whether they have one unit, which is larger, which stands first, whether one divides the other, whether they stand in one sentence and whether their sentences start with one word");

pub const LEAST_MARK_LESSONS: u32 = 5;
because!(LEAST_MARK_LESSONS, ProblemMarks, "a word is a mark as it is when this many lessons or more hold it; a rarer word would make a mark that no tree can count");

pub const MOST_TEXTS_SHARE: f64 = 0.1;
because!(MOST_TEXTS_SHARE, ProblemMarks, "a word that this share of the lessons or more holds is a word of most texts: it is no unit and no name of the question");

const UNIT_REACH: usize = 3;
because!(UNIT_REACH, ProblemMarks, "the unit of a number stands within this many tokens after it");

const NEAR_PLACES: &[i64] = &[-2, -1, 1, 2];
because!(NEAR_PLACES, ProblemMarks, "the places next to a number whose words are marks: one or two tokens before it and after it");

const PLURAL_ENDS: &[&str] = &["es", "s"];
because!(PLURAL_ENDS, ProblemMarks, "the ends of a plural whose singular, when the same problem holds it, is the word that a unit or a name is read as");

const STEM_LEAST: usize = 2;
because!(STEM_LEAST, ProblemMarks, "a plural end is taken off only when more than this many letters are left, so that a short word is not cut");

pub struct ProblemView {
    ask: usize,
    local: NumberBag<u32>,
    asked: NumberBag<u32>,
}
because!(ProblemView, ProblemMarks, "what the marks of a problem start from: the place of the mark of the question, the words of the problem, and the words that the question names (a word of the question that the story holds too and that is no word of most texts)");

#[derive(Clone, Debug, Default)]
pub struct MarkReader {
    pub words: PatternWords,
    pub as_mark: NumberBag<u32>,
    pub most_texts: NumberBag<u32>,
    pub end: u32,
    pub ask: u32,
}
because!(MarkReader, ProblemMarks, "the reader of the marks of a problem: the word model of the pattern learner, the words that are marks as they are, the words of most texts, and the tokens of the end of a sentence and of the start of the question. view reads where a problem asks and what it names; marks gives the marks of the numbers of a problem in one order; marks_one gives the marks of one number");

pub fn number_name(at: usize, by_size: bool) -> String {
    match (by_size, at) {
        (true, 0) => "the largest number".to_string(),
        (true, _) => format!("number {} by size", at + 1),
        (false, _) => format!("number {}", at + 1),
    }
}
because!(number_name, ProblemMarks, "the name of a number in a mark: its place in the order of the text, or its place by size");

pub fn by_size_order(amounts: &[Amount]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..amounts.len()).collect();
    order.sort_by(|one, other| amounts[*other].cmp(&amounts[*one]).then(one.cmp(other)));
    order
}
because!(by_size_order, ProblemMarks, "the places of the numbers from the largest to the smallest, equal numbers in the order of the text");

pub fn is_letter_token<N: WordTexts>(names: &N, word: u32) -> bool {
    names.name(word).as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
}
because!(is_letter_token, ProblemMarks, "whether a token is a word: it starts with a letter");

impl MarkReader {
    pub fn norm<N: WordTexts>(&self, names: &N, word: u32, local: &NumberBag<u32>) -> u32 {
        let found = self.words.norm(word);
        let text = names.name(found);
        for end in PLURAL_ENDS {
            if let Some(stem) = text.strip_suffix(end)
                && stem.len() > STEM_LEAST
                && let Some(base) = names.find(stem)
                && local.contains(&base)
            {
                return base;
            }
        }
        found
    }

    pub fn view<N: WordTexts>(&self, names: &N, words: &[u32]) -> ProblemView {
        let ask = words.iter().rposition(|word| *word == self.ask).unwrap_or_default();
        let local: NumberBag<u32> = words.iter().copied().filter(|word| is_letter_token(names, *word)).collect();
        let told: NumberBag<u32> = words[..ask].iter().filter(|word| is_letter_token(names, **word)).map(|word| self.norm(names, *word, &local)).collect();
        let asked: NumberBag<u32> = words[ask + 1..].iter().filter(|word| is_letter_token(names, **word) && !self.most_texts.contains(*word)).map(|word| self.norm(names, *word, &local)).filter(|word| told.contains(word)).collect();
        ProblemView { ask, local, asked }
    }

    fn around<N: WordTexts>(&self, names: &N, words: &[u32], place: usize, view: &ProblemView, name: &str, out: &mut Vec<String>) -> (Option<u32>, (usize, usize)) {
        let is_end = |word: u32| word == self.end || word == self.ask;
        let start = (0..place).rev().find(|at| is_end(words[*at])).map_or(0, |at| at + 1);
        let end = (place..words.len()).find(|at| is_end(words[*at])).unwrap_or(words.len());
        for word in &words[start..end] {
            if self.as_mark.contains(word) {
                out.push(format!("in the sentence of {name}: {}", names.name(*word)));
            }
        }
        if start < end && is_letter_token(names, words[start]) {
            let first = self.norm(names, words[start], &view.local);
            let how = if self.most_texts.contains(&words[start]) {
                "a word of most texts"
            } else if view.asked.contains(&first) {
                "a word that the question names"
            } else {
                "another word"
            };
            out.push(format!("the sentence of {name} starts with {how}"));
        }
        for step in NEAR_PLACES {
            let at = place as i64 + *step;
            if (start as i64) <= at && at < end as i64 && self.as_mark.contains(&words[at as usize]) {
                let side = if *step < 0 { "before" } else { "after" };
                out.push(format!("{} {side} {name}: {}", step.unsigned_abs(), names.name(words[at as usize])));
            }
        }
        let reach = (place + 1 + UNIT_REACH).min(end);
        let unit = (place + 1..reach).find(|at| is_letter_token(names, words[*at]) && !self.most_texts.contains(&words[*at])).map(|at| self.norm(names, words[at], &view.local));
        let how = match unit {
            Some(found) if view.asked.contains(&found) => "a word that the question names",
            Some(_) => "another word",
            None => "not told",
        };
        out.push(format!("the unit of {name} is {how}"));
        if place > view.ask {
            out.push(format!("{name} stands in the question"));
        }
        (unit, (start, end))
    }

    fn question_marks<N: WordTexts>(&self, names: &N, words: &[u32], view: &ProblemView, count: usize, out: &mut Vec<String>) {
        out.push(format!("numbers: {count}"));
        for word in &words[view.ask + 1..] {
            if self.as_mark.contains(word) {
                out.push(format!("in the question: {}", names.name(*word)));
            }
        }
    }

    pub fn marks<N: WordTexts>(&self, names: &N, words: &[u32], values: &[Option<Amount>], by_size: bool) -> Vec<String> {
        let view = self.view(names, words);
        let mut at: Vec<usize> = (0..values.len()).filter(|place| values[*place].is_some()).collect();
        if by_size {
            let amounts: Vec<Amount> = at.iter().filter_map(|place| values[*place]).collect();
            at = by_size_order(&amounts).into_iter().map(|place| at[place]).collect();
        }
        let mut out: Vec<String> = Vec::new();
        self.question_marks(names, words, &view, at.len(), &mut out);
        let mut units: Vec<Option<u32>> = Vec::with_capacity(at.len());
        let mut spans: Vec<(usize, usize)> = Vec::with_capacity(at.len());
        for (place, token) in at.iter().enumerate() {
            let (unit, span) = self.around(names, words, *token, &view, &number_name(place, by_size), &mut out);
            units.push(unit);
            spans.push(span);
        }
        for one in 0..at.len() {
            for other in one + 1..at.len() {
                let (first, second) = (number_name(one, by_size), number_name(other, by_size));
                if spans[one] == spans[other] {
                    out.push(format!("{first} and {second} stand in one sentence"));
                } else if words[spans[one].0] == words[spans[other].0] {
                    out.push(format!("the sentences of {first} and {second} start with one word"));
                }
                if units[one].is_some() && units[one] == units[other] {
                    out.push(format!("{first} and {second} have one unit"));
                }
                let (Some(left), Some(right)) = (values[at[one]], values[at[other]]) else { continue };
                if !by_size && left > right {
                    out.push(format!("{first} is larger than {second}"));
                }
                if by_size && at[one] < at[other] {
                    out.push(format!("{first} stands before {second} in the text"));
                }
                if left == right {
                    out.push(format!("{first} is as large as {second}"));
                }
                if right != Amount::whole(0) && left.over(right).is_some_and(|got| got.is_whole()) {
                    out.push(format!("{second} divides {first}"));
                }
                if left != Amount::whole(0) && right.over(left).is_some_and(|got| got.is_whole()) {
                    out.push(format!("{first} divides {second}"));
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn marks_one<N: WordTexts>(&self, names: &N, words: &[u32], values: &[Option<Amount>], at: &[usize], own: usize) -> Vec<String> {
        let view = self.view(names, words);
        let mut out: Vec<String> = Vec::new();
        let (unit, span) = self.around(names, words, at[own], &view, "the number", &mut out);
        self.question_marks(names, words, &view, at.len(), &mut out);
        let mine = values[at[own]];
        let others: Vec<Option<Amount>> = at.iter().enumerate().filter(|(place, _)| *place != own).map(|(_, token)| values[*token]).collect();
        if own == 0 {
            out.push("the number is the first number".to_string());
        }
        if own + 1 == at.len() {
            out.push("the number is the last number".to_string());
        }
        if !others.is_empty() && others.iter().all(|other| mine > *other) {
            out.push("the number is the largest number".to_string());
        }
        if !others.is_empty() && others.iter().all(|other| mine < *other) {
            out.push("the number is the smallest number".to_string());
        }
        for (place, token) in at.iter().enumerate() {
            if place == own {
                continue;
            }
            let mut other: Vec<String> = Vec::new();
            let (unit_other, span_other) = self.around(names, words, *token, &view, "another number", &mut other);
            if unit.is_some() && unit == unit_other {
                out.push("another number has the unit of the number".to_string());
            }
            if span == span_other {
                out.push("another number stands in the sentence of the number".to_string());
            }
            out.extend(other.into_iter().filter(|mark| mark.starts_with("the unit of another number")));
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}
