use crate::base::words::{answer_words, words};
use crate::tools::frames::story::{THING_ENDS, ThingPair};
use patterns::{because, source};

pub struct ToldSteps;
source!(ToldSteps, "the steps of a lesson story from its label lines: a unit line opens a written unit of the story, and each fact line of a unit is a step when its fact holds, a first name, a relation and a second name, which says that the second is the relation of the first. Labels that hold a rule, a fact that does not hold, or a name of more than one word give no steps; the steps of a lesson count only when they make a chain from the first thing of its question to the second");

const STEP_FIELD: char = '\t';
because!(STEP_FIELD, ToldSteps, "the mark between the fields of a label line");

const STEP_UNIT: &str = "U";
because!(STEP_UNIT, ToldSteps, "the mark of a label line that opens a written unit of a story");

const STEP_FACT: &str = "F";
because!(STEP_FACT, ToldSteps, "the mark of a label line that holds a fact of its unit");

const STEP_RULE: &[&str] = &["C", "H"];
because!(STEP_RULE, ToldSteps, "the marks of the label lines that hold a part of a rule, which is no step");

const STEP_HOLDS: &str = "yes";
because!(STEP_HOLDS, ToldSteps, "how a label line writes that its fact holds");

pub(super) const STEP_BETWEEN: &str = " ";
because!(STEP_BETWEEN, ToldSteps, "the mark between the words of a relation or of an answer in its text as a label");

pub(super) type LabelStep = (String, String, String);
because!(LabelStep, ToldSteps, "one step: the word of the first name, the relation, and the word of the second name");

fn one_word(text: &str) -> Option<String> {
    match words(text).as_slice() {
        [word] => Some(word.clone()),
        _ => None,
    }
}
because!(one_word, ToldSteps, "the word of a name that is one word, and nothing for any other name");

pub(super) fn label_steps(lines: &[String]) -> Option<Vec<LabelStep>> {
    let mut opened = false;
    let mut out: Vec<LabelStep> = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(STEP_FIELD).collect();
        let Some(mark) = fields.first() else { continue };
        if *mark == STEP_UNIT {
            opened = true;
        } else if STEP_RULE.contains(mark) && opened {
            return None;
        } else if *mark == STEP_FACT && opened {
            let [_, first, relation, second, holds] = fields.as_slice() else { return None };
            if *holds != STEP_HOLDS {
                return None;
            }
            out.push((one_word(first)?, answer_words(relation).join(STEP_BETWEEN), one_word(second)?));
        }
    }
    opened.then_some(out)
}
because!(label_steps, ToldSteps, "the steps of a lesson story from its label lines, in the order of the set; nothing when the lines open no unit or hold anything that is no step");

pub(super) fn chain_of(lines: &[String], ends: &ThingPair) -> Option<Vec<LabelStep>> {
    let steps = label_steps(lines)?;
    let (first, last) = (steps.first()?, steps.last()?);
    if first.0 != ends.0 || last.2 != ends.1 || steps.windows(THING_ENDS).any(|pair| pair[0].2 != pair[1].0) {
        return None;
    }
    Some(steps)
}
because!(chain_of, ToldSteps, "the steps of a lesson when they lead from the first thing of its question to the second, each from the name where the step before it ends");
