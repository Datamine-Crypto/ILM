use crate::controls::file::ChangedCase;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::{BTreeSet, HashSet};

pub struct DeductionChanges;
source!(DeductionChanges, "changed theories of deduction with told rules: the sets give, for their lesson theories, every told fact and every told rule in a formal notation. An exact reading uses all rules on the facts and on their own results until nothing new comes. In the closed form a condition with not holds when its fact does not follow_rules, found by turns of what surely follows and what can follow_rules, so that no rule uses the absence of a fact that comes out later; in the open form a fact with not must be told or follow_rules, and a statement that does not follow_rules and whose opposite does not follow_rules is unknown. A told unit is needed when the reading gives another answer once the unit is taken out, so the true answer of the changed theory is known; the first unit that is not needed is taken out with the old answer");

const FORMAL_ANY: &str = "?";
because!(FORMAL_ANY, DeductionChanges, "how the notation writes any thing in the place of a subject or an object of a rule");

const FORMAL_FIELD: char = '\t';
because!(FORMAL_FIELD, DeductionChanges, "the mark between the fields of a label line");

const FORMAL_UNIT: &str = "U";
because!(FORMAL_UNIT, DeductionChanges, "the mark of a label line that opens a written unit with its first told sentence and the sentence after its last");

const FORMAL_FACT: &str = "F";
because!(FORMAL_FACT, DeductionChanges, "the mark of a label line with a told fact of its unit");

const FORMAL_CONDITION: &str = "C";
because!(FORMAL_CONDITION, DeductionChanges, "the mark of a label line with a condition of a told rule of its unit");

const FORMAL_HEAD: &str = "H";
because!(FORMAL_HEAD, DeductionChanges, "the mark of a label line with the head of a told rule, which closes the rule");

const FORMAL_STATEMENT: &str = "T";
because!(FORMAL_STATEMENT, DeductionChanges, "the mark of the label line with the fact that the statement says");

const FORMAL_HOLDS: &str = "yes";
because!(FORMAL_HOLDS, DeductionChanges, "how a label line writes that a fact holds");

const DEDUCTION_TRUE: &str = "True";
because!(DEDUCTION_TRUE, DeductionChanges, "the answer when the theory makes the statement hold");

const DEDUCTION_FALSE: &str = "False";
because!(DEDUCTION_FALSE, DeductionChanges, "the answer when the theory makes the statement not hold");

const DEDUCTION_UNKNOWN: &str = "Unknown";
because!(DEDUCTION_UNKNOWN, DeductionChanges, "the answer of the open form when neither the statement nor its opposite follows; a kind with it in its lessons is read in the open form");

const DEDUCTION_TURNS: usize = 50;
because!(DEDUCTION_TURNS, DeductionChanges, "the most turns of the estimates of the closed form; a theory of the sets settles in a few");

const SORT_NEEDED_FACT: &str = "a needed told fact taken out";
because!(SORT_NEEDED_FACT, DeductionChanges, "the sort of the change where a unit of facts that the answer needs is taken out");

const SORT_NEEDED_RULE: &str = "a needed told rule taken out";
because!(SORT_NEEDED_RULE, DeductionChanges, "the sort of the change where a unit with a rule that the answer needs is taken out");

const SORT_NOT_NEEDED: &str = "a sentence that is not needed taken out";
because!(SORT_NOT_NEEDED, DeductionChanges, "the sort of the change where the first unit that the answer does not need is taken out");

pub(crate) type FormalFact = (String, String, String, bool);
because!(FormalFact, DeductionChanges, "a fact of the notation: a subject, a relation, an object, and whether it holds");

#[derive(Clone, Debug)]
pub(crate) enum FormalItem {
    Told(FormalFact),
    Rule(Vec<FormalFact>, FormalFact),
}
because!(FormalItem, DeductionChanges, "a form of a unit: a told fact, or a rule with its conditions and its head");

#[derive(Clone, Debug)]
pub(crate) struct FormalUnit {
    pub(crate) first: usize,
    pub(crate) end: usize,
    pub(crate) items: Vec<FormalItem>,
}
because!(FormalUnit, DeductionChanges, "a written unit of a theory: its told sentences from the first to the one before the end, and its forms");

fn formal_line_fact(fields: &[&str]) -> Option<FormalFact> {
    let [subject, relation, thing, holds] = fields else { return None };
    Some((subject.to_string(), relation.to_string(), thing.to_string(), *holds == FORMAL_HOLDS))
}
because!(formal_line_fact, DeductionChanges, "the fact of the fields of a label line after its mark");

fn formal_forms(lines: &[String]) -> (Vec<FormalUnit>, Option<FormalFact>) {
    let mut units: Vec<FormalUnit> = Vec::new();
    let mut statement: Option<FormalFact> = None;
    let mut open: Vec<FormalFact> = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(FORMAL_FIELD).collect();
        let Some((mark, rest)) = fields.split_first() else { continue };
        if *mark == FORMAL_UNIT {
            if let [first, end] = rest
                && let (Ok(first), Ok(end)) = (first.parse(), end.parse())
            {
                units.push(FormalUnit { first, end, items: Vec::new() });
            }
            continue;
        }
        let Some(fact) = formal_line_fact(rest) else { continue };
        if *mark == FORMAL_STATEMENT {
            statement = Some(fact);
            continue;
        }
        let Some(unit) = units.last_mut() else { continue };
        match *mark {
            FORMAL_FACT => unit.items.push(FormalItem::Told(fact)),
            FORMAL_CONDITION => open.push(fact),
            FORMAL_HEAD => unit.items.push(FormalItem::Rule(std::mem::take(&mut open), fact)),
            _ => {}
        }
    }
    (units, statement)
}
because!(formal_forms, DeductionChanges, "the units of a lesson theory with their forms and the fact of its statement, from the label lines of the lesson");

fn filled_fact(fact: &FormalFact, thing: Option<&str>) -> FormalFact {
    let put = |held: &String| match thing {
        Some(thing) if held == FORMAL_ANY => thing.to_string(),
        _ => held.clone(),
    };
    (put(&fact.0), fact.1.clone(), put(&fact.2), fact.3)
}
because!(filled_fact, DeductionChanges, "a fact of a rule with a thing in the place of any thing");

fn follow_rules(facts: &[FormalFact], rules: &[(Vec<FormalFact>, FormalFact)], things: &[String], holds: &dyn Fn(&FormalFact, &HashSet<FormalFact>) -> bool) -> HashSet<FormalFact> {
    let mut known: HashSet<FormalFact> = facts.iter().cloned().collect();
    loop {
        let mut new: HashSet<FormalFact> = HashSet::new();
        for (conditions, head) in rules {
            let open = conditions.iter().chain([head]).any(|fact| fact.0 == FORMAL_ANY || fact.2 == FORMAL_ANY);
            let each: Vec<Option<&str>> = if open { things.iter().map(|thing| Some(thing.as_str())).collect() } else { vec![None] };
            for thing in each {
                if conditions.iter().all(|condition| holds(&filled_fact(condition, thing), &known)) {
                    new.insert(filled_fact(head, thing));
                }
            }
        }
        if new.is_subset(&known) {
            return known;
        }
        known.extend(new);
    }
}
because!(follow_rules, DeductionChanges, "the facts that follow_rules: in each round every rule is used for every thing on what was known at the start of the round, until a round brings nothing new");

fn holding(fact: &FormalFact) -> FormalFact {
    (fact.0.clone(), fact.1.clone(), fact.2.clone(), true)
}
because!(holding, DeductionChanges, "the same fact said to hold");

pub(crate) fn deduction_exact(units: &[&FormalUnit], statement: &FormalFact, closed: bool) -> String {
    let mut facts: Vec<FormalFact> = Vec::new();
    let mut rules: Vec<(Vec<FormalFact>, FormalFact)> = Vec::new();
    for item in units.iter().flat_map(|unit| &unit.items) {
        match item {
            FormalItem::Told(fact) => facts.push(fact.clone()),
            FormalItem::Rule(conditions, head) => rules.push((conditions.clone(), head.clone())),
        }
    }
    let mut named: BTreeSet<String> = facts.iter().flat_map(|fact| [fact.0.clone(), fact.2.clone()]).collect();
    named.extend(rules.iter().flat_map(|(conditions, head)| conditions.iter().chain([head]).flat_map(|fact| [fact.0.clone(), fact.2.clone()])).filter(|thing| thing != FORMAL_ANY));
    let things: Vec<String> = named.into_iter().collect();
    if !closed {
        let known = follow_rules(&facts, &rules, &things, &|condition, known| known.contains(condition));
        if known.contains(statement) {
            return DEDUCTION_TRUE.to_string();
        }
        let opposite = (statement.0.clone(), statement.1.clone(), statement.2.clone(), !statement.3);
        return if known.contains(&opposite) { DEDUCTION_FALSE } else { DEDUCTION_UNKNOWN }.to_string();
    }
    let mut low = follow_rules(&facts, &rules, &things, &|condition, known| condition.3 && known.contains(condition));
    for _ in 0..DEDUCTION_TURNS {
        let high = follow_rules(&facts, &rules, &things, &|condition, known| if condition.3 { known.contains(condition) } else { !low.contains(&holding(condition)) });
        let again = follow_rules(&facts, &rules, &things, &|condition, known| if condition.3 { known.contains(condition) } else { !high.contains(&holding(condition)) });
        if again == low {
            break;
        }
        low = again;
    }
    let holds = low.contains(&holding(statement)) == statement.3;
    if holds { DEDUCTION_TRUE } else { DEDUCTION_FALSE }.to_string()
}
because!(deduction_exact, DeductionChanges, "the answer of a theory for a statement by the exact reading of its form");

pub fn deduction_changes(lessons: &[Case], labels: &[Vec<String>], whole: bool) -> Vec<ChangedCase> {
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let closed = !lessons.iter().any(|case| case.answer == DEDUCTION_UNKNOWN);
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        let (units, statement) = formal_forms(labels.get(place).map_or(&[][..], Vec::as_slice));
        let Some(statement) = statement else { continue };
        let all: Vec<&FormalUnit> = units.iter().collect();
        if units.is_empty() || deduction_exact(&all, &statement, closed) != case.answer {
            continue;
        }
        let without = |unit: &FormalUnit| -> Vec<String> { case.sentences.iter().enumerate().filter(|(at, _)| !(unit.first..unit.end).contains(at)).map(|(_, sentence)| sentence.clone()).collect() };
        let mut free: Option<usize> = None;
        for (at, unit) in units.iter().enumerate() {
            let rest: Vec<&FormalUnit> = units.iter().enumerate().filter(|(other, _)| *other != at).map(|(_, other)| other).collect();
            let truth = deduction_exact(&rest, &statement, closed);
            if truth == case.answer {
                free = free.or(Some(at));
                continue;
            }
            let facts_only = unit.items.iter().all(|item| matches!(item, FormalItem::Told(_)));
            let sort = if facts_only { SORT_NEEDED_FACT } else { SORT_NEEDED_RULE };
            out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences: without(unit), question: case.question.clone(), truth });
        }
        if let Some(free) = free {
            out.push(ChangedCase { place: Some(place), sort: SORT_NOT_NEEDED.to_string(), sentences: without(&units[free]), question: case.question.clone(), truth: case.answer.clone() });
        }
    }
    out
}
because!(deduction_changes, DeductionChanges, "the changed theories of the lessons that the model does not learn, or of all lessons for a model of all lessons, whose forms the lessons give and whose exact reading gives the answer of the set");
