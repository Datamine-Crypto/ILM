use crate::base::keys::KeyNumbers;
use crate::base::lanes::{on_lanes, shards};
use crate::base::runs::RunNumbers;
use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::split::told_texts;
use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::rule::deduce::{RuleClause, rule_add, rule_at_end, rule_state};
use crate::tools::rule::forms::{RULE_ANY, RuleFact, RulePart, RuleRole, rule_labels};
use crate::tools::rule::lessons::{RuleLesson, rule_lesson_sentences};
use crate::tools::rule::marks::{RuleClass, rule_mark_order};
use crate::tools::rule::pieces::RuleHand;
use crate::tools::rule::tables::{RULE_ALL_WAYS, RULE_BOTH, RULE_SURE, RULE_WAYS, RuleReader, RuleWay, rule_thing_words};
use crate::tools::rule::tokens::{RuleMarks, RuleSentence, rule_sentence};
use patterns::{because, source};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SecondReading;
source!(SecondReading, "the tables of a reader are counted on the first part of the lesson theories and read on the part that is held back: a key that reads too many of its sentences wrong there is dropped, and a way of reading is switched off when too few of the sentences that needed it are right. The model that answers counts its tables on all lessons. The meaning of a condition with no, and the answer for each state of a statement at the end of a theory, are taken from the lessons too");

static RULE_MODELS: AtomicU64 = AtomicU64::new(0);
because!(RULE_MODELS, SecondReading, "the models that this run of the program made: each model gets the next number, so that a theory which a thread keeps for the questions of one story is never taken for the theory of another model");

const RULE_USABLE: f64 = 0.9;
because!(RULE_USABLE, SecondReading, "the least share of right sentences for the reading by the marks of the sentence: it is the last way, and without it a sentence is not read at all, so it may fail more often than a cut");

const RULE_FIRST_STORIES: usize = 5;
because!(RULE_FIRST_STORIES, SecondReading, "when this many first stories of the lessons have no formal form, the kind has none, and the tool says the most frequent answer");

const RULE_EARLY: usize = 2;
because!(RULE_EARLY, SecondReading, "of the equal parts of the lesson theories, this many are the first part, where the tables of the second reading are counted");

const RULE_THIRDS: usize = 3;
because!(RULE_THIRDS, SecondReading, "the equal parts in which the lesson theories are counted for the second reading; the last one is held back to test the tables");

#[derive(Debug, Default)]
struct RuleStory {
    told: Vec<RuleLesson>,
    statements: Vec<RuleLesson>,
    stated: Vec<(String, Vec<String>)>,
    asked: Vec<(RuleFact, Vec<String>)>,
    facts: Vec<RuleFact>,
    rules: Vec<RuleClause>,
}
because!(RuleStory, SecondReading, "one lesson theory: its told sentences with their formal forms, its different statements as lesson sentences, the text and the answer of each of these, every statement that was asked with its answer, and the facts and rules of the theory as the set gives them");

#[derive(Debug, Default)]
struct RuleTheory {
    model: u64,
    sentences: Vec<String>,
    strangers: Symbols,
    hand: RuleHand,
    end: KeyNumbers,
    unread: usize,
    facts: usize,
    rules: usize,
}
because!(RuleTheory, SecondReading, "a told theory as one thread read it, kept for the next questions of the same story: the model and the sentences that it belongs to, the tokens and the things that no lesson held, the facts at the end, and how much was read");

thread_local! {
    static RULE_THEORIES: RefCell<RuleTheory> = RefCell::new(RuleTheory::default());
}

type RuleSays = Vec<(u128, Vec<String>)>;
because!(RuleSays, SecondReading, "the answer of each state of a statement at the end of a theory: the most frequent lesson answer of the state");

#[derive(Debug)]
pub(super) struct RuleModel {
    id: u64,
    symbols: Symbols,
    terms: RunNumbers,
    default: Vec<String>,
    reader: Option<RuleReader>,
    closed: bool,
    says: RuleSays,
}
because!(RuleModel, SecondReading, "what the tool learned: the reader with its tables, the meaning of a condition with no, the answer of each state of a statement, and the most frequent answer of the lessons for a statement that is not read or a state that no lesson had. A theory is read once for all questions of its story; the note of an answer says how many facts and rules were read and how many sentences were not");

fn rule_default(cases: &[Case]) -> Vec<String> {
    let mut answers: Vec<(Vec<String>, u32)> = Vec::new();
    for case in cases {
        rule_bump(&mut answers, answer_words(&case.answer));
    }
    rule_most(&answers).map(|(words, _)| words.clone()).unwrap_or_default()
}
because!(rule_default, SecondReading, "the most frequent answer of the lessons, for a statement that is not read");

fn rule_bump(answers: &mut Vec<(Vec<String>, u32)>, words: Vec<String>) {
    match answers.iter_mut().find(|(held, _)| *held == words) {
        Some((_, count)) => *count += 1,
        None => answers.push((words, 1)),
    }
}
because!(rule_bump, SecondReading, "one more lesson with an answer, in a short list of answers with their counts");

fn rule_most(answers: &[(Vec<String>, u32)]) -> Option<&(Vec<String>, u32)> {
    answers.iter().max_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)))
}
because!(rule_most, SecondReading, "the most frequent answer of a list; of answers with equal counts the one whose words come last in the order of texts, so the choice does not depend on the order of the lessons");

fn rule_same_forms(read: &[RulePart], parts: &[RulePart]) -> bool {
    let inside = |left: &[RulePart], right: &[RulePart]| left.iter().all(|(fact, role)| right.iter().any(|(other, held)| other == fact && (*held == RuleRole::Head) == (*role == RuleRole::Head)));
    let all_of = |parts: &[RulePart], role: RuleRole| parts.iter().all(|(_, held)| *held == role);
    let goes_on = |parts: &[RulePart]| !parts.is_empty() && all_of(parts, RuleRole::More);
    inside(read, parts) && inside(parts, read) && all_of(read, RuleRole::Fact) == all_of(parts, RuleRole::Fact) && goes_on(read) == goes_on(parts)
}
because!(rule_same_forms, SecondReading, "does a reading give the formal forms of the set: the same facts as heads, the same facts as other parts, and the same make (all facts, or all more of the rule before)");

fn rule_lessons_of(stories: &[RuleStory]) -> Vec<&RuleLesson> {
    stories.iter().flat_map(|story| story.told.iter().chain(&story.statements)).collect()
}
because!(rule_lessons_of, SecondReading, "the lesson sentences of some theories: the told sentences of each with their forms, then its different statements");

impl RuleModel {
    fn stories(lessons: Lessons, symbols: &mut Symbols, terms: &mut RunNumbers, marks: &RuleMarks) -> Vec<RuleStory> {
        let cases = lessons.cases;
        let (keys, sizes) = told_texts(cases);
        let mut first: Vec<Option<usize>> = vec![None; sizes.len()];
        for (place, key) in keys.iter().enumerate() {
            first[*key].get_or_insert(place);
        }
        let mut stories: Vec<RuleStory> = Vec::new();
        let mut story_of: Vec<Option<usize>> = vec![None; sizes.len()];
        let mut seen = 0usize;
        for (key, place) in first.iter().enumerate() {
            let Some(place) = place else { continue };
            if cases[*place].sentences.is_empty() {
                continue;
            }
            if seen == RULE_FIRST_STORIES && stories.is_empty() {
                break;
            }
            seen += 1;
            let Some(lines) = lessons.labels.get(*place) else { continue };
            let labels = rule_labels(lines, terms, &mut |word: &str| symbols.number(word));
            if labels.units.is_empty() {
                continue;
            }
            let sentences: Vec<RuleSentence> = cases[*place].sentences.iter().map(|text| rule_sentence(text, &mut |word: &str| symbols.number(word))).collect();
            let mut story = RuleStory { told: rule_lesson_sentences(&sentences, &labels.units, marks, terms), ..RuleStory::default() };
            for item in labels.units.iter().flat_map(|unit| &unit.items) {
                rule_add(item, &mut story.facts, &mut story.rules);
            }
            story_of[key] = Some(stories.len());
            stories.push(story);
        }
        for (place, case) in cases.iter().enumerate() {
            let Some(story) = story_of[keys[place]] else { continue };
            let Some(lines) = lessons.labels.get(place) else { continue };
            let Some(fact) = rule_labels(lines, terms, &mut |word: &str| symbols.number(word)).statement else { continue };
            let words = answer_words(&case.answer);
            let story = &mut stories[story];
            story.asked.push((fact, words.clone()));
            let held = story.stated.iter().zip(&story.statements).any(|((question, answer), lesson)| *question == case.question && *answer == words && lesson.parts.first().is_some_and(|(other, _)| *other == fact));
            if !held {
                story.stated.push((case.question.clone(), words));
                story.statements.push(RuleLesson { sent: rule_sentence(&case.question, &mut |word: &str| symbols.number(word)), parts: vec![(fact, RuleRole::Fact)], before: None, told: false });
            }
        }
        stories
    }

    fn second_reading(reader: &mut RuleReader, read: &[&RuleLesson], terms: &RunNumbers, lanes: usize) {
        let mut new = [0u32; RULE_BOTH];
        for lesson in read {
            let (subjects, things) = rule_thing_words(&lesson.parts, terms);
            for word in &lesson.sent.toks {
                if reader.seen_of(*word) == 0 {
                    new[usize::from(subjects.contains(word) || things.contains(word))] += 1;
                }
            }
        }
        let [marks, slots] = new;
        reader.new_is_slot = slots >= marks;
        let runs = shards(read.len(), lanes);
        let tried = {
            let reader = &*reader;
            on_lanes(runs.len(), lanes, &|run| {
                let mut hand = RuleHand::default();
                let mut found: Vec<(usize, u32, bool)> = Vec::new();
                for lesson in &read[runs[run].clone()] {
                    let got = reader.sentence(&lesson.sent, lesson.before, lesson.told, terms, &mut hand);
                    let right = got.as_ref().is_some_and(|got| rule_same_forms(&got.parts, &lesson.parts));
                    let used = got.map(|got| got.used).unwrap_or_default();
                    for (place, entry) in used.iter().enumerate() {
                        if !used[..place].contains(entry) {
                            found.push((entry.0, entry.1, right));
                        }
                    }
                }
                found
            })
        };
        let mut counts: Vec<Vec<[u32; RULE_BOTH]>> = reader.tables.iter().map(|table| vec![[0; RULE_BOTH]; table.stats.len()]).collect();
        for (table, number, right) in tried.into_iter().flatten() {
            counts[table][number as usize][usize::from(right)] += 1;
        }
        for (table, numbers) in counts.iter().enumerate() {
            for (number, [wrong, right]) in numbers.iter().enumerate() {
                if *wrong > 0 && f64::from(*wrong) > (f64::from(1u8) - RULE_SURE) * f64::from(wrong + right) {
                    reader.tables[table].dropped[number] = true;
                }
            }
        }
        let by_way = {
            let reader = &*reader;
            on_lanes(runs.len(), lanes, &|run| {
                let mut hand = RuleHand::default();
                let mut counts = [[0u32; RULE_BOTH]; RULE_WAYS];
                for lesson in &read[runs[run].clone()] {
                    if let Some(got) = reader.sentence(&lesson.sent, lesson.before, lesson.told, terms, &mut hand) {
                        counts[got.way as usize][usize::from(rule_same_forms(&got.parts, &lesson.parts))] += 1;
                    }
                }
                counts
            })
        };
        for way in RULE_ALL_WAYS {
            let [wrong, right] = by_way.iter().fold([0u32; RULE_BOTH], |[wrong, right], counts| [wrong + counts[*way as usize][0], right + counts[*way as usize][1]]);
            let needed = if *way == RuleWay::Marks { RULE_USABLE } else { RULE_SURE };
            reader.ways[*way as usize] = f64::from(right) >= needed * f64::from(right + wrong);
        }
    }

    fn meaning(stories: &[RuleStory], lanes: usize) -> (bool, RuleSays) {
        let says_no = stories.iter().any(|story| story.facts.iter().chain(story.rules.iter().map(|(_, head)| head)).any(|fact| !fact.yes));
        let mut best: Option<(u32, bool, RuleSays)> = None;
        for closed in [!says_no, says_no] {
            let ends = on_lanes(stories.len(), lanes, &|place| {
                let story = &stories[place];
                if story.asked.is_empty() {
                    return Vec::new();
                }
                let end = rule_at_end(&story.facts, &story.rules, closed);
                story.asked.iter().map(|(fact, _)| rule_state(fact, &end)).collect::<Vec<u128>>()
            });
            let mut states: Vec<(u128, Vec<(Vec<String>, u32)>)> = Vec::new();
            for (story, found) in stories.iter().zip(&ends) {
                for ((_, answer), state) in story.asked.iter().zip(found) {
                    let place = match states.iter().position(|(held, _)| held == state) {
                        Some(place) => place,
                        None => {
                            states.push((*state, Vec::new()));
                            states.len() - 1
                        }
                    };
                    rule_bump(&mut states[place].1, answer.clone());
                }
            }
            let mut right = 0u32;
            let mut says: RuleSays = Vec::new();
            for (state, answers) in &states {
                if let Some((words, count)) = rule_most(answers) {
                    says.push((*state, words.clone()));
                    right += count;
                }
            }
            if best.as_ref().is_none_or(|(held, _, _)| right > *held) {
                best = Some((right, closed, says));
            }
        }
        best.map(|(_, closed, says)| (closed, says)).unwrap_or_default()
    }

    fn learn(lessons: Lessons, lanes: usize) -> RuleModel {
        let mut symbols = Symbols::new();
        let mut terms = RunNumbers::default();
        let marks = RuleMarks::new(&mut symbols);
        let stories = RuleModel::stories(lessons, &mut symbols, &mut terms, &marks);
        let mut model = RuleModel { id: RULE_MODELS.fetch_add(1, Ordering::Relaxed) + 1, symbols, terms, default: rule_default(lessons.cases), reader: None, closed: false, says: Vec::new() };
        if stories.is_empty() {
            return model;
        }
        let order = rule_mark_order(&model.symbols);
        let (first, last) = stories.split_at(stories.len() * RULE_EARLY / RULE_THIRDS);
        let parts: Vec<&[RuleStory]> = if first.is_empty() || last.is_empty() { vec![&stories] } else { vec![&stories, first] };
        let mut readers = on_lanes(parts.len(), lanes, &|part| RuleReader::count(&rule_lessons_of(parts[part]), &model.symbols, &marks, &model.terms, &order)).into_iter();
        let Some(mut reader) = readers.next() else { return model };
        if let Some(mut early) = readers.next() {
            RuleModel::second_reading(&mut early, &rule_lessons_of(last), &model.terms, lanes);
            reader.ways = early.ways;
            reader.new_is_slot = early.new_is_slot;
            for (table, held) in early.tables.iter().enumerate() {
                for (number, dropped) in held.dropped.iter().enumerate() {
                    if *dropped && let Some(place) = reader.tables[table].runs.find(held.runs.run(number as u32)) {
                        reader.tables[table].dropped[place as usize] = true;
                    }
                }
            }
        }
        (model.closed, model.says) = RuleModel::meaning(&stories, lanes);
        model.reader = Some(reader);
        model
    }

    fn theory(&self, reader: &RuleReader, sentences: &[String]) -> RuleTheory {
        let mut theory = RuleTheory { model: self.id, sentences: sentences.to_vec(), ..RuleTheory::default() };
        let known = self.symbols.len() as u32;
        let strangers = &mut theory.strangers;
        let mut told: Vec<RuleSentence> = sentences.iter().map(|text| rule_sentence(text, &mut |word: &str| self.symbols.find(word).unwrap_or_else(|| known + strangers.number(word)))).collect();
        if reader.usual.is_some() {
            let count = told.len();
            let mut later = false;
            for (number, sent) in told.iter_mut().enumerate().rev() {
                sent.left = Some((count - 1 - number) as u32);
                sent.later = Some(later);
                later = later || reader.classes(sent).contains(&RuleClass::Subject);
            }
        }
        let mut facts: Vec<RuleFact> = Vec::new();
        let mut rules: Vec<RuleClause> = Vec::new();
        let mut before: Option<u32> = None;
        for sent in &told {
            match reader.sentence(sent, before, true, &self.terms, &mut theory.hand) {
                Some(read) => {
                    before = read.before;
                    rule_add(&read.parts, &mut facts, &mut rules);
                }
                None => theory.unread += 1,
            }
        }
        theory.end = rule_at_end(&facts, &rules, self.closed);
        theory.facts = facts.len();
        theory.rules = rules.len();
        theory
    }

    fn stated(&self, reader: &RuleReader, theory: &mut RuleTheory, question: &str) -> Said {
        let known = self.symbols.len() as u32;
        let strangers = &mut theory.strangers;
        let sent = rule_sentence(question, &mut |word: &str| self.symbols.find(word).unwrap_or_else(|| known + strangers.number(word)));
        let note = format!("{} facts and {} rules read, {} of {} sentences not read", theory.facts, theory.rules, theory.unread, theory.sentences.len());
        let read = reader.sentence(&sent, None, false, &self.terms, &mut theory.hand);
        let fact = match read.as_ref().map(|read| read.parts.as_slice()) {
            Some([(fact, RuleRole::Fact)]) if fact.subject != RULE_ANY => *fact,
            _ => return Said { words: self.default.clone(), note: format!("{note}; the statement was not read") },
        };
        let state = rule_state(&fact, &theory.end);
        match self.says.iter().find(|(held, _)| *held == state) {
            Some((_, words)) => Said { words: words.clone(), note: format!("{note}; the statement was read") },
            None => Said { words: self.default.clone(), note: format!("{note}; no lesson had the state of the statement") },
        }
    }
}

impl Learner for RuleModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let Some(reader) = &self.reader else { return Said { words: self.default.clone(), note: String::new() } };
        RULE_THEORIES.with_borrow_mut(|theory| {
            if theory.model != self.id || theory.sentences != sentences {
                *theory = self.theory(reader, sentences);
            }
            self.stated(reader, theory, question)
        })
    }
}

pub(super) fn learn_rule(lessons: Lessons, lanes: usize) -> Learned {
    Ok(Box::new(RuleModel::learn(lessons, lanes)))
}
because!(learn_rule, SecondReading, "the rule tool as a tool of the one call: it learns from lessons that have formal forms of their told sentences, and it says the most frequent answer of the lessons where the first stories have none");
