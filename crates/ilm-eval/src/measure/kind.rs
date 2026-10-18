use ilm_core::base::lanes::{BUDGET_STOPPED, MOST_LEARNING_WORK, in_work_scope_offering};
use ilm_core::tools::explain::costs::{close_costs, open_costs};
use ilm_core::cases::Case;
use ilm_core::classes::WordClasses;
use ilm_core::choice::{picked, picked_labels, take};
use ilm_core::base::lanes::on_spare_lanes;
use ilm_core::base::shares::share;
use ilm_core::tools::call::{AnswerMark, Learner, Lessons, Reading, Tool};
use ilm_core::tools::plain::Plain;
use crate::measure::question_list::question_list_share;
use ilm_core::split::{parts, told_texts};
use ilm_core::base::words::{answer_words, words};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use std::time::Instant;

pub struct ExactMeasure;
source!(ExactMeasure, "the measure of a kind of task: a model learns from the learned part and reads the read part, and a case is right only when the words of the answer are exactly the words of the true answer; beside the model stand plain controls, and a kind is learned only when the model is at the bar and clearly above every control, so a kind whose answers need no reading of the text is not learned");

pub struct NewWords;
source!(NewWords, "the test of a learned step with words that the lessons never held: the names, places and things of the read cases are replaced by new words, the same word by the same new word in the story, the question and the answer; a model that learned the step and not the words answers as before");

pub const LEARNED: f64 = 95.0;
because!(LEARNED, ExactMeasure, "the bar of a learned kind in hundredths of the read cases: high enough that a rule which holds for a part of the cases does not pass, with room for the faults that the public sets themselves hold");

pub const ABOVE: f64 = 5.0;
because!(ABOVE, ExactMeasure, "a learned kind is this many hundredths or more above every control: a difference that the chance of a read part of a few hundred cases does not give");

const LEAST_READ: usize = 100;
because!(LEAST_READ, ExactMeasure, "a kind is measured only with this many read cases or more: with fewer, a single case moves the share by more than a hundredth");

const LEAST_FINAL: usize = 50;
because!(LEAST_FINAL, ExactMeasure, "the final part of a kind is read with this many cases or more: it is smaller than a read part because cases that another kind has touched are taken out of it");

const LEAST_SMALL: usize = 1;
because!(LEAST_SMALL, ExactMeasure, "the least read part when a run asks to read a part of any size: a kind of an official test split is read however few cases it has, as every case of the split counts in its total");

const LEAST_LESSONS: usize = 25;
because!(LEAST_LESSONS, ExactMeasure, "a kind is measured only with this many lessons or more: the smallest size of the curve of lessons");

const EMPTY_MOST: f64 = 0.02;
because!(EMPTY_MOST, ExactMeasure, "a kind has no result when more than this share of its answers hold no word for the cutter of words: such answers are signs that the measure cannot compare");

const LEAST_FRESH: usize = 50;
because!(LEAST_FRESH, ExactMeasure, "with copies of lessons among the read cases, the share on the other read cases counts, and only when there are this many of them or more");

const SIZES: &[usize] = &[25, 50, 100, 200, 400, 600, 1000, 2000];
because!(SIZES, ExactMeasure, "the numbers of lessons of the curve: each about the double of the one before, from the least lessons of a measured kind to more than a kind of the board learns from");

const SHOWN_TOLD: usize = 6;
because!(SHOWN_TOLD, ExactMeasure, "the last told sentences that are printed with a wrong lesson case: enough to see the step that went wrong, and a line that a person still reads");

const SHOWN_PARTS: usize = 2;
because!(SHOWN_PARTS, ExactMeasure, "of the parts of the lessons, this many are learned from when wrong lesson cases are shown, and the rest is read");

const SHOWN_OF: usize = 3;
because!(SHOWN_OF, ExactMeasure, "the parts that the lessons are cut in when wrong lesson cases are shown");

const TOO_FEW: &str = "too few cases";
because!(TOO_FEW, ExactMeasure, "the note of a kind with too few lessons or too few cases to read");

const NO_WORD_NOTE: &str = "answers hold no word for the tokeniser: no result";
because!(NO_WORD_NOTE, ExactMeasure, "the note of a kind whose answers the cutter of words cannot read");

const CHAIN_WORK_HEAD: &str = "chain work ";
because!(CHAIN_WORK_HEAD, ExactMeasure, "the words before the work of the chains of the learning of a kind in its line of costs (facts read and fillings taken, the work that the bound of a learning counts)");

const CHAIN_WORK_STOPPED: &str = ", stopped at the budget";
because!(CHAIN_WORK_STOPPED, ExactMeasure, "the mark after the work of the chains of a learning whose learners stopped at the bound of the work of one learning");

const CHAIN_WORK_BETWEEN: &str = " | ";
because!(CHAIN_WORK_BETWEEN, ExactMeasure, "the mark between the costs of a learning and the work of its chains, and before the mark of a learning stopped at its bound");

const LEARNER_FAILED: &str = "the run of the learner ended with an error";
because!(LEARNER_FAILED, ExactMeasure, "the note of a kind where the learner or its model broke, before where it broke; a kind where no tool gave a model has what the tools said as its note; the controls of the kind are still given");

const LEARNER_BROKE: &str = "the learner broke while it learned";
because!(LEARNER_BROKE, ExactMeasure, "what the note of a kind says when the learner broke while it learned, so that it said nothing");

const MODEL_BROKE: &str = "the model broke while it read the read part";
because!(MODEL_BROKE, ExactMeasure, "what the note of a kind says when its model broke while it read the read part");

#[derive(Clone, Debug, Default)]
pub struct Row {
    pub kind: String,
    pub cases: usize,
    pub learn: usize,
    pub read: usize,
    pub note: String,
    pub costs: String,
    pub spent_ways: Vec<(&'static str, f64)>,
    pub copies: usize,
    pub answers: usize,
    pub one_answer: f64,
    pub plain: f64,
    pub question: f64,
    pub learner: Option<f64>,
    pub not_copies: f64,
    pub fit_seconds: f64,
    pub all_seconds: f64,
    pub tool: String,
    pub stopped: Option<(String, String)>,
    pub bound_stops: usize,
    pub learning_work: usize,
    pub budget_stopped: bool,
    pub marks: MarkCounts,
    pub one_case: Option<(f64, f64)>,
    pub named: Option<(usize, usize)>,
    pub labels: bool,
    pub curriculum: bool,
    pub curriculum_from: Vec<usize>,
    pub stored_from: Vec<String>,
    pub reading: Option<Reading>,
    pub rules: (usize, usize),
    pub facts: (usize, usize),
    pub asked: Option<(usize, usize)>,
    pub new_words: Option<(f64, f64, usize)>,
    pub default_shares: Option<(f64, f64)>,
    pub curve: Vec<(usize, f64)>,
    pub wrong: Vec<String>,
}
because!(Row, ExactMeasure, "what the measure gives for one kind: the sizes of its parts, the controls (one answer for all, plain counting, the question alone), the share of the model on the read part and on the read cases that are no copies of lessons, the tool that was taken, how its model reads the kind (learned, written or mixed), how many rules of the library (given) and rules learned from the lessons (discovered) the proofs of its answers rest on, how many readings of the read part ended at a bound of the model's work, the answers of the read part by their marks (by proof, by default and guesses, each right and wrong, and no answer), the share of the cases whose told text the kind tells once in the lessons and in the read part (none for a final part), how many learned forms of the model name a word that a form with fewer fixed words reads too, and what the asked extras gave (new words, with the shares of the read cases answered by a default without and with them, the curve by lessons, wrong lesson cases, questions of the tool); and where the learning spent its time, as a line and by way; and the places, in the order of its set before it, of the kinds whose lessons gave lines of its model; and the kinds, other than itself, whose lines of the known-good store its learning took, as a task that rests on them stands on more than its own lessons");

#[derive(Clone, Copy, Debug)]
pub struct Asked {
    pub fit: usize,
    pub held: usize,
    pub sealed: bool,
    pub new_words: bool,
    pub curve: bool,
    pub show: usize,
    pub questions: usize,
    pub tool: Option<usize>,
    pub lanes: usize,
    pub small: bool,
    pub control_fit: Option<usize>,
}
because!(Asked, ExactMeasure, "what a run of the measure asks for: the most lessons and the most read cases, the final part in place of the read part, the extras, one tool alone in place of the choice among all, the threads, whether a read part of any size is read, as the official test split of a benchmark is read whole however few cases a kind of it has, and for a run that learns from fewer lessons than its set gives, the lessons of the set from which the controls are still learned, so that a weaker control does not make a kind look learned");

#[derive(Clone, Copy)]
pub struct Ground<'a> {
    pub name: &'a str,
    pub cases: &'a [Case],
    pub labels: &'a [Vec<String>],
    pub start: Option<usize>,
    pub stories: bool,
    pub final_part: Option<&'a [Case]>,
    pub sealed: Option<&'a [Case]>,
    pub new_words: Option<&'a [(String, String)]>,
    pub answerer: Option<&'a (dyn Fn(&str) -> Option<String> + Sync)>,
    pub classes: Option<&'a WordClasses>,
    pub before: Option<&'a (dyn Fn() -> Vec<Vec<Case>> + Sync)>,
    pub stored: bool,
}
because!(Ground, ExactMeasure, "what a kind gives to the measure: its name, its cases with their label lines, the place of the first case to read for a kind of two halves, whether lessons are counted in stories, the final part when it is read, the sealed part when it is not the one of the cut, the table of new words, for a kind that a program made a way to get the answer of any text, the table of word classes of the run, and a way to read the lessons of the kinds before it in its set; and whether its learning may take lines of the known-good store, as a run that must learn from its own lessons alone may not");

fn most_frequent(lessons: &[Case]) -> (Vec<String>, usize) {
    let mut place: HashMap<Vec<String>, usize> = HashMap::new();
    let mut seen: Vec<(Vec<String>, usize)> = Vec::new();
    for lesson in lessons {
        let said = answer_words(&lesson.answer);
        match place.get(&said) {
            Some(at) => seen[*at].1 += 1,
            None => {
                place.insert(said.clone(), seen.len());
                seen.push((said, 1));
            }
        }
    }
    let mut best: Option<&(Vec<String>, usize)> = None;
    for entry in &seen {
        if best.is_none_or(|top| entry.1 > top.1) {
            best = Some(entry);
        }
    }
    (best.map(|(answer, _)| answer.clone()).unwrap_or_default(), seen.len())
}
because!(most_frequent, ExactMeasure, "the answer that the lessons have most often, the first of equals in the order of the lessons, and the number of different answers: the control that gives one answer for every case");

fn right_and_fresh(model: &dyn Learner, read: &[Case], truths: &[Vec<String>], copy: &[bool], lanes: usize) -> ReadCounts {
    let counted = on_spare_lanes(read.len(), lanes, &|place| {
        let said = model.answer(&read[place].sentences, &read[place].question);
        let right = said.words == truths[place];
        ReadCounts { right: usize::from(right), fresh: usize::from(right && !copy[place]), marks: MarkCounts::of(said.mark, right) }
    });
    counted.into_iter().fold(ReadCounts::default(), |sum, one| ReadCounts { right: sum.right + one.right, fresh: sum.fresh + one.fresh, marks: sum.marks.added(one.marks) })
}

#[derive(Clone, Copy, Default)]
struct ReadCounts {
    right: usize,
    fresh: usize,
    marks: MarkCounts,
}
because!(ReadCounts, ExactMeasure, "what the reading of the read part counts: the cases answered right, those of them that are no copies of lessons, and the answers by their marks");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MarkCounts {
    pub proof: (usize, usize),
    pub default: (usize, usize),
    pub guess: (usize, usize),
    pub none: usize,
}
because!(MarkCounts, ExactMeasure, "the answers of the read part by their marks: by proof, by default and guesses, each right and wrong, and the cases with no answer, so that the board shows whether an answer that claims a proof can be trusted and how many answers claim one");

impl MarkCounts {
    fn of(mark: AnswerMark, right: bool) -> MarkCounts {
        let one = (usize::from(right), usize::from(!right));
        match mark {
            AnswerMark::Proof => MarkCounts { proof: one, ..MarkCounts::default() },
            AnswerMark::Default => MarkCounts { default: one, ..MarkCounts::default() },
            AnswerMark::Guess => MarkCounts { guess: one, ..MarkCounts::default() },
            AnswerMark::NoAnswer => MarkCounts { none: 1, ..MarkCounts::default() },
        }
    }

    pub fn added(self, other: MarkCounts) -> MarkCounts {
        let sum = |one: (usize, usize), two: (usize, usize)| (one.0 + two.0, one.1 + two.1);
        MarkCounts { proof: sum(self.proof, other.proof), default: sum(self.default, other.default), guess: sum(self.guess, other.guess), none: self.none + other.none }
    }

    pub fn read_cases(&self) -> usize {
        self.proof.0 + self.proof.1 + self.default.0 + self.default.1 + self.guess.0 + self.guess.1 + self.none
    }

    pub fn shares(&self) -> String {
        let all = self.read_cases();
        let right = |(good, bad): (usize, usize)| if good + bad > usize::default() { format!("{:.places$}", share(good, good + bad), places = SHARE_PLACES) } else { NO_MARKED.to_string() };
        format!(
            "proof {:.places$} ({} right), default {:.places$}, guess {:.places$} ({} right), none {:.places$}",
            share(self.proof.0 + self.proof.1, all),
            right(self.proof),
            share(self.default.0 + self.default.1, all),
            share(self.guess.0 + self.guess.1, all),
            right(self.guess),
            share(self.none, all),
            places = SHARE_PLACES
        )
    }
}

impl MarkCounts {
    pub fn counts(&self) -> String {
        format!(
            "proof {} ({} wrong), default {} ({} wrong), guess {} ({} wrong), none {}",
            self.proof.0 + self.proof.1,
            self.proof.1,
            self.default.0 + self.default.1,
            self.default.1,
            self.guess.0 + self.guess.1,
            self.guess.1,
            self.none
        )
    }
}

const NO_MARKED: &str = "-";
because!(NO_MARKED, ExactMeasure, "what the share right of the answers of a mark says when no answer has that mark");

fn one_case_shares(cases: &[Case], learn: &[usize], read: &[usize]) -> (f64, f64) {
    let (keys, sizes) = told_texts(cases);
    let once = |part: &[usize]| share(part.iter().filter(|at| sizes[keys[**at]] == 1).count(), part.len());
    (once(learn), once(read))
}
because!(one_case_shares, ExactMeasure, "the share of the cases whose told text the kind tells once, in the lessons and in the read part: the cut takes whole told texts, the lessons those seen first and the read part those seen last, so where told texts of many cases and told texts of one case are different kinds of story, the read part may hold mostly one kind and the lessons the other");

pub const DEFAULT_RISE_POINTS: f64 = 5.0;
because!(DEFAULT_RISE_POINTS, NewWords, "the points by which the share of the read cases answered by a default must grow with new words before the board names the kind: as many as a model must stand above its controls: a reading that needs the words of its lessons sends cases to the answer otherwise (lonli_negation without its text, by the names of names.txt: seven points)");

pub const READ_UNLIKE_POINTS: f64 = 30.0;
because!(READ_UNLIKE_POINTS, ExactMeasure, "the points by which the share of told texts told once in the read part must differ from that share in the lessons before the measure says that the read part is unlike the lessons: a few points come by chance, while a read part of one kind of story beside lessons of another differs by most of the scale (lonli_quantifier: four and ninety-seven)");
fn rules_of(model: &dyn Learner, read: &[Case], lanes: usize) -> ((usize, usize), (usize, usize)) {
    let picked: Vec<&Case> = read.iter().step_by((read.len() / RULE_CASES).max(1)).take(RULE_CASES).collect();
    let used = on_spare_lanes(picked.len(), lanes, &|place| (model.rules_used(&picked[place].sentences, &picked[place].question), model.facts_used(&picked[place].sentences, &picked[place].question)));
    let counted = |lines: Vec<&Vec<(String, bool)>>| {
        let given: HashSet<&str> = lines.iter().copied().flatten().filter(|(_, held)| *held).map(|(line, _)| line.as_str()).collect();
        let found: HashSet<&str> = lines.iter().copied().flatten().filter(|(_, held)| !*held).map(|(line, _)| line.as_str()).collect();
        (given.len(), found.len())
    };
    (counted(used.iter().map(|(rules, _)| rules).collect()), counted(used.iter().map(|(_, facts)| facts).collect()))
}
because!(rules_of, ExactMeasure, "how many rules and how many known facts of the library (given), and how many that no text holds (discovered, as the model learned them from the lessons), the proofs of the answers of some read cases rest on, each rule once");

const RULE_CASES: usize = 20;
because!(RULE_CASES, ExactMeasure, "the read cases, spread over the read part, whose proofs tell which rules a model rests on: a rule that the kind needs stands in many of its proofs, and each proof chains a story again");

because!(right_and_fresh, ExactMeasure, "the read cases that a model answers right, and those of them that are no copies of lessons, the answers that the model gave by a default, right and wrong, and the cases that it gave no answer; the cases are read in runs on the threads");

pub fn renamed(text: &str, table: &HashMap<&str, &str>) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut at = 0usize;
    while at < bytes.len() {
        let mut end = at;
        while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
            end += 1;
        }
        if end == at {
            let mut next = at + 1;
            while next < bytes.len() && !bytes[next].is_ascii_alphabetic() {
                next += 1;
            }
            out.push_str(&text[at..next]);
            at = next;
            continue;
        }
        let word = &text[at..end];
        match table.get(word.to_ascii_lowercase().as_str()) {
            None => out.push_str(word),
            Some(new) if bytes[at].is_ascii_uppercase() => {
                let mut signs = new.chars();
                if let Some(first) = signs.next() {
                    out.extend(first.to_uppercase());
                    out.push_str(&signs.as_str().to_lowercase());
                }
            }
            Some(new) => out.push_str(new),
        }
        at = end;
    }
    out
}
because!(renamed, NewWords, "a text with each word of the table replaced by its new word: a word is a run of letters, it is looked up in small letters, and the new word starts with a capital letter when the old one did");

fn with_new_words(model: &dyn Learner, learn: &[Case], read: &[Case], pairs: &[(String, String)], lanes: usize) -> ((f64, f64, usize), f64) {
    let table: HashMap<&str, &str> = pairs.iter().map(|(old, new)| (old.as_str(), new.as_str())).collect();
    let mut held: HashSet<String> = HashSet::new();
    for lesson in learn {
        for text in lesson.sentences.iter().chain([&lesson.question, &lesson.answer]) {
            held.extend(words(text));
        }
    }
    let known = table.values().filter(|new| held.contains(**new)).count();
    let counted = on_spare_lanes(read.len(), lanes, &|place| {
        let case = &read[place];
        let sentences: Vec<String> = case.sentences.iter().map(|sentence| renamed(sentence, &table)).collect();
        let question = renamed(&case.question, &table);
        let answer = renamed(&case.answer, &table);
        let changed = sentences != case.sentences || question != case.question || answer != case.answer;
        let said = model.answer(&sentences, &question);
        (usize::from(said.words == answer_words(&answer)), usize::from(changed), usize::from(said.mark == AnswerMark::Default))
    });
    let (right, changed, guessed) = counted.into_iter().fold((0, 0, 0), |sum, one| (sum.0 + one.0, sum.1 + one.1, sum.2 + one.2));
    ((share(right, read.len()), share(changed, read.len()), known), share(guessed, read.len()))
}
because!(with_new_words, NewWords, "the read cases again with the new words: the share right, the share of the cases that the new words changed, and how many of the new words stand in the lessons after all; and the share of the read cases that the model answers by a default with the new words, which grows when a reading needs the words of its lessons (names that a list knows) and falls back to the answer otherwise");

struct Made {
    model: Box<dyn Learner>,
    tool: String,
    stopped: Option<(String, String)>,
    labels: bool,
    curriculum: bool,
    kinds_before: Vec<usize>,
}
because!(Made, ExactMeasure, "the model of a kind as the measure reads it: the model, the name of its tool, the tools that stopped before it, whether it got labels of its lessons, whether it learned over the kinds before its kind in its set, and the places of those that gave it lines in the order of the set");

fn made(tools: &[Tool], lessons: Lessons, asked: &Asked) -> Result<Made, String> {
    match asked.tool {
        None => {
            let quiet = Lessons { ask: None, ..lessons };
            let taken = take(tools, quiet, asked.lanes, false)?;
            let stopped: Vec<&str> = taken.stopped.iter().map(|(tool, _)| tools[*tool].name).collect();
            let stopped = (!stopped.is_empty()).then(|| (stopped.join(", "), tools[taken.tool].name.to_string()));
            let curriculum = taken.model.used_kinds_before();
            let kinds_before = taken.model.kinds_used_before();
            Ok(Made { model: taken.model, tool: tools[taken.tool].name.to_string(), stopped, labels: taken.labels, curriculum, kinds_before })
        }
        Some(tool) => {
            let model = (tools[tool].learn)(lessons, asked.lanes)?;
            let curriculum = model.used_kinds_before();
            let kinds_before = model.kinds_used_before();
            Ok(Made { model, tool: tools[tool].name.to_string(), stopped: None, labels: tools[tool].needs_labels && lessons.labelled(), curriculum, kinds_before })
        }
    }
}
because!(made, ExactMeasure, "the model of a kind: by the choice among all tools, or by one tool alone");

fn wrong_lessons(tools: &[Tool], lessons: Lessons, asked: &Asked) -> Vec<String> {
    let cut = lessons.cases.len() * SHOWN_PARTS / SHOWN_OF;
    let labels = if lessons.labels.is_empty() { lessons.labels } else { &lessons.labels[..cut] };
    let Ok(inner) = made(tools, Lessons { cases: &lessons.cases[..cut], labels, ask: None, classes: lessons.classes, before: lessons.before }, asked) else { return Vec::new() };
    let mut wrong: Vec<String> = Vec::new();
    for case in &lessons.cases[cut..] {
        let got = inner.model.answer(&case.sentences, &case.question).words;
        if got != answer_words(&case.answer) && wrong.len() < asked.show {
            let told = &case.sentences[case.sentences.len().saturating_sub(SHOWN_TOLD)..];
            wrong.push(format!("      WRONG (a lesson case, read by a model of the first two thirds of the lessons): {} | {} -> {}; said {}", told.join(" / "), case.question, case.answer, got.join(" ")));
        }
    }
    wrong
}
because!(wrong_lessons, ExactMeasure, "wrong cases for the design of a tool, taken from the lessons only: a model of the first lessons reads the last ones; no case of the read part is ever printed, because a tool is designed from lessons");

pub fn measure(ground: &Ground, tools: &[Tool], asked: &Asked) -> Row {
    let started = Instant::now();
    let cut = parts(ground.cases, asked.fit, asked.held, ground.start, ground.stories);
    let learn = picked(ground.cases, &cut.learn);
    let learn_labels = picked_labels(ground.labels, &cut.learn);
    let read: Vec<Case> = match (asked.sealed, ground.final_part) {
        (true, Some(given)) => given.to_vec(),
        (true, None) => picked(ground.cases, &cut.sealed),
        _ => picked(ground.cases, &cut.read),
    };
    let mut row = Row { kind: ground.name.to_string(), cases: ground.cases.len(), learn: learn.len(), read: read.len(), ..Row::default() };
    let least_read = if asked.small { LEAST_SMALL } else if asked.sealed { LEAST_FINAL } else { LEAST_READ };
    if read.len() < least_read || learn.len() < LEAST_LESSONS {
        row.note = TOO_FEW.to_string();
        return row;
    }
    let truths: Vec<Vec<String>> = read.iter().map(|case| answer_words(&case.answer)).collect();
    let empty = truths.iter().filter(|truth| truth.is_empty()).count() + learn.iter().filter(|lesson| answer_words(&lesson.answer).is_empty()).count();
    if (empty as f64) > EMPTY_MOST * ((read.len() + learn.len()) as f64) {
        row.note = format!("{empty} {NO_WORD_NOTE}");
        return row;
    }
    let seen: HashSet<(&[String], &str)> = learn.iter().map(|lesson| (lesson.sentences.as_slice(), lesson.question.as_str())).collect();
    let controls_from: Vec<Case> = match asked.control_fit {
        Some(full) => {
            let asked_cases: HashSet<usize> = cut.read.iter().chain(cut.sealed.iter()).copied().collect();
            parts(ground.cases, full, asked.held, ground.start, ground.stories).learn.into_iter().filter(|place| !asked_cases.contains(place)).map(|place| ground.cases[place].clone()).collect()
        }
        None => learn.clone(),
    };
    let copy: Vec<bool> = read.iter().map(|case| seen.contains(&(case.sentences.as_slice(), case.question.as_str()))).collect();
    let (most, answers) = most_frequent(&controls_from);
    row.copies = copy.iter().filter(|is| **is).count();
    row.one_answer = share(truths.iter().filter(|truth| **truth == most).count(), read.len());
    row.answers = answers;
    let base = Plain::learn(&controls_from);
    row.plain = share((0..read.len()).filter(|place| base.best(&read[*place].sentences, &read[*place].question) == Some(truths[*place].as_slice())).count(), read.len());
    let bare: Vec<Case> = controls_from.iter().map(|lesson| Case { sentences: Vec::new(), question: lesson.question.clone(), answer: lesson.answer.clone() }).collect();
    let alone = Plain::learn(&bare);
    row.question = share((0..read.len()).filter(|place| alone.best(&[], &read[*place].question) == Some(truths[*place].as_slice())).count(), read.len());
    if controls_from.iter().any(|lesson| !lesson.sentences.is_empty())
        && let Some(listed) = question_list_share(&controls_from, &read)
    {
        row.question = row.question.max(listed);
    }
    let fit_started = Instant::now();
    let questions: Mutex<(usize, usize)> = Mutex::new((0, 0));
    let closed: HashSet<Vec<String>> = if asked.questions > 0 && ground.answerer.is_some() {
        let sealed = match ground.sealed {
            Some(given) => given.to_vec(),
            None => picked(ground.cases, &cut.sealed),
        };
        read.iter().chain(sealed.iter()).map(|case| words(&case.question)).collect()
    } else {
        HashSet::new()
    };
    let ask = |text: &str| -> Option<String> {
        let answerer = ground.answerer?;
        let mut count = questions.lock().ok()?;
        if count.0 >= asked.questions {
            return None;
        }
        let said = if closed.contains(&words(text)) { None } else { answerer(text) };
        count.0 += 1;
        count.1 += usize::from(said.is_some());
        said
    };
    let asks = asked.questions > 0 && ground.answerer.is_some() && asked.tool.is_some();
    let lessons = Lessons { cases: &learn, labels: &learn_labels, ask: if asks { Some(&ask) } else { None }, classes: ground.classes, before: ground.before };
    open_costs();
    let worked = in_work_scope_offering(MOST_LEARNING_WORK, ground.stored && asked.control_fit.is_none(), || catch_unwind(AssertUnwindSafe(|| made(tools, lessons, asked))));
    let spent = close_costs();
    let learned = worked.value;
    row.learning_work = worked.work;
    row.budget_stopped = worked.stopped;
    row.stored_from = worked.taken.into_iter().filter(|kind| kind.as_str() != ground.name).collect();
    let work = format!("{CHAIN_WORK_HEAD}{}{}", worked.work, if worked.stopped { CHAIN_WORK_STOPPED } else { "" });
    row.costs = if spent.line.is_empty() { work } else { format!("{}{CHAIN_WORK_BETWEEN}{work}", spent.line) };
    row.spent_ways = spent.ways;
    let made_model = match learned {
        Ok(Ok(made_model)) => made_model,
        failed => {
            row.note = failed.map_or_else(|_| format!("{LEARNER_FAILED}: {LEARNER_BROKE}"), |said| said.err().unwrap_or_default());
            row.all_seconds = started.elapsed().as_secs_f64();
            return row;
        }
    };
    row.fit_seconds = fit_started.elapsed().as_secs_f64();
    row.tool = made_model.tool.clone();
    row.stopped = made_model.stopped.clone();
    row.labels = made_model.labels;
    row.curriculum = made_model.curriculum;
    row.curriculum_from = made_model.kinds_before.clone();
    row.reading = made_model.model.reading();
    if asks {
        row.asked = questions.lock().ok().map(|count| *count);
    }
    let model = made_model.model.as_ref();
    let stops_before = model.bound_stops();
    let Ok(ReadCounts { right, fresh, marks }) = catch_unwind(AssertUnwindSafe(|| right_and_fresh(model, &read, &truths, &copy, asked.lanes))) else {
        row.note = format!("{LEARNER_FAILED}: {MODEL_BROKE}");
        row.all_seconds = started.elapsed().as_secs_f64();
        return row;
    };
    if asked.show > 0 {
        row.wrong = wrong_lessons(tools, lessons, asked);
    }
    row.bound_stops = model.bound_stops().saturating_sub(stops_before);
    (row.rules, row.facts) = catch_unwind(AssertUnwindSafe(|| rules_of(model, &read, asked.lanes))).unwrap_or_default();
    row.marks = marks;
    row.one_case = (!asked.sealed).then(|| one_case_shares(ground.cases, &cut.learn, &cut.read));
    row.named = model.named_words();
    row.learner = Some(share(right, read.len()));
    row.not_copies = share(fresh, (read.len() - row.copies).max(1));
    if asked.curve {
        for size in SIZES {
            if *size >= learn.len() {
                break;
            }
            let labels = if learn_labels.is_empty() { &learn_labels[..] } else { &learn_labels[..*size] };
            if let Ok(small) = made(tools, Lessons { cases: &learn[..*size], labels, ask: None, classes: ground.classes, before: ground.before }, asked) {
                row.curve.push((*size, share(right_and_fresh(small.model.as_ref(), &read, &truths, &copy, asked.lanes).right, read.len())));
            }
        }
        row.curve.push((learn.len(), share(right, read.len())));
    }
    if let (true, Some(table)) = (asked.new_words, ground.new_words) {
        let (counts, guessed) = with_new_words(model, &learn, &read, table, asked.lanes);
        row.new_words = Some(counts);
        row.default_shares = Some((share(marks.default.0 + marks.default.1, read.len()), guessed));
    }
    row.all_seconds = started.elapsed().as_secs_f64();
    row
}
because!(measure, ExactMeasure, "one kind through the measure: the cut, the controls, the model from the learned part, its share on the read part, and the asked extras; labels of the set reach the model for its lessons alone, and a model that fails leaves the controls of the kind in the row; the known-good store reaches the model only when the run learns from as many lessons as its set gives, as a run of another number of lessons asks what those lessons teach alone, and stored lines that discovery found on other lessons would answer for them");

pub fn share_that_counts(row: &Row) -> Option<f64> {
    if row.copies > 0 {
        return if row.read - row.copies >= LEAST_FRESH { Some(row.not_copies) } else { None };
    }
    row.learner
}
because!(share_that_counts, ExactMeasure, "the share of the model that counts: on the read cases that are no copies of lessons when there are copies, and nothing when too few read cases are no copies");

pub fn learned(row: &Row) -> bool {
    row.learner.is_some() && share_that_counts(row).is_some_and(|counts| counts >= LEARNED && counts >= row.one_answer.max(row.question) + ABOVE)
}
because!(learned, ExactMeasure, "a kind is learned when the share that counts is at the bar and clearly above one answer for all and above the question alone");

const KIND_WIDTH: usize = 34;
because!(KIND_WIDTH, ExactMeasure, "the signs of the column of the kind in a line of the table: the longest names of the kinds of the public sets fit");

const CASES_WIDTH: usize = 6;
because!(CASES_WIDTH, ExactMeasure, "the signs of the column of the cases of a kind and of each share: a kind of some hundred thousand cases fits, and a share with its place after the point");

const PART_WIDTH: usize = 5;
because!(PART_WIDTH, ExactMeasure, "the signs of the columns of the parts of a kind, of the copies, of the answers and of the seconds");

pub const SHARE_PLACES: usize = 1;
because!(SHARE_PLACES, ExactMeasure, "the places after the point of a printed share: enough to show a difference of a single case in a read part of a thousand cases, and no more, so that noise is not read as a result");

const MARK_LEARNED: &str = "LEARNED";
because!(MARK_LEARNED, ExactMeasure, "the mark of a learned kind in a line of the table");

const MARK_COPIES: &str = "copies ";
because!(MARK_COPIES, ExactMeasure, "the mark of a kind that cannot be measured because too few read cases are no copies of lessons");

const MARK_NO_TEXT: &str = "no text";
because!(MARK_NO_TEXT, ExactMeasure, "the mark of a kind at the bar that a control reaches too: its answers need no reading of the text");

const MARK_NONE: &str = "       ";
because!(MARK_NONE, ExactMeasure, "the mark of a kind that is not learned, as wide as the other marks");

pub fn line(row: &Row) -> String {
    let kind = format!("{:<wide$} | {:>count$}", row.kind, row.cases, wide = KIND_WIDTH, count = CASES_WIDTH);
    if row.note == TOO_FEW {
        return format!("{kind} cases: too few to learn from and to read ({} and {})", row.learn, row.read);
    }
    if row.note.contains(NO_WORD_NOTE) {
        return format!("{kind} cases: {}", row.note);
    }
    let head = format!(
        "{kind} {:>part$} {:>part$} | {:>part$} | {:>part$} | {:>count$.places$} | {:>count$.places$} | {:>count$.places$} |",
        row.learn,
        row.read,
        row.copies,
        row.answers,
        row.one_answer,
        row.plain,
        row.question,
        part = PART_WIDTH,
        count = CASES_WIDTH,
        places = SHARE_PLACES
    );
    let Some(learner) = row.learner else { return format!("{head} no result of the learner: {}", row.note) };
    let mark = if learned(row) {
        MARK_LEARNED
    } else if share_that_counts(row).is_none() {
        MARK_COPIES
    } else if learner >= LEARNED {
        MARK_NO_TEXT
    } else {
        MARK_NONE
    };
    let mut text = format!("{head} {learner:>count$.places$} {mark} | {:>count$.places$} | {:>part$.none$}", row.not_copies, row.fit_seconds, count = CASES_WIDTH, places = SHARE_PLACES, part = PART_WIDTH, none = 0);
    if !row.curve.is_empty() {
        let mut sizes = row.curve.clone();
        sizes.sort_by_key(|(size, _)| *size);
        let need = if learned(row) { sizes.iter().find(|(size, _)| sizes.iter().all(|(other, at)| other < size || *at >= LEARNED)).map(|(size, _)| *size) } else { None };
        let points: Vec<String> = sizes.iter().map(|(size, at)| format!("{size}:{at:.places$}", places = SHARE_PLACES)).collect();
        text += &format!(" | {} | needed: {}", points.join(" "), need.map_or("not learned".to_string(), |size| size.to_string()));
    }
    if let Some((right, changed, known)) = row.new_words {
        let stand = if known > 0 { format!("; {known} of the new words stand in the lessons") } else { String::new() };
        text += &format!(" | new words: {right:.places$} ({changed:.none$}% of the read cases changed{stand})", places = SHARE_PLACES, none = 0);
    }
    if let Some((without, with)) = row.default_shares {
        text += &format!(" | answered by default: {without:.places$} of the read cases, {with:.places$} with new words", places = SHARE_PLACES);
    }
    if row.labels {
        text += " | STRONGER LESSONS: the formal forms that the set gives for the lesson sentences";
    }
    if row.curriculum {
        text += " | EARLIER KINDS: learned over the rules that the kinds before it in its set taught, from their lessons";
    }
    if let Some((asked, answered)) = row.asked {
        text += &format!(" | QUESTIONS: {asked} asked, {answered} answered");
    }
    if let Some((stopped, tool)) = &row.stopped {
        text += &format!(" | {stopped} was taken and stopped on all lessons: {tool} is the tool");
    }
    if row.bound_stops > usize::default() {
        text += &format!(" | STOPPED AT A BOUND: {} readings of the read part", row.bound_stops);
    }
    if row.budget_stopped {
        text += &format!("{CHAIN_WORK_BETWEEN}{BUDGET_STOPPED} {}", row.learning_work);
    }
    if row.marks.read_cases() > usize::default() {
        text += &format!(" | answer marks of the read cases: {}", row.marks.shares());
    }
    if let Some((named, forms)) = row.named
        && named > usize::default()
    {
        text += &format!(" | named words: {named} of {forms} learned forms name a word that a learned form with fewer fixed words reads too");
    }
    if let Some((lessons, read)) = row.one_case
        && (lessons - read).abs() >= READ_UNLIKE_POINTS
    {
        text += &format!(" | read part unlike the lessons: told texts told once in {lessons:.places$} of the lessons and {read:.places$} of the read part", places = SHARE_PLACES);
    }
    text
}
because!(line, ExactMeasure, "one line of the table for a kind: the sizes, the controls, the share of the model with its mark, and what the extras gave");

pub fn table_head(curve: bool) -> String {
    let more = if curve { " | share right by learned cases | needed" } else { "" };
    format!("{:<wide$} |  cases learn  read | copies | answers | one answer | plain | question | learner | not copies | fit s{more}", "kind", wide = KIND_WIDTH)
}
because!(table_head, ExactMeasure, "the names of the columns of the table of a measure");
