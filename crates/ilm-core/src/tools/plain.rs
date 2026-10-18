use crate::cases::Case;
use crate::tools::call::{AnswerMark, Learned, Learner, Lessons, Said, Tool};
use crate::base::sums::{CompensatedSum, compensated};
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use patterns::{because, source};
use std::collections::HashMap;

pub struct PlainCounting;
source!(PlainCounting, "naive Bayes, the independence model of Maron for the choice of a class from the words of a text: the answer with the best odds under the words of the question, the whole question, and the words of the last told sentence and of the one before it; each of these is one count and the counts are added as logarithms; it has no slots and points at nothing, so it is the line that a learner of patterns is compared with");

pub struct HalfCount;
source!(HalfCount, "the estimate of Krichevsky and Trofimov for the share of one of two outcomes: half a reading is added to the count and one reading to the total, so that a key that an answer never had is rare and not impossible");

const BEFORE_LESSONS: f64 = 0.5;
because!(BEFORE_LESSONS, HalfCount, "the reading that every answer has under every key before the lessons, as the estimate gives it for one of two outcomes");

const BOTH_OUTCOMES: f64 = 1.0;
because!(BOTH_OUTCOMES, HalfCount, "what the estimate adds to the readings of an answer: the readings before the lessons of the two outcomes together, a key that stands and a key that does not");

const WORD_OF_QUESTION: &str = "q:";
because!(WORD_OF_QUESTION, PlainCounting, "the mark of a key that is one word of the question");

const WHOLE_QUESTION: &str = "Q:";
because!(WHOLE_QUESTION, PlainCounting, "the mark of the key that holds every word of the question in its order, so that a question that comes again word for word has a count of its own");

const TOLD_BACK: &[&str] = &["last:", "before:"];
because!(TOLD_BACK, PlainCounting, "the marks of the keys that are a word of the last told sentence and of the sentence before it, in this order");

const KEY_BETWEEN: &str = " ";
because!(KEY_BETWEEN, PlainCounting, "the mark between the words of the whole question in its key");

fn once(found: Vec<String>) -> Vec<String> {
    let mut kept: Vec<String> = Vec::with_capacity(found.len());
    for word in found {
        if !kept.contains(&word) {
            kept.push(word);
        }
    }
    kept
}
because!(once, PlainCounting, "the different words of a text in the order of their first place, so that a word counts one time for a case");

fn keys_of(sentences: &[String], question: &str) -> Vec<String> {
    let asked = words(question);
    let mut keys: Vec<String> = once(asked.clone()).iter().map(|word| format!("{WORD_OF_QUESTION}{word}")).collect();
    keys.push(format!("{WHOLE_QUESTION}{}", asked.join(KEY_BETWEEN)));
    for (back, mark) in TOLD_BACK.iter().enumerate() {
        if sentences.len() > back {
            let told = &sentences[sentences.len() - 1 - back];
            keys.extend(once(words(told)).iter().map(|word| format!("{mark}{word}")));
        }
    }
    keys
}
because!(keys_of, PlainCounting, "the keys of a case: each different word of the question, the whole question, each different word of the last told sentence and of the one before it");

pub struct Plain {
    keys: Symbols,
    answers: Vec<Vec<String>>,
    readings: Vec<u32>,
    total: u32,
    counts: Vec<Vec<(u32, u32)>>,
    unseen: Vec<f64>,
}
because!(Plain, PlainCounting, HalfCount, CompensatedSum, "the counts of plain counting: the answers in the order of their first lesson with their readings, for each key the readings of each answer that had it, and for each answer the logarithm of its estimate under a key that it never had. learn reads each lesson once; best gives the answer with the highest sum of logarithms for a case, the first of equals in the order of the lessons, and it sums the keys first, with the lost parts of the additions kept, and adds the share of the answer to that sum: answers with equal counts then have equal sums, and the first of them is taken");

fn estimate_log(count: u32, readings: f64) -> f64 {
    ((f64::from(count) + BEFORE_LESSONS) / (readings + BOTH_OUTCOMES)).ln()
}
because!(estimate_log, HalfCount, "the logarithm of the estimate of an answer under a key: its count under the key and half a reading, over its readings and one reading");

fn hits_by_answer(rows: &[&Vec<(u32, u32)>], answers: usize) -> (Vec<usize>, Vec<(usize, u32)>) {
    let mut start: Vec<usize> = vec![0; answers + 1];
    for row in rows {
        for (seen, _) in row.iter() {
            start[*seen as usize + 1] += 1;
        }
    }
    for answer in 0..answers {
        start[answer + 1] += start[answer];
    }
    let mut fill = start.clone();
    let mut hits: Vec<(usize, u32)> = vec![(0, 0); start[answers]];
    for (place, row) in rows.iter().enumerate() {
        for (seen, count) in row.iter() {
            let at = &mut fill[*seen as usize];
            hits[*at] = (place, *count);
            *at += 1;
        }
    }
    (start, hits)
}
because!(hits_by_answer, PlainCounting, "the counts of the keys of a case grouped by answer, each group in the order of the keys: the keys that an answer had are found by a walk, and every other key of the answer has the estimate of a key that it never had");

pub(crate) fn bump<T: PartialEq>(row: &mut Vec<(T, u32)>, answer: T) {
    match row.iter_mut().find(|(seen, _)| *seen == answer) {
        Some((_, count)) => *count += 1,
        None => row.push((answer, 1)),
    }
}
because!(bump, PlainCounting, "one more reading of an answer under a key, in a short list of answers with their counts");

impl Plain {
    pub fn learn(cases: &[Case]) -> Plain {
        let mut plain = Plain { keys: Symbols::new(), answers: Vec::new(), readings: Vec::new(), total: 0, counts: Vec::new(), unseen: Vec::new() };
        let mut answer_of: HashMap<Vec<String>, u32> = HashMap::new();
        for case in cases {
            let said = answer_words(&case.answer);
            let fresh = plain.answers.len() as u32;
            let answer = *answer_of.entry(said.clone()).or_insert(fresh);
            if answer == fresh {
                plain.answers.push(said);
                plain.readings.push(0);
            }
            plain.readings[answer as usize] += 1;
            plain.total += 1;
            for key in keys_of(&case.sentences, &case.question) {
                let number = plain.keys.number(&key) as usize;
                if number == plain.counts.len() {
                    plain.counts.push(Vec::new());
                }
                bump(&mut plain.counts[number], answer);
            }
        }
        plain.unseen = plain.readings.iter().map(|readings| estimate_log(0, f64::from(*readings))).collect();
        plain
    }

    pub fn best(&self, sentences: &[String], question: &str) -> Option<&[String]> {
        let rows: Vec<&Vec<(u32, u32)>> = keys_of(sentences, question).iter().filter_map(|key| self.keys.find(key)).map(|number| &self.counts[number as usize]).collect();
        let (start, hits) = hits_by_answer(&rows, self.readings.len());
        let mut best: Option<(usize, f64)> = None;
        for (answer, readings) in self.readings.iter().enumerate() {
            let readings = f64::from(*readings);
            let mine = &hits[start[answer]..start[answer + 1]];
            let mut next = 0usize;
            let under = compensated((0..rows.len()).map(|place| match mine.get(next) {
                Some((held, count)) if *held == place => {
                    next += 1;
                    estimate_log(*count, readings)
                }
                _ => self.unseen[answer],
            }));
            let score = (readings / f64::from(self.total)).ln() + under;
            if best.is_none_or(|(_, top)| score > top) {
                best = Some((answer, score));
            }
        }
        best.map(|(answer, _)| self.answers[answer].as_slice())
    }
}

impl Learner for Plain {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        plain_said(self.best(sentences, question))
    }
}

fn plain_said(best: Option<&[String]>) -> Said {
    let words = best.map(<[String]>::to_vec).unwrap_or_default();
    let mark = if words.is_empty() { AnswerMark::NoAnswer } else { AnswerMark::Guess };
    Said { words, note: String::new(), mark }
}
because!(plain_said, PlainCounting, "the answer of plain counting, marked as a guess: a count of the lessons proves nothing of the case");

pub fn learn_plain(lessons: Lessons, _lanes: usize) -> Learned {
    Ok(Box::new(Plain::learn(lessons.cases)))
}
because!(learn_plain, PlainCounting, "plain counting as a tool of the one call, so that the line which every learner is compared with can be measured as a learner is");

pub const PLAIN_TOOL: Tool = Tool { name: "plain", needs_labels: false, learn: learn_plain };
because!(PLAIN_TOOL, PlainCounting, "plain counting on the whole case, as a tool with a name");

pub struct QuestionAlone {
    counts: Plain,
}
because!(QuestionAlone, PlainCounting, "plain counting that learns from the questions and answers of the lessons with every told sentence taken away, and reads a question in the same way: what the question alone gives");

impl Learner for QuestionAlone {
    fn answer(&self, _sentences: &[String], question: &str) -> Said {
        plain_said(self.counts.best(&[], question))
    }
}

pub fn learn_question(lessons: Lessons, _lanes: usize) -> Learned {
    let bare: Vec<Case> = lessons.cases.iter().map(|lesson| Case { sentences: Vec::new(), question: lesson.question.clone(), answer: lesson.answer.clone() }).collect();
    Ok(Box::new(QuestionAlone { counts: Plain::learn(&bare) }))
}
because!(learn_question, PlainCounting, "plain counting of the question alone as a tool of the one call");

pub const QUESTION_TOOL: Tool = Tool { name: "question", needs_labels: false, learn: learn_question };
because!(QUESTION_TOOL, PlainCounting, "plain counting of the question alone, as a tool with a name");
