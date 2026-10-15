use crate::base::runs::RunNumbers;
use crate::tools::rule::forms::{RULE_ANY, RuleFact, RulePart, RuleRole, RuleUnit};
use crate::tools::rule::tokens::{RuleMarks, RuleSentence};
use crate::base::runs::run_place;
use patterns::{because, source};

pub struct LessonPieces;
source!(LessonPieces, "a lesson sentence beside its formal form: the things of the form are found in the sentence in their order, and the sentence is cut into one piece for each part of the form. What stands before a piece, between its things and after the last piece are the marks that say the relation, yes or no, and the role of the part; where a part names no subject, the lesson shows where its subject comes from");

pub(super) const RULE_SOURCES: usize = 4;
because!(RULE_SOURCES, LessonPieces, "the places that the subject of a piece can come from");

const RULE_UNIT_OF_TWO: usize = 2;
because!(RULE_UNIT_OF_TWO, LessonPieces, "the told sentences of a unit whose one rule lies over both of them, and the least things of its first sentence: a condition and the head that the second sentence turns into a condition");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuleSource {
    Own,
    Any,
    Same,
    Near,
}
because!(RuleSource, LessonPieces, "where the subject of a piece comes from: its own thing, any thing, the subject of the part before, or the subject that another piece of the sentence names. The order is fixed: with equal counts the earlier one is taken");

pub(super) const RULE_SOURCE_ORDER: &[RuleSource] = &[RuleSource::Own, RuleSource::Any, RuleSource::Same, RuleSource::Near];
because!(RULE_SOURCE_ORDER, LessonPieces, "the places that a subject can come from, in the order in which equal counts are decided");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuleKind {
    ARule,
    ItsOwn,
    TheSame,
    GoesOn,
}
because!(RuleKind, LessonPieces, "what a told sentence is for the reading by its marks: a rule about any thing, facts about a thing that the sentence names, facts about the subject before, or more of the rule that the sentence before began. The order is fixed: of kinds with equal scores the earlier one is taken");

pub(super) const RULE_KIND_ORDER: &[RuleKind] = &[RuleKind::ARule, RuleKind::ItsOwn, RuleKind::TheSame, RuleKind::GoesOn];
because!(RULE_KIND_ORDER, LessonPieces, "the kinds of a told sentence in the order in which equal scores are decided");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RuleRest {
    pub(super) relation: u32,
    pub(super) yes: bool,
    pub(super) role: RuleRole,
}
because!(RuleRest, LessonPieces, "what the marks of a piece say: the relation, whether it holds, and the role of the part");

#[derive(Clone, Debug)]
pub(super) struct RulePiece {
    pub(super) lead: Vec<u32>,
    pub(super) inner: Vec<u32>,
    pub(super) tail: Vec<u32>,
    pub(super) rest: RuleRest,
    pub(super) fits: [bool; RULE_SOURCES],
}
because!(RulePiece, LessonPieces, "one piece of a lesson sentence: the marks before it back to the piece before, its things as slots with the marks between them, the marks after it when it is the last piece, what these marks say, and every place that its subject can come from in this lesson");

#[derive(Clone, Debug, Default)]
pub(super) struct RuleCut {
    pub(super) pieces: Vec<RulePiece>,
    pub(super) spans: Vec<(usize, usize)>,
    pub(super) last: Option<u32>,
}
because!(RuleCut, LessonPieces, "a lesson sentence cut by its formal form: its pieces, the places of its things as runs of tokens, and the subject of its last part");

#[derive(Clone, Debug)]
pub(super) struct RuleLesson {
    pub(super) sent: RuleSentence,
    pub(super) parts: Vec<RulePart>,
    pub(super) before: Option<u32>,
    pub(super) told: bool,
}
because!(RuleLesson, LessonPieces, "one lesson sentence with the parts of its formal form, the subject of the part before it, and whether it is a told sentence or the statement of a question");


pub(super) fn rule_words(terms: &RunNumbers, thing: u32) -> &[u32] {
    thing.checked_sub(1).map_or(&[], |number| terms.run(number))
}
because!(rule_words, LessonPieces, "the tokens of a thing, and none for any thing");

fn rule_place(toks: &[u32], parts: &[RulePart], terms: &RunNumbers) -> Option<Vec<(Option<usize>, usize)>> {
    let mut cursor = 0;
    let mut out = Vec::with_capacity(parts.len());
    for (fact, _) in parts {
        if fact.thing == RULE_ANY {
            return None;
        }
        let thing = rule_words(terms, fact.thing);
        let subject = rule_words(terms, fact.subject);
        let mut subject_at = if fact.subject == RULE_ANY { None } else { run_place(toks, subject, cursor) };
        let mut thing_at = subject_at.and_then(|at| run_place(toks, thing, at + subject.len()));
        if thing_at.is_none() {
            subject_at = None;
            thing_at = run_place(toks, thing, cursor);
        }
        let thing_at = thing_at?;
        out.push((subject_at, thing_at));
        cursor = thing_at + thing.len();
    }
    Some(out)
}
because!(rule_place, LessonPieces, "where the subject and the object of each part stand in the sentence, the parts taken in the order given: the subject is looked for first and the object after it, and a part whose subject is not there has its object alone");

pub(super) fn rule_cut(toks: &[u32], parts: &[RulePart], before: Option<u32>, marks: &RuleMarks, terms: &RunNumbers) -> Option<RuleCut> {
    let moved: Vec<RulePart>;
    let mut parts = parts;
    let mut spots = rule_place(toks, parts, terms);
    if spots.is_none() {
        let first: Vec<usize> = parts.iter().map(|(fact, _)| run_place(toks, rule_words(terms, fact.thing), 0)).collect::<Option<Vec<usize>>>()?;
        let mut order: Vec<usize> = (0..parts.len()).collect();
        order.sort_by_key(|place| (first[*place], *place));
        moved = order.iter().map(|place| parts[*place]).collect();
        parts = &moved;
        spots = rule_place(toks, parts, terms);
    }
    let spots = spots?;
    let named: Vec<u32> = parts.iter().zip(&spots).filter(|(_, (at, _))| at.is_some()).map(|((fact, _), _)| fact.subject).collect();
    let mut cut = RuleCut::default();
    let mut end = 0usize;
    let mut before = before;
    for (number, ((fact, role), (subject_at, thing_at))) in parts.iter().zip(&spots).enumerate() {
        let stop = thing_at + rule_words(terms, fact.thing).len();
        let first = subject_at.unwrap_or(*thing_at);
        let mut lead: Vec<u32> = Vec::new();
        if number == 0 {
            lead.push(marks.start);
        }
        lead.extend_from_slice(toks.get(end..first).unwrap_or(&[]));
        let mut inner = vec![marks.slot];
        let mut fits = [false; RULE_SOURCES];
        match subject_at {
            Some(at) => {
                let from = at + rule_words(terms, fact.subject).len();
                inner.extend_from_slice(&toks[from..*thing_at]);
                inner.push(marks.slot);
                fits[RuleSource::Own as usize] = true;
                cut.spans.push((*at, from));
            }
            None => {
                fits[RuleSource::Any as usize] = fact.subject == RULE_ANY;
                fits[RuleSource::Same as usize] = before == Some(fact.subject);
                fits[RuleSource::Near as usize] = named.contains(&fact.subject);
                if !fits.contains(&true) {
                    return None;
                }
            }
        }
        let mut tail: Vec<u32> = Vec::new();
        if number + 1 == parts.len() {
            tail.extend_from_slice(&toks[stop..]);
            tail.push(marks.end);
        }
        cut.spans.push((*thing_at, stop));
        cut.pieces.push(RulePiece { lead, inner, tail, rest: RuleRest { relation: fact.relation, yes: fact.yes, role: *role }, fits });
        end = stop;
        before = Some(fact.subject);
    }
    cut.last = before;
    Some(cut)
}
because!(rule_cut, LessonPieces, "the pieces of a lesson sentence, or nothing when its parts cannot be found one after the other. A set may give the parts in another order than the sentence: they are then taken in the order of their objects in the sentence. A part with no subject of its own must be about any thing, about the subject before, or about a subject that another piece names, else the lesson does not show where its subject comes from");

fn rule_over_two(toks: &[&RuleSentence], items: &[Vec<RulePart>], terms: &RunNumbers) -> Option<Vec<Vec<RulePart>>> {
    let [parts] = items else { return None };
    if toks.len() != RULE_UNIT_OF_TWO || parts.last().is_none_or(|(_, role)| *role != RuleRole::Head) {
        return None;
    }
    if parts.iter().any(|(fact, _)| fact.subject != RULE_ANY || fact.thing == RULE_ANY) {
        return None;
    }
    let mut places: Vec<(usize, usize, RuleFact)> = Vec::new();
    for (fact, _) in parts {
        let (sentence, at) = toks.iter().enumerate().find_map(|(sentence, sent)| run_place(&sent.toks, rule_words(terms, fact.thing), 0).map(|at| (sentence, at)))?;
        places.push((sentence, at, *fact));
    }
    places.sort_by_key(|(sentence, at, _)| (*sentence, *at));
    let first: Vec<RuleFact> = places.iter().filter(|(sentence, _, _)| *sentence == 0).map(|(_, _, fact)| *fact).collect();
    let second: Vec<RuleFact> = places.iter().filter(|(sentence, _, _)| *sentence == 1).map(|(_, _, fact)| *fact).collect();
    let (head, more) = second.split_last()?;
    let (turned, conditions) = first.split_last()?;
    if first.len() < RULE_UNIT_OF_TWO {
        return None;
    }
    let made: Vec<&RuleFact> = first.iter().chain(more).collect();
    let given: Vec<&RuleFact> = parts.iter().filter(|(_, role)| *role == RuleRole::Condition).map(|(fact, _)| fact).collect();
    let true_head = parts.iter().find(|(_, role)| *role == RuleRole::Head).map(|(fact, _)| fact)?;
    if head != true_head || !made.iter().all(|fact| given.contains(fact)) || !given.iter().all(|fact| made.contains(fact)) {
        return None;
    }
    let mut one: Vec<RulePart> = conditions.iter().map(|fact| (*fact, RuleRole::Condition)).collect();
    one.push((*turned, RuleRole::Head));
    Some(vec![one, second.iter().map(|fact| (*fact, RuleRole::More)).collect()])
}
because!(rule_over_two, LessonPieces, "a rule about any thing that lies over the two told sentences of its unit, as the two readings that make it: the first sentence is a rule by itself, its last thing the head; the second sentence holds more of that rule, where the head of the first becomes a condition and the last thing of the second is the head. Nothing when the unit is not of this make or the two readings do not give the rule of the set");

pub(super) fn rule_kind_of(toks: &[u32], parts: &[RulePart], before: Option<u32>, terms: &RunNumbers) -> Option<RuleKind> {
    let (first, _) = parts.first()?;
    if parts.iter().all(|(_, role)| *role == RuleRole::More) {
        return Some(RuleKind::GoesOn);
    }
    if parts.iter().any(|(_, role)| *role != RuleRole::Fact) {
        return parts.iter().all(|(fact, _)| fact.subject == RULE_ANY).then_some(RuleKind::ARule);
    }
    if parts.iter().all(|(fact, _)| fact.subject != RULE_ANY && run_place(toks, rule_words(terms, fact.subject), 0).is_some()) {
        return Some(RuleKind::ItsOwn);
    }
    (first.subject != RULE_ANY && before == Some(first.subject) && parts.iter().all(|(fact, _)| fact.subject == first.subject)).then_some(RuleKind::TheSame)
}
because!(rule_kind_of, LessonPieces, "the kind of a lesson sentence for the reading by marks, from its formal form: more of the rule before; a rule, when every part is about any thing; facts about things that the sentence names; facts that are all about the subject before; and no kind for any other make");

pub(super) fn rule_lesson_sentences(sentences: &[RuleSentence], units: &[RuleUnit], marks: &RuleMarks, terms: &RunNumbers) -> Vec<RuleLesson> {
    let mut out: Vec<RuleLesson> = Vec::new();
    let mut before: Option<u32> = None;
    for unit in units {
        let toks: Vec<&RuleSentence> = (unit.first..unit.end).filter_map(|place| sentences.get(place)).collect();
        let mut per: Option<Vec<Vec<RulePart>>> = Some(vec![Vec::new(); toks.len()]);
        for item in &unit.items {
            let home = toks.iter().position(|sent| item.iter().all(|(fact, _)| run_place(&sent.toks, rule_words(terms, fact.thing), 0).is_some()));
            match (home, per.as_mut()) {
                (Some(home), Some(per)) => per[home].extend_from_slice(item),
                _ => {
                    per = None;
                    break;
                }
            }
        }
        let Some(per) = per.or_else(|| rule_over_two(&toks, &unit.items, terms)) else {
            before = None;
            continue;
        };
        for (number, (sent, parts)) in toks.iter().zip(per).enumerate() {
            let mut sent = (*sent).clone();
            sent.left = Some((sentences.len() - 1 - (unit.first + number)) as u32);
            let mut next = before;
            if let Some((last, _)) = parts.last() {
                next = rule_cut(&sent.toks, &parts, before, marks, terms).map_or(Some(last.subject), |cut| cut.last);
            }
            out.push(RuleLesson { sent, parts, before, told: true });
            before = next;
        }
    }
    let mut later = false;
    for lesson in out.iter_mut().rev() {
        lesson.sent.later = Some(later);
        later = later || rule_kind_of(&lesson.sent.toks, &lesson.parts, lesson.before, terms) == Some(RuleKind::ItsOwn);
    }
    out
}
because!(rule_lesson_sentences, LessonPieces, "the told sentences of a lesson story with their parts, in their order. A unit of more than one sentence gives each form to its first sentence that holds every object of the form; a rule that lies over the two sentences of its unit is given as the two readings that make it; a unit with a form that fits neither way is left out. Each sentence also gets how many told sentences stand after it, and whether a later sentence speaks about a thing that it names");
