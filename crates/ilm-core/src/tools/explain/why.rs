use crate::base::words::answer_words;
use crate::tools::explain::chain::{Facts, Filling, chain_facts, empty_filling, fill, fit_into, holding, none_fits, told_with_question};
use crate::tools::explain::derive::{filled_text, reading_note, timed_readings};
use crate::tools::explain::text::{Condition, LogicPiece, Reply, Topic, fact_numbers};
use crate::tools::explain::words::{word_text, word_texts};
use patterns::{because, source};
use std::sync::Arc;

pub struct MissingStep;
source!(MissingStep, "the diagnosis of a missing answer, as the algorithmic debugging of Shapiro asks of a logic program why an answer that it should give is missing: of the answers of the library that could give the answer that a lesson wants, the one whose conditions, in the order in which the text writes them, hold furthest, and the first of its conditions under which no filling of the conditions before it holds any longer, which is the step that the facts of the story miss");

const MISSING_HEAD: &str = "the first missing step for the answer";
because!(MISSING_HEAD, MissingStep, "the first line of the diagnosis, before the wanted answer");

const MISSING_STEP: &str = "  ";
because!(MISSING_STEP, MissingStep, "the indent of the lines of a diagnosis under its first line");

const HELD_MARK: &str = "holds:";
because!(HELD_MARK, MissingStep, "the mark of a condition of the answer that holds with the facts of the story");

const MISSING_MARK: &str = "missing:";
because!(MISSING_MARK, MissingStep, "the mark of the first condition of the answer that no filling holds, with the words that the conditions before it filled");

pub const MISSING_AS_WRITTEN: &str = "missing, as the text writes it:";
because!(MISSING_AS_WRITTEN, MissingStep, "the mark of the first missing condition as the text writes it, with its open places as letters, so that the missing steps of many lessons are counted together");

const ABSENT_HOLDS: &str = "every condition holds, but one that must not hold holds too:";
because!(ABSENT_HOLDS, MissingStep, "the diagnosis of an answer whose conditions all hold while a condition that must be absent holds as well");

const OTHER_WORDS: &str = "every condition holds, and the answer comes out as";
because!(OTHER_WORDS, MissingStep, "the diagnosis of an answer with open places whose conditions hold and fill it with other words than the lesson wants");

const EARLIER_FIRST: &str = "every condition holds: an earlier answer of the library comes first";
because!(EARLIER_FIRST, MissingStep, "the diagnosis of an answer that holds, so that the model gave another one only because an answer before it in the library held first");

fn can_give(answer: &Reply, expected: &[String]) -> bool {
    let mut words: Vec<&str> = Vec::new();
    for logic_piece in &answer.words {
        match logic_piece {
            LogicPiece::Word(word) => words.push(word_text(*word)),
            _ => return true,
        }
    }
    answer_words(&words.join(" ")) == expected
}
because!(can_give, MissingStep, "whether an answer of the library can give the wanted words: an answer of fixed words gives them only when they are the wanted words, and an answer with open places may give any words that its conditions fill");

fn diagnosis(name: &str, answer: &Reply, facts: &Facts, start: &Filling, expected: &[String]) -> (usize, Vec<String>) {
    let conditions: Vec<&Condition> = answer.conditions.iter().collect();
    let mut held = 0usize;
    let mut shown = start.clone();
    for count in 1..=conditions.len() {
        let Some(filling) = holding(&conditions[..count], facts, start).into_iter().next() else { break };
        held = count;
        shown = filling;
    }
    let readings = timed_readings(&answer.conditions, start);
    let note = |at: usize| readings.get(at).copied().flatten().map_or_else(String::new, |open| reading_note(conditions[at], open, facts));
    let mut lines = vec![format!("{MISSING_STEP}{name}: {}", answer.text)];
    lines.extend(conditions[..held].iter().enumerate().map(|(at, condition)| format!("{MISSING_STEP}{HELD_MARK} {}{}", filled_text(condition, &shown), note(at))));
    if held < conditions.len() {
        lines.push(format!("{MISSING_STEP}{MISSING_MARK} {}{}", filled_text(conditions[held], &shown), note(held)));
        lines.push(format!("{MISSING_STEP}{MISSING_AS_WRITTEN} {name}: {}", filled_text(conditions[held], &empty_filling())));
        return (held, lines);
    }
    let whole = holding(&conditions, facts, start);
    let passing: Vec<&Filling> = whole.iter().filter(|filling| none_fits(&answer.absent, facts, filling)).collect();
    if passing.is_empty() {
        let absent = answer.absent.iter().map(|condition| filled_text(condition, &shown)).collect::<Vec<String>>().join("; ");
        lines.push(format!("{MISSING_STEP}{ABSENT_HOLDS} {absent}"));
        lines.push(format!("{MISSING_STEP}{MISSING_AS_WRITTEN} {name}: {ABSENT_HOLDS} {}", answer.absent.iter().map(|condition| filled_text(condition, &empty_filling())).collect::<Vec<String>>().join("; ")));
        return (held, lines);
    }
    let gives = passing.iter().filter_map(|filling| fill(&answer.words, filling)).map(|words| answer_words(&word_texts(&words).join(" "))).find(|words| words != expected);
    match gives {
        Some(other) if !passing.iter().filter_map(|filling| fill(&answer.words, filling)).any(|words| answer_words(&word_texts(&words).join(" ")) == expected) => {
            lines.push(format!("{MISSING_STEP}{OTHER_WORDS} {}", other.join(" ")));
            lines.push(format!("{MISSING_STEP}{MISSING_AS_WRITTEN} {name}: {OTHER_WORDS} other words"));
        }
        _ => {
            lines.push(format!("{MISSING_STEP}{EARLIER_FIRST}"));
            lines.push(format!("{MISSING_STEP}{MISSING_AS_WRITTEN} {name}: {EARLIER_FIRST}"));
        }
    }
    (held, lines)
}
because!(diagnosis, MissingStep, "the diagnosis of one answer of the library from one filling of the question: the conditions, in the order of the text, that hold together with the facts of the story, and the first one that does not, with the words that the conditions before it filled, a condition held at a time with which fact it reads (the state of its thing, or the newest fact of any thing while a place of that thing is open); when all hold, why the answer still did not come: a condition that must not hold holds, the answer comes out with other words, or an earlier answer of the library came first");

pub(super) fn missing_step(topic: &Topic, base: &Arc<Facts>, sentences: &[String], question: &str, expected: &[String]) -> Option<Vec<String>> {
    let asked = fact_numbers(question);
    let facts = chain_facts(&told_with_question(sentences, question, &topic.rules), base, &topic.rules, topic.rules_key);
    let mut best: Option<(usize, Vec<String>)> = None;
    for form in &topic.questions {
        let mut starts = Vec::new();
        fit_into(&form.question, &asked, &empty_filling(), &mut starts);
        for answer in form.replies.iter().filter(|answer| can_give(answer, expected)) {
            for start in &starts {
                let (held, lines) = diagnosis(&topic.name, answer, &facts, start, expected);
                if best.as_ref().is_none_or(|(known, _)| held > *known) {
                    best = Some((held, lines));
                }
            }
        }
    }
    let (_, lines) = best?;
    let mut out = vec![format!("{MISSING_HEAD} \"{}\":", expected.join(" "))];
    out.extend(lines);
    Some(out)
}
because!(missing_step, MissingStep, "why a topic did not give the answer that a lesson wants: of the answers of its question forms that fit the question and could give those words, the one whose conditions hold furthest with the facts of the story, with the conditions that hold and the first that does not; nothing when no question form of the topic fits the question or no answer can give the words");
