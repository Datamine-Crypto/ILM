use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::amount::AMOUNT_TOOL;
use crate::tools::amount::learn::amount_late;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::together::asked::{AskedModel, AskedRead};
use crate::tools::together::roles::written_way;
use crate::tools::together::tokens::answer_amount;
use crate::tools::together::tokens::{RESULT_PLACES, TokenMarks};
use crate::tools::together::trees::TREE_COUNT;
use crate::tools::together::vote::RoleModel;
use crate::tools::together::walk::Walk;
use patterns::{because, source};
use std::sync::OnceLock;

pub struct ToolsTogether;
source!(ToolsTogether, "tools that work together inside one problem, each doing the step it was made for: the amount tool reads the numbers and makes each step of a calculation exactly, the pattern learner gives the words, the state tool reads a story into units; this tool holds what joins them, the roles of the numbers and the walk of a place, and a kind is read by the one that its lessons follow; where neither reaches a problem, the amount tool answers it as it answers alone");

pub struct SureVotes;
source!(SureVotes, "an answer is sure or a guess: a model of the first part of the lessons reads the part held back, the answers in the order of the share of their vote; the lowest share at or above which nearly all answers are right, among enough answers, marks a sure answer");

const SURE_COUNT: usize = 20;
because!(SURE_COUNT, SureVotes, "the answers at or above a share that the mark of a sure answer needs, the sure count of the tools");

const TOGETHER_SURE_PART: f64 = 0.98;
because!(TOGETHER_SURE_PART, SureVotes, "the share of right answers at or above a share that makes it the mark of a sure answer, the sure share of the tools");

const NOT_READ: &str = "the together tool does not read this kind: no rule of a walk, and no way of roles for half of the lessons";
because!(NOT_READ, ToolsTogether, "the words with which the tool says that neither of its ways reads a kind, so that the choice goes on with the other tools");

const ROLE_PARTS: &str = "amount tool: the numbers, each step of the calculation, the written result; pattern learner: the words; together tool: the roles";
because!(ROLE_PARTS, ToolsTogether, "the part of the note that says which tool did which step of an answer by the roles");

const ASKED_PARTS: &str = "amount tool: the numbers and the step; together tool: the numbers that the question names and the program of the nearest lessons";
because!(ASKED_PARTS, ToolsTogether, "the part of the note that says which tool did which step of an answer by a program of the question");

const ALONE: &str = "amount tool alone";
because!(ALONE, ToolsTogether, "the start of the note of an answer that the amount tool gave alone");

const LIST_END: &str = "list";
because!(LIST_END, SureVotes, "the last word of the note of a sure answer, the word of the pattern learner");

const BEST_END: &str = "best";
because!(BEST_END, SureVotes, "the last word of the note of a guess, the word of the pattern learner");

const NOTE_GAP: &str = " | ";
because!(NOTE_GAP, ToolsTogether, "the mark between the parts of a note");

const VOTE_PLACES: usize = 3;
because!(VOTE_PLACES, SureVotes, "the places after the point of a share in a note");

pub struct TogetherModel {
    symbols: Symbols,
    marks: TokenMarks,
    roles: Option<RoleModel>,
    asked: Option<AskedModel>,
    sure: Option<f64>,
    walk: Option<Walk>,
    lessons: Vec<Case>,
    lanes: usize,
    amount: OnceLock<Option<Box<dyn Learner>>>,
}
because!(TogetherModel, ToolsTogether, SureVotes, "the model of the tools together: the table of words, the marks of the tokens, the roles when they read the kind, the programs of the questions beside the roles, the mark of a sure answer, the walk when it is a rule, and the lessons with the number of threads for the amount tool, which is learned when a problem first needs it. learn makes it, or says that it does not read the kind; its answer is the walk's, else the sure program of the question, else the vote of the roles, else the amount tool's");

fn sure_share(cases: &[Case]) -> Option<f64> {
    let late = amount_late(cases);
    let first: Vec<Case> = cases.iter().zip(&late).filter(|(_, late)| !**late).map(|(case, _)| case.clone()).collect();
    let mut symbols = Symbols::new();
    let marks = TokenMarks::of(&mut symbols);
    let inner = RoleModel::learn(&first, &[], &mut symbols, &marks)?;
    let mut read: Vec<(f64, bool)> = Vec::new();
    for (case, late) in cases.iter().zip(&late) {
        if !late {
            continue;
        }
        let problem = marks.read(&mut symbols, &case.sentences, &case.question);
        let Some((value, _, share)) = inner.vote(&symbols, &problem) else { continue };
        if let Some(text) = value.written(RESULT_PLACES) {
            read.push((share, answer_words(&text) == answer_words(&case.answer)));
        }
    }
    read.sort_by(|one, other| other.0.total_cmp(&one.0));
    let mut best: Option<f64> = None;
    let mut right = 0usize;
    for (at, (share, good)) in read.iter().enumerate() {
        let count = at + 1;
        right += usize::from(*good);
        if count >= SURE_COUNT && right as f64 >= TOGETHER_SURE_PART * count as f64 && (count == read.len() || read[count].0 < *share) {
            best = Some(*share);
        }
    }
    best
}
because!(sure_share, SureVotes, "the share of the vote that marks a sure answer, or none: a model of the first part of the lessons reads the part held back; the lowest share with nearly all answers right at or above it, among enough answers, and where the next answer has a lower share");

pub struct ForeignSentences;
source!(ForeignSentences, "a told sentence of another problem does not change the answer of a problem: the roles learn from the lessons and from each lesson once more with one told sentence that holds a number, taken from a lesson far from it in the order of the lessons, at a place that its own place in that order gives, and with the same answer; so the roles learn that a number of a text can stand there and not be used, which lessons where every number is used never show");

const FAR_PART: usize = 2;
because!(FAR_PART, ForeignSentences, "the lesson that gives a sentence stands this part of the lessons away, half way round: another problem, and not a variation of the same problem that stands next to it");

fn with_foreign(cases: &[Case]) -> Vec<(usize, Case)> {
    let count = cases.len();
    let mut out: Vec<(usize, Case)> = Vec::new();
    for (place, case) in cases.iter().enumerate() {
        let from = (place + count / FAR_PART) % count.max(1);
        let Some(extra) = cases[from].sentences.iter().find(|sentence| sentence.chars().any(|sign| sign.is_ascii_digit())) else { continue };
        if from == place || case.sentences.contains(extra) {
            continue;
        }
        let mut sentences = case.sentences.clone();
        sentences.insert(place % (case.sentences.len() + 1), extra.clone());
        out.push((place, Case { sentences, question: case.question.clone(), answer: case.answer.clone() }));
    }
    out
}
because!(with_foreign, ForeignSentences, "the changed lessons of the roles, each with the place of its own lesson: every lesson again with a told sentence that holds a number from the lesson half way round, when that sentence is not its own");

impl TogetherModel {
    pub fn learn(lessons: &Lessons, lanes: usize) -> Result<TogetherModel, String> {
        let cases = lessons.cases;
        let mut symbols = Symbols::new();
        let walk = Walk::learn(cases, &mut symbols).filter(|walk| walk.rule);
        let marks = TokenMarks::of(&mut symbols);
        let roles = RoleModel::learn(cases, &with_foreign(cases), &mut symbols, &marks);
        let sure = if roles.is_some() { sure_share(cases) } else { None };
        if roles.is_none() && walk.is_none() {
            return Err(NOT_READ.to_string());
        }
        let asked = roles.as_ref().and_then(|roles| {
            let reads: Vec<(AskedRead, crate::base::exact::Amount)> = cases
                .iter()
                .filter_map(|case| {
                    let truth = answer_amount(&case.answer)?;
                    let problem = marks.read(&mut symbols, &case.sentences, &case.question);
                    Some((AskedRead::of(&roles.reader, &symbols, &problem), truth))
                })
                .collect();
            AskedModel::learn(&reads)
        });
        Ok(TogetherModel { symbols, marks, roles, asked, sure, walk, lessons: cases.to_vec(), lanes, amount: OnceLock::new() })
    }

    fn alone(&self, sentences: &[String], question: &str) -> Said {
        let model = self.amount.get_or_init(|| (AMOUNT_TOOL.learn)(Lessons::of(&self.lessons), self.lanes).ok());
        match model {
            Some(model) => {
                let said = model.answer(sentences, question);
                Said { words: said.words, note: format!("{ALONE}{NOTE_GAP}{}", said.note) }
            }
            None => Said { words: Vec::new(), note: ALONE.to_string() },
        }
    }
}

impl Learner for TogetherModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        if let Some(walk) = &self.walk
            && let Some(said) = walk.answer(&self.symbols, sentences)
        {
            return said;
        }
        if let (Some(roles), Some(asked)) = (&self.roles, &self.asked) {
            let mut names = Fresh::over(&self.symbols);
            let problem = self.marks.read(&mut names, sentences, question);
            let read = AskedRead::of(&roles.reader, &names, &problem);
            if let Some((value, program)) = asked.answer(&read)
                && roles.fits(value)
                && let Some(text) = value.written(RESULT_PLACES)
            {
                let note = [format!("{:?} {}", program.step, names.name(program.word)), ASKED_PARTS.to_string()].join(NOTE_GAP);
                return Said { words: answer_words(&text), note };
            }
        }
        if let Some(roles) = &self.roles {
            let mut names = Fresh::over(&self.symbols);
            let problem = self.marks.read(&mut names, sentences, question);
            if let Some((value, way, share)) = roles.vote(&names, &problem) && let Some(text) = value.written(RESULT_PLACES) {
                let amounts: Vec<String> = problem.values.iter().flatten().map(|amount| amount.written(RESULT_PLACES).unwrap_or_default()).collect();
                let trees = TREE_COUNT * (roles.forests.len() as u64 + 1);
                let end = if self.sure.is_some_and(|bar| share >= bar) { LIST_END } else { BEST_END };
                let note = [written_way(&way, &amounts), ROLE_PARTS.to_string(), format!("{share:.*}", VOTE_PLACES), trees.to_string(), end.to_string()].join(NOTE_GAP);
                return Said { words: answer_words(&text), note };
            }
        }
        self.alone(sentences, question)
    }
}

pub fn learn_together(lessons: Lessons, lanes: usize) -> Learned {
    let model = TogetherModel::learn(&lessons, lanes)?;
    Ok(Box::new(model))
}
because!(learn_together, ToolsTogether, "the tools together as a tool of the one call: a model when the walk or the roles read the kind, and else the words with which it does not read it");
