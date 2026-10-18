use crate::tools::explain::chain::{Facts, Filling, Time, adds, apart, asks, condition_fits, empty_filling, fill, fit_into, known_rivals, none_fits, overruling_fact, rule_text};
use crate::tools::explain::clauses::refutation;
use crate::tools::explain::text::{AskedForm, Condition, LineKind, LogicPiece, Moment, Pattern, ToldRule, condition_text, place_letter};
use crate::tools::explain::words::{WordId, word_text, word_texts};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct Proofs;
source!(Proofs, "an explanation as a proof found backward (as a logic program answers 'why' by the derivation tree of a fact, Shapiro's algorithmic debugging of Prolog): from the facts that an answer rests on, a rule whose outcome is the fact and whose conditions hold on facts that came before it, down to the told sentences; it is searched only when an explanation is asked, so that a run that asks none pays nothing, and a fact only rests on earlier facts, so that the search cannot go round");

pub enum ProofStep {
    Told(String),
    Known(String),
    Made { fact: String, rule: String, from: Vec<ProofStep> },
    Worked(String),
    Absent(String, Vec<String>),
    Refuted { statement: String, from: Vec<Option<String>>, holds: bool },
    Unfound(String),
    Timed(String),
    Overruled { fact: String, rule: String, by: Vec<ProofStep> },
}
because!(ProofStep, Proofs, "one step of a proof: a told sentence, a fact that the topic knows before any story, a fact made by a rule from the steps of its conditions, a condition that the engine works out, a condition that no fact may fit (with the known facts that the rules that would make its fact read), a statement that the clause block proves or proves false from the told facts that its refutation rests on, a fact whose derivation the search did not find within its bound, or how a condition held at a place in the story read its fact, before the step of that fact");

const TIMED_MARK: &str = "read at its time:";
because!(TIMED_MARK, Proofs, "the mark of a condition that only one fact may hold, by its place in the story, before the words that say which fact it read");

const STATE_READING: &str = "the latest value of this state at that point of the story";
because!(STATE_READING, Proofs, "what a condition held at a time reads when its sentence names its thing: the latest fact of the same thing up to that point, which must hold the same value, so a later fact of that thing with another value (an end) would make it fail");

const NEWEST_READING: &str = "the newest fact of its sentence at that point of the story, whatever fills its open place, not the state of one thing";
because!(NEWEST_READING, Proofs, "what a condition held at a time reads when a place of its sentence is still open: the one newest fact that fits it, whatever thing fills the open place, so a later fact of the same thing with another value is not read; a reader of the proof sees that the rule did not read the state of the thing that it names");

const NO_HISTORY_READING: &str = "; the facts keep no history, so a fact that came twice is read only at its latest place";
because!(NO_HISTORY_READING, Proofs, "what a proof adds to a condition at the time of a clause before it when the facts of the story keep only the latest place of each fact: a fact made again later, such as a state that started again, is then missing at the earlier time, and the reader sees why such a rule may fail");

const MOST_STEPS: usize = 4096;
because!(MOST_STEPS, Proofs, "the most facts whose derivation one explanation searches: a proof of a story fact rests on far fewer, and a bound keeps an explanation of a fact with many ways to it short");

struct Prover<'a> {
    rules: &'a [ToldRule],
    facts: &'a Facts,
    known: usize,
    left: usize,
    found: HashMap<usize, bool>,
    consulting: bool,
}
because!(Prover, Proofs, "the search for a proof in one story: the rules of the topic, the facts of the story, how many facts the topic knew before the story, how many more facts the search may try, the facts already tried with whether a derivation was found, and whether it is reading the known facts behind a fact that must be absent (consulted), which does not nest; fact gives the step of the fact at a place, which rests only on facts of earlier places; holding gives the steps of conditions that hold together under a filling, each fact before the given place, and before the step of a condition held at a time, how it read its fact: the state of the thing that it names, or, while a place of its sentence is open, the newest fact of any thing; for a rule that a learner made, only a filling whose places hold different things, as the chain fills it, so that a proof never shows a filling that the chain refused");

pub(super) fn filled_text(condition: &Condition, filling: &Filling) -> String {
    condition_text(condition, &|logic_piece| match logic_piece {
        LogicPiece::Word(word) => word_text(*word).to_string(),
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => filling.get(*place).map_or_else(|| place_letter(*place), |words| word_texts(words).join(" ")),
    })
}
because!(filled_text, Proofs, "a condition with the words that fill its places, as a step of a proof shows it");

fn fact_said(words: &[WordId]) -> String {
    word_texts(words).join(" ")
}
because!(fact_said, Proofs, "a fact as a step of a proof shows it");

const MOST_CONSULTED: usize = 64;
because!(MOST_CONSULTED, Proofs, "the most fillings that the rules which would make an absent fact try, as far as their conditions hold, for the known facts that they read: a person is seen busy a few times, each read with a few hours, and the search for a step that the answer only rests on should cost far less than the proof");

fn known_leaves(step: &ProofStep, out: &mut Vec<String>) {
    match step {
        ProofStep::Known(fact) => out.push(fact.clone()),
        ProofStep::Made { from, .. } | ProofStep::Overruled { by: from, .. } => from.iter().for_each(|part| known_leaves(part, out)),
        _ => {}
    }
}
because!(known_leaves, Proofs, "the known facts that a step of a proof rests on, down through the facts that rules made");

impl Prover<'_> {
    fn consulted(&mut self, condition: &Condition, filling: &Filling) -> Vec<String> {
        let Condition::Fact { pattern, .. } = condition else { return Vec::new() };
        let Some(words) = fill(pattern, filling) else { return Vec::new() };
        if self.consulting {
            return Vec::new();
        }
        self.consulting = true;
        let mut out: Vec<String> = Vec::new();
        let mut left = MOST_CONSULTED;
        for rule in self.rules.iter().filter(|rule| adds(rule)) {
            let mut fits = Vec::new();
            fit_into(&rule.outcome, &words, &empty_filling(), &mut fits);
            if let Some(made) = fits.into_iter().next() {
                self.tried(&rule.conditions, made, (&mut out, &mut left));
            }
        }
        self.consulting = false;
        let mut seen: Vec<String> = Vec::new();
        out.retain(|fact| if seen.contains(fact) { false } else { seen.push(fact.clone()); true });
        out
    }

    fn tried(&mut self, conditions: &[Condition], filling: Filling, (out, left): (&mut Vec<String>, &mut usize)) {
        let Some((first, rest)) = conditions.split_first() else { return };
        for (next, _) in condition_fits(first, self.facts, &filling, Time::MAX) {
            if *left == usize::default() {
                return;
            }
            *left -= 1;
            if let Condition::Fact { pattern, .. } = first
                && let Some(place) = fill(pattern, &next).and_then(|words| self.facts.place_of(&words))
            {
                if place < self.known {
                    out.push(fact_said(self.facts.fact(place)));
                } else {
                    let step = self.fact(place);
                    known_leaves(&step, out);
                }
            }
            self.tried(rest, next, (&mut *out, &mut *left));
        }
    }

    fn fact(&mut self, place: usize) -> ProofStep {
        let words = self.facts.fact(place).to_vec();
        if self.facts.is_told(&words) {
            return ProofStep::Told(fact_said(&words));
        }
        if place < self.known {
            return ProofStep::Known(fact_said(&words));
        }
        if self.found.get(&place) == Some(&false) || self.left == usize::default() {
            return ProofStep::Unfound(fact_said(&words));
        }
        self.left -= 1;
        let exact = self.rules.iter().filter(|rule| adds(rule) && !rule.conditions.is_empty());
        let defaults = self.rules.iter().filter(|rule| rule.kind == LineKind::AddsByDefault && !rule.conditions.is_empty());
        for rule in exact.chain(defaults) {
            let mut fits = Vec::new();
            fit_into(&rule.outcome, &words, &empty_filling(), &mut fits);
            for filling in fits {
                let readings = timed_readings(&rule.conditions, &empty_filling());
                if let Some(from) = self.holding((&rule.conditions, &readings), &rule.absent, filling, (Time::MAX, place), (Some((&rule.outcome, &words)), Some(rule))) {
                    self.found.insert(place, true);
                    return ProofStep::Made { fact: fact_said(&words), rule: rule_text(rule), from };
                }
            }
        }
        self.found.insert(place, false);
        ProofStep::Unfound(fact_said(&words))
    }

    fn holding(&mut self, (conditions, readings): (&[Condition], &[Option<bool>]), absent: &[Condition], filling: Filling, (anchor, before): (Time, usize), (goal, guard): (Option<(&Pattern, &[WordId])>, Option<&ToldRule>)) -> Option<Vec<ProofStep>> {
        let Some((first, rest)) = conditions.split_first() else {
            if !none_fits(absent, self.facts, &filling) || goal.is_some_and(|(outcome, made)| fill(outcome, &filling).as_deref() != Some(made)) || guard.is_some_and(|rule| !apart(rule, &filling)) {
                return None;
            }
            return Some(absent.iter().map(|condition| ProofStep::Absent(filled_text(condition, &filling), self.consulted(condition, &filling))).collect());
        };
        let timed = readings.first().copied().flatten();
        let later = readings.get(1..).unwrap_or_default();
        for (next, at) in condition_fits(first, self.facts, &filling, anchor) {
            let step = match first {
                Condition::Fact { pattern, .. } => {
                    let Some(place) = fill(pattern, &next).and_then(|words| self.facts.place_of(&words)) else { continue };
                    if place >= before {
                        continue;
                    }
                    self.fact(place)
                }
                Condition::Proves { statement, holds } => {
                    let Some(words) = fill(statement, &next) else { continue };
                    let from = refutation(&words, self.facts, !*holds).unwrap_or_default().into_iter().map(|told| told.map(|words| fact_said(&words))).collect();
                    ProofStep::Refuted { statement: fact_said(&words), from, holds: *holds }
                }
                _ => ProofStep::Worked(filled_text(first, &next)),
            };
            let read = timed.map(|open| ProofStep::Timed(format!("{TIMED_MARK} {}{}", filled_text(first, &next), reading_note(first, open, self.facts))));
            if let Some(mut steps) = self.holding((rest, later), absent, next, (at, before), (goal, guard)) {
                steps.insert(0, step);
                if let Some(read) = read {
                    steps.insert(0, read);
                }
                return Some(steps);
            }
        }
        None
    }
}

fn condition_places(condition: &Condition) -> Vec<usize> {
    let seen = std::cell::RefCell::new(Vec::new());
    condition_text(condition, &|logic_piece| {
        if let LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) = logic_piece {
            seen.borrow_mut().push(*place);
        }
        String::new()
    });
    seen.into_inner()
}
because!(condition_places, Proofs, "the open places that a condition names, in the order in which it writes them");

pub(super) fn reading_note(condition: &Condition, open: bool, facts: &Facts) -> String {
    let unkept = matches!(condition, Condition::Fact { when: Moment::AtThatTime | Moment::BeforeThatTime, .. }) && !facts.has_history();
    format!(" ({}{})", if open { NEWEST_READING } else { STATE_READING }, if unkept { NO_HISTORY_READING } else { "" })
}
because!(reading_note, Proofs, "the words after a condition held at a time that say which fact it read: the latest value of the state of the thing that it names, or, while a place of that thing is open, the newest fact of any thing, and that the facts keep no history when a condition at the time of a clause before it reads them so");

pub(super) fn timed_readings(conditions: &[Condition], start: &Filling) -> Vec<Option<bool>> {
    let mut bound: Vec<usize> = Vec::new();
    conditions
        .iter()
        .map(|condition| {
            let reading = match condition {
                Condition::Fact { pattern, when } if *when != Moment::Any => {
                    let thing = &pattern[..pattern.len().saturating_sub(1)];
                    Some(thing.iter().any(|logic_piece| matches!(logic_piece, LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) if !start.filled(*place) && !bound.contains(place))))
                }
                _ => None,
            };
            bound.extend(condition_places(condition));
            reading
        })
        .collect()
}
because!(timed_readings, Proofs, "for each condition of a rule or an answer, when only one fact may hold it by its place in the story, whether the thing that its sentence names (every piece but the last, the value, as English puts the new part last) still has an open place when the chain reads it, after the places that the start and the conditions before it fill: then it reads the one newest fact of any thing, and else the latest value of the state of that thing; nothing for a condition read at any time. The rules are read in the order in which they are written while a condition at a time is left, as the chain reads them");

pub fn overruled_proof(rules: &[ToldRule], facts: &Facts, known: usize, (forms, asked): (&[AskedForm], &[WordId])) -> Vec<ProofStep> {
    let defaults: Vec<&ToldRule> = rules.iter().filter(|rule| rule.kind == LineKind::AddsByDefault).collect();
    if defaults.is_empty() {
        return Vec::new();
    }
    let rivals = known_rivals(facts);
    let mut prover = Prover { rules, facts, known, left: MOST_STEPS, found: HashMap::new(), consulting: false };
    let mut out: Vec<ProofStep> = Vec::new();
    for form in forms.iter().filter(|form| asks(form, asked)) {
        let mut questions = Vec::new();
        fit_into(&form.question, asked, &empty_filling(), &mut questions);
        for (filling, reply) in questions.iter().flat_map(|filling| form.replies.iter().map(move |reply| (filling, reply))) {
            for condition in &reply.conditions {
                let Condition::Fact { pattern, .. } = condition else { continue };
                let Some(words) = fill(pattern, filling) else { continue };
                if facts.place_of(&words).is_some() {
                    continue;
                }
                for rule in &defaults {
                    let mut fits = Vec::new();
                    fit_into(&rule.outcome, &words, &empty_filling(), &mut fits);
                    let values: HashSet<WordId> = rule.outcome.iter().filter_map(|piece| if let LogicPiece::Word(word) = piece { Some(*word) } else { None }).collect();
                    let readings = timed_readings(&rule.conditions, &empty_filling());
                    let holds = fits.into_iter().any(|start| prover.holding((&rule.conditions, &readings), &rule.absent, start, (Time::MAX, usize::MAX), (Some((&rule.outcome, &words)), Some(rule))).is_some());
                    let rival = overruling_fact(&words, facts, (&values, &rivals)).and_then(|fact| facts.place_of(&fact));
                    if let (true, Some(place)) = (holds, rival) {
                        let step = ProofStep::Overruled { fact: fact_said(&words), rule: rule_text(rule), by: vec![prover.fact(place)] };
                        if !out.iter().any(|known| matches!((known, &step), (ProofStep::Overruled { fact: one, .. }, ProofStep::Overruled { fact: other, .. }) if one == other)) {
                            out.push(step);
                        }
                    }
                }
            }
        }
    }
    out
}
because!(overruled_proof, Proofs, "for an answer that holds otherwise, the defaults that a fact of the story ruled out on the way: each sentence of an answer before it, filled from the question, that a rule that holds usually would make, with the fact that overrules it and its proof (by default john is male; the story says that john is a mother, so john is female)");

pub fn answer_proof(rules: &[ToldRule], facts: &Facts, known: usize, conditions: &[Condition], absent: &[Condition], filling: &Filling) -> Vec<ProofStep> {
    let mut prover = Prover { rules, facts, known, left: MOST_STEPS, found: HashMap::new(), consulting: false };
    prover.holding((conditions, &timed_readings(conditions, filling)), absent, filling.clone(), (Time::MAX, usize::MAX), (None, None)).unwrap_or_default()
}
because!(answer_proof, Proofs, "the proof of an answer of a question form: the steps of its conditions under the filling with which it held, each fact with the derivation found for it; nothing when the conditions are found not to hold again");

const OVERRULED_MARK: &str = "a default is overruled:";
because!(OVERRULED_MARK, Proofs, "the words before the sentence that a rule that holds usually would make, when a fact of the story rules it out");

const OVERRULED_BY: &str = "as the story holds";
because!(OVERRULED_BY, Proofs, "the words before the steps of the fact that rules a default out");

const PROOF_INDENT: &str = "  ";
because!(PROOF_INDENT, Proofs, "the indent of the steps that a step rests on, so that the tree reads from the answer down to the told sentences");

const TOLD_MARK: &str = "told:";
because!(TOLD_MARK, Proofs, "the word before a sentence that the story told, where a proof ends");

pub(super) const KNOWN_MARK: &str = "known:";
because!(KNOWN_MARK, Proofs, "the mark of a fact that the topic knows before any story");

pub(super) const BY_MARK: &str = "by the rule:";
because!(BY_MARK, Proofs, "the words between a made fact and the rule that made it");

const SHOWN_MARK: &str = "(its proof is shown above)";
because!(SHOWN_MARK, Proofs, "what a proof says of a made fact that it already showed with its steps, so that a fact that two steps rest on is shown once");

const WORKED_MARK: &str = "worked out:";
because!(WORKED_MARK, Proofs, "the mark of a condition that the engine works out");

const ABSENT_MARK: &str = "no fact fits:";

const CONSULTED_MARK: &str = "read by the rules that would make it:";
because!(CONSULTED_MARK, Proofs, "the words before the known facts that the rules which would make an absent fact read, as the answer rests on those facts too: without them the fact could not be made, and the condition would hold for every case");
because!(ABSENT_MARK, Proofs, "the mark of a condition that must not hold");

const PROVED_MARK: &str = "proved by refutation:";
because!(PROVED_MARK, Proofs, "the mark of a statement that the clause block proves");

const DISPROVED_MARK: &str = "proved false by refutation:";
because!(DISPROVED_MARK, Proofs, "the mark of a statement that the clause block proves false");

const ASKED_MARK: &str = "(the statement asked, taken the other way)";
because!(ASKED_MARK, Proofs, "what a proof by refutation shows for the clauses of the asked statement, which the refutation adds as its opposite");

const UNFOUND_MARK: &str = "(no derivation found within the bound)";
because!(UNFOUND_MARK, Proofs, "the mark of a fact whose derivation the search did not find");

pub fn proof_lines(steps: &[ProofStep]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut shown = Vec::new();
    for step in steps {
        step_lines(step, usize::default(), &mut lines, &mut shown);
    }
    lines
}
because!(proof_lines, Proofs, "the steps of a proof as lines, each step with the steps that it rests on indented under it");

fn step_lines(step: &ProofStep, depth: usize, lines: &mut Vec<String>, shown: &mut Vec<String>) {
    let indent = PROOF_INDENT.repeat(depth);
    match step {
        ProofStep::Told(fact) => lines.push(format!("{indent}{TOLD_MARK} {fact}")),
        ProofStep::Known(fact) => lines.push(format!("{indent}{KNOWN_MARK} {fact}")),
        ProofStep::Made { fact, rule, from } => {
            if shown.contains(fact) {
                lines.push(format!("{indent}{fact} {SHOWN_MARK}"));
                return;
            }
            shown.push(fact.clone());
            lines.push(format!("{indent}{fact} {BY_MARK} {rule}"));
            for part in from {
                step_lines(part, depth + 1, lines, shown);
            }
        }
        ProofStep::Overruled { fact, rule, by } => {
            lines.push(format!("{indent}{OVERRULED_MARK} {fact} ({rule}), {OVERRULED_BY}"));
            for part in by {
                step_lines(part, depth + 1, lines, shown);
            }
        }
        ProofStep::Worked(text) => lines.push(format!("{indent}{WORKED_MARK} {text}")),
        ProofStep::Absent(text, known) => {
            lines.push(format!("{indent}{ABSENT_MARK} {text}"));
            if !known.is_empty() {
                lines.push(format!("{indent}{PROOF_INDENT}{CONSULTED_MARK}"));
            }
            for fact in known {
                lines.push(format!("{indent}{PROOF_INDENT}{PROOF_INDENT}{KNOWN_MARK} {fact}"));
            }
        }
        ProofStep::Refuted { statement, from, holds } => {
            lines.push(format!("{indent}{} {statement}", if *holds { PROVED_MARK } else { DISPROVED_MARK }));
            for told in from {
                let inner = PROOF_INDENT.repeat(depth + 1);
                match told {
                    Some(fact) => lines.push(format!("{inner}{TOLD_MARK} {fact}")),
                    None => lines.push(format!("{inner}{ASKED_MARK}")),
                }
            }
        }
        ProofStep::Unfound(fact) => lines.push(format!("{indent}{fact} {UNFOUND_MARK}")),
        ProofStep::Timed(text) => lines.push(format!("{indent}{text}")),
    }
}
because!(step_lines, Proofs, "one step of a proof and the steps under it as lines, at a depth, a made fact that was shown before given once");
