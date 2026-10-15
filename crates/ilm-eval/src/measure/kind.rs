use ilm_core::cases::Case;
use ilm_core::classes::WordClasses;
use ilm_core::choice::{picked, picked_labels, take};
use ilm_core::base::lanes::{on_lanes, shards};
use ilm_core::base::shares::share;
use ilm_core::tools::call::{Learner, Lessons, Tool};
use ilm_core::tools::plain::Plain;
use ilm_core::split::parts;
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

const PROBE_MOST: usize = 1000;
because!(PROBE_MOST, ExactMeasure, "the most lessons for the model that gives the control by the form of the question when another tool is measured alone, so that a run with very many lessons does not wait for a second model");

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

const LEARNER_FAILED: &str = "the run of the learner ended with an error";
because!(LEARNER_FAILED, ExactMeasure, "the note of a kind where no tool gave a model or the model broke; the controls of the kind are still given");

#[derive(Clone, Debug, Default)]
pub struct Row {
    pub kind: String,
    pub cases: usize,
    pub learn: usize,
    pub read: usize,
    pub note: String,
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
    pub labels: bool,
    pub asked: Option<(usize, usize)>,
    pub new_words: Option<(f64, f64, usize)>,
    pub curve: Vec<(usize, f64)>,
    pub wrong: Vec<String>,
}
because!(Row, ExactMeasure, "what the measure gives for one kind: the sizes of its parts, the controls (one answer for all, plain counting, the question alone), the share of the model on the read part and on the read cases that are no copies of lessons, the tool that was taken, and what the asked extras gave (new words, the curve by lessons, wrong lesson cases, questions of the tool)");

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
}
because!(Asked, ExactMeasure, "what a run of the measure asks for: the most lessons and the most read cases, the final part in place of the read part, the extras, one tool alone in place of the choice among all, the threads, and whether a read part of any size is read, as the official test split of a benchmark is read whole however few cases a kind of it has");

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
}
because!(Ground, ExactMeasure, "what a kind gives to the measure: its name, its cases with their label lines, the place of the first case to read for a kind of two halves, whether lessons are counted in stories, the final part when it is read, the sealed part when it is not the one of the cut, the table of new words, for a kind that a program made a way to get the answer of any text, and the table of word classes of the run");

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

fn right_and_fresh(model: &dyn Learner, read: &[Case], truths: &[Vec<String>], copy: &[bool], lanes: usize) -> (usize, usize) {
    let runs = shards(read.len(), lanes);
    let counted = on_lanes(runs.len(), lanes, &|run| {
        let mut right = 0usize;
        let mut fresh = 0usize;
        for place in runs[run].clone() {
            if model.answer(&read[place].sentences, &read[place].question).words == truths[place] {
                right += 1;
                fresh += usize::from(!copy[place]);
            }
        }
        (right, fresh)
    });
    counted.into_iter().fold((0, 0), |sum, one| (sum.0 + one.0, sum.1 + one.1))
}
because!(right_and_fresh, ExactMeasure, "the read cases that a model answers right, and those of them that are no copies of lessons; the cases are read in runs on the threads");

fn by_question(probe: &dyn Learner, read: &[Case], truths: &[Vec<String>], lanes: usize) -> Option<f64> {
    let runs = shards(read.len(), lanes);
    let counted = on_lanes(runs.len(), lanes, &|run| {
        let mut right = 0usize;
        for place in runs[run].clone() {
            right += usize::from(probe.question_alone(&read[place].sentences, &read[place].question)? == truths[place]);
        }
        Some(right)
    });
    let right: Option<usize> = counted.into_iter().sum();
    right.map(|right| share(right, read.len()))
}
because!(by_question, ExactMeasure, "the share right when a model answers by its rule of the form of the question alone; nothing when the model has no such rule for a case");

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

fn with_new_words(model: &dyn Learner, learn: &[Case], read: &[Case], pairs: &[(String, String)], lanes: usize) -> (f64, f64, usize) {
    let table: HashMap<&str, &str> = pairs.iter().map(|(old, new)| (old.as_str(), new.as_str())).collect();
    let mut held: HashSet<String> = HashSet::new();
    for lesson in learn {
        for text in lesson.sentences.iter().chain([&lesson.question, &lesson.answer]) {
            held.extend(words(text));
        }
    }
    let known = table.values().filter(|new| held.contains(**new)).count();
    let runs = shards(read.len(), lanes);
    let counted = on_lanes(runs.len(), lanes, &|run| {
        let mut right = 0usize;
        let mut changed = 0usize;
        for case in &read[runs[run].clone()] {
            let sentences: Vec<String> = case.sentences.iter().map(|sentence| renamed(sentence, &table)).collect();
            let question = renamed(&case.question, &table);
            let answer = renamed(&case.answer, &table);
            changed += usize::from(sentences != case.sentences || question != case.question || answer != case.answer);
            right += usize::from(model.answer(&sentences, &question).words == answer_words(&answer));
        }
        (right, changed)
    });
    let (right, changed) = counted.into_iter().fold((0, 0), |sum, one| (sum.0 + one.0, sum.1 + one.1));
    (share(right, read.len()), share(changed, read.len()), known)
}
because!(with_new_words, NewWords, "the read cases again with the new words: the share right, the share of the cases that the new words changed, and how many of the new words stand in the lessons after all");

struct Made {
    model: Box<dyn Learner>,
    probe: Option<Box<dyn Learner>>,
    own_probe: bool,
    tool: String,
    stopped: Option<(String, String)>,
    labels: bool,
}
because!(Made, ExactMeasure, "the model of a kind as the measure reads it: the model, the model for the control by the form of the question (or the mark that the model itself gives that control), the name of its tool, the tools that stopped before it, and whether it got labels of its lessons");

fn made(tools: &[Tool], lessons: Lessons, asked: &Asked) -> Result<Made, String> {
    match asked.tool {
        None => {
            let quiet = Lessons { ask: None, ..lessons };
            let taken = take(tools, quiet, asked.lanes, false)?;
            let stopped: Vec<&str> = taken.stopped.iter().map(|(tool, _)| tools[*tool].name).collect();
            let stopped = (!stopped.is_empty()).then(|| (stopped.join(", "), tools[taken.tool].name.to_string()));
            Ok(Made { model: taken.model, probe: taken.probe, own_probe: false, tool: tools[taken.tool].name.to_string(), stopped, labels: taken.labels })
        }
        Some(tool) => {
            let model = (tools[tool].learn)(lessons, asked.lanes)?;
            let own_probe = tools[tool].probe;
            let probe = match tools.iter().find(|other| other.probe) {
                Some(other) if !own_probe => {
                    let most = lessons.cases.len().min(PROBE_MOST);
                    let labels = if lessons.labels.is_empty() { lessons.labels } else { &lessons.labels[..most] };
                    (other.learn)(Lessons { cases: &lessons.cases[..most], labels, ask: None, classes: lessons.classes }, asked.lanes).ok()
                }
                _ => None,
            };
            Ok(Made { model, probe, own_probe, tool: tools[tool].name.to_string(), stopped: None, labels: tools[tool].needs_labels && lessons.labelled() })
        }
    }
}
because!(made, ExactMeasure, "the model of a kind: by the choice among all tools, or by one tool alone; a tool that is measured alone gets the same control by the form of the question as the choice has, from a model of the tool that gives it");

fn wrong_lessons(tools: &[Tool], lessons: Lessons, asked: &Asked) -> Vec<String> {
    let cut = lessons.cases.len() * SHOWN_PARTS / SHOWN_OF;
    let labels = if lessons.labels.is_empty() { lessons.labels } else { &lessons.labels[..cut] };
    let Ok(inner) = made(tools, Lessons { cases: &lessons.cases[..cut], labels, ask: None, classes: lessons.classes }, asked) else { return Vec::new() };
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
    let copy: Vec<bool> = read.iter().map(|case| seen.contains(&(case.sentences.as_slice(), case.question.as_str()))).collect();
    let (most, answers) = most_frequent(&learn);
    row.copies = copy.iter().filter(|is| **is).count();
    row.one_answer = share(truths.iter().filter(|truth| **truth == most).count(), read.len());
    row.answers = answers;
    let base = Plain::learn(&learn);
    row.plain = share((0..read.len()).filter(|place| base.best(&read[*place].sentences, &read[*place].question) == Some(truths[*place].as_slice())).count(), read.len());
    let bare: Vec<Case> = learn.iter().map(|lesson| Case { sentences: Vec::new(), question: lesson.question.clone(), answer: lesson.answer.clone() }).collect();
    let alone = Plain::learn(&bare);
    row.question = share((0..read.len()).filter(|place| alone.best(&[], &read[*place].question) == Some(truths[*place].as_slice())).count(), read.len());
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
    let lessons = Lessons { cases: &learn, labels: &learn_labels, ask: if asks { Some(&ask) } else { None }, classes: ground.classes };
    let Ok(Ok(made_model)) = catch_unwind(AssertUnwindSafe(|| made(tools, lessons, asked))) else {
        row.note = LEARNER_FAILED.to_string();
        row.all_seconds = started.elapsed().as_secs_f64();
        return row;
    };
    row.fit_seconds = fit_started.elapsed().as_secs_f64();
    row.tool = made_model.tool.clone();
    row.stopped = made_model.stopped.clone();
    row.labels = made_model.labels;
    if asks {
        row.asked = questions.lock().ok().map(|count| *count);
    }
    let model = made_model.model.as_ref();
    if learn.iter().any(|lesson| !lesson.sentences.is_empty()) {
        let probe = if made_model.own_probe { Some(model) } else { made_model.probe.as_deref() };
        if let Some(pattern) = probe.and_then(|probe| by_question(probe, &read, &truths, asked.lanes)) {
            row.question = row.question.max(pattern);
        }
    }
    let Ok((right, fresh)) = catch_unwind(AssertUnwindSafe(|| right_and_fresh(model, &read, &truths, &copy, asked.lanes))) else {
        row.note = LEARNER_FAILED.to_string();
        row.all_seconds = started.elapsed().as_secs_f64();
        return row;
    };
    if asked.show > 0 {
        row.wrong = wrong_lessons(tools, lessons, asked);
    }
    row.learner = Some(share(right, read.len()));
    row.not_copies = share(fresh, (read.len() - row.copies).max(1));
    if asked.curve {
        for size in SIZES {
            if *size >= learn.len() {
                break;
            }
            let labels = if learn_labels.is_empty() { &learn_labels[..] } else { &learn_labels[..*size] };
            if let Ok(small) = made(tools, Lessons { cases: &learn[..*size], labels, ask: None, classes: ground.classes }, asked) {
                row.curve.push((*size, share(right_and_fresh(small.model.as_ref(), &read, &truths, &copy, asked.lanes).0, read.len())));
            }
        }
        row.curve.push((learn.len(), share(right, read.len())));
    }
    if let (true, Some(table)) = (asked.new_words, ground.new_words) {
        row.new_words = Some(with_new_words(model, &learn, &read, table, asked.lanes));
    }
    row.all_seconds = started.elapsed().as_secs_f64();
    row
}
because!(measure, ExactMeasure, "one kind through the measure: the cut, the controls, the model from the learned part, its share on the read part, and the asked extras; labels of the set reach the model for its lessons alone, and a model that fails leaves the controls of the kind in the row");

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
    if row.labels {
        text += " | STRONGER LESSONS: the formal forms that the set gives for the lesson sentences";
    }
    if let Some((asked, answered)) = row.asked {
        text += &format!(" | QUESTIONS: {asked} asked, {answered} answered");
    }
    if let Some((stopped, tool)) = &row.stopped {
        text += &format!(" | {stopped} was taken and stopped on all lessons: {tool} is the tool");
    }
    text
}
because!(line, ExactMeasure, "one line of the table for a kind: the sizes, the controls, the share of the model with its mark, and what the extras gave");

pub fn table_head(curve: bool) -> String {
    let more = if curve { " | share right by learned cases | needed" } else { "" };
    format!("{:<wide$} |  cases learn  read | copies | answers | one answer | plain | question | learner | not copies | fit s{more}", "kind", wide = KIND_WIDTH)
}
because!(table_head, ExactMeasure, "the names of the columns of the table of a measure");
