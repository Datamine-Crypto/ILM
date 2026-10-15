use patterns::{because, source};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CaseFile;
source!(CaseFile, "the cases of a kind of task as plain lines, written once from the public set by its loader: a line is a mark, a tab and a text; the told sentences of a case, then its question, then its answer, then an empty line; the first line names the kind and where its cases to read start; a line of a case with another mark is a label of the set for that case, kept as it stands for a tool that learns from labels");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    pub sentences: Vec<String>,
    pub question: String,
    pub answer: String,
}
because!(Case, CaseFile, "one case of a kind: the sentences that are told, the question, and the exact answer");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Kind {
    pub head: Vec<(String, String)>,
    pub cases: Vec<Case>,
    pub labels: Vec<Vec<String>>,
}
because!(Kind, CaseFile, "the cases of a kind in the order of their file, with the named values of the first line, and for each case the label lines that the set gives for it, in their order and with their marks");

const HEAD: char = '#';
because!(HEAD, CaseFile, "the mark of the first line, which holds named values of the kind");

const SENTENCE: char = 'S';
because!(SENTENCE, CaseFile, "the mark of a line that holds one told sentence");

const QUESTION: char = 'Q';
because!(QUESTION, CaseFile, "the mark of the line that holds the question");

const ANSWER: char = 'A';
because!(ANSWER, CaseFile, "the mark of the line that holds the answer and closes the case");

const FIELD: char = '\t';
because!(FIELD, CaseFile, "the mark between the mark of a line and its text, and between the named values of the first line");

const NAMED: char = '=';
because!(NAMED, CaseFile, "the mark between a name and its value in the first line");

fn head_values(text: &str) -> Vec<(String, String)> {
    text.split(FIELD)
        .filter_map(|part| part.split_once(NAMED))
        .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        .collect()
}
because!(head_values, CaseFile, "the named values of the first line of a case file");

pub fn read_kind(path: &Path) -> std::io::Result<Kind> {
    let mut kind = Kind::default();
    let mut sentences: Vec<String> = Vec::new();
    let mut question = String::new();
    let mut labels: Vec<String> = Vec::new();
    for line in BufReader::new(std::fs::File::open(path)?).lines() {
        let line = line?;
        let mut marks = line.chars();
        let Some(mark) = marks.next() else { continue };
        let text = marks.as_str().strip_prefix(FIELD).unwrap_or(marks.as_str());
        if mark == HEAD {
            kind.head = head_values(text);
        } else if mark == SENTENCE {
            sentences.push(text.to_string());
        } else if mark == QUESTION {
            question = text.to_string();
        } else if mark == ANSWER {
            kind.cases.push(Case { sentences: std::mem::take(&mut sentences), question: std::mem::take(&mut question), answer: text.to_string() });
            kind.labels.push(std::mem::take(&mut labels));
        } else {
            labels.push(line);
        }
    }
    Ok(kind)
}
because!(read_kind, CaseFile, "the cases of one case file in their order; a line with another mark is kept whole as a label of the case that its answer line closes");

pub fn head_value<'a>(kind: &'a Kind, name: &str) -> Option<&'a str> {
    kind.head.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
}
pub const START_NAME: &str = "read_from";
because!(START_NAME, CaseFile, "the name of the value of the first line of a case file that gives the place of the first case to read, for a kind made of two halves");

because!(head_value, CaseFile, "one named value of the first line of a case file, when it is there");

pub fn read_head(path: &Path) -> std::io::Result<Kind> {
    let mut first = String::new();
    BufReader::new(std::fs::File::open(path)?).read_line(&mut first)?;
    let mut kind = Kind::default();
    let mut marks = first.trim_end().chars();
    if marks.next() == Some(HEAD) {
        kind.head = head_values(marks.as_str().strip_prefix(FIELD).unwrap_or(marks.as_str()));
    }
    Ok(kind)
}
because!(read_head, CaseFile, "the first line of a case file alone, as a kind with no cases, for a look at a kind that does not read its cases");
