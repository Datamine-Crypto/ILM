use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said, Tool};
use crate::tools::pattern::PATTERNS_TOOL;
use crate::tools::pattern::learn::PatternLearner;
use patterns::{because, source};
use std::cmp::Ordering;

pub struct OrdinalReading;
source!(OrdinalReading, "the ordinal scale of Stevens: a reading that keeps the order of amounts and drops their size. Each number of a case is read as its rank among the different numbers that its told text and its question hold, so a form that compares two amounts, two years or two distances is one form whatever the amounts are, and the pattern learner counts it on the texts read in this way");

const RANK_MARK: &str = "rank";
because!(RANK_MARK, OrdinalReading, "the start of the word that stands for a number by its rank; the rank follows it, so the word cutter keeps the whole as one word");

const NO_NUMBERS: &str = "the ranks tool does not read this kind: no lesson orders two different numbers in its told text and its question";
because!(NO_NUMBERS, OrdinalReading, "what the tool says for lessons where no case holds two different numbers: an order needs two amounts, and with fewer the reading only loses the amounts");

const MARK_IN_TEXT: &str = "the ranks tool does not read this kind: a text of its lessons holds a word of the form of a rank word";
because!(MARK_IN_TEXT, OrdinalReading, "what the tool says when a word of the lessons has the form of a rank word: such a word could be taken for a number, so the tool reads nothing rather than read it wrong");

const RANK_NOTE_BETWEEN: &str = ": ";
because!(RANK_NOTE_BETWEEN, OrdinalReading, "the mark between the name of the reading and the note of the pattern learner in the note of an answer");

const NOTE_NAME: &str = "numbers by rank";
because!(NOTE_NAME, OrdinalReading, "the start of the note of an answer, which says that the numbers of the case were read by their ranks");

fn runs(text: &str) -> Vec<(usize, usize, bool)> {
    let bytes = text.as_bytes();
    let mut found: Vec<(usize, usize, bool)> = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        if !bytes[at].is_ascii_alphanumeric() {
            at += 1;
            continue;
        }
        let start = at;
        while at < bytes.len() && bytes[at].is_ascii_alphanumeric() {
            at += 1;
        }
        found.push((start, at, bytes[start..at].iter().all(u8::is_ascii_digit)));
    }
    found
}
because!(runs, OrdinalReading, "the words of a text as the word cutter finds them, as places in the text, each with whether it is all digits: a run of letters and digits is one word, and a word of digits alone is a number");

const ZERO_SIGN: char = '0';
because!(ZERO_SIGN, OrdinalReading, "the digit that adds nothing to a whole number when it stands before its first other digit");

fn value_of(digits: &str) -> &str {
    let kept = digits.trim_start_matches(ZERO_SIGN);
    if kept.is_empty() { &digits[digits.len() - 1..] } else { kept }
}
because!(value_of, OrdinalReading, "the digits of a number with no zero before its first other digit, so that one amount written in two ways is one value");

fn by_value(left: &str, right: &str) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}
because!(by_value, OrdinalReading, "the order of two whole numbers given as their digits: the one with more digits is larger, and of two with as many digits the order of the digits decides; so a number of any length is ordered and none overflows");

fn rank_word(rank: usize) -> String {
    format!("{RANK_MARK}{rank}")
}
because!(rank_word, OrdinalReading, "the word that stands for the number of a rank, the smallest number having the first rank");

fn is_rank_word(word: &str) -> bool {
    word.len() > RANK_MARK.len() && word.to_ascii_lowercase().starts_with(RANK_MARK) && word.as_bytes()[RANK_MARK.len()..].iter().all(u8::is_ascii_digit)
}
because!(is_rank_word, OrdinalReading, "whether a word has the form of a rank word, as the cutter gives it in small letters");

#[derive(Clone, Debug, Default)]
struct Ranked {
    values: Vec<String>,
    written: Vec<String>,
}
because!(Ranked, OrdinalReading, "the different numbers of one case in rising order, each as its value and as the case first wrote it; the rank of a number is its place in this list, counted from one. of reads a case; rewrite writes a text with each number that the case holds as its rank word; back gives the number for a rank word");

impl Ranked {
    fn of(sentences: &[String], question: &str) -> Ranked {
        let mut ranked = Ranked::default();
        for text in sentences.iter().map(String::as_str).chain(std::iter::once(question)) {
            for (start, end, number) in runs(text) {
                if !number {
                    continue;
                }
                let value = value_of(&text[start..end]);
                if let Err(place) = ranked.values.binary_search_by(|held| by_value(held, value)) {
                    ranked.values.insert(place, value.to_string());
                    ranked.written.insert(place, text[start..end].to_string());
                }
            }
        }
        ranked
    }

    fn rank(&self, digits: &str) -> Option<usize> {
        self.values.binary_search_by(|held| by_value(held, value_of(digits))).ok().map(|place| place + 1)
    }

    fn rewrite(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut from = 0usize;
        for (start, end, number) in runs(text) {
            if !number {
                continue;
            }
            let Some(rank) = self.rank(&text[start..end]) else { continue };
            out.push_str(&text[from..start]);
            out.push_str(&rank_word(rank));
            from = end;
        }
        out.push_str(&text[from..]);
        out
    }

    fn back(&self, word: &str) -> Option<&str> {
        if !is_rank_word(word) {
            return None;
        }
        let rank: usize = word[RANK_MARK.len()..].parse().ok()?;
        self.written.get(rank.checked_sub(1)?).map(String::as_str)
    }
}

fn holds_rank_word(case: &Case) -> bool {
    case.sentences.iter().chain([&case.question, &case.answer]).any(|text| runs(text).iter().any(|(start, end, _)| is_rank_word(&text[*start..*end])))
}
because!(holds_rank_word, OrdinalReading, "whether a text of a case holds a word of the form of a rank word");

pub struct RankedPatterns {
    inner: PatternLearner,
}
because!(RankedPatterns, OrdinalReading, "the pattern learner over texts whose numbers are read by their ranks: a case is read in the same way, and a rank word of its answer is given as the number of that rank in the case");

impl Learner for RankedPatterns {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let ranked = Ranked::of(sentences, question);
        let told: Vec<String> = sentences.iter().map(|sentence| ranked.rewrite(sentence)).collect();
        let (words, note) = self.inner.said(&told, &ranked.rewrite(question));
        let words = words.into_iter().map(|word| ranked.back(&word).map_or(word, str::to_string)).collect();
        Said { words, note: [NOTE_NAME, note.as_str()].join(RANK_NOTE_BETWEEN) }
    }
}

pub fn learn_ranks(lessons: Lessons, lanes: usize) -> Learned {
    if lessons.cases.iter().any(holds_rank_word) {
        return Err(MARK_IN_TEXT.to_string());
    }
    let mut ordered = false;
    let cases: Vec<Case> = lessons
        .cases
        .iter()
        .map(|case| {
            let ranked = Ranked::of(&case.sentences, &case.question);
            ordered |= ranked.values.len() > 1;
            Case { sentences: case.sentences.iter().map(|sentence| ranked.rewrite(sentence)).collect(), question: ranked.rewrite(&case.question), answer: ranked.rewrite(&case.answer) }
        })
        .collect();
    if !ordered {
        return Err(NO_NUMBERS.to_string());
    }
    Ok(Box::new(RankedPatterns { inner: PatternLearner::learn(&cases, lanes) }))
}
because!(learn_ranks, OrdinalReading, "the tool on lessons: each lesson is read with the ranks of its own numbers, also in its answer, where a number that the told text and the question do not hold stays as it is; the pattern learner learns from the lessons read in this way. The tool says that it does not read a kind where no lesson orders two numbers, or where a word of a text has the form of a rank word");

pub const RANKS_TOOL: Tool = Tool { name: "ranks", needs_labels: false, probe: false, asks: false, over: Some(PATTERNS_TOOL.name), learn: learn_ranks };
because!(RANKS_TOOL, OrdinalReading, "the tool in the list of the tools: it needs no labels, and it gives no control by the form of the question, because the rank of a number of the question depends on the numbers of the told text");
