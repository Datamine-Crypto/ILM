use crate::cases::Case;
use crate::tools::explain::chain::{Facts, empty_filling, fit_into};
use crate::tools::explain::text::{Condition, LogicPiece, Moment, PLAIN_PLACES, Pattern, ToldRule, Topic, plain_piece};
use crate::tools::explain::wording::{CONDITIONS_JOINED, LEAST_SUPPORT, STATEMENT_END, TOLD_IF, TOLD_THEN, bounded_facts, fact_values, library_writing, places_of, renamed};
use patterns::{because, source};
use std::collections::HashMap;
use std::sync::Arc;

pub struct NamingBridge;
source!(NamingBridge, "a bridge between two names of one fact, as schema matching maps the terms of one vocabulary onto another (Rahm and Bernstein): when rules of a text read a sentence that no fact of the stories fits (the place of A is B), because the rule that made it was learned under another name (A is in the B) or is gone, a rule from a sentence that the stories hold with as many places to that sentence lets the rules that read it chain again; it is judged as any learned rule, never added unasked");

pub(super) const MOST_BRIDGE_PLACES: usize = 3;
because!(MOST_BRIDGE_PLACES, NamingBridge, "the most places of a sentence that a bridge reaches: each order of the places of the source is a candidate, so their number grows as the factorial of the places, and the facts that rules read name a few things");

const MOST_BRIDGES: usize = 32;
because!(MOST_BRIDGES, NamingBridge, "the most bridges that a search judges, those whose source the most lessons tell first: each is judged with the whole sample");

pub(super) fn orders(count: usize) -> Vec<Vec<usize>> {
    if count == usize::default() {
        return vec![Vec::new()];
    }
    let mut out: Vec<Vec<usize>> = Vec::new();
    for shorter in orders(count - 1) {
        for at in 0..=shorter.len() {
            let mut longer = shorter.clone();
            longer.insert(at, count - 1);
            out.push(longer);
        }
    }
    out
}
because!(orders, NamingBridge, "every order of the given number of places, each place once, as a bridge may give the places of its source to the places of the missing sentence in any order (who is where: A is in the B, the place of A is B)");

pub(super) fn missing_sentences(topic: &Topic, stories: &[Facts], base: &Facts) -> Vec<Pattern> {
    let first = base.added_to(&Facts::default());
    let holds = |pattern: &Pattern| {
        stories.iter().any(|facts| (usize::default()..first + facts.added_to(base)).any(|place| {
            let mut fits = Vec::new();
            fit_into(pattern, facts.fact(place), &empty_filling(), &mut fits);
            !fits.is_empty()
        }))
    };
    let mut out: Vec<Pattern> = Vec::new();
    for rule in &topic.rules {
        let sentences: Vec<&Pattern> = rule.conditions.iter().filter_map(|condition| match condition {
            Condition::Fact { pattern, .. } => Some(pattern),
            _ => None,
        }).collect();
        let missing: Vec<&Pattern> = sentences.iter().copied().filter(|pattern| !holds(pattern)).collect();
        if let [wanted] = missing.as_slice()
            && sentences.len() > missing.len()
            && !places_of(wanted).is_empty()
            && !out.contains(wanted)
        {
            out.push((*wanted).clone());
        }
    }
    out
}
because!(missing_sentences, NamingBridge, "the sentences that block a rule of a text: a rule whose other sentences all hold in some story and one sentence holds in none, nor in what the text knows, is kept from its outcome by that one fact alone, which nothing makes any more; a rule of one sentence that never holds is only an event that these stories do not tell, and a rule that misses two sentences needs more than a bridge");

pub fn bridge_candidates(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], patterns: &[Pattern]) -> Vec<(String, usize)> {
    let first = base.added_to(&Facts::default());
    let stories: Vec<Facts> = lessons.iter().filter_map(|case| bounded_facts(case, base, topic, false)).collect();
    let missing = missing_sentences(topic, &stories, base);
    if missing.is_empty() {
        return Vec::new();
    }
    let mut told: HashMap<usize, (usize, usize)> = HashMap::new();
    for (lesson, facts) in stories.iter().enumerate() {
        for place in first..first + facts.added_to(base) {
            let Some((pattern, _)) = fact_values(facts.fact(place), patterns) else { continue };
            let Some(source) = patterns.iter().position(|known| *known == pattern) else { continue };
            let entry = told.entry(source).or_insert((usize::default(), usize::MAX));
            if entry.1 != lesson {
                entry.0 += 1;
                entry.1 = lesson;
            }
        }
    }
    let mut out: Vec<(String, usize)> = Vec::new();
    for wanted in &missing {
        let places = places_of(wanted);
        if places.len() > MOST_BRIDGE_PLACES {
            continue;
        }
        for (source, (lessons_told, _)) in &told {
            let given = places_of(&patterns[*source]);
            if given.len() != places.len() || patterns[*source] == *wanted || *lessons_told < LEAST_SUPPORT {
                continue;
            }
            for order in orders(places.len()) {
                let to: Vec<usize> = order.iter().map(|at| given[*at]).collect();
                let line = format!("{TOLD_IF}{}{TOLD_THEN}{}{STATEMENT_END}", library_writing(&patterns[*source]), library_writing(&renamed(wanted, &places, &to)));
                if !out.iter().any(|(known, _)| *known == line) {
                    out.push((line, *lessons_told));
                }
            }
        }
    }
    out.sort_by(|one, other| other.1.cmp(&one.1).then_with(|| one.0.cmp(&other.0)));
    out.truncate(MOST_BRIDGES);
    out
}
because!(bridge_candidates, NamingBridge, "the bridges that the stories suggest: for each sentence that rules of the text read and nothing makes, a rule to it from each sentence of the stories with as many places, the places given in every order, counted by the lessons whose stories tell that source sentence, the most told first; nothing when every sentence that a rule reads holds somewhere");

pub(super) fn place_map(reading: &Pattern, given: &Pattern) -> Option<Vec<(usize, usize)>> {
    if reading.len() != given.len() {
        return None;
    }
    let mut map: Vec<(usize, usize)> = Vec::new();
    for (one, other) in reading.iter().zip(given) {
        let (from, to) = match (plain_piece(one), plain_piece(other)) {
            (LogicPiece::Word(word), LogicPiece::Word(again)) if word == again => continue,
            (LogicPiece::Open(from), LogicPiece::Open(to)) => (from, to),
            (LogicPiece::OpenWithEnd(from, ending), LogicPiece::OpenWithEnd(to, again)) if ending == again => (from, to),
            _ => return None,
        };
        match map.iter().find(|(known, _)| *known == from) {
            Some((_, known)) if *known != to => return None,
            Some(_) => {}
            None if map.iter().any(|(_, known)| *known == to) => return None,
            None => map.push((from, to)),
        }
    }
    Some(map)
}
because!(place_map, NamingBridge, "how the places of the sentence that a reading line makes stand for the places of a sentence of a rule, when the two are one sentence whatever their letters (C has A as B and A has B as C): the same words at the same spots, each place of one to one place of the other, a place with an empty ending as a plain place; nothing when they differ");

pub fn worded_rules(given: &ToldRule, readings: &[ToldRule]) -> Vec<(ToldRule, usize)> {
    let mut out: Vec<(ToldRule, usize)> = Vec::new();
    let most = given.conditions.iter().chain(&given.absent).filter_map(|condition| if let Condition::Fact { pattern, .. } = condition { places_of(pattern).into_iter().max() } else { None }).chain(places_of(&given.outcome).into_iter().max()).max().unwrap_or_default();
    for (at, reading) in readings.iter().enumerate() {
        let told: Option<Vec<&Pattern>> = reading.conditions.iter().map(|condition| if let Condition::Fact { pattern, when: Moment::Any } = condition { Some(pattern) } else { None }).collect();
        let Some(told) = told.filter(|told| !told.is_empty() && told.len() <= MOST_READING_SENTENCES) else { continue };
        if !reading.absent.is_empty() {
            continue;
        }
        let made = places_of(&reading.outcome);
        if !places_of(told[0]).iter().all(|place| made.contains(place)) {
            continue;
        }
        for (spot, condition) in given.conditions.iter().enumerate() {
            let Condition::Fact { pattern, when } = condition else { continue };
            let Some(map) = place_map(&reading.outcome, pattern) else { continue };
            let (mut from, mut to): (Vec<usize>, Vec<usize>) = map.into_iter().unzip();
            let mut next = most + 1;
            for place in told.iter().flat_map(|sentence| places_of(sentence)) {
                if !from.contains(&place) {
                    from.push(place);
                    to.push(next);
                    next += 1;
                }
            }
            if next > PLAIN_PLACES {
                continue;
            }
            let mut worded = given.clone();
            let written: Vec<Condition> = told.iter().map(|sentence| Condition::Fact { pattern: renamed(sentence, &from, &to).iter().map(plain_piece).collect(), when: *when }).collect();
            worded.conditions.splice(spot..=spot, written);
            out.push((without_subsumed(worded), at));
        }
    }
    out
}
because!(worded_rules, NamingBridge, "a given rule written in the words of the told sentences: for each reading line of a few sentences that a text of the library writes (If A is B of C then C has A as B; If A R B and R means a D E then A has B as R), each condition of the rule that is the sentence that the line makes is written as the sentences that it reads, with its places and new places for the line's own, one condition at a time, with the line that wrote it, and a condition that another says more of is left out (without_subsumed); a rule learned from the told sentences of a kind whose text reads them so says the same as the given rule, in the words of that kind");

const MOST_READING_SENTENCES: usize = 2;
because!(MOST_READING_SENTENCES, NamingBridge, "the most sentences of a reading line that a given rule is written in: a told sentence and a known fact that names its word (A R B and R means a D E), as a line that reads more is a rule of its own, not a reading");

fn without_subsumed(rule: ToldRule) -> ToldRule {
    let mut rule = rule;
    loop {
        let facts: Vec<(usize, Pattern)> = rule.conditions.iter().enumerate().filter_map(|(at, condition)| if let Condition::Fact { pattern, .. } = condition { Some((at, pattern.clone())) } else { None }).collect();
        let found = facts.iter().find_map(|(at, general)| {
            let elsewhere: Vec<usize> = rule.conditions.iter().enumerate().filter(|(other, _)| other != at).filter_map(|(_, condition)| if let Condition::Fact { pattern, .. } = condition { Some(places_of(pattern)) } else { None }).flatten().chain(places_of(&rule.outcome)).collect();
            facts.iter().filter(|(other, _)| other != at).any(|(_, special)| subsumes(general, special, &elsewhere)).then_some(*at)
        });
        let Some(at) = found else { return rule };
        rule.conditions.remove(at);
    }
}
because!(without_subsumed, NamingBridge, "a rule with each condition left out that another condition says more of: a condition whose places that no other sentence reads may take any words, and whose other words stand as in the other condition (R means a D E beside R means a D child), holds wherever the other holds");

fn subsumes(general: &Pattern, special: &Pattern, elsewhere: &[usize]) -> bool {
    if general.len() != special.len() {
        return false;
    }
    let mut map: Vec<(usize, LogicPiece)> = Vec::new();
    for (one, other) in general.iter().zip(special) {
        match one {
            LogicPiece::Word(_) if one == other => {}
            LogicPiece::Open(place) if elsewhere.contains(place) => {
                if one != other {
                    return false;
                }
            }
            LogicPiece::Open(place) => match map.iter().find(|(known, _)| known == place) {
                Some((_, known)) if known != other => return false,
                Some(_) => {}
                None => map.push((*place, other.clone())),
            },
            _ => return false,
        }
    }
    true
}
because!(subsumes, NamingBridge, "whether a sentence says no more than another: the same length, the same words at the same spots, each place that other sentences of the rule read the same place, and each place that only it reads standing for one piece of the other throughout");

pub fn condition_orders(rule: &ToldRule) -> Vec<ToldRule> {
    if rule.conditions.len() > MOST_ORDERED_CONDITIONS || !rule.absent.is_empty() {
        return vec![rule.clone()];
    }
    orders(rule.conditions.len()).into_iter().map(|order| ToldRule { conditions: order.iter().map(|at| rule.conditions[*at].clone()).collect(), ..rule.clone() }).collect()
}
because!(condition_orders, NamingBridge, "a rule with its conditions in every order: a learned rule may name the same facts in another order (B is a parent of C and A is a parent of B), which says the same, as every condition must hold");

const MOST_ORDERED_CONDITIONS: usize = 3;
because!(MOST_ORDERED_CONDITIONS, NamingBridge, "the most conditions of a given rule whose orders are tried: the orders grow as the factorial of the conditions, and the rules of a concept join a few facts each");

fn kept_sentences(topic: &Topic) -> Vec<Pattern> {
    let mut out: Vec<Pattern> = Vec::new();
    for condition in topic.questions.iter().flat_map(|form| &form.replies).flat_map(|reply| &reply.conditions) {
        if let Condition::Fact { pattern, .. } = condition
            && places_of(pattern).len() == KEPT_PLACES
            && pattern.iter().any(|piece| matches!(piece, LogicPiece::Word(_)))
            && !out.iter().any(|known| place_map(known, pattern).is_some())
        {
            out.push(pattern.clone());
        }
    }
    out
}
because!(kept_sentences, NamingBridge, "the sentences of two places that the question forms of a text read (A is a parent of B, A is a spouse of B), each once whatever its letters: the facts in the text's own words that its answers rest on, which a learned rule may name in place of the words of a kind");

const KEPT_PLACES: usize = 2;
because!(KEPT_PLACES, NamingBridge, "the places of a sentence of the text that a learned rule may be split through: a relation of one person to another, as the outcome of a rule learned from a statement about people names each of them, and the sentence in between must name the same people");

fn defined_in(kept: &Pattern, told: &Pattern, (place, words): (usize, &[LogicPiece])) -> Option<Pattern> {
    let at = told.iter().position(|piece| *piece == LogicPiece::Open(place))?;
    let run = (kept.len() + 1).checked_sub(told.len()).filter(|run| *run > usize::default())?;
    let middle = &kept[at..at + run];
    if !middle.iter().all(|piece| matches!(piece, LogicPiece::Word(_)) && words.contains(piece)) {
        return None;
    }
    let rest: Pattern = kept[..at].iter().chain(&kept[at + run..]).cloned().collect();
    let around: Pattern = told[..at].iter().chain(&told[at + 1..]).cloned().collect();
    let (from, to): (Vec<usize>, Vec<usize>) = place_map(&rest, &around)?.into_iter().unzip();
    Some(renamed(kept, &from, &to))
}
because!(defined_in, NamingBridge, "a sentence of the text as a told sentence with one of its places written as the words that a known fact gives that place (A is B of C, with B means a D parent: A is a parent of C), with the places of the told sentence; nothing when the sentence differs from the told sentence elsewhere, or takes a word that the known fact does not hold");

pub(super) fn kept_splits(topic: &Topic, rule: &ToldRule) -> Vec<(String, String)> {
    let said = places_of(&rule.outcome);
    let facts: Vec<&Pattern> = rule.conditions.iter().filter_map(|condition| if let Condition::Fact { pattern, when: Moment::Any } = condition { Some(pattern) } else { None }).collect();
    let kept = kept_sentences(topic);
    if said.len() != KEPT_PLACES || facts.len() != rule.conditions.len() || !rule.absent.is_empty() || kept.iter().any(|known| place_map(known, &rule.outcome).is_some()) || facts.iter().all(|pattern| kept.iter().any(|known| place_map(known, pattern).is_some())) {
        return Vec::new();
    }
    let joined: String = facts.iter().enumerate().map(|(at, pattern)| format!("{}{}", if at == usize::default() { "" } else { CONDITIONS_JOINED }, library_writing(pattern))).collect();
    let mut out: Vec<(String, String)> = Vec::new();
    for (one, told) in facts.iter().enumerate() {
        for place in places_of(told).into_iter().filter(|place| !said.contains(place)) {
            let words: Vec<LogicPiece> = facts.iter().enumerate().filter(|(other, pattern)| *other != one && pattern.contains(&LogicPiece::Open(place))).flat_map(|(_, pattern)| pattern.iter().filter(|piece| matches!(piece, LogicPiece::Word(_))).cloned()).collect();
            for known in &kept {
                let Some(middle) = defined_in(known, told, (place, &words)) else { continue };
                if places_of(&middle).iter().any(|stands| !said.contains(stands)) {
                    continue;
                }
                out.push((format!("{TOLD_IF}{joined}{TOLD_THEN}{}{STATEMENT_END}", library_writing(&middle)), format!("{TOLD_IF}{}{TOLD_THEN}{}{STATEMENT_END}", library_writing(&middle), library_writing(&rule.outcome))));
            }
        }
    }
    out
}
because!(kept_splits, NamingBridge, "the ways to split a learned rule through a sentence that the question forms of its text read: when that sentence is a told sentence of the rule with one place written as the words of a known fact of that place (A is B of C, B means a D parent: A is a parent of C), a bridge from the rule's conditions to that sentence and a rule from that sentence to the outcome (If A is a parent of C then C is child of A), as the known fact defines the word of the told sentence in the words of the text; nothing for a rule whose outcome or every one of whose conditions is already such a sentence, or whose outcome names other than two things");
