pub mod asked;
pub mod classes;
pub mod facts;
pub mod letters;
pub mod store;
pub mod text_facts;
pub mod titles;

use crate::sets::export::Described;
use patterns::{because, source};
use std::path::Path;

pub struct KnowledgeSets;
source!(KnowledgeSets, "the sets of the reader of facts and of the reader of questions: a probe of facts with the sentence form of each relation, and questions that people wrote about single facts; a description says which of them it holds by one line, and its set is written as one file of lines");

const KNOWLEDGE_KEY: &str = "knowledge";
because!(KNOWLEDGE_KEY, KnowledgeSets, "the key of the line of a description that makes a set a set of knowledge and says which file it gives");

const FACTS_FORM: &str = "facts";
because!(FACTS_FORM, KnowledgeSets, "the value of that line for a probe of facts, whose file of facts the reader of facts learns from and is asked from");

const QUESTIONS_FORM: &str = "questions";
because!(QUESTIONS_FORM, KnowledgeSets, "the value of that line for a set of questions, whose file of questions the reader of questions learns from and is measured on");

const FACTS_NAME: &str = "facts.txt";
because!(FACTS_NAME, KnowledgeSets, "the name of the file of facts in the folder that a run names");

const QUESTIONS_NAME: &str = "questions.txt";
because!(QUESTIONS_NAME, KnowledgeSets, "the name of the file of questions in the folder that a run names");

pub fn write_knowledge(described: &Described, folder: &Path) -> Option<Result<Vec<String>, String>> {
    let form = described.value(KNOWLEDGE_KEY)?;
    Some((|| {
        let (name, text) = match form {
            FACTS_FORM => (FACTS_NAME, facts::facts_file(described)?),
            QUESTIONS_FORM => (QUESTIONS_NAME, asked::questions_file(described)?),
            other => return Err(format!("no set of knowledge of the form {other}")),
        };
        std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
        let place = folder.join(name);
        std::fs::write(&place, &text).map_err(|error| format!("{}: {error}", place.display()))?;
        Ok(vec![format!("{}: {} lines", place.display(), text.lines().count())])
    })())
}
because!(write_knowledge, KnowledgeSets, "the file of a set of knowledge written into a folder from the raw files that its description names; nothing for a description of another set");
