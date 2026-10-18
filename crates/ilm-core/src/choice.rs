use crate::cases::Case;
use crate::base::lanes::{on_lanes, on_spare_lanes};
use crate::tools::call::{Learner, Lessons, Reading, Said, StoryAsk, Tool, read_story};
use crate::split::told_texts;
use crate::base::words::answer_words;
use patterns::{because, source};
use std::cmp::Ordering;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;

pub struct HeldChoice;
source!(HeldChoice, "the choice of a model by lessons that it did not learn from (the cross-validatory choice of Stone): every tool learns from the first part of the lessons and reads the rest; the tool that is right most often there is taken and learns again from all lessons; no tool is told which kind it reads and no setting changes between kinds, so a new kind of task needs no choice by a person");

const INNER: f64 = 0.6666666666666666;
because!(INNER, HeldChoice, "the share of the lessons that every tool learns from before the choice: most of them, so that a tool learns nearly what it learns from all, with enough left to read that tools which differ by a few cases are told apart");

const CLEARLY: f64 = 1e-9;
because!(CLEARLY, HeldChoice, "a later tool is taken only when its share is higher by more than this: shares that differ by the rounding of a division are equal, and of equal tools the earlier one is taken");

const NO_SHARE: f64 = -1.0;
because!(NO_SHARE, HeldChoice, "the share of a tool that gave no model: below the share of every model, so such a tool is taken only when no tool gave a model");

pub const NO_LABELS: &str = "the lessons of this kind have no labels";
because!(NO_LABELS, HeldChoice, "what is said for a tool that learns from labels of the lesson sentences where the set gives none");

const BROKE: &str = "the tool broke";
because!(BROKE, HeldChoice, "what is said for a tool that ended with a fault of its own on a kind: the other tools go on, and the fault is seen in the table of the shares");

const NO_TOOL: &str = "no tool learns from these lessons";
because!(NO_TOOL, HeldChoice, "what is said when every tool that was tried says that it does not read the kind");

pub fn inner_parts(cases: &[Case]) -> (Vec<usize>, Vec<usize>) {
    let (keys, sizes) = told_texts(cases);
    let most = (cases.len() as f64) * INNER;
    let mut mine = vec![false; sizes.len()];
    let mut count = 0usize;
    for (key, size) in sizes.iter().enumerate() {
        if (count as f64) >= most {
            break;
        }
        mine[key] = true;
        count += size;
    }
    let first = (0..cases.len()).filter(|place| mine[keys[*place]]).collect();
    let last = (0..cases.len()).filter(|place| !mine[keys[*place]]).collect();
    (first, last)
}
because!(inner_parts, HeldChoice, "the places of the lessons that the tools learn from before the choice and of the lessons that they read: told texts are taken whole in the order of first sight until the first part holds its share, so a told text that comes again with another question is never in both parts");

pub fn picked(cases: &[Case], places: &[usize]) -> Vec<Case> {
    places.iter().map(|place| cases[*place].clone()).collect()
}
because!(picked, HeldChoice, "the cases at the given places, in the order of the places");

pub fn picked_labels(labels: &[Vec<String>], places: &[usize]) -> Vec<Vec<String>> {
    if labels.is_empty() {
        return Vec::new();
    }
    places.iter().map(|place| labels[*place].clone()).collect()
}
because!(picked_labels, HeldChoice, "the label lines of the cases at the given places, and none for a kind that has no labels");

pub fn right_of(model: &dyn Learner, cases: &[Case], lanes: usize) -> usize {
    on_spare_lanes(cases.len(), lanes, &|place| usize::from(model.answer(&cases[place].sentences, &cases[place].question).words == answer_words(&cases[place].answer))).into_iter().sum()
}
because!(right_of, HeldChoice, "the cases that a model answers with exactly the words of the true answer; the cases are read in runs on the threads, and a count does not depend on their number");

struct Tried {
    share: f64,
    all: bool,
    skipped: bool,
    said: String,
    seconds: f64,
}
because!(Tried, HeldChoice, "what one tool gave on the first part of the lessons: its share of right answers on the rest, whether it read every case of the rest right, whether it was not tried because an earlier tool did, what it said when it gave no model, and its seconds");

fn untried() -> Tried {
    Tried { share: NO_SHARE, all: false, skipped: true, said: String::new(), seconds: f64::default() }
}
because!(untried, HeldChoice, "a tool that was not tried because an earlier tool read every case of the rest of the lessons right: it has no share until it is tried");

fn tried(tool: &Tool, lessons: Lessons, first: Lessons, last: &[Case], lanes: usize) -> Tried {
    let started = Instant::now();
    let learned = if tool.needs_labels && !lessons.labelled() {
        Err(NO_LABELS.to_string())
    } else {
        catch_unwind(AssertUnwindSafe(|| (tool.learn)(first, lanes).map(|model| (right_of(model.as_ref(), last, lanes), model)))).unwrap_or_else(|_| Err(BROKE.to_string()))
    };
    let seconds = started.elapsed().as_secs_f64();
    match learned {
        Ok((right, _)) => Tried { share: (right as f64) / (last.len().max(1) as f64), all: right == last.len(), skipped: false, said: String::new(), seconds },
        Err(said) => Tried { share: NO_SHARE, all: false, skipped: false, said, seconds },
    }
}
because!(tried, HeldChoice, "one tool on the first part of the lessons: a tool that says that it does not read the kind, or that breaks, has no share and does not stop the others");

pub struct Taken {
    pub model: Box<dyn Learner>,
    pub tool: usize,
    pub named: String,
    pub shares: Vec<f64>,
    pub said: Vec<String>,
    pub seconds: Vec<f64>,
    pub stopped: Vec<(usize, String)>,
    pub labels: bool,
}
because!(Taken, HeldChoice, "the model of the tool that was taken, made from all lessons, with what the choice rests on: the share of every tool on the lessons that it did not learn from, what a tool said that gave no model, the seconds, the tools that were taken and then said on all lessons that they do not read the kind, and whether the tool got labels of its lessons. It answers as its model does and names the tool in the note");

impl Learner for Taken {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let said = self.model.answer(sentences, question);
        Said { words: said.words, note: [self.named.as_str(), said.note.as_str()].join(NOTE_BETWEEN), mark: said.mark }
    }

    fn story<'a>(&'a self, sentences: &[String]) -> Option<Box<dyn StoryAsk + 'a>> {
        Some(Box::new(NamedStory { inner: read_story(self.model.as_ref(), sentences), named: &self.named }))
    }

    fn bound_stops(&self) -> usize {
        self.model.bound_stops()
    }

    fn reading(&self) -> Option<Reading> {
        self.model.reading()
    }

    fn proof(&self, sentences: &[String], question: &str) -> Vec<String> {
        self.model.proof(sentences, question)
    }

    fn missing(&self, sentences: &[String], question: &str, expected: &[String]) -> Vec<String> {
        self.model.missing(sentences, question, expected)
    }

    fn rules_used(&self, sentences: &[String], question: &str) -> Vec<(String, bool)> {
        self.model.rules_used(sentences, question)
    }

    fn lines_from_lessons(&self) -> Vec<String> {
        self.model.lines_from_lessons()
    }
}

struct NamedStory<'a> {
    inner: Box<dyn StoryAsk + 'a>,
    named: &'a str,
}
because!(NamedStory, HeldChoice, "a story read once by the model of the tool that was taken: it answers as that model does and names the tool in the note, as an answer of the choice does");

impl StoryAsk for NamedStory<'_> {
    fn ask(&mut self, question: &str) -> Said {
        let said = self.inner.ask(question);
        Said { words: said.words, note: [self.named, said.note.as_str()].join(NOTE_BETWEEN), mark: said.mark }
    }
}

const NOTE_BETWEEN: &str = ": ";
because!(NOTE_BETWEEN, HeldChoice, "the mark between the name of the tool and the note of its model in the note of an answer");

fn named(tools: &[Tool], tool: usize, stopped: &[(usize, String)]) -> String {
    if stopped.is_empty() {
        return tools[tool].name.to_string();
    }
    let names: Vec<&str> = stopped.iter().map(|(place, _)| tools[*place].name).collect();
    format!("{} (taken after {} stopped on all lessons)", tools[tool].name, names.join(", "))
}
because!(named, HeldChoice, "the name of the tool in the note of an answer; it also names every tool that was taken before it and stopped on all lessons, so that an answer shows that the first choice did not hold");

pub fn take(tools: &[Tool], lessons: Lessons, lanes: usize, every: bool) -> Result<Taken, String> {
    let (first, last) = inner_parts(lessons.cases);
    let first_cases = picked(lessons.cases, &first);
    let first_labels = picked_labels(lessons.labels, &first);
    let last_cases = picked(lessons.cases, &last);
    let inner = lessons.part(&first_cases, &first_labels);
    let mut tries: Vec<Tried> = if every {
        let each = (lanes / tools.len().max(1)).max(1);
        on_lanes(tools.len(), lanes, &|tool| tried(&tools[tool], lessons, inner, &last_cases, each))
    } else if tools.len() == 1 {
        vec![untried()]
    } else {
        let mut found: Vec<Tried> = Vec::with_capacity(tools.len());
        for tool in tools {
            let one = if found.iter().any(|held| held.all) { untried() } else { tried(tool, lessons, inner, &last_cases, lanes) };
            found.push(one);
        }
        found
    };
    let mut best: Option<usize> = None;
    for (tool, one) in tries.iter().enumerate() {
        if best.is_none_or(|top| one.share > tries[top].share + CLEARLY) {
            best = Some(tool);
        }
    }
    let Some(best) = best else { return Err(NO_TOOL.to_string()) };
    let mut order: Vec<usize> = vec![best];
    let mut stopped: Vec<(usize, String)> = Vec::new();
    let mut at = 0;
    while at < order.len() {
        let tool = order[at];
        at += 1;
        match (tools[tool].learn)(lessons, lanes) {
            Ok(model) => {
                let labels = tools[tool].needs_labels || model.used_labels();
                return Ok(Taken {
                    model,
                    tool,
                    named: named(tools, tool, &stopped),
                    shares: tries.iter().map(|one| one.share).collect(),
                    said: tries.iter_mut().map(|one| std::mem::take(&mut one.said)).collect(),
                    seconds: tries.iter().map(|one| one.seconds).collect(),
                    stopped,
                    labels,
                });
            }
            Err(why) => stopped.push((tool, why)),
        }
        if order.len() == 1 && tools.len() > 1 {
            for (place, one) in tries.iter_mut().enumerate() {
                if one.skipped {
                    *one = tried(&tools[place], lessons, inner, &last_cases, lanes);
                }
            }
            let mut rest: Vec<usize> = (0..tools.len()).filter(|tool| *tool != best && tries[*tool].share >= f64::default()).collect();
            rest.sort_by(|one, other| tries[*other].share.partial_cmp(&tries[*one].share).unwrap_or(Ordering::Equal).then(one.cmp(other)));
            order.extend(rest);
        }
    }
    let said: Vec<String> = stopped.iter().map(|(tool, why)| format!("{}: {why}", tools[*tool].name)).collect();
    Err(format!("{NO_TOOL}: {}", said.join("; ")))
}
because!(take, HeldChoice, "the choice among the tools for one kind: the tools on the first part of the lessons, each with its share on the rest; the highest share is taken, of equals the earlier tool; that tool learns again from all lessons, and when it says there that it does not read the kind, the next tool in the order of the shares is taken. The tools are tried in their order, and a tool after one that read every case of the rest right is not tried, because a later tool is taken only with a higher share; when the tool taken stops on all lessons, the tools not tried are tried then. One tool alone is no choice, so it is not tried on the first part and learns from all lessons at once. With every, all tools are tried side by side, for a run that shows the share of each tool");
