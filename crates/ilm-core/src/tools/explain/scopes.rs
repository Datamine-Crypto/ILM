use crate::tools::explain::chain::{Facts, empty_filling, fit_into};
use crate::tools::explain::text::{LogicPiece, PLAIN_PLACES, Pattern, word_not_mark};
use crate::tools::explain::wording::fixed_count;
use crate::tools::explain::words::{WordId, word_text};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct KnownScope;
source!(KnownScope, "the least general generalization relative to background knowledge (Plotkin; Muggleton and Feng's Golem): a place of a learned rule that its outcome does not name and no other sentence of the rule reads takes any word, while the lessons show the rule only for the words that they gave it; when every one of those words is known in one shape (father means a male parent, mother means a female parent), the rule reads that shape too, with a place where the known facts differ, so that it holds for the words of that kind and not for every word");

const SCOPED: usize = 0;
because!(SCOPED, KnownScope, "the place of a shape that the words of the rule's place fill; the places where the known facts differ come after it");

const MOST_SCOPE_PLACES: usize = 2;
because!(MOST_SCOPE_PLACES, KnownScope, "the most places where the known facts of the words of a place may differ (male and female; a sex and the general word): with more, the shape fits any known fact of its length and says nothing of the words");

const MOST_SCOPE_FACTS: usize = 32;
because!(MOST_SCOPE_FACTS, KnownScope, "the most known facts of one word that are read as its shapes: a word of a kind (a word of a relation, a city) holds in a few known facts, while a common word holds in many");

const LEAST_SCOPE_WORDS: usize = 2;
because!(LEAST_SCOPE_WORDS, KnownScope, "the fewest different words of a place that a shape is sought for: the shape of one word is that word's own fact, and the place may as well take the word itself");

pub struct MergedRule {
    pub conditions: Vec<(String, Pattern)>,
    pub place: usize,
    pub members: Vec<usize>,
    pub words: Vec<WordId>,
}
because!(MergedRule, KnownScope, "rules that differ only in one word of a condition, made one: the conditions with a new place where the word stood, the new place, the rules made one, and the word that each of them had there");

const BLANK: usize = usize::MAX;
because!(BLANK, KnownScope, "the mark that stands for the word that rules of one shape may differ in, when they are grouped");

pub fn merged_rules(rules: &[(&[(String, Pattern)], &Pattern)]) -> Vec<MergedRule> {
    let mut groups: HashMap<(Vec<(String, Pattern)>, Pattern), Vec<(usize, WordId)>> = HashMap::new();
    let mut order: Vec<(Vec<(String, Pattern)>, Pattern)> = Vec::new();
    for (at, (conditions, outcome)) in rules.iter().enumerate() {
        for (one, (_, condition)) in conditions.iter().enumerate() {
            for (spot, piece) in condition.iter().enumerate() {
                let LogicPiece::Word(word) = piece else { continue };
                if !word_not_mark(word_text(*word)) {
                    continue;
                }
                let mut blanked = conditions.to_vec();
                blanked[one].1[spot] = LogicPiece::Open(BLANK);
                let key = (blanked, (*outcome).clone());
                let members = groups.entry(key.clone()).or_insert_with(|| {
                    order.push(key);
                    Vec::new()
                });
                members.push((at, *word));
            }
        }
    }
    let mut out: Vec<MergedRule> = Vec::new();
    for key in order {
        let members = &groups[&key];
        let mut words: Vec<WordId> = members.iter().map(|(_, word)| *word).collect();
        words.sort_unstable();
        words.dedup();
        if words.len() < LEAST_SCOPE_WORDS {
            continue;
        }
        let (blanked, outcome) = key;
        let used = blanked.iter().flat_map(|(_, condition)| condition.iter()).chain(outcome.iter()).filter_map(|piece| match piece {
            LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) if *place != BLANK => Some(*place),
            _ => None,
        });
        let place = used.max().map_or(usize::default(), |most| most + 1);
        if place >= PLAIN_PLACES {
            continue;
        }
        let conditions = blanked.into_iter().map(|(joined, condition)| (joined, condition.into_iter().map(|piece| if piece == LogicPiece::Open(BLANK) { LogicPiece::Open(place) } else { piece }).collect())).collect();
        out.push(MergedRule { conditions, place, members: members.iter().map(|(at, _)| *at).collect(), words: members.iter().map(|(_, word)| *word).collect() });
    }
    out
}
because!(merged_rules, KnownScope, "the least general generalization of rules that differ in one word (Plotkin): rules suggested with the same outcome and the same conditions but for one word of a condition (A And his B went to C, A And her B went to C) are one rule with a place there, which the lessons of all of them suggest; for each group of two or more different words, in the order in which the rules come, with the rules that it makes one and the word of each");

fn word_shapes(base: &Facts, known: &HashMap<WordId, Vec<usize>>, value: &[WordId]) -> Vec<Pattern> {
    let Some(first) = value.first() else { return Vec::new() };
    let mut out: Vec<Pattern> = Vec::new();
    for place in known.get(first).into_iter().flatten().take(MOST_SCOPE_FACTS) {
        let fact = base.fact(*place);
        let starts: Vec<usize> = (0..fact.len()).filter(|at| fact[*at..].starts_with(value)).collect();
        if starts.len() != 1 || fact.len() == value.len() {
            continue;
        }
        let mut shape: Pattern = fact[..starts[0]].iter().map(|word| LogicPiece::Word(*word)).collect();
        shape.push(LogicPiece::Open(SCOPED));
        shape.extend(fact[starts[0] + value.len()..].iter().map(|word| LogicPiece::Word(*word)));
        if !out.contains(&shape) {
            out.push(shape);
        }
    }
    out
}
because!(word_shapes, KnownScope, "the known facts that hold a word of a place once, each as a shape with the word as the place and the other words as they are (father means a male parent as B means a male parent)");

fn shared_shape(general: &Pattern, shape: &Pattern) -> Option<Pattern> {
    if general.len() != shape.len() {
        return None;
    }
    let mut next = general.iter().filter_map(|piece| if let LogicPiece::Open(place) = piece { Some(*place + 1) } else { None }).max().unwrap_or(SCOPED + 1);
    let mut out: Pattern = Vec::new();
    for (one, other) in general.iter().zip(shape) {
        out.push(match (one, other) {
            (LogicPiece::Open(SCOPED), LogicPiece::Open(SCOPED)) => LogicPiece::Open(SCOPED),
            (LogicPiece::Open(SCOPED), _) | (_, LogicPiece::Open(SCOPED)) => return None,
            (LogicPiece::Word(word), LogicPiece::Word(again)) if word == again => LogicPiece::Word(*word),
            (LogicPiece::Open(place), _) => LogicPiece::Open(*place),
            _ => {
                next += 1;
                LogicPiece::Open(next - 1)
            }
        });
    }
    Some(out)
}
because!(shared_shape, KnownScope, "the shape that two shapes of the same length share: the place of the words where both have it, a word where both have the same word, and a new place where they differ (male and female as D); nothing when the words of the place stand at different spots");

fn in_order(shape: &Pattern) -> Pattern {
    let mut seen: Vec<usize> = vec![SCOPED];
    shape
        .iter()
        .map(|piece| match piece {
            LogicPiece::Open(place) => {
                if !seen.contains(place) {
                    seen.push(*place);
                }
                LogicPiece::Open(seen.iter().position(|known| known == place).unwrap_or_default())
            }
            other => other.clone(),
        })
        .collect()
}
because!(in_order, KnownScope, "a shape with its new places numbered in the order in which they stand, so that the same shape found from the words in another order is written with the same letters");


fn reach(base: &Facts, known: &HashMap<WordId, Vec<usize>>, shape: &Pattern) -> usize {
    let Some(LogicPiece::Word(word)) = shape.iter().find(|piece| matches!(piece, LogicPiece::Word(_))) else { return usize::MAX };
    let mut filled: HashSet<Vec<WordId>> = HashSet::new();
    for place in known.get(word).into_iter().flatten() {
        let mut fits = Vec::new();
        fit_into(shape, base.fact(*place), &empty_filling(), &mut fits);
        filled.extend(fits.iter().filter_map(|filling| filling.get(SCOPED).map(<[WordId]>::to_vec)));
    }
    filled.len()
}
because!(reach, KnownScope, "how many words the known facts give the place of a shape: the fewer, the less the shape adds to what the lessons show (B means a D parent reaches father and mother, B means a D E every word of a relation)");

pub fn shape_values(base: &Facts, known: &HashMap<WordId, Vec<usize>>, shape: &Pattern) -> HashSet<WordId> {
    let Some(LogicPiece::Word(word)) = shape.iter().find(|piece| matches!(piece, LogicPiece::Word(_))) else { return HashSet::new() };
    let places: Vec<usize> = shape.iter().filter_map(|piece| if let LogicPiece::Open(place) = piece { Some(*place) } else { None }).filter(|place| *place != SCOPED).collect();
    let mut out: HashSet<WordId> = HashSet::new();
    for at in known.get(word).into_iter().flatten() {
        let mut fits = Vec::new();
        fit_into(shape, base.fact(*at), &empty_filling(), &mut fits);
        for filling in &fits {
            out.extend(places.iter().filter_map(|place| filling.get(*place)).flatten().copied());
        }
    }
    out
}
because!(shape_values, KnownScope, "every word that the known facts of a shape give the places where those facts differ (male and female in B means a D parent; the countries in S is the capital of C), not the place of the rule's own words: a rule scoped by the shape that keeps one of them in its outcome names that one thing (R lives in Argentina), as its outcome should take the place that stands for it");

pub fn known_scope(base: &Facts, known: &HashMap<WordId, Vec<usize>>, values: &[Vec<WordId>]) -> Option<Pattern> {
    if values.len() < LEAST_SCOPE_WORDS {
        return None;
    }
    let shapes: Vec<Vec<Pattern>> = values.iter().map(|value| word_shapes(base, known, value)).collect();
    if shapes.iter().any(Vec::is_empty) {
        return None;
    }
    let mut best: Option<(usize, usize, Pattern)> = None;
    for seed in &shapes[0] {
        let mut general = Some(seed.clone());
        for others in &shapes[1..] {
            general = general.and_then(|known_shape| others.iter().filter_map(|shape| shared_shape(&known_shape, shape)).max_by_key(|shape| fixed_count(shape)));
        }
        let Some(general) = general.map(|shape| in_order(&shape)) else { continue };
        let places = general.iter().filter(|piece| matches!(piece, LogicPiece::Open(place) if *place != SCOPED)).count();
        if places > MOST_SCOPE_PLACES || fixed_count(&general) == usize::default() {
            continue;
        }
        let reached = reach(base, known, &general);
        if best.as_ref().is_none_or(|(least, most, _)| (reached, std::cmp::Reverse(fixed_count(&general))) < (*least, std::cmp::Reverse(*most))) {
            best = Some((reached, fixed_count(&general), general));
        }
    }
    best.map(|(_, _, general)| general)
}
because!(known_scope, KnownScope, "the shape of the known facts that every word given to a place of a rule holds (father means a male parent and mother means a female parent: B means a D parent), with the words of the place at its first place and a new place for each spot where the facts differ, of the shapes shared by all the words the one that reaches the fewest words, then the one with the most fixed words; nothing for a place of one word, a word with no known fact, or words whose facts share no shape");
