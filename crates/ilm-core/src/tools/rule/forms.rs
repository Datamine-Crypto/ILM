use crate::base::keys::packed;
use crate::base::runs::RunNumbers;
use crate::tools::rule::tokens::rule_sentence;
use patterns::{because, source};

pub struct FormalForms;
source!(FormalForms, "the formal notation of the sets of deduction with told rules, after RuleTaker of Clark, Tafjord and Richardson: a told sentence is a fact, a thing with a relation and a value that holds or does not hold, or a rule from conditions to a head, where a thing can be any thing. A set that gives these forms beside its lesson sentences gives stronger lessons than a story with its answer");

pub(super) const RULE_ANY: u32 = 0;
because!(RULE_ANY, FormalForms, "the number of the thing that stands for any thing; the things that a text names have the numbers after it");

const RULE_ANY_TEXT: &str = "?";
because!(RULE_ANY_TEXT, FormalForms, "how a label line writes any thing in the place of a subject or of an object");

const RULE_FIELD: char = '\t';
because!(RULE_FIELD, FormalForms, "the mark between the fields of a label line");

const RULE_UNIT_LINE: &str = "U";
because!(RULE_UNIT_LINE, FormalForms, "the mark of a label line that opens one written unit of a story and names its first told sentence and the sentence after its last");

const RULE_FACT_LINE: &str = "F";
because!(RULE_FACT_LINE, FormalForms, "the mark of a label line that holds a fact of its unit");

const RULE_CONDITION_LINE: &str = "C";
because!(RULE_CONDITION_LINE, FormalForms, "the mark of a label line that holds a condition of a rule of its unit");

const RULE_HEAD_LINE: &str = "H";
because!(RULE_HEAD_LINE, FormalForms, "the mark of a label line that holds the head of a rule and closes the rule");

const RULE_STATEMENT_LINE: &str = "T";
because!(RULE_STATEMENT_LINE, FormalForms, "the mark of the label line that holds the fact which the statement of the question says");

const RULE_HOLDS: &str = "yes";
because!(RULE_HOLDS, FormalForms, "how a label line writes that a fact holds; any other text says that it does not");

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum RuleRole {
    Condition,
    Fact,
    Head,
    More,
}
because!(RuleRole, FormalForms, "what a part of a sentence is in its formal form: a condition of a rule, a fact, the head of a rule, or more of the rule that the sentence before began. The order of the roles is fixed, so that of two roles with equal counts the same one is taken in every run");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct RuleFact {
    pub(super) subject: u32,
    pub(super) relation: u32,
    pub(super) thing: u32,
    pub(super) yes: bool,
}
because!(RuleFact, FormalForms, "one fact: a subject thing, a relation, an object thing, and whether it holds. key gives the fact as one wide number for a set of facts; open says that it speaks of any thing; of puts a thing in the place of any thing; said gives the same fact with the other value of holding or not");

impl RuleFact {
    pub(super) fn key(&self) -> u128 {
        packed(&[self.subject, self.relation, self.thing, u32::from(self.yes)])
    }

    pub(super) fn open(&self) -> bool {
        self.subject == RULE_ANY || self.thing == RULE_ANY
    }

    pub(super) fn of(&self, thing: u32) -> RuleFact {
        let named = |held: u32| if held == RULE_ANY { thing } else { held };
        RuleFact { subject: named(self.subject), thing: named(self.thing), ..*self }
    }

    pub(super) fn said(&self, yes: bool) -> RuleFact {
        RuleFact { yes, ..*self }
    }
}

pub(super) type RulePart = (RuleFact, RuleRole);
because!(RulePart, FormalForms, "one part of the formal form of a sentence: a fact with its role");

#[derive(Clone, Debug, Default)]
pub(super) struct RuleUnit {
    pub(super) first: usize,
    pub(super) end: usize,
    pub(super) items: Vec<Vec<RulePart>>,
}
because!(RuleUnit, FormalForms, "one written unit of a lesson story: its told sentences from the first to the one before the end, and its formal forms in the order of the set, each as its parts (one fact, or the conditions of a rule and then its head)");

#[derive(Clone, Debug, Default)]
pub(super) struct RuleLabels {
    pub(super) units: Vec<RuleUnit>,
    pub(super) statement: Option<RuleFact>,
}
because!(RuleLabels, FormalForms, "what the label lines of one lesson case say: the units of its story with their formal forms, and the fact that its statement says");

pub(super) fn rule_thing<N: FnMut(&str) -> u32>(text: &str, terms: &mut RunNumbers, name: &mut N) -> u32 {
    if text == RULE_ANY_TEXT {
        return RULE_ANY;
    }
    terms.number(&rule_sentence(&text.to_lowercase(), name).toks) + 1
}
because!(rule_thing, FormalForms, "the number of a thing of a formal form: any thing, or the run of the tokens of its text in small letters, so that a thing is found in a sentence token by token");

pub(super) fn rule_labels<N: FnMut(&str) -> u32>(lines: &[String], terms: &mut RunNumbers, name: &mut N) -> RuleLabels {
    let mut out = RuleLabels::default();
    let mut open: Vec<RulePart> = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(RULE_FIELD).collect();
        match fields.as_slice() {
            [RULE_UNIT_LINE, first, end] => {
                if let (Ok(first), Ok(end)) = (first.parse(), end.parse()) {
                    out.units.push(RuleUnit { first, end, items: Vec::new() });
                }
            }
            [mark, subject, relation, thing, holds] => {
                let fact = RuleFact { subject: rule_thing(subject, terms, name), relation: name(&relation.to_lowercase()), thing: rule_thing(thing, terms, name), yes: *holds == RULE_HOLDS };
                if *mark == RULE_STATEMENT_LINE {
                    out.statement = Some(fact);
                }
                let Some(unit) = out.units.last_mut() else { continue };
                if *mark == RULE_FACT_LINE {
                    unit.items.push(vec![(fact, RuleRole::Fact)]);
                } else if *mark == RULE_CONDITION_LINE {
                    open.push((fact, RuleRole::Condition));
                } else if *mark == RULE_HEAD_LINE {
                    open.push((fact, RuleRole::Head));
                    unit.items.push(std::mem::take(&mut open));
                }
            }
            _ => {}
        }
    }
    out
}
because!(rule_labels, FormalForms, "the formal forms of one lesson case from its label lines: a unit line opens a unit, a fact line is a form of its own, condition lines are gathered until the head line closes their rule, and the statement line gives the fact of the statement; a line of another make is passed over");
