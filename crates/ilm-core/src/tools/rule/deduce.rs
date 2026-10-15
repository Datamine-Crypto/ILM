use crate::base::keys::{KeyNumbers, packed};
use crate::tools::rule::forms::{RULE_ANY, RuleFact, RulePart, RuleRole};
use patterns::{because, source};

pub struct RuleChaining;
source!(RuleChaining, "forward chaining over facts and rules with one open thing: a rule takes each subject of the theory in turn, and each rule is used on the facts and on its own results until nothing new comes. A condition that says no has two meanings: the fact with no must be there, or the fact with yes must not come out. The second is found by turns of a low and a high estimate, the alternating fixpoint of Van Gelder, so a rule cannot use the absence of a fact that comes out later");

const RULE_MOST_ROUNDS: usize = 200;
because!(RULE_MOST_ROUNDS, RuleChaining, "the most rounds of one chaining and the most turns of the two estimates: the theory of a story ends long before, and a fault of a set cannot hold a run for ever");

pub(super) type RuleClause = (Vec<RuleFact>, RuleFact);
because!(RuleClause, RuleChaining, "one rule in formal form: its conditions and its head");

pub(super) fn rule_add(parts: &[RulePart], facts: &mut Vec<RuleFact>, rules: &mut Vec<RuleClause>) {
    if let Some(((last, _), more)) = parts.split_last()
        && parts.iter().all(|(_, role)| *role == RuleRole::More)
    {
        if let Some((conditions, head)) = rules.last_mut() {
            conditions.push(*head);
            conditions.extend(more.iter().map(|(fact, _)| *fact));
            *head = *last;
        }
        return;
    }
    if parts.iter().all(|(_, role)| *role == RuleRole::Fact) {
        for (fact, _) in parts {
            if fact.open() {
                rules.push((Vec::new(), *fact));
            } else if !facts.contains(fact) {
                facts.push(*fact);
            }
        }
        return;
    }
    if let Some((head, _)) = parts.iter().find(|(_, role)| *role == RuleRole::Head) {
        rules.push((parts.iter().filter(|(_, role)| *role == RuleRole::Condition).map(|(fact, _)| *fact).collect(), *head));
    }
}
because!(rule_add, RuleChaining, "the parts of one sentence as facts or as one rule. A fact about any thing is a rule with no condition. Parts that are more of the rule before change that rule: its head becomes a condition, the parts but the last are conditions too, and the last part is the head");

fn rule_each_thing(facts: &[RuleFact], rules: &[RuleClause]) -> Vec<RuleClause> {
    let mut things: Vec<u32> = Vec::new();
    let subjects = facts.iter().chain(rules.iter().flat_map(|(conditions, head)| conditions.iter().chain([head])));
    for fact in subjects {
        if fact.subject != RULE_ANY && !things.contains(&fact.subject) {
            things.push(fact.subject);
        }
    }
    let mut out: Vec<RuleClause> = Vec::new();
    for (conditions, head) in rules {
        if !head.open() && !conditions.iter().any(RuleFact::open) {
            out.push((conditions.clone(), *head));
            continue;
        }
        for thing in &things {
            out.push((conditions.iter().map(|fact| fact.of(*thing)).collect(), head.of(*thing)));
        }
    }
    out
}
because!(rule_each_thing, RuleChaining, "every rule with its open thing replaced by each subject of the theory; a rule with no open thing stays as it is");

fn rule_results(facts: &[RuleFact], rules: &[RuleClause], absent: Option<&KeyNumbers>) -> KeyNumbers {
    let mut have = KeyNumbers::default();
    for fact in facts {
        have.number(fact.key());
    }
    let mut new: Vec<u128> = Vec::new();
    for _ in 0..RULE_MOST_ROUNDS {
        new.clear();
        for (conditions, head) in rules {
            if have.find(head.key()).is_some() {
                continue;
            }
            let holds = conditions.iter().all(|fact| match absent {
                Some(absent) if !fact.yes => absent.find(fact.said(true).key()).is_none(),
                _ => have.find(fact.key()).is_some(),
            });
            if holds {
                new.push(head.key());
            }
        }
        if new.is_empty() {
            break;
        }
        for key in &new {
            have.number(*key);
        }
    }
    have
}
because!(rule_results, RuleChaining, "the facts and all that the rules give, in rounds: every rule is tried on the facts of the round before, so the outcome does not depend on the order of the rules. A condition with no holds when the fact with no is there, or, with a set of absent facts given, when the fact with yes is not in that set");

pub(super) fn rule_at_end(facts: &[RuleFact], rules: &[RuleClause], closed: bool) -> KeyNumbers {
    let rules = rule_each_thing(facts, rules);
    if !closed || rules.iter().all(|(conditions, _)| conditions.iter().all(|fact| fact.yes)) {
        return rule_results(facts, &rules, None);
    }
    let mut low = KeyNumbers::default();
    for _ in 0..RULE_MOST_ROUNDS {
        let high = rule_results(facts, &rules, Some(&low));
        let new = rule_results(facts, &rules, Some(&high));
        if new.keys().len() == low.keys().len() && new.keys().iter().all(|key| low.find(*key).is_some()) {
            break;
        }
        low = new;
    }
    low
}
because!(rule_at_end, RuleChaining, "the facts at the end of a theory. In the closed meaning a condition with no holds when its fact with yes does not come out: the low estimate holds the facts that come out for certain, the high estimate all that can come out, and each is made from the other until the low one stays");

pub(super) fn rule_state(statement: &RuleFact, end: &KeyNumbers) -> u128 {
    packed(&[u32::from(statement.yes), u32::from(end.find(statement.said(true).key()).is_some()), u32::from(end.find(statement.said(false).key()).is_some())])
}
because!(rule_state, RuleChaining, "the state of a statement at the end of a theory, as one key: whether the statement says yes or no, whether its fact with yes is there, and whether its fact with no is there; the answer of each state is the most frequent lesson answer of the state");
