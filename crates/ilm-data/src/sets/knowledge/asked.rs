use ilm_core::base::lines::one_line;
use crate::sets::export::{Described, EVERY_KIND};
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::rows_where;
use patterns::{because, source};

pub struct AskedFacts;
source!(AskedFacts, "SimpleQuestions of Bordes and others in the copy with the facts of Wikidata and their labels: people wrote a question for a given fact of a knowledge base; a row gives the question as the person wrote it, the subject, the relation with its label, and the answer; a case is a question that asks for the object of a fact, so a row whose subject has no label, or that has no answer, is left out; the train file holds the lessons and the validation file the held questions");

const LESSON_ROWS: &str = "lessons";
because!(LESSON_ROWS, AskedFacts, "the role in the description of the file of the lesson questions");

const HELD_ROWS: &str = "held";
because!(HELD_ROWS, AskedFacts, "the role in the description of the file of the held questions");

const LESSON_QUESTION: &str = "L";
because!(LESSON_QUESTION, AskedFacts, "the mark of a lesson question in the file of questions");

const HELD_QUESTION: &str = "H";
because!(HELD_QUESTION, AskedFacts, "the mark of a held question in the file of questions");

const NO_LABEL: &str = "UNDEFINED_LABEL";
because!(NO_LABEL, AskedFacts, "what the copy writes for a subject that has no label, so that its question names no subject that a reader could find");

const PYTHON_NONE: &str = "None";
because!(PYTHON_NONE, AskedFacts, "how Python writes a missing value as a text, which a line of the file of questions holds for a field that the copy leaves out");

const QUESTION_FIELD: &str = "\t";
because!(QUESTION_FIELD, AskedFacts, "the mark between the fields of a line of the file of questions");

const QUESTION_LINE: &str = "\n";
because!(QUESTION_LINE, AskedFacts, "the end of each line of the file of questions");

pub struct AskedCase {
    pub held: bool,
    pub fields: [String; ASKED_FIELDS],
}
because!(AskedCase, AskedFacts, "one question of the set: whether it is held, and its question, subject, relation, label of the relation and answer as texts; subject gives the name of the subject");

impl AskedCase {
    pub fn subject(&self) -> &str {
        &self.fields[1]
    }
}

const ASKED_FIELDS: usize = 5;
because!(ASKED_FIELDS, AskedFacts, "the fields of a question that the file of questions holds after its mark: the question, the subject, the relation, the label of the relation and the answer");

fn as_python_text(value: &Value) -> String {
    value.text().map_or_else(|| PYTHON_NONE.to_string(), str::to_string)
}
because!(as_python_text, AskedFacts, "a field as a text, a missing one as Python writes nothing");

fn field_filled(value: &Value) -> bool {
    value.text().is_some_and(|text| !text.is_empty())
}
because!(field_filled, AskedFacts, "whether a field holds a text that is not empty");

pub fn asked_cases(described: &Described) -> Result<Vec<AskedCase>, String> {
    let mut out: Vec<AskedCase> = Vec::new();
    for (held, role) in [(false, LESSON_ROWS), (true, HELD_ROWS)] {
        for place in described.files(EVERY_KIND, role) {
            for [question, subject, relation, label, answer, named] in rows_where(&place, ["original_nl_question", "nl_subject", "rdf_property", "nl_property", "nl_answer", "rdf_subject"], None)? {
                if field_filled(&named) && field_filled(&subject) && subject.text() != Some(NO_LABEL) && field_filled(&answer) {
                    out.push(AskedCase { held, fields: [as_python_text(&question), as_python_text(&subject), as_python_text(&relation), as_python_text(&label), as_python_text(&answer)] });
                }
            }
        }
    }
    Ok(out)
}
because!(asked_cases, AskedFacts, "the questions that ask for the object of a fact and whose subject has a label, the lessons first, each file in its order");

pub fn questions_file(described: &Described) -> Result<String, String> {
    let lines = asked_cases(described)?.into_iter().map(|case| {
        let mark = if case.held { HELD_QUESTION } else { LESSON_QUESTION };
        let fields: Vec<String> = case.fields.iter().map(|field| one_line(field)).collect();
        [mark.to_string(), fields.join(QUESTION_FIELD)].join(QUESTION_FIELD) + QUESTION_LINE
    });
    Ok(lines.collect())
}
because!(questions_file, AskedFacts, "the file of questions: each question a line of its mark, the question, the subject, the relation, the label of the relation and the answer");
