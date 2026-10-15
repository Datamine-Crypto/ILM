use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::readers::common::{LESSON_FILES, READ_FILES, lines_of, stripped, text_of};
use patterns::{because, source};

pub struct LookupSet;
source!(LookupSet, "the lookup tables of Liska, Kruszewski and Baroni in the files of Hupkes and others: a sample of the set has eight tables, each giving for each row of three bits another row; a text is a row of bits and the names of the tables in the order of their use, and the answer is the row and what each table gives in turn. A file is a table of lines with the text and the answer between tabs, and a line with an empty text or answer is no case; the train file of a sample gives the lessons of each of its kinds, and the test files of a kind follow in their order, where the cases to read start");

const LOOKUP_FIELD: char = '\t';
because!(LOOKUP_FIELD, LookupSet, "the mark between the text, the answer and the other columns of a line");

const LOOKUP_LINE_END: char = '\n';
because!(LOOKUP_LINE_END, LookupSet, "the end of a line, which is no part of its last column");

const LOOKUP_FIELDS: usize = 2;
because!(LOOKUP_FIELDS, LookupSet, "the columns that a line of a case has at least: the text and the answer; a column for an attention network may follow and is not read");

fn lookup_rows(place: &str) -> Result<Vec<Case>, String> {
    let mut out: Vec<Case> = Vec::new();
    for line in lines_of(&text_of(place)?) {
        let fields: Vec<&str> = line.trim_end_matches(LOOKUP_LINE_END).split(LOOKUP_FIELD).collect();
        if fields.len() < LOOKUP_FIELDS {
            continue;
        }
        let (question, answer) = (stripped(fields[0]), stripped(fields[1]));
        if !question.is_empty() && !answer.is_empty() {
            out.push(Case { sentences: Vec::new(), question: question.to_string(), answer: answer.to_string() });
        }
    }
    Ok(out)
}
because!(lookup_rows, LookupSet, "the cases of one file in its order: the text as the question and the answer, with no told sentence");

pub fn lookup_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let mut cases = lookup_rows(&described.file(kind, LESSON_FILES)?)?;
    let read_from = Some(cases.len());
    for place in described.files(kind, READ_FILES) {
        cases.extend(lookup_rows(&place)?);
    }
    Ok(KindCases { cases, read_from, ..KindCases::default() })
}
because!(lookup_cases, LookupSet, "the cases of a kind: the lessons of the train file of its sample, then the cases of its test files, and the place of the first of them");
