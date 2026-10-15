use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::cases::Case;
use patterns::{because, source};

pub struct MostLessons;
source!(MostLessons, "the answer that most lessons have: a learner says it for a case that none of its rules reads, and a way that is no sure rule is used only when it is right more often than this answer. Of answers with equal counts the one that came first in the order of the lessons is taken, so the same lessons give the same answer in every run");

const FREQUENT_BETWEEN: &str = " ";
because!(FREQUENT_BETWEEN, MostLessons, "the mark between the words of an answer in the key by which equal answers are counted together");

pub fn frequent_answer(cases: &[Case]) -> (Vec<String>, f64) {
    let mut answers = Symbols::new();
    let mut counts: Vec<u32> = Vec::new();
    let mut said: Vec<Vec<String>> = Vec::new();
    for case in cases {
        let words = answer_words(&case.answer);
        let number = answers.number(&words.join(FREQUENT_BETWEEN)) as usize;
        if number == said.len() {
            said.push(words);
            counts.push(0);
        }
        counts[number] += 1;
    }
    let mut best: Option<usize> = None;
    for (number, count) in counts.iter().enumerate() {
        if best.is_none_or(|held| *count > counts[held]) {
            best = Some(number);
        }
    }
    match best {
        Some(number) => (said[number].clone(), f64::from(counts[number]) / (cases.len().max(1) as f64)),
        None => (Vec::new(), f64::default()),
    }
}
because!(frequent_answer, MostLessons, "the most frequent answer of the lessons in words, and the share of the lessons that have it; no words and no share for no lessons");
