use crate::cases::Case;
use crate::tools::explain::chain::{Facts, keeps_history, knowledge_base};
use crate::tools::explain::text::{LineKind, Topic, fact_numbers};
use crate::tools::explain::words::WordId;
use patterns::{because, source};
use std::sync::Arc;

pub struct LessonFacts;
source!(LessonFacts, "a topic may read the lessons of a kind as told facts, as a pupil reads the worked examples of a chapter: each lesson of the part that the tool learns from becomes the fact that the answer of its question is its answer, beside the facts that the library tells, so that the rules of the topic take what the lessons show (the actions of a one-word command, from the lessons where the command is that word) in place of a list written in the library; the lessons that the tool reads afterwards are never facts");

const ANSWER_OF: &str = "the answer of";
because!(ANSWER_OF, LessonFacts, "the words before the question of a lesson in the fact that it becomes");

const ANSWER_IS: &str = "is";
because!(ANSWER_IS, LessonFacts, "the word between the question of a lesson and its answer in the fact that it becomes");

pub fn reads_lessons(topic: &Topic) -> bool {
    topic.rules.iter().any(|rule| rule.kind == LineKind::ReadsLessons)
}
because!(reads_lessons, LessonFacts, "whether a topic reads the lessons of a kind as facts: only a topic whose text has the line that says so, so that no other topic changes");

fn lesson_fact(case: &Case) -> Vec<WordId> {
    let mut fact = fact_numbers(ANSWER_OF);
    fact.extend(fact_numbers(&case.question));
    fact.extend(fact_numbers(ANSWER_IS));
    fact.extend(fact_numbers(&case.answer));
    fact
}
because!(lesson_fact, LessonFacts, "the fact that one lesson becomes: the answer of its question, in the words of a told sentence, is its answer");

pub fn lesson_base(topic: &Topic, cases: &[Case]) -> Arc<Facts> {
    let mut known = topic.facts.clone();
    known.extend(cases.iter().map(lesson_fact));
    Arc::new(knowledge_base(&known, &topic.rules, keeps_history(&topic.rules, &topic.questions)))
}
because!(lesson_base, LessonFacts, "the facts that a topic knows before any story of a kind: its told facts and a fact for each lesson that the tool learns from, with every outcome of its rules on them, found once for the lessons of one learning");
