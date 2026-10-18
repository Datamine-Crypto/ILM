use crate::tools::explain::chain::{Facts, chain_facts, empty_filling, fit_into, reply, rule_text};
use crate::tools::explain::derive::{ProofStep, answer_proof, overruled_proof};
use crate::tools::explain::learn::{folder_names, library};
use crate::tools::explain::text::{Condition, LineKind, ToldRule, Topic, fact_numbers};
use crate::tools::explain::words::{WordId, word_texts};
use patterns::{because, source};
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

pub struct RuleProvenance;
source!(RuleProvenance, "where the rules that a model reads come from: the rules that the proof of an answer rests on and the lines that wrote its told sentences again, each told as given, when a text of the library holds it, or as discovered, when no text holds it, as the model learned it from the lessons of its kind");

pub(super) struct AnswerSteps {
    pub(super) words: Vec<WordId>,
    pub(super) text: String,
    pub(super) by_default: bool,
    pub(super) steps: Vec<ProofStep>,
    pub(super) rewrites: Vec<String>,
}
because!(AnswerSteps, RuleProvenance, "the answer of a topic to a question with what it rests on: its words, the answer of the library that held, whether it held otherwise (with no condition), the steps of its proof back to the told sentences, and the lines that wrote a told sentence of the case again");

pub(super) fn answer_steps(topic: &Topic, base: &Arc<Facts>, told: &[String], asked: &[WordId], told_only: bool) -> Option<AnswerSteps> {
    let facts = chain_facts(told, base, &topic.rules, topic.rules_key);
    let answered = reply(&topic.questions, asked, &facts)?;
    if told_only && answered.reply.conditions.is_empty() {
        return None;
    }
    let known = base.added_to(&Facts::default());
    let by_default = answered.reply.conditions.is_empty();
    let mut steps = answer_proof(&topic.rules, &facts, known, &answered.reply.conditions, &answered.reply.absent, &answered.filling);
    if by_default {
        steps.extend(overruled_proof(&topic.rules, &facts, known, (&topic.questions, asked)));
    }
    Some(AnswerSteps { words: answered.words.clone(), text: answered.reply.text.clone(), by_default, steps, rewrites: rewrites_used(&topic.rules, told) })
}
because!(answer_steps, RuleProvenance, "the answer of a topic to the told sentences of a case with its proof, as the topic answers it: nothing when no answer holds, or when only an answer that holds otherwise holds and the caller asks for one that the told sentences prove; an answer that holds otherwise shows the defaults that a fact of the story overruled on the way (overruled_proof)");

fn rewrites_used(rules: &[ToldRule], told: &[String]) -> Vec<String> {
    let sentences: Vec<Vec<WordId>> = told.iter().map(|sentence| fact_numbers(sentence)).collect();
    rules
        .iter()
        .filter(|rule| matches!(rule.kind, LineKind::Replaces | LineKind::ReplacesInside))
        .filter(|rule| match rule.conditions.first() {
            Some(Condition::Fact { pattern, .. }) => sentences.iter().any(|words| {
                let mut fits = Vec::new();
                fit_into(pattern, words, &empty_filling(), &mut fits);
                !fits.is_empty()
            }),
            _ => false,
        })
        .map(rule_text)
        .collect()
}
because!(rewrites_used, RuleProvenance, "the lines of a topic that write a told sentence again whose old form fits a told sentence of the case: such a line does its work before the rules, so no proof names it");

pub(super) fn proof_rules(steps: &[ProofStep], out: &mut Vec<String>) {
    for step in steps {
        if let ProofStep::Made { rule, from, .. } = step {
            if !out.contains(rule) {
                out.push(rule.clone());
            }
            proof_rules(from, out);
        }
    }
}
because!(proof_rules, RuleProvenance, "the rules that the steps of a proof rest on, each once, in the order in which the proof first names them");

pub(super) fn used_rules(answered: &AnswerSteps) -> Vec<(String, bool)> {
    let mut rules = answered.rewrites.clone();
    proof_rules(&answered.steps, &mut rules);
    let given = given_rules();
    rules.into_iter().map(|rule| {
        let held = given.contains(&rule);
        (rule, held)
    }).collect()
}
because!(used_rules, RuleProvenance, "the rules that an answer rests on, each with whether a text of the library holds it (given) or none does (discovered from the lessons)");

pub(super) fn used_facts(answered: &AnswerSteps) -> Vec<(String, bool)> {
    let mut facts: Vec<String> = Vec::new();
    proof_facts(&answered.steps, &mut facts);
    let given = given_facts();
    facts.into_iter().map(|fact| {
        let held = given.contains(&fact);
        (fact, held)
    }).collect()
}
because!(used_facts, RuleProvenance, "the facts known before any story that an answer rests on, each with whether a text of the library holds it (given) or none does (discovered from the lessons, as the fact search learns a fact of an order of times)");

fn proof_facts(steps: &[ProofStep], out: &mut Vec<String>) {
    for step in steps {
        match step {
            ProofStep::Known(fact) if !out.contains(fact) => out.push(fact.clone()),
            ProofStep::Made { from, .. } => proof_facts(from, out),
            _ => {}
        }
    }
}
because!(proof_facts, RuleProvenance, "the known facts that the steps of a proof rest on, each once, in the order in which the proof first names them");

static GIVEN_FACTS: OnceLock<HashSet<String>> = OnceLock::new();
because!(GIVEN_FACTS, RuleProvenance, "the facts of every text of the library of the run, as a proof writes a known fact, read once");

fn given_facts() -> &'static HashSet<String> {
    GIVEN_FACTS.get_or_init(|| library().map(|library| library.texts.iter().filter(|text| !folder_names().learned.contains(&text.name)).flat_map(|text| text.facts.iter().map(|fact| word_texts(fact).join(FACT_WORD_GAP))).collect()).unwrap_or_default())
}
because!(given_facts, RuleProvenance, "the facts that the texts of the library that people wrote hold, as a proof writes a known fact, so that a fact of a model is found among them by its words; the facts of a learned text are discovered, not given");

const FACT_WORD_GAP: &str = " ";
because!(FACT_WORD_GAP, RuleProvenance, "the mark between the words of a known fact as a proof writes it");

static GIVEN_RULES: OnceLock<HashSet<String>> = OnceLock::new();
because!(GIVEN_RULES, RuleProvenance, "the rules of every text of the library of the run, as each text writes them, read once");

fn given_rules() -> &'static HashSet<String> {
    GIVEN_RULES.get_or_init(|| library().map(|library| library.texts.iter().flat_map(|text| text.rules.iter().map(rule_text)).collect()).unwrap_or_default())
}
because!(given_rules, RuleProvenance, "the rules that the texts of the library hold, as the profile writes a rule, so that a rule of a model is found among them by its text");
