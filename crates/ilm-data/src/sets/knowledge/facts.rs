use ilm_core::base::lines::one_line;
use crate::sets::export::{Described, EVERY_KIND};
use crate::sets::readers::common::{rows_where, text_value};
use patterns::{because, source};
use std::collections::HashSet;

pub struct ProbeFacts;
source!(ProbeFacts, "the T-REx part of the LAMA probe of Petroni and others in the parquet files that the hub made from it: facts of Wikidata, each a subject, a relation and an object with the sentence form of the relation, one row for each sentence of an encyclopedia that says the fact, so that a fact is known by its identifier and its first row; the first fifth of the facts of each relation, in the order of the set, are the lesson facts, the others are asked");

const PROBE_ROWS: &str = "rows";
because!(PROBE_ROWS, ProbeFacts, "the role in the description of the files of the probe, in the order in which they are read");

const LESSON_SHARE: usize = 5;
because!(LESSON_SHARE, ProbeFacts, "one fact in so many of each relation is a lesson fact, the first ones in the order of the set; the others are asked, so that most facts must be found in text");

const LESSON_MARK: &str = "L";
because!(LESSON_MARK, ProbeFacts, "the mark of a lesson fact in the file of facts");

const PROBE_MARK: &str = "P";
because!(PROBE_MARK, ProbeFacts, "the mark of an asked fact in the file of facts");

const FACT_FIELD: &str = "\t";
because!(FACT_FIELD, ProbeFacts, "the mark between the fields of a line of the file of facts");

const PROBE_LINE: &str = "\n";
because!(PROBE_LINE, ProbeFacts, "the end of each line of the file of facts");

pub struct ProbeFact {
    pub relation: String,
    pub subject: String,
    pub object: String,
    pub template: String,
}
because!(ProbeFact, ProbeFacts, "one fact of the probe: its relation, its subject, its object, and the sentence form of its relation");

pub fn probe_facts(described: &Described) -> Result<Vec<ProbeFact>, String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<ProbeFact> = Vec::new();
    for place in described.files(EVERY_KIND, PROBE_ROWS) {
        for [mark, subject, object, relation, template] in rows_where(&place, ["uuid", "sub_label", "obj_label", "predicate_id", "template"], None)? {
            let text = |value: &crate::sets::parquet::records::Value| text_value(value, &place);
            if seen.insert(text(&mark)?) {
                out.push(ProbeFact { relation: text(&relation)?, subject: text(&subject)?, object: text(&object)?, template: text(&template)? });
            }
        }
    }
    Ok(out)
}
because!(probe_facts, ProbeFacts, "the facts of the probe in the order of the set, each once, with the fields of its first row");

pub fn facts_file(described: &Described) -> Result<String, String> {
    let facts = probe_facts(described)?;
    let mut by_relation: Vec<(&str, Vec<&ProbeFact>)> = Vec::new();
    for fact in &facts {
        match by_relation.iter_mut().find(|(relation, _)| *relation == fact.relation) {
            Some((_, held)) => held.push(fact),
            None => by_relation.push((&fact.relation, vec![fact])),
        }
    }
    let line = |mark: &str, fact: &ProbeFact| [mark, &one_line(&fact.relation), &one_line(&fact.subject), &one_line(&fact.object), &one_line(&fact.template)].join(FACT_FIELD) + PROBE_LINE;
    let mut out = String::new();
    for (mark, lessons) in [(LESSON_MARK, true), (PROBE_MARK, false)] {
        for (_, held) in &by_relation {
            let cut = held.len() / LESSON_SHARE;
            let part = if lessons { &held[..cut] } else { &held[cut..] };
            out.extend(part.iter().map(|fact| line(mark, fact)));
        }
    }
    Ok(out)
}
because!(facts_file, ProbeFacts, "the file of facts: the lesson facts of every relation, the relations in the order in which the set first names them, then the asked facts in the same order; each fact a line of its mark, its relation, subject, object and sentence form");
