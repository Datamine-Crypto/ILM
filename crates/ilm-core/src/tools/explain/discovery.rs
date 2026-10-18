use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::explain::known_good::stored_source;
use crate::tools::explain::chain::{Facts, empty_filling, fill, fit_into, rule_text};
use crate::tools::explain::derive::{BY_MARK, KNOWN_MARK, ProofStep};
use crate::tools::explain::learn::{Library, proof_text};
use crate::tools::explain::provenance::{AnswerSteps, answer_steps};
use crate::tools::explain::bridges::{condition_orders, worded_rules};
use crate::tools::explain::states::{StateCounts, aligned, state_covers};
use crate::tools::explain::text::{Condition, LineKind, LogicPiece, Moment, Pattern, ToldRule, Topic, condition_text, fact_numbers, logic_piece, place_letter, read_topic};
use crate::base::lanes::{BUDGET_STOPPED, MOST_LEARNING_WORK, hardware_lanes, in_work_scope};
use crate::tools::call::{Lessons, Reading};
use crate::tools::explain::costs::{close_costs, open_costs};
use crate::tools::explain::EXPLAIN_TOOL;
use crate::tools::explain::wording::{common_words, fact_values, places_of, story_patterns, LEARNED_LINE_BREAK, LEAST_SUPPORT, SEED_SAMPLE, bounded_facts, case_place, checked_answers, form_line, learn_forms, library_writing, rules_and_readings, spread_sample, told_of, with_learned, person_lines, told_frame_and_forms};
use crate::tools::explain::words::{WordId, word_number, word_text, word_texts};
use patterns::{because, source};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use std::sync::{Arc, OnceLock};

pub struct DiscoveryReport;
source!(DiscoveryReport, "why a rule of a concept is or is not discovered from the lessons, so that a person sees into the search: for each kind in turn, the fact that its questions ask and where its name comes from, the rules that the stories suggest, each step of the search with the rules that it judged (right and wrong answers on the sample, and why a rule was refused), the rules discovered with what they build on and the proof of a lesson under them, and for each given rule of the concept whether it was discovered, and if not, the closest rules that were judged and the fact that the lessons miss");

#[derive(Clone)]
pub(super) struct Judged {
    pub(super) line: String,
    pub(super) right: usize,
    pub(super) wrong: usize,
    pub(super) wrong_before: usize,
    pub(super) allowed: bool,
    pub(super) narrow: bool,
    pub(super) touched: usize,
    pub(super) uses: usize,
    pub(super) bound: Option<(usize, usize)>,
    pub(super) marks: Vec<(usize, bool, bool)>,
}
because!(Judged, DiscoveryReport, "a rule as one step of the search judged it: its line (in its form of places of one word when that form took its place), the lessons of the sample that it makes right with the rules before it, the wrong answers then and before it, whether it was allowed (it adds no wrong answer), whether its form of places of one word was judged in place of it, and the keys of the order of ties: how many of the lessons still wrong it reads, and how many lessons suggest it; for a rule that the step did not judge, as the most lessons that it could make right fell below the score that the rules it keeps reached, those two numbers, so that the report still names every rule of the step; and for the rules that a report may show, each sample lesson's case place and whether the rule answers it right or wrong");

#[derive(Clone)]
pub(super) struct WrongLesson {
    pub(super) lesson: usize,
    pub(super) said: String,
    pub(super) proof: Vec<String>,
}
because!(WrongLesson, DiscoveryReport, "a lesson of the sample that a refused rule answers wrong where the rules before it did not: its place in the sample, what it said, and the proof of that answer");

#[derive(Clone)]
pub(super) struct SearchStep {
    pub(super) refused: Option<(String, Vec<WrongLesson>)>,
    pub(super) before: usize,
    pub(super) judged: Vec<Judged>,
    pub(super) pair: Option<(String, String, usize)>,
    pub(super) added: Vec<String>,
    pub(super) reached: usize,
    pub(super) stop: Option<&'static str>,
    pub(super) sample: usize,
}
because!(SearchStep, DiscoveryReport, "one step of the search for rules: the refused rule with the most right and the lessons that it answers wrong, the lessons of the sample right before it, every rule that it judged, the best pair of rules that it judged together with their right lessons, the rules that it added and the right lessons after them, and why the search stopped at this step, if it did, and how many lessons the sample of the step holds (more in a second round)");

pub(super) struct FormJoin {
    pub(super) members: Vec<String>,
    pub(super) joined: String,
    pub(super) beyond: Vec<String>,
}
because!(FormJoin, DiscoveryReport, "told sentence forms that the told learner joined into one form with a place where they differ: the forms, the joined form, and the words that its place reads in the told sentences of the lessons beyond the words of the forms (a join of took and grabbed that also reads dropped), as a joined place takes every word that a story puts there");

pub(super) struct RuleSearch {
    pub(super) way: String,
    pub(super) target: String,
    pub(super) answer_targets: Vec<(String, String)>,
    pub(super) states: Vec<String>,
    pub(super) candidates: Vec<(String, usize)>,
    pub(super) dropped: Vec<(String, usize)>,
    pub(super) steps: Vec<SearchStep>,
    pub(super) chosen: Vec<String>,
    pub(super) score: usize,
    pub(super) sample: usize,
    pub(super) kept: bool,
    pub(super) tried_states: Vec<(Vec<String>, usize, Vec<SearchStep>)>,
    pub(super) state_counts: Option<StateCounts>,
    pub(super) splits: Vec<(String, String, String)>,
    pub(super) stored: Option<(usize, usize)>,
}
because!(RuleSearch, DiscoveryReport, "one search for the rules of a concept for one kind, in one way of naming the asked fact: the way and the asked fact of a lesson in it, and of the first lesson of each other length of answer, with its answer (how the learner parsed an answer of several words, cellar no more), the invented state that the search ran over (its rules), if any, every state that the learner tried with the lessons of the sample that the rules over it made right and the steps of its search, and the counts of what its search for states read, the rules judged and the rules suggested but not judged, each with the lessons that suggest it, its steps, the rules that it chose and the lessons of the sample that they make right, whether the kind kept this search (of the ways tried, the one that answers most), and each rule chosen that was split through a sentence of the text, with the bridge and the rule that took its place; and, for a search that took the stored lines of its text in place of its steps, how many it kept of how many the store offered");

pub(super) const STOP_ALL_RIGHT: &str = "every lesson of the sample is right";
because!(STOP_ALL_RIGHT, DiscoveryReport, "why a search stops when nothing is left to learn on its sample");

pub(super) const STOP_NO_GAIN: &str = "no rule and no pair of rules makes more lessons of the sample right without a wrong answer";
because!(STOP_NO_GAIN, DiscoveryReport, "why a search stops when the best rule or pair that it may add gains nothing");

pub(super) const STOP_NO_RULE: &str = "no rule is left that adds no wrong answer";
because!(STOP_NO_RULE, DiscoveryReport, "why a search stops when every rule left was refused or its facts ran past the bound");

pub(super) const STOP_RAN_AWAY: &str = "the facts of a lesson ran past the bound under the rules chosen";
because!(STOP_RAN_AWAY, DiscoveryReport, "why a search stops when the facts of a lesson of the sample do not stop within their bound");

pub(super) const WAY_REPLY: &str = "named by a reply line of the text";
because!(WAY_REPLY, DiscoveryReport, "where the name of the asked fact comes from when a reply line of the text says the answer in a sentence");

pub(super) const WAY_LABEL: &str = "the question as a statement, for the lessons answered ";
because!(WAY_LABEL, DiscoveryReport, "where the name of the asked fact comes from for a kind of a few labels: the question itself, for the lessons of one label");

pub(super) const WAY_SPOKEN: &str = "named from the question and its answer, as no reply line names it";
because!(WAY_SPOKEN, DiscoveryReport, "where the name of the asked fact comes from when nothing else names it: the words of the question followed by the answer");

pub(super) const WAY_FORMED: &str = "named by the text's own question form ";
because!(WAY_FORMED, DiscoveryReport, "where the name of the asked fact comes from when the kind's questions have the form of a question form of the text whose answers read facts that no rule of the text makes and no story tells: the facts of that form, which the rules must make");

pub(super) const WAY_ALIGNED: &str = "named by the reply line ";
because!(WAY_ALIGNED, DiscoveryReport, "where the name of the asked fact comes from when a reply line of the text answers a question of another form with as many places: the reply line, then the order in which the lesson's values take its places");

const SHOWN_WRONG: usize = 2;
because!(SHOWN_WRONG, DiscoveryReport, "the lessons that a refused rule answers wrong whose proofs the report shows: a few show how it reads them wrong, as its faults repeat from story to story, and more make the tree long");

const INDENT: &str = "  ";
because!(INDENT, DiscoveryReport, "the indent of one level of the tree of the report, as a proof indents its steps");

const NO_TEXT: &str = "the library holds no text named";
because!(NO_TEXT, DiscoveryReport, "what the report says when the text that it is asked about is not in the library of the run");

const NOT_MARK: &str = "not ";
because!(NOT_MARK, DiscoveryReport, "the word before a condition of a rule that must not hold, as the library writes it");

const EXAMPLE_TRIES: usize = 40;
because!(EXAMPLE_TRIES, DiscoveryReport, "the last lessons of a kind whose proofs the report reads to find a lesson that the learned rules answer and the proof that each discovered rule stands in: a rule of the kind stands in some proof among a few dozen stories, and each proof chains a story again");

const FACT_SAMPLE: usize = 40;
because!(FACT_SAMPLE, DiscoveryReport, "the lessons of each kind whose facts, under all the rules discovered, tell whether a condition of a given rule fits any fact: a sentence form that the kind tells stands in many of its stories");

const CLOSEST_SHOWN: usize = 3;
because!(CLOSEST_SHOWN, DiscoveryReport, "the rules judged by the search that the report shows as the closest to a given rule that was not discovered: the first few tell whether the search came near it");

struct DiscoveredRule {
    line: String,
    text: String,
    kind: usize,
    whole: String,
    conditions: String,
    outcome: Pattern,
    roles: Vec<String>,
}
because!(DiscoveredRule, DiscoveryReport, "a rule or a fact (a rule with no condition, whose sentences are empty and whose text is the fact) that the search chose for a kind: its line, its text as a proof names it, the kind, and its sentences with their places named in the order in which they stand, whole and the conditions alone, its outcome, and the places of its outcome in the order in which they stand there, so that a given rule is matched to it whatever letters each uses, and a fact named in another way must hold the same places in the same order");

struct SeenRule {
    line: String,
    kind: usize,
    right: usize,
    wrong: usize,
    allowed: bool,
    reason: String,
    conditions: Vec<String>,
    alike: String,
    roles: Vec<String>,
}
because!(SeenRule, DiscoveryReport, "a rule that a step of the search judged, at its best step: its line, the kind, its right and wrong answers then, whether it was allowed, why it was not added (or that it was), its conditions each with its places named in order, so that a given rule finds the rules that came closest to it, and its conditions together with their places named in order and the places of its outcome, so that a given rule finds the rule that says the same");

static DISCOVERY_CLOCK: OnceLock<Instant> = OnceLock::new();
because!(DISCOVERY_CLOCK, DiscoveryReport, "when the report of a discovery started in this run, for the progress lines that it writes while it runs; never set on the board");

const PROGRESS_HEAD: &str = "discover-why: ";
because!(PROGRESS_HEAD, DiscoveryReport, "the words before a progress line of a discovery on the error output, as the board's progress lines name the board");

pub(super) fn progress(what: &str) {
    if let Some(clock) = DISCOVERY_CLOCK.get() {
        eprintln!("{PROGRESS_HEAD}{what} ({:.0} s so far)", clock.elapsed().as_secs_f64());
    }
}
because!(progress, DiscoveryReport, "a progress line of a discovery on the error output with the seconds since it started, so that a long run shows which kind and step it is at before its report is written (a run stopped at a limit leaves its last line)");

pub fn discovery_report(library: &Library, name: &str, kinds: &[(String, Vec<&Case>, Vec<&Case>)], given: &[(String, String)], (show, sources): (usize, &[Vec<String>])) -> Result<Vec<String>, String> {
    DISCOVERY_CLOCK.get_or_init(Instant::now);
    let place = library.topics.iter().position(|topic| topic.name == name).ok_or_else(|| format!("{NO_TEXT} {name}"))?;
    let mut topic = library.topics[place].clone();
    let mut base = Arc::clone(library.base(place));
    let names: Vec<String> = kinds.iter().map(|(file, _, _)| Path::new(file).file_stem().map_or_else(|| file.clone(), |stem| stem.to_string_lossy().to_string())).collect();
    let mut lines = vec![format!("discovery of the rules of the text {name}, {} kinds in turn, each on top of the rules discovered for the kinds before it", kinds.len())];
    let mut found: Vec<DiscoveredRule> = Vec::new();
    let mut seen: Vec<SeenRule> = Vec::new();
    let mut targets: Vec<(usize, Vec<WordId>)> = Vec::new();
    let mut unjudged: Vec<(String, usize, usize)> = Vec::new();
    let mut board_held: Vec<String> = Vec::new();
    for (at, (file, fit, check)) in kinds.iter().enumerate() {
        progress(&format!("kind {} of {}: {file}: the search for rules and readings", at + 1, kinds.len()));
        let mut searches: Vec<RuleSearch> = Vec::new();
        let mut reading_steps: Vec<SearchStep> = Vec::new();
        let mut joins: Vec<FormJoin> = Vec::new();
        let worked = in_work_scope(MOST_LEARNING_WORK, || rules_and_readings(&topic, &base, fit, (Some(&mut searches), Some(&mut reading_steps), Some(&mut joins))));
        let taught = worked.value;
        let rules = taught.lines;
        progress(&format!("kind {} of {}: {} lines found; the wording", at + 1, kinds.len(), rules.len()));
        let started = Instant::now();
        let (learned, learned_base) = with_learned(&topic, &base, &rules);
        let (forms, words) = learn_forms(&learned, &learned_base, fit);
        let checked = checked_answers(&learned, &learned_base, &forms, &words, check);
        let right = checked.learned;
        let source_of: HashMap<usize, &str> = fit.iter().zip(sources.get(at).map(Vec::as_slice).unwrap_or_default()).filter(|(_, source)| !source.is_empty()).map(|(case, source)| (case_place(case), source.as_str())).collect();
        let told_counts: HashMap<usize, usize> = fit.iter().map(|case| (case_place(case), case.sentences.len())).collect();
        let form_seconds = started.elapsed().as_secs_f64();
        lines.push(String::new());
        lines.push(format!("kind {} of {}: {file} ({} lessons to learn from, {} last lessons to check; seconds: rules {:.1}, readings {:.1}, wording {form_seconds:.1}; work of the chains of the searches for rules and readings: {})", at + 1, kinds.len(), fit.len(), check.len(), taught.rule_seconds, taught.reading_seconds, worked.work));
        if worked.stopped {
            lines.push(format!("{INDENT}{BUDGET_STOPPED} {} (the bound is {MOST_LEARNING_WORK}): the searches for rules and readings stopped at their next step, and the rules below are those that they found before it", worked.work));
        }
        lines.push(format!("{INDENT}readings of the told sentences into the facts that the rules of the text start from, sought only when the rules leave the sample wrong: {}", taught.readings.len()));
        if let Some(forms) = taught.forms_not_read {
            lines.push(format!("{INDENT}{INDENT}{FORMS_NOT_READ_HEAD}{forms}{FORMS_NOT_READ_TAIL}"));
        }
        for reading in &taught.readings {
            lines.push(format!("{INDENT}{INDENT}{reading}"));
        }
        lines.push(format!("{INDENT}readings learned and dropped, as the rules over them answered no more of the sample than the rules alone: {}", taught.dropped.len()));
        for reading in &taught.dropped {
            lines.push(format!("{INDENT}{INDENT}{reading}"));
        }
        let people = person_lines(fit);
        if !people.is_empty() {
            lines.push(format!("{INDENT}{PERSON_HEAD}{}", people.join(WORD_GAP)));
        }
        let (frame, told_forms, read) = told_frame_and_forms(fit);
        if !told_forms.is_empty() {
            lines.push(format!("{INDENT}{FRAME_HEAD}{}", frame.join(FRAME_GAP)));
            let shown: Vec<String> = told_forms.iter().take(FORMS_SHOWN).map(|(form, count)| format!("{form} ({count} of {read})")).collect();
            let more = if told_forms.len() > FORMS_SHOWN { format!("{FORMS_MORE_HEAD}{}{FORMS_MORE_TAIL}", told_forms.len() - FORMS_SHOWN) } else { String::new() };
            lines.push(format!("{INDENT}{TOLD_FORMS_HEAD}{}{more}", shown.join(BELOW_BAR_GAP)));
        }
        for join in &joins {
            let beyond = if join.beyond.is_empty() { JOINED_ONLY.to_string() } else { format!("{JOINED_BEYOND}{}", join.beyond.join(WORD_GAP)) };
            lines.push(format!("{INDENT}{JOINED_HEAD}{}{JOINED_FROM}{}{beyond}", join.joined, join.members.join(BELOW_BAR_GAP)));
        }
        if !reading_steps.is_empty() {
            lines.push(format!("{INDENT}the search for readings, judged by the filled answers of the facts that the rules of the text make:"));
            lines.extend(sample_line(&reading_steps, (&source_of, &told_counts)).map(|line| format!("{INDENT}{INDENT}{line}")));
            lines.extend(steps_lines(&reading_steps, sample_of(&reading_steps), (show, &source_of)).into_iter().map(|line| format!("{INDENT}{line}")));
        }
        for search in &searches {
            lines.extend(search_lines(search, (show, &source_of, &told_counts)));
            if search.kept {
                targets.push((at, fact_numbers(&search.target)));
            }
            unjudged.extend(search.dropped.iter().map(|(line, lessons)| (line.clone(), *lessons, at)));
            for step in &search.steps {
                let best = step.judged.iter().filter(|judged| judged.allowed).map(|judged| judged.right).max().unwrap_or_default();
                for judged in &step.judged {
                    let reason = if step.added.contains(&judged.line) { ADDED_MARK.to_string() } else { judged_reason(judged, best, step) };
                    match seen.iter_mut().find(|known| known.line == judged.line && known.kind == at) {
                        Some(known) if known.right >= judged.right && known.reason != ADDED_MARK => {}
                        Some(known) if known.reason == ADDED_MARK => {}
                        Some(known) => {
                            known.right = judged.right;
                            known.wrong = judged.wrong;
                            known.allowed = judged.allowed;
                            known.reason = reason;
                        }
                        None => {
                            let (sentences, outcome) = read_rule(name, &judged.line).map(|rule| rule_sentences(&rule)).unwrap_or_default();
                            let mut whole = sentences.clone();
                            whole.push(outcome);
                            seen.push(SeenRule { line: judged.line.clone(), kind: at, right: judged.right, wrong: judged.wrong, allowed: judged.allowed, reason, conditions: sentences.iter().map(|sentence| place_free(&[sentence.clone()])).collect(), alike: place_free(&sentences), roles: outcome_roles(&place_free(&whole)) });
                        }
                    }
                }
            }
        }
        for line in &rules {
            if let Some(rule) = read_rule(name, line) {
                let (conditions, outcome) = rule_sentences(&rule);
                let mut whole = conditions.clone();
                whole.push(outcome);
                let named = place_free(&whole);
                found.push(DiscoveredRule { line: line.clone(), text: rule_text(&rule), kind: at, roles: outcome_roles(&named), whole: named, conditions: place_free(&conditions), outcome: rule.outcome.clone() });
            } else if let Ok(read) = read_topic(name, line) {
                found.extend(read.facts.into_iter().map(|fact| DiscoveredRule { line: line.clone(), text: word_texts(&fact).join(WORD_GAP), kind: at, whole: String::new(), conditions: String::new(), outcome: fact.into_iter().map(LogicPiece::Word).collect(), roles: Vec::new() }));
            }
        }
        let model = read_topic(&topic.name, &forms.iter().map(form_line).collect::<Vec<String>>().join(LEARNED_LINE_BREAK)).ok().map(|read| Topic { questions: read.questions, ..learned.clone() });
        let proofs: Vec<(&Case, AnswerSteps)> = model.as_ref().map(|model| check.iter().take(EXAMPLE_TRIES).filter_map(|case| {
            let answered = answer_steps(model, &learned_base, &told_of(&case.sentences, &case.question, &model.rules), &fact_numbers(&case.question), false)?;
            (answer_words(&word_texts(&answered.words).join(WORD_GAP)) == answer_words(&case.answer)).then_some((*case, answered))
        }).collect()).unwrap_or_default();
        lines.push(format!("{INDENT}rules discovered for this kind: {}", rules.len()));
        let question_words: Vec<String> = fit.iter().flat_map(|case| fact_numbers(&case.question).into_iter().chain(case.sentences.iter().flat_map(|sentence| fact_numbers(sentence)))).map(|word| word_text(word).to_string()).collect();
        let replied = reply_words(&topic);
        for rule in found.iter().filter(|rule| rule.kind == at) {
            if rule.whole.is_empty() {
                let used = proofs.iter().filter(|(_, answered)| known_in(&answered.steps, &rule.text)).count();
                lines.push(format!("{INDENT}{INDENT}{} (a fact, with no condition; it stands in {used} of the {} proofs of the last lessons read)", rule.line, proofs.len()));
                continue;
            }
            lines.push(format!("{INDENT}{INDENT}{}", rule.line));
            if let Some(source) = stored_source(&rule.line) {
                lines.push(format!("{INDENT}{INDENT}{INDENT}{FROM_STORE_HEAD}{source}"));
            }
            match proofs.iter().find_map(|(_, answered)| find_made(&answered.steps, &rule.text)) {
                Some(ProofStep::Made { from, .. }) => {
                    for part in from {
                        lines.push(format!("{INDENT}{INDENT}{INDENT}builds on {}", rests_on(part, &found, &names)));
                    }
                }
                _ => lines.push(format!("{INDENT}{INDENT}{INDENT}it stands in no proof of the first {EXAMPLE_TRIES} last lessons that the learned forms answer right")),
            }
            let condition_words: Vec<String> = read_rule(name, &rule.line).map(|read| rule_sentences(&read).0.join(WORD_GAP).split_whitespace().map(str::to_lowercase).collect()).unwrap_or_default();
            let strange: Vec<String> = rule.outcome.iter().filter_map(|piece| match piece {
                LogicPiece::Word(word) => Some(word_text(*word).to_string()),
                _ => None,
            }).filter(|word| !question_words.contains(word) && !condition_words.contains(word) && !replied.contains(word)).collect();
            if !strange.is_empty() {
                lines.push(format!("{INDENT}{INDENT}{INDENT}audit: its fact holds words that neither its conditions nor the stories and questions of the kind nor a reply line hold ({}), so it may read against the wording of the text", strange.join(WORD_GAP)));
            }
            if let Some(read) = read_rule(name, &rule.line) {
                for condition in &read.conditions {
                    let Condition::Fact { pattern, .. } = condition else { continue };
                    let unsaid: Vec<String> = pattern.iter().filter_map(|piece| if let LogicPiece::Word(word) = piece { Some(word_text(*word).to_string()) } else { None }).filter(|word| !question_words.contains(word)).collect();
                    if !unsaid.is_empty() && unsaid.len() == pattern.iter().filter(|piece| matches!(piece, LogicPiece::Word(_))).count() {
                        lines.push(format!("{INDENT}{INDENT}{INDENT}audit: it reads a fact that the stories and the questions of the kind never state ({}), so it may be right through another thing that this fact marks (a name that can be she marks a person)", pattern_said(pattern)));
                    }
                }
            }
        }
        lines.push(format!("{INDENT}wording learned with them, judged on the last lessons: {right} of {} right, {} wrong, {} with no answer", check.len(), checked.wrong, checked.silent));
        for form in &forms {
            lines.push(format!("{INDENT}{INDENT}{} ({} of {} lessons)", form_line(form), form.right, form.lessons));
        }
        progress(&format!("kind {} of {}: wording {right} of {} right; the board's own model", at + 1, kinds.len(), check.len()));
        lines.extend(board_lines(name, (at, kinds), &mut board_held));
        match proofs.first() {
            Some((case, answered)) => {
                lines.push(format!("{INDENT}a last lesson and the proof of its answer under the rules:"));
                let patterns = story_patterns(&learned);
                for sentence in &case.sentences {
                    lines.push(format!("{INDENT}{INDENT}told: {sentence}"));
                    let read = match fact_values(&fact_numbers(sentence), &patterns) {
                        Some((pattern, values)) => format!("by the sentence \"{}\" as {}", pattern_said(&pattern), values.iter().map(|value| word_texts(value).join(WORD_GAP)).collect::<Vec<String>>().join(VALUE_GAP)),
                        None => NOT_READ.to_string(),
                    };
                    lines.push(format!("{INDENT}{INDENT}{INDENT}{READ_HEAD}{read}"));
                }
                lines.push(format!("{INDENT}{INDENT}question: {} answer: {}", case.question, case.answer));
                for proof_line in proof_text(name, answered) {
                    lines.push(format!("{INDENT}{INDENT}{}", annotated(&proof_line, &found, &names)));
                }
            }
            None => lines.push(format!("{INDENT}no last lesson among the first {EXAMPLE_TRIES} is answered right, so no proof is shown")),
        }
        topic = learned;
        base = learned_base;
    }
    if !given.is_empty() {
        progress(&format!("the given rules of {} texts against the {} rules discovered", given.len(), found.len()));
        lines.extend(given_lines((library, name, given), (&topic, &base), kinds, (&found, &seen, &targets, &names), (&unjudged, &board_held)));
    }
    progress("report written");
    Ok(lines)
}
because!(discovery_report, DiscoveryReport, "the report of the discovery of the rules of one text over some kinds in turn, as lines of a tree: for each kind its searches of the rules of the concept, and when they leave the sample wrong the readings of its told sentences (the told learner, with its steps) and the searches again on top of them, the rules discovered with what each builds on in a proof, an audit of words that its fact holds from nowhere and an audit of conditions that read a fact whose words the stories and questions never state (knowledge read as a mark of something else), the wording learned with them and its share on the last lessons, and a last lesson with the proof of its answer, each rule of the proof marked as discovered on a kind or given; then, when the given texts of one or more concepts are handed in, each of their rules with whether it was discovered and why not, and a summary line for each that a program reads");

const WORD_GAP: &str = " ";
because!(WORD_GAP, DiscoveryReport, "the space between the words of a sentence that the report writes");

fn search_lines(search: &RuleSearch, (show, sources, told): (usize, &HashMap<usize, &str>, &HashMap<usize, usize>)) -> Vec<String> {
    let mut lines = vec![format!("{INDENT}the asked fact: {}, as \"{}\"{}", search.way, search.target, if search.kept { "" } else { " (not kept: another way answers more of the sample)" })];
    for (answer, target) in &search.answer_targets {
        lines.push(format!("{INDENT}{INDENT}{ANSWER_TARGET_HEAD}\"{answer}\": \"{target}\""));
    }
    for state in &search.states {
        lines.push(format!("{INDENT}over an invented state: {state}"));
    }
    if let Some(counts) = &search.state_counts {
        lines.push(format!("{INDENT}the search for states read {} lessons: {} events of two places, {} pairs of events that name the same things ({} told in enough lessons); {} told events of {} asked things, whose roles explain {} of {} steps", counts.lessons, counts.events, counts.pairs, counts.kept, counts.told, counts.things, counts.explained, counts.steps));
        if !counts.below.is_empty() {
            lines.push(format!("{INDENT}{BELOW_BAR_HEAD}{}", counts.below.iter().map(|(form, count, of)| format!("{} ({count} of {of})", library_writing(form))).collect::<Vec<String>>().join(BELOW_BAR_GAP)));
        }
    }
    for (state, right, steps) in &search.tried_states {
        lines.push(format!("{INDENT}state tried: right {right} of {} with the rules over it: {}", search.sample, state.join(WORD_GAP)));
        if *state == search.states {
            lines.push(format!("{INDENT}{INDENT}{TRIED_KEPT}"));
            continue;
        }
        lines.extend(steps_lines(steps, search.sample, (show, sources)).into_iter().map(|line| format!("{INDENT}{INDENT}{line}")));
    }
    if let Some((taken, offered)) = search.stored {
        lines.push(format!("{INDENT}{STORED_TAKEN_HEAD}{taken} of {offered} stored lines{STORED_TAKEN_END}"));
    }
    lines.push(format!("{INDENT}the stories suggest {} rules; {} of them, each suggested by at least {LEAST_SUPPORT} lessons (and the lines that write a sentence again), are judged on a sample of {} lessons", search.candidates.len() + search.dropped.len(), search.candidates.len(), search.sample));
    lines.extend(sample_line(&search.steps, (sources, told)).map(|line| format!("{INDENT}{line}")));
    if search.steps.is_empty() {
        lines.push(format!("{INDENT}no step: {} of the sample are right before any rule", search.score));
    }
    lines.extend(steps_lines(&search.steps, search.sample, (show, sources)));
    lines.push(format!("{INDENT}rules chosen: {}, right on {} of {}", search.chosen.len(), search.score, search.sample));
    for (rule, bridge, step) in &search.splits {
        lines.push(format!("{INDENT}{INDENT}{SPLIT_HEAD}{rule}{SPLIT_AS}{bridge}{SPLIT_GAP}{step}"));
    }
    lines
}

fn sample_line(steps: &[SearchStep], (sources, told): (&HashMap<usize, &str>, &HashMap<usize, usize>)) -> Option<String> {
    let places: Vec<usize> = steps.iter().flat_map(|step| &step.judged).find(|judged| !judged.marks.is_empty())?.marks.iter().map(|(place, _, _)| *place).collect();
    let mean = |values: &[usize]| values.iter().sum::<usize>() as f64 / values.len().max(1) as f64;
    let sampled: Vec<usize> = places.iter().filter_map(|place| told.get(place).copied()).collect();
    let all: Vec<usize> = told.values().copied().collect();
    let mut kinds: Vec<(&str, usize)> = Vec::new();
    for place in &places {
        let Some(source) = sources.get(place).copied() else { continue };
        match kinds.iter_mut().find(|(known, _)| *known == source) {
            Some((_, count)) => *count += 1,
            None => kinds.push((source, 1)),
        }
    }
    let by_source = if kinds.is_empty() { String::new() } else { format!("{SAMPLE_SOURCES}{}", kinds.iter().map(|(source, count)| format!("{source} {count}")).collect::<Vec<String>>().join(BY_SOURCE_BETWEEN)) };
    Some(format!("{SAMPLE_HEAD}{} lessons with {:.1} told sentences on the mean, against {:.1} in all {} lessons{by_source}", places.len(), mean(&sampled), mean(&all), all.len()))
}
because!(sample_line, DiscoveryReport, "what the sample of a search holds beside all the lessons of the kind: its size, its told sentences on the mean against those of all the lessons, and its lessons by source kind for a pooled file, so that a sample that takes every n-th lesson of lessons in a periodic order (the first question of each story, one answer of three) shows; read from the lessons that the judged rules of the steps were marked on");

const SAMPLE_HEAD: &str = "the sample: ";
because!(SAMPLE_HEAD, DiscoveryReport, "the words that open the line of what the sample of a search holds");

const SAMPLE_SOURCES: &str = "; by source kind: ";
because!(SAMPLE_SOURCES, DiscoveryReport, "the words before the lessons of a sample by source kind");

const STORED_TAKEN_HEAD: &str = "the known-good store of the text: ";
because!(STORED_TAKEN_HEAD, DiscoveryReport, "the words that open the line of a search that took the stored lines of its text, before how many it kept of how many the store offered");

const STORED_TAKEN_END: &str = " taken, each judged on the sample (a line that made a wrong answer dropped), and no search ran, as they answer every lesson of the sample right";
because!(STORED_TAKEN_END, DiscoveryReport, "what the line of a search that took the store says after its counts: why no step follows");

const SPLIT_HEAD: &str = "split through a sentence of the text: ";
because!(SPLIT_HEAD, DiscoveryReport, "the words that open the line of a rule chosen that a bridge and a rule in the text's own words took the place of");

const SPLIT_AS: &str = " as ";
because!(SPLIT_AS, DiscoveryReport, "the words between a rule that was split and the two rules that took its place");

const SPLIT_GAP: &str = " / ";
because!(SPLIT_GAP, DiscoveryReport, "the mark between the bridge and the rule that took the place of a rule that was split");
const ANSWER_TARGET_HEAD: &str = "the asked fact of the first lesson whose answer has another length, answered ";
because!(ANSWER_TARGET_HEAD, DiscoveryReport, "the words that open the line of the asked fact of a lesson whose answer has another number of words than the lesson shown first, so that a reader sees how the learner parsed an answer of several words");

const FRAME_HEAD: &str = "the frame words of the told sentences (the words that stay when the told learner cuts a sentence into a form): ";
because!(FRAME_HEAD, DiscoveryReport, "the words that open the line of the frame words of the told sentences of a kind, by which a reader sees a word of a value that was taken as a frame word (room of store room)");

const FRAME_GAP: &str = ", ";
because!(FRAME_GAP, DiscoveryReport, "the gap between two frame words in their line");

const TOLD_FORMS_HEAD: &str = "the told sentence forms that the frame words cut, the most told first, with the lessons that tell each: ";
because!(TOLD_FORMS_HEAD, DiscoveryReport, "the words that open the line of the forms of the told sentences of a kind, which shows one sentence split into two forms by a word of a value");

const FORMS_SHOWN: usize = 12;
because!(FORMS_SHOWN, DiscoveryReport, "the told sentence forms shown in the line, the most told first: enough to see the common forms and a split of one sentence, while a kind of many forms keeps the line short");

const FORMS_MORE_HEAD: &str = "; and ";
because!(FORMS_MORE_HEAD, DiscoveryReport, "the words before the number of the told sentence forms that the line does not show");

const FORMS_MORE_TAIL: &str = " more forms";
because!(FORMS_MORE_TAIL, DiscoveryReport, "the words after the number of the told sentence forms that the line does not show");

const PERSON_HEAD: &str = "the invented state of the persons that the questions name, which the learned wording may read: ";
because!(PERSON_HEAD, DiscoveryReport, "the words that open the line of the lines of the state of the persons that the questions of a kind name, read from its told sentences (person_lines)");

const FORMS_NOT_READ_HEAD: &str = "no readings sought, as the board's own told side would not read this kind with this text (a text of rules and no question forms whose rules speak of the told sentences but do not yet read the stories, for a kind told in a few dozen forms); the first lessons tell ";
because!(FORMS_NOT_READ_HEAD, DiscoveryReport, "the words that open the line of a kind whose readings the report does not seek, as the board's told side would not read them, before the number of sentence forms that its first lessons tell");

const FORMS_NOT_READ_TAIL: &str = " sentence forms with enough uses";
because!(FORMS_NOT_READ_TAIL, DiscoveryReport, "the words after the number of sentence forms of a kind whose readings the report does not seek");

const JOINED_HEAD: &str = "told forms joined into one for the readings: ";
because!(JOINED_HEAD, DiscoveryReport, "the words that open the line of a join of told forms in the report, before the joined form");

const JOINED_FROM: &str = " from ";
because!(JOINED_FROM, DiscoveryReport, "the words between a joined told form and the forms that it joined");

const JOINED_BEYOND: &str = "; its place also reads ";
because!(JOINED_BEYOND, DiscoveryReport, "the words before the words that the place of a joined form reads beyond the words of the forms that it joined");

const JOINED_ONLY: &str = "; its place reads no other word";
because!(JOINED_ONLY, DiscoveryReport, "what the report says of a joined form whose place reads only the words of the forms that it joined");

pub const BELOW_BAR_HEAD: &str = "told sentences of asked things that too few lessons hold for a state to read them, with the lessons that hold each: ";
because!(BELOW_BAR_HEAD, DiscoveryReport, "the words that open the list of the told sentences that the search for states left out, as fewer lessons than the least share of a state hold them");

pub const BELOW_BAR_GAP: &str = "; ";
because!(BELOW_BAR_GAP, DiscoveryReport, "the mark between two told sentences in the list of those that the search for states left out");

const TRIED_KEPT: &str = "the search over this state is the one below, with its steps";
because!(TRIED_KEPT, DiscoveryReport, "what the report says under the state tried that the search kept, whose steps it writes once, as the steps of the search");
because!(search_lines, DiscoveryReport, "one search as lines of the tree: the asked fact and where its name comes from, the states tried with the steps of the search over each that the search did not keep, how many rules were suggested and judged, its steps, and the rules chosen");

const SECOND_ROUND: &str = "a second round, on ";
because!(SECOND_ROUND, DiscoveryReport, "the words that open the line of a second round of a search, before the number of lessons of its sample");

const SECOND_ROUND_WHY: &str = ": the sample and the lessons whose stories suggested the rules that the rules chosen answer wrong";
because!(SECOND_ROUND_WHY, DiscoveryReport, "what the sample of a second round of a search holds, after its number of lessons");

fn by_source(judged: &Judged, sources: &HashMap<usize, &str>) -> Option<String> {
    let mut counted: Vec<(&str, usize, usize)> = Vec::new();
    for (place, right, wrong) in &judged.marks {
        let source = sources.get(place).copied().unwrap_or(NO_SOURCE);
        match counted.iter_mut().find(|(known, _, _)| *known == source) {
            Some((_, rights, wrongs)) => {
                *rights += usize::from(*right);
                *wrongs += usize::from(*wrong);
            }
            None => counted.push((source, usize::from(*right), usize::from(*wrong))),
        }
    }
    (counted.len() > 1).then(|| format!("{BY_SOURCE_HEAD}{}", counted.iter().map(|(source, rights, wrongs)| format!("{source} right {rights} wrong {wrongs}")).collect::<Vec<String>>().join(BY_SOURCE_BETWEEN)))
}
because!(by_source, DiscoveryReport, "right and wrong of a shown rule on the sample by the source kind of each lesson, in the order in which the sources first come, when its lessons come from more than one kind (a pooled case file names the source of each case), so that a rule that one kind's lessons pay for with another kind's wrong answers shows; nothing for lessons of one source");

const NO_SOURCE: &str = "no source named";
because!(NO_SOURCE, DiscoveryReport, "the source of a sample lesson whose case names none, in the count by source kind");

const BY_SOURCE_HEAD: &str = "by source kind: ";
because!(BY_SOURCE_HEAD, DiscoveryReport, "the words before the counts of a shown rule by source kind");

const BY_SOURCE_BETWEEN: &str = "; ";
because!(BY_SOURCE_BETWEEN, DiscoveryReport, "the mark between the source kinds of a shown rule's counts");

fn steps_lines(steps: &[SearchStep], sample: usize, (show, sources): (usize, &HashMap<usize, &str>)) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for (at, step) in steps.iter().enumerate() {
        let alone = !step.judged.is_empty() && step.judged.iter().all(|judged| judged.line.starts_with(RULE_STARTS[1]));
        if step.sample > sample && (at == usize::default() || steps[at - 1].sample != step.sample) {
            lines.push(format!("{INDENT}{SECOND_ROUND}{} lessons{SECOND_ROUND_WHY}", step.sample));
        }
        let unjudged = step.judged.iter().filter(|judged| judged.bound.is_some()).count();
        let unjudged_note = if unjudged > usize::default() { format!("{UNJUDGED_BETWEEN}{unjudged}{UNJUDGED_COUNTED}") } else { String::new() };
        lines.push(format!("{INDENT}step {}: {} of {} right before it; {} rules judged{unjudged_note}{}", at + 1, step.before, step.sample.max(sample), step.judged.len() - unjudged, if alone { " (the lines that write a sentence again, tried alone first)" } else { "" }));
        for line in &step.added {
            let alone = step.judged.iter().find(|judged| judged.line == *line).map(|judged| format!(" (alone: right {}, wrong {}{})", judged.right, judged.wrong, if judged.narrow { "; judged with its places of one word" } else { "" })).unwrap_or_default();
            lines.push(format!("{INDENT}{INDENT}added: {line}{alone}"));
            if let Some(counts) = step.judged.iter().find(|judged| judged.line == *line).and_then(|judged| by_source(judged, sources)) {
                lines.push(format!("{INDENT}{INDENT}{INDENT}{counts}"));
            }
        }
        if let Some((one, other, right)) = &step.pair {
            let taken = step.added.len() > 1;
            lines.push(format!("{INDENT}{INDENT}best pair{}: {one} / {other}: right {right} together", if taken { " (added)" } else { " (not added)" }));
        }
        if !step.added.is_empty() {
            lines.push(format!("{INDENT}{INDENT}right after it: {} of {}", step.reached, step.sample.max(sample)));
        }
        let best = step.judged.iter().filter(|judged| judged.allowed).map(|judged| judged.right).max().unwrap_or_default();
        let mut beaten: Vec<&Judged> = step.judged.iter().filter(|judged| !step.added.contains(&judged.line)).collect();
        beaten.sort_by(|one, other| other.right.cmp(&one.right).then_with(|| other.allowed.cmp(&one.allowed)));
        if !beaten.is_empty() && show > 0 {
            lines.push(format!("{INDENT}{INDENT}beaten (most right first):"));
        }
        for judged in beaten.into_iter().take(show) {
            if judged.bound.is_some() {
                lines.push(format!("{INDENT}{INDENT}{INDENT}{}: {}", judged.line, judged_reason(judged, best, step)));
                continue;
            }
            lines.push(format!("{INDENT}{INDENT}{INDENT}{}: right {}, wrong {}: {}", judged.line, judged.right, judged.wrong, judged_reason(judged, best, step)));
            if let Some(counts) = by_source(judged, sources) {
                lines.push(format!("{INDENT}{INDENT}{INDENT}{INDENT}{counts}"));
            }
        }
        if let Some((rule, wrongs)) = &step.refused {
            lines.push(format!("{INDENT}{INDENT}the refused rule with the most right of the rules judged, {rule}, answers wrong where the rules before it did not: {} lessons", wrongs.len()));
            for wrong in wrongs.iter().take(SHOWN_WRONG) {
                lines.push(format!("{INDENT}{INDENT}{INDENT}lesson {} of the sample: {}", wrong.lesson + 1, wrong.said));
                lines.extend(wrong.proof.iter().map(|line| format!("{INDENT}{INDENT}{INDENT}{INDENT}{line}")));
            }
        }
        if let Some(stop) = step.stop {
            lines.push(format!("{INDENT}{INDENT}stopped: {stop}"));
        }
    }
    lines
}
because!(steps_lines, DiscoveryReport, "the steps of a search as lines of the tree: each step with the rules that it added (with their counts alone), the best pair, the right lessons after it, the rules that it beat with their counts and why each lost, and why the search stopped");

const ADDED_MARK: &str = "added";
because!(ADDED_MARK, DiscoveryReport, "what the report says of a rule that a step added, where it says of the others why they lost");

fn sample_of(steps: &[SearchStep]) -> usize {
    steps.iter().map(|step| step.reached.max(step.before)).max().unwrap_or_default().max(SEED_SAMPLE)
}
because!(sample_of, DiscoveryReport, "the lessons of the sample of a search for readings, which its steps do not carry: the sample of a first reading, or more when a step made more right (a second pass adds the lessons that the first left wrong)");

fn board_lines(name: &str, (at, kinds): (usize, &[(String, Vec<&Case>, Vec<&Case>)]), held: &mut Vec<String>) -> Vec<String> {
    let (_, fit, check) = &kinds[at];
    let all: Vec<Case> = fit.iter().chain(check.iter()).map(|case| (*case).clone()).collect();
    let earlier: Vec<Vec<Case>> = kinds[..at].iter().map(|(_, fit, check)| fit.iter().chain(check.iter()).map(|case| (*case).clone()).collect()).collect();
    let before = || earlier.clone();
    open_costs();
    let worked = in_work_scope(MOST_LEARNING_WORK, || (EXPLAIN_TOOL.learn)(Lessons { cases: &all, labels: &[], ask: None, classes: None, before: Some(&before) }, hardware_lanes()));
    let spent = close_costs();
    let stop = if worked.stopped { format!(" ({BUDGET_STOPPED} {})", worked.work) } else { String::new() };
    let costs = format!("{INDENT}{COSTS_HEAD}{}", spent.line);
    let model = match worked.value {
        Ok(model) => model,
        Err(why) => return vec![format!("{INDENT}the board's own model for this kind{stop}: none ({why})"), costs],
    };
    let note = check.first().map(|case| model.answer(&case.sentences, &case.question).note).unwrap_or_default();
    let used: Vec<(String, bool)> = check.iter().take(BOARD_PROOFS).flat_map(|case| model.rules_used(&case.sentences, &case.question)).collect();
    let mut given: Vec<&str> = used.iter().filter(|(_, held)| *held).map(|(rule, _)| rule.as_str()).collect();
    given.sort_unstable();
    given.dedup();
    let mut found: Vec<&str> = used.iter().filter(|(_, held)| !*held).map(|(rule, _)| rule.as_str()).collect();
    found.sort_unstable();
    found.dedup();
    let learned: Vec<String> = model.lines_from_lessons().into_iter().filter(|line| !line.starts_with(FORM_START)).collect();
    let reading = match model.reading() {
        Some(Reading::Learned) => READ_LEARNED,
        Some(Reading::Mixed) => READ_MIXED,
        Some(Reading::Written) => READ_WRITTEN,
        None => READ_NONE,
    };
    let mut lines = vec![format!("{INDENT}the board's own model for this kind ({reading}){stop} answers a last lesson by: {note}; the proofs of {} last lessons rest on {} given and {} discovered rules; it learned {} lines of rules and facts from the lessons", check.len().min(BOARD_PROOFS), given.len(), found.len(), learned.len())];
    for line in learned {
        lines.push(format!("{INDENT}{INDENT}{line}"));
        held.push(match read_rule(name, &line) {
            Some(rule) => {
                let (conditions, outcome) = rule_sentences(&rule);
                let mut whole = conditions;
                whole.push(outcome);
                place_free(&whole)
            }
            None => read_topic(name, &line).ok().and_then(|read| read.facts.into_iter().next()).map(|fact| word_texts(&fact).join(WORD_GAP)).unwrap_or(line),
        });
    }
    let forms: Vec<String> = model.lines_from_lessons().into_iter().filter(|line| line.starts_with(FORM_START)).collect();
    if !forms.is_empty() {
        lines.push(format!("{INDENT}{BOARD_FORMS_HEAD}"));
        lines.extend(forms.iter().map(|form| format!("{INDENT}{INDENT}{form}")));
    }
    lines.push(costs);
    lines
}
because!(board_lines, DiscoveryReport, "what the board itself learns for a kind, beside the curriculum of the report: the explain tool on all the lessons of the kind, with the kinds before it in the run as the kinds before it in its set, as the board learns it, under the same bound of work; the text and the answer by which it reads a last lesson, the given and discovered rules that the proofs of some last lessons rest on, and the rules and facts that it learned, which the report keeps to count a given rule as discovered only when the board can use the line found");

const BOARD_PROOFS: usize = 20;
because!(BOARD_PROOFS, DiscoveryReport, "the last lessons whose proofs tell which rules the board's own model rests on, as the board's column of the rules reads them");

const FORM_START: &str = "To answer";

const BOARD_FORMS_HEAD: &str = "the learned question forms of the board's own model, in the order that the engine tries them:";
because!(BOARD_FORMS_HEAD, DiscoveryReport, "the words that open the learned question forms of the board's own model, each a decision list of answers with their conditions, so that a reader sees which branch answers a lesson and which branch comes first");
because!(FORM_START, DiscoveryReport, "the start of a learned question form among the learned lines of a model, which the count of rules and facts leaves out");

const READ_LEARNED: &str = "learned";
because!(READ_LEARNED, DiscoveryReport, "the word for a model that reads its kind by lines learned from its lessons alone, as the board writes it");

const READ_MIXED: &str = "mixed";
because!(READ_MIXED, DiscoveryReport, "the word for a model that reads its kind by learned lines and lines of the library");

const READ_WRITTEN: &str = "written";
because!(READ_WRITTEN, DiscoveryReport, "the word for a model that reads its kind by lines of the library alone");

const READ_NONE: &str = "no reading said";
because!(READ_NONE, DiscoveryReport, "the word for a model that does not say how it reads its kind");

const NOT_JUDGED: &str = "not judged: at most ";
because!(NOT_JUDGED, DiscoveryReport, "the words before the most lessons that a rule could make right, for a rule that a step of the search did not judge as that falls below the score that the rules it keeps reached");

const NOT_JUDGED_BELOW: &str = " right, below ";
because!(NOT_JUDGED_BELOW, DiscoveryReport, "the words between the most lessons that a rule not judged could make right and the score below which that falls");

const UNJUDGED_BETWEEN: &str = ", ";
because!(UNJUDGED_BETWEEN, DiscoveryReport, "the marks between the rules that a step of the search judged and those it did not judge, in the line of the step");

const UNJUDGED_COUNTED: &str = " not judged, as the most lessons that each could make right fall below the score of the rules that the step keeps";
because!(UNJUDGED_COUNTED, DiscoveryReport, "the words after the count of the rules that a step of the search did not judge, in the line of the step");

fn judged_reason(judged: &Judged, best: usize, step: &SearchStep) -> String {
    if let Some((most, least)) = judged.bound {
        return format!("{NOT_JUDGED}{most}{NOT_JUDGED_BELOW}{least}");
    }
    let narrow = if judged.narrow { " (judged with its places of one word)" } else { "" };
    if !judged.allowed {
        return format!("refused, as the wrong answers grow from {} to {}{narrow}", judged.wrong_before, judged.wrong);
    }
    if judged.right < best {
        return format!("fewer right{narrow}");
    }
    let Some(winner) = step.judged.iter().find(|other| other.allowed && other.right == best && step.added.contains(&other.line)) else {
        return format!("as many right as the best rule, which a pair of rules that makes more right together went past{narrow}");
    };
    let words = |line: &str| line.split_whitespace().count();
    let key = if judged.touched != winner.touched {
        format!("it reads {} of the lessons still wrong, the rule added {}", judged.touched, winner.touched)
    } else if words(&judged.line) != words(&winner.line) {
        format!("it is {} words long, the rule added {}", words(&judged.line), words(&winner.line))
    } else if judged.uses != winner.uses {
        format!("{} lessons suggest it, {} the rule added", judged.uses, winner.uses)
    } else {
        "it comes later in the order of the rules suggested".to_string()
    };
    format!("as many right as the rule added, behind on the order of ties: {key}{narrow}")
}
because!(judged_reason, DiscoveryReport, "why a rule that a step judged was not added: it adds wrong answers (the search adds only a rule that adds none), it makes fewer lessons right than the best allowed rule, or it ties with the rule added and loses on the first key of the order of ties that differs (the lessons still wrong that it reads, its length, the lessons that suggest it), or the step added a pair that makes more right together");

fn read_rule(name: &str, line: &str) -> Option<ToldRule> {
    read_topic(name, line).ok()?.rules.into_iter().next()
}
because!(read_rule, DiscoveryReport, "the rule of one line of the library, as the reader reads it");

fn rule_sentences(rule: &ToldRule) -> (Vec<String>, String) {
    let piece = |logic_piece: &LogicPiece| match logic_piece {
        LogicPiece::Word(word) => word_text(*word).to_string(),
        LogicPiece::Open(place) => place_letter(*place),
        LogicPiece::OpenWithEnd(place, ending) => format!("{}{ending}", place_letter(*place)),
    };
    let conditions = rule.conditions.iter().map(|condition| condition_text(condition, &piece)).chain(rule.absent.iter().map(|condition| format!("{NOT_MARK}{}", condition_text(condition, &piece)))).collect();
    (conditions, rule.outcome.iter().map(piece).collect::<Vec<String>>().join(WORD_GAP))
}
because!(rule_sentences, DiscoveryReport, "the sentences of a rule with the letters of their places: each condition (one that must not hold after its word not) and the outcome");

fn place_free(sentences: &[String]) -> String {
    let mut places: Vec<String> = Vec::new();
    let mut named = |token: &str, ending: &str| -> String {
        let at = places.iter().position(|known| known == token).unwrap_or_else(|| {
            places.push(token.to_string());
            places.len() - 1
        });
        format!("{}{ending}", place_letter(at))
    };
    sentences
        .iter()
        .map(|sentence| {
            sentence
                .split_whitespace()
                .map(|token| match logic_piece(token) {
                    LogicPiece::Open(_) => named(token, ""),
                    LogicPiece::OpenWithEnd(_, ending) => {
                        let stem = token.strip_suffix(ending.as_str()).unwrap_or(token).to_string();
                        named(&stem, &ending)
                    }
                    LogicPiece::Word(_) => token.to_lowercase(),
                })
                .collect::<Vec<String>>()
                .join(WORD_GAP)
        })
        .collect::<Vec<String>>()
        .join(LEARNED_LINE_BREAK)
}
because!(place_free, DiscoveryReport, "sentences with their places named again by the order in which they first stand, a place of one word as any place, and every word in small letters, so that two rules that say the same with other letters read the same");

fn find_made<'a>(steps: &'a [ProofStep], rule: &str) -> Option<&'a ProofStep> {
    steps.iter().find_map(|step| match step {
        ProofStep::Made { rule: made, from, .. } => if made == rule { Some(step) } else { find_made(from, rule) },
        _ => None,
    })
}
fn known_in(steps: &[ProofStep], fact: &str) -> bool {
    steps.iter().any(|step| match step {
        ProofStep::Known(known) => known == fact,
        ProofStep::Made { from, .. } => known_in(from, fact),
        _ => false,
    })
}
because!(known_in, DiscoveryReport, "whether a proof rests on a given known fact, searched through the steps that each step rests on");

because!(find_made, DiscoveryReport, "the first step of a proof that a given rule made, searched through the steps that each step rests on");

fn rests_on(step: &ProofStep, found: &[DiscoveredRule], names: &[String]) -> String {
    match step {
        ProofStep::Told(fact) => format!("the told sentence \"{fact}\""),
        ProofStep::Known(fact) => match found.iter().find(|known| known.whole.is_empty() && known.text == *fact) {
            Some(known) => format!("the fact \"{fact}\", discovered on {}", names[known.kind]),
            None => format!("the known fact \"{fact}\""),
        },
        ProofStep::Made { fact, rule, .. } => match found.iter().find(|known| known.text == *rule) {
            Some(known) => format!("\"{fact}\", made by a rule discovered on {}: {}", discovered_on(known, names), known.line),
            None => format!("\"{fact}\", made by a given rule: {rule}"),
        },
        ProofStep::Worked(text) => format!("a condition worked out: {text}"),
        ProofStep::Absent(text, _) => format!("a condition that no fact fits: {text}"),
        ProofStep::Refuted { statement, .. } => format!("a statement proved by refutation: {statement}"),
        ProofStep::Unfound(fact) => format!("\"{fact}\", whose derivation was not found within the bound"),
        ProofStep::Timed(text) => text.clone(),
        ProofStep::Overruled { fact, rule, .. } => format!("a default that the story overrules: \"{fact}\" ({rule})"),
    }
}
because!(rests_on, DiscoveryReport, "what a step under a discovered rule is, in words: a told sentence, a known fact, a fact made by a rule discovered on an earlier kind or by a given rule, or a condition of another sort");

fn annotated(line: &str, found: &[DiscoveredRule], names: &[String]) -> String {
    if line.trim_start().starts_with(KNOWN_MARK) {
        return match found.iter().find(|known| known.whole.is_empty() && line.ends_with(&known.text)) {
            Some(known) => format!("{line} [discovered on {}]", names[known.kind]),
            None => line.to_string(),
        };
    }
    if !line.contains(BY_MARK) {
        return line.to_string();
    }
    match found.iter().find(|known| line.ends_with(&known.text)) {
        Some(known) => format!("{line} [discovered on {}]", discovered_on(known, names)),
        None => format!("{line} [given]"),
    }
}

fn discovered_on(known: &DiscoveredRule, names: &[String]) -> String {
    stored_source(&known.line).map_or_else(|| names[known.kind].clone(), |source| format!("{source}{STORED_SOURCE_END}"))
}
because!(discovered_on, DiscoveryReport, "the kind that a rule of a proof was discovered on: the source that the known-good store names for a stored line, or the kind of the report that discovered it");

const STORED_SOURCE_END: &str = ", from the store";
because!(STORED_SOURCE_END, DiscoveryReport, "the words after the source of a stored rule in a proof, so that a rule that an earlier kind discovered is told from one that the kind of the report discovered");

const FROM_STORE_HEAD: &str = "from the known-good store, discovered on ";
because!(FROM_STORE_HEAD, DiscoveryReport, "the words before the source of a rule of the report that the kind took from the store in place of a search");
because!(annotated, DiscoveryReport, "a line of a proof with the rule that it names marked as discovered on a kind or given");

fn reply_words(topic: &Topic) -> Vec<String> {
    topic.reply_lines.iter().flat_map(|line| line.said.iter().filter_map(|piece| match piece {
        LogicPiece::Word(word) => Some(word_text(*word).to_string()),
        _ => None,
    })).collect()
}
because!(reply_words, DiscoveryReport, "the words that the reply lines of a text say, which an asked fact named by a reply line may hold");

fn lesson_facts(topic: &Topic, base: &Arc<Facts>, kinds: &[(String, Vec<&Case>, Vec<&Case>)]) -> Vec<Vec<Vec<WordId>>> {
    let first = base.added_to(&Facts::default());
    kinds
        .iter()
        .map(|(_, fit, _)| {
            spread_sample(fit, FACT_SAMPLE)
                .into_iter()
                .filter_map(|case| bounded_facts(case, base, topic, false))
                .flat_map(|facts| (usize::default()..first + facts.added_to(base)).map(|place| facts.fact(place).to_vec()).collect::<Vec<Vec<WordId>>>())
                .collect()
        })
        .collect()
}
because!(lesson_facts, DiscoveryReport, "for each kind, the facts of some of its lessons under all the rules discovered, with the facts that the text knows before any story");

fn fits_any(pattern: &Pattern, facts: &[Vec<WordId>]) -> bool {
    facts.iter().any(|words| {
        let mut fits = Vec::new();
        fit_into(pattern, words, &empty_filling(), &mut fits);
        !fits.is_empty()
    })
}
because!(fits_any, DiscoveryReport, "whether some fact fits a sentence of a rule");

type FoundRules<'a> = (&'a [DiscoveredRule], &'a [SeenRule], &'a [(usize, Vec<WordId>)], &'a [String]);
because!(FoundRules, DiscoveryReport, "what the searches of the kinds found, for the report of the given rules: the rules discovered, every rule judged at its best step, the asked fact of each kind, and the names of the kinds");

const STATE_COVERED: &str = "two ways to start or to end one invented state, so the line says no more than the state";
because!(STATE_COVERED, DiscoveryReport, "why a given line of one sentence counts as discovered when two discovered rules make its sentence and its outcome one fact of the same shape: a reading of told words into another (If A took the B then A got the B) only says that both start the same state");

const STATE_PAIR_GAP: &str = " / ";
because!(STATE_PAIR_GAP, DiscoveryReport, "the mark between the two discovered rules that together cover a given line");

fn given_lines((library, name, texts): (&Library, &str, &[(String, String)]), (topic, base): (&Topic, &Arc<Facts>), kinds: &[(String, Vec<&Case>, Vec<&Case>)], (found, seen, targets, names): FoundRules, (unjudged, board_held): (&[(String, usize, usize)], &[String])) -> Vec<String> {
    let given: Vec<(String, ToldRule, &str)> = texts.iter().flat_map(|(concept, text)| text.lines().map(str::trim).filter(|line| RULE_STARTS.iter().any(|start| line.starts_with(start))).filter_map(move |line| Some((line.to_string(), read_rule(name, line)?, concept.as_str())))).collect();
    let facts = lesson_facts(topic, base, kinds);
    let builders: Vec<Topic> = library.texts.iter().filter(|text| text.builds_on.iter().any(|built| built == name || texts.iter().any(|(concept, _)| built == concept))).cloned().collect();
    let readers: Vec<Pattern> = texts.iter().filter_map(|(concept, text)| read_topic(concept, text).ok()).chain(builders.iter().cloned()).flat_map(|read| read.rules.into_iter().flat_map(|rule| rule.conditions.into_iter()).chain(read.questions.into_iter().flat_map(|form| form.replies.into_iter().flat_map(|reply| reply.conditions.into_iter())))).filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } => Some(pattern),
        _ => None,
    }).collect();
    let read_by_others = |outcome: &Pattern| -> bool {
        let words = stand_in(outcome);
        readers.iter().any(|reader| {
            let mut fits = Vec::new();
            fit_into(reader, &words, &empty_filling(), &mut fits);
            !fits.is_empty()
        })
    };
    let unused = |known: &DiscoveredRule| -> Option<String> {
        let held = if known.whole.is_empty() { board_held.contains(&known.text) } else { board_held.contains(&known.whole) };
        (!held).then(|| format!("{NOT_HELD_MARK}{} on {}, but no model that the board learns for these kinds holds it", known.line, names[known.kind]))
    };
    let own: Vec<String> = given.iter().flat_map(|(_, rule, _)| {
        let (sentences, said) = rule_sentences(rule);
        sentences.into_iter().chain(std::iter::once(said)).map(|sentence| place_free(&[sentence])).collect::<Vec<String>>()
    }).collect();
    let readings: Vec<(ToldRule, String)> = builders.iter().flat_map(|text| text.rules.iter().map(move |rule| (rule.clone(), text.name.clone()))).filter(|(rule, _)| rule_sentences(rule).0.first().is_some_and(|told| !own.contains(&place_free(std::slice::from_ref(told))))).collect();
    let reading_rules: Vec<ToldRule> = readings.iter().map(|(rule, _)| rule.clone()).collect();
    let asked_facts: Vec<String> = texts.iter().filter_map(|(concept, text)| read_topic(concept, text).ok()).chain(builders).flat_map(|read| read.questions.into_iter()).flat_map(|form| form.replies.into_iter()).flat_map(|reply| reply.conditions.into_iter()).filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } => Some(place_free(&[pattern_said(&pattern)])),
        _ => None,
    }).collect();
    let told_raw: Vec<Vec<WordId>> = kinds.iter().flat_map(|(_, fit, _)| spread_sample(fit, FACT_SAMPLE).into_iter().flat_map(|case| case.sentences.iter().map(|sentence| fact_numbers(sentence)))).collect();
    let fits_words = |pattern: &Pattern, words: &[WordId]| {
        let mut out = Vec::new();
        fit_into(pattern, words, &empty_filling(), &mut out);
        !out.is_empty()
    };
    let written_first = |pattern: &Pattern| -> String {
        let Some(words) = told_raw.iter().find(|words| fits_words(pattern, words)) else { return String::new() };
        let first = topic.rules.iter().find(|rule| matches!(rule.kind, LineKind::Replaces | LineKind::ReplacesInside) && rule.conditions.first().is_some_and(|condition| matches!(condition, Condition::Fact { pattern: old, .. } if fits_words(old, words))));
        first.map(|rule| format!("; a told sentence holds it (\"{}\"), but the line \"{}\" writes that sentence again first", word_texts(words).join(WORD_GAP), rule_text(rule))).unwrap_or_default()
    };
    let mut lines = vec![String::new(), format!("the given rules of {}: {}", texts.iter().map(|(concept, _)| concept.as_str()).collect::<Vec<&str>>().join(KINDS_GAP), given.len())];
    let mut summary: Vec<String> = Vec::new();
    let mut renamings: Vec<(String, String)> = Vec::new();
    let mut through_readings = usize::default();
    let found_rules: Vec<(usize, ToldRule)> = found.iter().enumerate().filter_map(|(at, known)| Some((at, read_rule(name, &known.line)?))).collect();
    let rules_found: Vec<ToldRule> = found_rules.iter().map(|(_, rule)| rule.clone()).collect();
    for (line, rule, concept) in &given {
        let (conditions, outcome) = rule_sentences(rule);
        let mut whole = conditions.clone();
        whole.push(outcome.clone());
        let whole = place_free(&whole);
        let alone = place_free(&conditions);
        let shown_kinds = |places: &[usize]| places.iter().map(|at| names[*at].clone()).collect::<Vec<String>>().join(KINDS_GAP);
        lines.push(format!("{INDENT}{line} ({concept})"));
        if let Some(known) = found.iter().find(|known| known.whole == whole) {
            match unused(known) {
                None => {
                    lines.push(format!("{INDENT}{INDENT}discovered as {} on {}", known.line, names[known.kind]));
                    summary.push([line.as_str(), &format!("discovered as {}", known.line), "the same rule", &names[known.kind]].join(SUMMARY_GAP));
                }
                Some(why) => {
                    lines.push(format!("{INDENT}{INDENT}not discovered: {why} (the same rule)"));
                    summary.push([line.as_str(), "not discovered", &why, &names[known.kind]].join(SUMMARY_GAP));
                }
            }
            continue;
        }
        if let Some(known) = found.iter().find(|known| known.conditions == alone && known.roles == outcome_roles(&whole)) {
            let named = format!("its fact named \"{}\" in place of \"{outcome}\"", pattern_said(&known.outcome));
            renamings.push((outcome.clone(), pattern_said(&known.outcome)));
            let why = if read_by_others(&known.outcome) { unused(known) } else { Some(format!("found only as an island, {} on {} ({named}): no given rule nor question form reads that fact, so it cannot stand for the given rule where other rules need its fact", known.line, names[known.kind])) };
            match why {
                None => {
                    lines.push(format!("{INDENT}{INDENT}discovered as {} on {} ({named})", known.line, names[known.kind]));
                    summary.push([line.as_str(), &format!("discovered as {}", known.line), &named, &names[known.kind]].join(SUMMARY_GAP));
                }
                Some(why) => {
                    lines.push(format!("{INDENT}{INDENT}not discovered: {why}"));
                    summary.push([line.as_str(), "not discovered", &why, &names[known.kind]].join(SUMMARY_GAP));
                }
            }
            continue;
        }
        let reordered = condition_orders(rule).into_iter().find_map(|form| {
            let (sentences, said) = rule_sentences(&form);
            let mut all = sentences.clone();
            all.push(said);
            let all = place_free(&all);
            let alike = place_free(&sentences);
            found.iter().find(|known| known.whole == all || (known.conditions == alike && known.roles == outcome_roles(&all))).map(|known| (known, known.whole != all))
        });
        if let Some((known, renamed)) = reordered {
            let named = if renamed { format!("its fact named \"{}\" in place of \"{outcome}\", {REORDERED}", pattern_said(&known.outcome)) } else { REORDERED.to_string() };
            let why = if !renamed || read_by_others(&known.outcome) { unused(known).map(|why| format!("{why} ({named})")) } else { Some(format!("found only as an island, {} on {} ({named}): no given rule nor question form reads that fact, so it cannot stand for the given rule where other rules need its fact", known.line, names[known.kind])) };
            match why {
                None => {
                    lines.push(format!("{INDENT}{INDENT}discovered as {} on {} ({named})", known.line, names[known.kind]));
                    summary.push([line.as_str(), &format!("discovered as {}", known.line), &named, &names[known.kind]].join(SUMMARY_GAP));
                }
                Some(why) => {
                    lines.push(format!("{INDENT}{INDENT}not discovered: {why}"));
                    summary.push([line.as_str(), "not discovered", &why, &names[known.kind]].join(SUMMARY_GAP));
                }
            }
            continue;
        }
        if let Some(known) = found.iter().filter(|known| !known.whole.is_empty()).find(|known| read_rule(name, &known.line).is_some_and(|wide| covers(&wide, rule))) {
            match unused(known) {
                None => {
                    lines.push(format!("{INDENT}{INDENT}discovered as a wider rule, {} on {} (the given rule is one case of it)", known.line, names[known.kind]));
                    summary.push([line.as_str(), &format!("discovered as {}", known.line), "a wider rule, of which the given rule is one case", &names[known.kind]].join(SUMMARY_GAP));
                }
                Some(why) => {
                    lines.push(format!("{INDENT}{INDENT}not discovered: {why} (a wider rule, of which the given rule is one case)"));
                    summary.push([line.as_str(), "not discovered", &why, &names[known.kind]].join(SUMMARY_GAP));
                }
            }
            continue;
        }
        let worded = worded_rules(rule, &reading_rules).into_iter().flat_map(|(worded, at)| condition_orders(&worded).into_iter().map(move |form| (form, at))).find_map(|(worded, at)| {
            let (sentences, said) = rule_sentences(&worded);
            let mut all = sentences.clone();
            all.push(said);
            let all = place_free(&all);
            let alike = place_free(&sentences);
            found.iter().find(|known| known.whole == all || (known.conditions == alike && known.roles == outcome_roles(&all))).map(|known| (known, at, known.whole != all))
        });
        if let Some((known, at, renamed)) = worded {
            let (reading, text) = &readings[at];
            let fact = if renamed { format!(", its fact named \"{}\" in place of \"{outcome}\"", pattern_said(&known.outcome)) } else { String::new() };
            let named = format!("{READING_MARK}\"{}\" of {text}{fact}", rule_text(reading));
            let why = if !renamed || read_by_others(&known.outcome) { unused(known).map(|why| format!("{why} ({named})")) } else { Some(format!("found only as an island, {} on {} ({named}): no given rule nor question form reads that fact, so it cannot stand for the given rule where other rules need its fact", known.line, names[known.kind])) };
            match why {
                None => {
                    lines.push(format!("{INDENT}{INDENT}discovered as {} on {} ({named})", known.line, names[known.kind]));
                    summary.push([line.as_str(), &format!("discovered as {}", known.line), &named, &names[known.kind]].join(SUMMARY_GAP));
                    through_readings += 1;
                }
                Some(why) => {
                    lines.push(format!("{INDENT}{INDENT}not discovered: {why}"));
                    summary.push([line.as_str(), "not discovered", &why, &names[known.kind]].join(SUMMARY_GAP));
                }
            }
            continue;
        }
        if let Some((one, other)) = state_covers(rule, &rules_found) {
            let (first, second) = (&found[found_rules[one].0], &found[found_rules[other].0]);
            let both = format!("{}{STATE_PAIR_GAP}{}", first.line, second.line);
            match unused(first).or_else(|| unused(second)) {
                None => {
                    lines.push(format!("{INDENT}{INDENT}discovered as {both} on {} ({STATE_COVERED})", names[first.kind]));
                    summary.push([line.as_str(), &format!("discovered as {both}"), STATE_COVERED, &names[first.kind]].join(SUMMARY_GAP));
                }
                Some(why) => {
                    lines.push(format!("{INDENT}{INDENT}not discovered: {why} ({STATE_COVERED})"));
                    summary.push([line.as_str(), "not discovered", &why, &names[first.kind]].join(SUMMARY_GAP));
                }
            }
            continue;
        }
        if let Some((at, rest)) = found_rules.iter().find_map(|(at, discovered)| renamed_with_rest(rule, discovered).map(|rest| (*at, rest))) {
            let known = &found[at];
            let named = format!("its fact named \"{}\" in place of \"{outcome}\", and its sentence holds the words \"{}\" after the last place", pattern_said(&known.outcome), pattern_said(&rest));
            let why = if read_by_others(&known.outcome) { unused(known) } else { Some(format!("found only as an island, {} on {} ({named}): no given rule nor question form reads that fact, so it cannot stand for the given rule where other rules need its fact", known.line, names[known.kind])) };
            match why {
                None => {
                    lines.push(format!("{INDENT}{INDENT}discovered as {} on {} ({named})", known.line, names[known.kind]));
                    summary.push([line.as_str(), &format!("discovered as {}", known.line), &named, &names[known.kind]].join(SUMMARY_GAP));
                }
                Some(why) => {
                    lines.push(format!("{INDENT}{INDENT}not discovered: {why}"));
                    summary.push([line.as_str(), "not discovered", &why, &names[known.kind]].join(SUMMARY_GAP));
                }
            }
            continue;
        }
        let shown: Vec<usize> = (0..kinds.len()).filter(|at| rule.conditions.iter().any(|condition| matches!(condition, Condition::Fact { pattern, .. } if fits_any(pattern, &facts[*at])))).collect();
        let missing = rule.conditions.iter().find_map(|condition| match condition {
            Condition::Fact { pattern, .. } if !facts.iter().any(|kind| fits_any(pattern, kind)) => {
                let said = condition_text(condition, &|piece: &LogicPiece| match piece {
                    LogicPiece::Word(word) => word_text(*word).to_string(),
                    LogicPiece::Open(place) => place_letter(*place),
                    LogicPiece::OpenWithEnd(place, ending) => format!("{}{ending}", place_letter(*place)),
                });
                let alike = place_free(&[pattern_said(pattern)]);
                let renamed = renamings.iter().find(|(given_fact, _)| place_free(&[given_fact.clone()]) == alike).map(|(_, found_fact)| format!(", which the rules discovered name \"{found_fact}\"")).unwrap_or_default();
                Some(format!("{said}\"{renamed}{}", written_first(pattern)))
            }
            _ => None,
        });
        let wanted: Vec<String> = conditions.iter().map(|sentence| place_free(&[sentence.clone()])).collect();
        let roles = outcome_roles(&whole);
        let formed = seen.iter().filter(|known| known.alike == alone && known.roles == roles).max_by(|one, other| one.right.cmp(&other.right));
        let asked = asked_facts.contains(&place_free(&[outcome.clone()])) || targets.iter().any(|(_, words)| {
            let mut fits = Vec::new();
            fit_into(&rule.outcome, words, &empty_filling(), &mut fits);
            !fits.is_empty()
        });
        let why = match (&missing, formed) {
            (Some(condition), _) => format!("it needs a fact that no rule makes on these lessons: \"{condition}"),
            (None, _) if !asked => format!("no question of the text or of the texts that build on it asks the fact that it makes{}: the rule learner forms only rules that make an asked fact, and a reading of told sentences into a fact of the text is the told learner's", targets.first().map(|(_, words)| format!(" (the kinds ask such facts as \"{}\")", word_texts(words).join(WORD_GAP))).unwrap_or_default()),
            (None, Some(known)) => format!("formed as {} on {}: {} (right {}, wrong {})", known.line, names[known.kind], known.reason, known.right, known.wrong),
            (None, None) => match unjudged.iter().filter(|(line, _, _)| read_rule(name, line).is_some_and(|read| {
                let (sentences, said) = rule_sentences(&read);
                let mut whole = sentences.clone();
                whole.push(said);
                place_free(&sentences) == alone && outcome_roles(&place_free(&whole)) == roles
            })).max_by_key(|(_, lessons, _)| *lessons) {
                Some((line, lessons, at)) if *lessons < LEAST_SUPPORT => format!("suggested as {line} on {} by {lessons} lessons, fewer than the {LEAST_SUPPORT} that a rule needs to be judged", names[*at]),
                Some((line, lessons, at)) => format!("suggested as {line} on {} by {lessons} lessons, below the bound of the most suggested rules that are judged", names[*at]),
                None => "not formed: no story of the lessons suggests it as one fact, or two facts that share a value, that lead to the asked fact".to_string(),
            },
        };
        let why = match nearest(&whole, found) {
            Some((known, missing, extra)) => format!("{why}; the nearest discovered rule is {} on {}, which lacks the words \"{missing}\" and holds the words \"{extra}\"", known.line, names[known.kind]),
            None => why,
        };
        lines.push(format!("{INDENT}{INDENT}not discovered: {why}"));
        let mut closest: Vec<&SeenRule> = seen.iter().filter(|known| wanted.iter().any(|sentence| known.conditions.contains(sentence))).collect();
        closest.sort_by(|one, other| wanted.iter().filter(|sentence| other.conditions.contains(sentence)).count().cmp(&wanted.iter().filter(|sentence| one.conditions.contains(sentence)).count()).then_with(|| other.right.cmp(&one.right)));
        if !closest.is_empty() {
            lines.push(format!("{INDENT}{INDENT}closest rules judged:"));
        }
        for known in closest.into_iter().take(CLOSEST_SHOWN) {
            lines.push(format!("{INDENT}{INDENT}{INDENT}{} on {}: right {}, wrong {}{}", known.line, names[known.kind], known.right, known.wrong, if known.allowed { "" } else { ", refused" }));
        }
        summary.push([line.as_str(), "not discovered", &why, &shown_kinds(&shown)].join(SUMMARY_GAP));
    }
    let plain = summary.iter().filter(|row| row.contains(DISCOVERED_ROW)).count() - through_readings;
    lines.push(String::new());
    let unheld = summary.iter().filter(|row| row.contains(NOT_HELD_MARK)).count();
    lines.push(format!("{COUNT_HEAD}{} of {} ({plain} plain, {through_readings} {COUNT_READINGS}); {COUNT_NOT_HELD}{unheld}", plain + through_readings, given.len()));
    lines.push("summary: rule | discovered as a line or not discovered | why | kinds".to_string());
    lines.extend(summary);
    lines
}

const REORDERED: &str = "its conditions in another order";
because!(REORDERED, DiscoveryReport, "why a given line counts as discovered when a discovered rule names the same facts in another order: every condition must hold, so the order says nothing");

const READING_MARK: &str = "renamed through the reading line ";
because!(READING_MARK, DiscoveryReport, "the mark of a given line found in the words of told sentences through a reading line of a text that builds on its concept, before the line and the text, so that such a match stands apart from a plain one");

const DISCOVERED_ROW: &str = " | discovered as ";
because!(DISCOVERED_ROW, DiscoveryReport, "the words of a summary row of a given line that was discovered, by which the rows are counted");

const COUNT_HEAD: &str = "given lines discovered: ";
because!(COUNT_HEAD, DiscoveryReport, "the words that open the count of the given lines discovered, before the summary");

const COUNT_READINGS: &str = "through a reading line";

const READ_HEAD: &str = "read ";
because!(READ_HEAD, DiscoveryReport, "the words before how a told sentence of the lesson of a proof is read: by which sentence of the text, with the words that fill each of its places, so that a reader sees a relation word and a name that became one value (mike / mother of ron)");

const VALUE_GAP: &str = " / ";
because!(VALUE_GAP, DiscoveryReport, "the mark between the values that a sentence of the text gives a told sentence");

const NOT_READ: &str = "by no sentence of the text";
because!(NOT_READ, DiscoveryReport, "what the report says of a told sentence that no sentence of the text reads");

const COSTS_HEAD: &str = "the ways and seconds of the board's own learning of this kind: ";
because!(COSTS_HEAD, DiscoveryReport, "the words before the line of costs of the board's own learning of a kind, as the jobs file of a board writes it (each way with its seconds, the chains, the replies, the searches, the notes), so that a long report shows which way of the board took the seconds");

const NOT_HELD_MARK: &str = "found by the curriculum as ";
because!(NOT_HELD_MARK, DiscoveryReport, "the words that open why a given line counts as not discovered when the curriculum of the report found it but no model that the board learns holds it, by which such lines are counted");

const COUNT_NOT_HELD: &str = "matched by the curriculum but not yet held by a model of the board: ";
because!(COUNT_NOT_HELD, DiscoveryReport, "the words of the count of the given lines that the curriculum found and the board's models do not hold yet, after the count of those discovered: progress on a concept that the board cannot use yet (family's lines matched while the board learns no model for clutrr)");
because!(COUNT_READINGS, DiscoveryReport, "the words of the count after the given lines found through a reading line of a text, apart from the plain matches");
because!(given_lines, DiscoveryReport, "each given rule of the concepts handed in (with its concept) and what the searches found for it: discovered as a rule that says the same (the same sentences whatever their letters), or as one with the same conditions whose fact has another name, or, for a line of one sentence, as two discovered rules that make its sentence and its outcome one fact of the same shape (two ways to start or end one invented state), or as a wider rule of which it is one case, or, written in the words of told sentences by a reading line of one sentence of a text that builds on it (If A is B of C then C has A as B, or If A R B and R means a D E then A has B as R, whose first sentence, the told one, is no sentence of the given rules), as a discovered rule that says the same or names its fact in another way, marked as renamed through that reading line and counted apart from the plain matches; a line counts as discovered only when a model that the board learns for the kinds holds it, and a line found with another name for its fact (also when its sentence holds a few words more after its last place, the B there for the B) only when a given rule or a question form reads that fact (else it is an island, which cannot stand for the given rule where other rules need its fact); else not discovered, with why and the nearest discovered rule with the words where they differ (a fact that it needs that no rule makes on the lessons, a fact that no question asks (the question forms of the given text and of the texts that build on it tell the asked facts), a rule like it that was refused or beaten, a rule like it suggested by too few lessons or below the bound of the rules judged, or no story that suggests it), the closest rules judged, and a summary line for each that a program reads");

fn renamed_with_rest(given: &ToldRule, discovered: &ToldRule) -> Option<Pattern> {
    let [Condition::Fact { pattern: told, when }] = given.conditions.as_slice() else { return None };
    let [Condition::Fact { pattern: found_told, when: found_when }] = discovered.conditions.as_slice() else { return None };
    if when != found_when || !given.absent.is_empty() || !discovered.absent.is_empty() {
        return None;
    }
    let (map, rest) = aligned(told, found_told)?;
    if rest.is_empty() {
        return None;
    }
    let given_roles: Vec<usize> = places_of(&given.outcome).into_iter().filter_map(|place| map.get(&place).copied()).collect();
    (given_roles.len() == places_of(&given.outcome).len() && given_roles == places_of(&discovered.outcome)).then_some(rest)
}
because!(renamed_with_rest, DiscoveryReport, "the words that a discovered rule of one sentence holds after the last place of the given rule's sentence when the two say the same with another name for the fact: the sentences align with those words after the last place (A got the B and A got the B there), and the outcome of the discovered rule holds the places of the given outcome in the same order (the holder of the B is A, the got of the B is A); nothing when they do not");

fn nearest<'a>(whole: &str, found: &'a [DiscoveredRule]) -> Option<(&'a DiscoveredRule, String, String)> {
    let mine: Vec<String> = whole.split_whitespace().map(str::to_string).collect();
    found
        .iter()
        .filter(|known| !known.whole.is_empty())
        .map(|known| {
            let theirs: Vec<String> = known.whole.split_whitespace().map(str::to_string).collect();
            let common = common_words(&mine, &theirs);
            (known, left_over(&mine, &common), left_over(&theirs, &common))
        })
        .min_by_key(|(_, missing, extra)| missing.len() + extra.len())
        .map(|(known, missing, extra)| (known, missing.join(WORD_GAP), extra.join(WORD_GAP)))
}
because!(nearest, DiscoveryReport, "the discovered rule nearest to a given rule that was not discovered, with the words of the given rule that it lacks and the words that it holds beyond them, both with their places named in order, the fewest words apart first, so that a reader sees what the search missed");

fn left_over(words: &[String], common: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut at = usize::default();
    for word in words {
        if common.get(at) == Some(word) {
            at += 1;
        } else {
            out.push(word.clone());
        }
    }
    out
}
because!(left_over, DiscoveryReport, "the words of a sentence that the run of words that it shares with another does not take, in their order");

fn stand_in(pattern: &Pattern) -> Vec<WordId> {
    pattern
        .iter()
        .map(|piece| match piece {
            LogicPiece::Word(word) => *word,
            LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => word_number(&format!("{STAND_IN}{}", place_letter(*place))),
        })
        .collect()
}
because!(stand_in, DiscoveryReport, "a sentence of a rule as words, each place standing for a word of its own, so that another sentence can be fitted to it as to a fact");

fn covers(wide: &ToldRule, given: &ToldRule) -> bool {
    let wide_facts: Vec<(&Pattern, &Moment)> = wide.conditions.iter().filter_map(|condition| match condition {
        Condition::Fact { pattern, when } => Some((pattern, when)),
        _ => None,
    }).collect();
    let given_facts: Vec<(&Pattern, &Moment)> = given.conditions.iter().filter_map(|condition| match condition {
        Condition::Fact { pattern, when } => Some((pattern, when)),
        _ => None,
    }).collect();
    if wide_facts.is_empty() || wide_facts.iter().any(|(pattern, _)| !pattern.iter().any(|piece| matches!(piece, LogicPiece::Word(_)))) || wide_facts.len() != given_facts.len() || wide.conditions.len() != given.conditions.len() || !wide.absent.is_empty() || !given.absent.is_empty() {
        return false;
    }
    let mut fillings = vec![empty_filling()];
    for ((wide_pattern, wide_when), (given_pattern, given_when)) in wide_facts.iter().zip(&given_facts) {
        if wide_when != given_when {
            return false;
        }
        let words = stand_in(given_pattern);
        let mut next = Vec::new();
        for filling in &fillings {
            fit_into(wide_pattern, &words, filling, &mut next);
        }
        if next.is_empty() {
            return false;
        }
        fillings = next;
    }
    let wanted = stand_in(&given.outcome);
    fillings.iter().any(|filling| fill(&wide.outcome, filling).is_some_and(|made| made == wanted))
}
because!(covers, DiscoveryReport, "whether a discovered rule holds a given rule as one case of it: the sentences of the given rule, each place standing for a word of its own, fit the conditions of the discovered rule in turn, at the same times, and the outcome of the discovered rule then is the outcome of the given rule (A moved to the B as a case of A, any word, to the B)");

const STAND_IN: &str = "stand-in-";
because!(STAND_IN, DiscoveryReport, "the start of the words that stand for the places of a given rule when a discovered rule is fitted to it, words that no story holds");

const RULE_STARTS: &[&str] = &["If ", "Write ", "Read "];
because!(RULE_STARTS, DiscoveryReport, "the openings of the rule lines of a text, as the library writes them: a rule, a line that writes a sentence again, and a line that reads told rules");

const KINDS_GAP: &str = ", ";
because!(KINDS_GAP, DiscoveryReport, "the mark between the names of the kinds in a summary line of the report");

fn pattern_said(pattern: &Pattern) -> String {
    pattern
        .iter()
        .map(|piece| match piece {
            LogicPiece::Word(word) => word_text(*word).to_string(),
            LogicPiece::Open(place) => place_letter(*place),
            LogicPiece::OpenWithEnd(place, ending) => format!("{}{ending}", place_letter(*place)),
        })
        .collect::<Vec<String>>()
        .join(WORD_GAP)
}
because!(pattern_said, DiscoveryReport, "a sentence of a rule as the library writes it, with the letters of its places");

fn outcome_roles(whole: &str) -> Vec<String> {
    let mut roles: Vec<String> = Vec::new();
    for token in whole.lines().last().unwrap_or_default().split_whitespace() {
        if !matches!(logic_piece(token), LogicPiece::Word(_)) && !roles.iter().any(|known| known == token) {
            roles.push(token.to_string());
        }
    }
    roles
}
because!(outcome_roles, DiscoveryReport, "the places of the outcome of a rule whose places were named again in order, in the order in which they first stand in the outcome: a fact named in another way says the same only when it holds the same places in the same order (the place of a person, where the person is), and not when it holds fewer (a place is a place) or the same in another order (the place has the person)");

const SUMMARY_GAP: &str = " | ";
because!(SUMMARY_GAP, DiscoveryReport, "the mark between the fields of a summary line of the report, as the board marks its fields");
