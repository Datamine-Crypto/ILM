use crate::base::lanes::{Worked, learned_within};
use crate::cases::Case;
use crate::choice::inner_parts;
use crate::tools::explain::chain::Facts;
use crate::tools::explain::learn::Library;
use crate::tools::explain::text::Topic;
use crate::tools::explain::wording::{RELEVANCE_SAMPLE, RULE_GROUND_PERCENT, WHOLE_SHARE, learn_rules, lessons_key, reply_target, rules_with_store, searched_nothing, with_learned};
use patterns::{because, source};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

pub struct SetCurriculum;
source!(SetCurriculum, "curriculum learning (Bengio, Louradour, Collobert and Weston): the kinds of a set are learned in the order of the set, and the rules that one kind discovers are kept for the kinds after it, so that a kind whose asked fact rests on facts that an earlier kind's rules make finds them made, as a lesson book teaches a step after the steps that it needs");

const MOST_TAUGHT_KINDS: usize = 2;
because!(MOST_TAUGHT_KINDS, SetCurriculum, "the most kinds before a kind, the nearest in the order of the set that the text grounds, whose rules the kind learns over: the step that a kind needs is most often the one just before it (where a thing is before where it was), and every kind taught costs a search for rules");

type TaughtKey = (usize, u64, u64);
because!(TaughtKey, SetCurriculum, "what the rules that a kind taught a text depend on: the text, the rules that it held before them, and the lessons of the kind");

type TaughtCell = Arc<Mutex<Option<Worked<Vec<String>>>>>;
because!(TaughtCell, SetCurriculum, "the rules that one kind taught a text, made by the first kind that needs them while the others wait, so that two kinds of a set that run side by side never learn them twice, with the work that they cost, which a kind that waited or came later adds as its own; kept under the bound of work of a kept learning (learned_within)");

static TAUGHT: OnceLock<Mutex<HashMap<TaughtKey, TaughtCell>>> = OnceLock::new();
because!(TAUGHT, SetCurriculum, "the rules that each kind before a kind taught a text, learned once in a run for each text, rules before them and lessons, as every later kind of a set learns over the same steps; the rules depend on the key and the work left of the learning alone, so a kind's rules never depend on which kind of its set ran first");

pub(super) fn asked_line(topic: &Topic, cases: &[&Case]) -> Option<usize> {
    let lines: Vec<usize> = cases.iter().filter_map(|case| reply_target(topic, case).map(|(_, _, line, _)| line)).collect();
    if lines.len() * WHOLE_SHARE < cases.len() * RULE_GROUND_PERCENT {
        return None;
    }
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for line in &lines {
        *counts.entry(*line).or_default() += 1;
    }
    counts.into_iter().max_by(|one, other| one.1.cmp(&other.1).then_with(|| other.0.cmp(&one.0))).map(|(line, _)| line)
}
because!(asked_line, SetCurriculum, "the reply line of a text that answers most questions of some lessons, when its reply lines answer most of them: the fact that those lessons ask, as a concept answers the kinds that it teaches step by step (where a thing is, then where it was before a place); nothing for lessons that the text grounds only by its known facts, which share no asked fact with other kinds");

pub fn taught_before(library: &Library, place: usize, (before, own): (&[Vec<Case>], usize)) -> (Topic, Arc<Facts>, Vec<String>, Vec<usize>) {
    let mut topic = library.topics[place].clone();
    let mut base = Arc::clone(library.base(place));
    let mut lines: Vec<String> = Vec::new();
    let mut kinds: Vec<usize> = Vec::new();
    let mut grounded: Vec<(usize, Vec<&Case>)> = Vec::new();
    for (from, kind) in before.iter().enumerate().rev() {
        let (first, _) = inner_parts(kind);
        let fit: Vec<&Case> = first.iter().map(|at| &kind[*at]).collect();
        let sampled: Vec<&Case> = fit.iter().take(RELEVANCE_SAMPLE).copied().collect();
        if asked_line(&topic, &sampled).is_some_and(|line| line != own) {
            grounded.push((from, fit));
        }
        if grounded.len() == MOST_TAUGHT_KINDS {
            break;
        }
    }
    for (from, fit) in grounded.into_iter().rev() {
        if let Some(found) = searched_nothing(library, place, (&topic, &fit)) {
            found.again();
            continue;
        }
        let key = (place, rules_with_store(&topic), lessons_key(&fit));
        let cell: TaughtCell = TAUGHT.get_or_init(Mutex::default).lock().map(|mut taught| Arc::clone(taught.entry(key).or_default())).unwrap_or_default();
        let mut kept = cell.lock().unwrap_or_else(PoisonError::into_inner);
        let (rules, fresh) = learned_within(kept.as_ref(), || learn_rules(&topic, &base, &fit));
        if fresh.is_some() {
            *kept = fresh;
        }
        drop(kept);
        if rules.is_empty() {
            continue;
        }
        let (learned, learned_base) = with_learned(&topic, &base, &rules);
        topic = learned;
        base = learned_base;
        lines.extend(rules);
        kinds.push(from);
    }
    (topic, base, lines, kinds)
}
because!(taught_before, SetCurriculum, "a text with the rules that the nearest kinds before a kind in its set taught it (a kind whose own search for these rules found nothing in the run is skipped), in the order of the set, each kind's rules learned on top of those before them from the first lessons of that kind, as the board learns a kind from its first lessons; a kind whose questions no reply line of the text answers, or the same reply line as the kind's own (the same fact asked of other stories), teaches it nothing and is passed over; with the lines of all the rules taught, which the kind's own rules then build on, and the places in the order of the set of the kinds that taught some rule");
