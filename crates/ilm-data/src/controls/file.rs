use ilm_core::base::lines::one_line;
use patterns::{because, source};
use std::path::{Path, PathBuf};

pub struct ChangeFile;
source!(ChangeFile, "the changed cases of a control written as a case file that the program reads back: each changed case with its told sentences, its question, a line that names the lesson it was made from and the sort of its change, and its new true answer; the first line names the kind, its set, the control and whether the changes were made from all lessons. The changes are made apart from every learner, so that a control asks the same of every tool");

const FILE_HEAD: &str = "#";
because!(FILE_HEAD, ChangeFile, "the mark of the first line, which holds the named values of the file");

const FILE_SENTENCE: &str = "S";
because!(FILE_SENTENCE, ChangeFile, "the mark of a line that holds one told sentence of a changed case");

const FILE_QUESTION: &str = "Q";
because!(FILE_QUESTION, ChangeFile, "the mark of the line that holds the question of a changed case");

const FILE_ORIGIN: &str = "O";
because!(FILE_ORIGIN, ChangeFile, "the mark of the line that names the lesson of a changed case and the sort of its change");

const FILE_ANSWER: &str = "A";
because!(FILE_ANSWER, ChangeFile, "the mark of the line that holds the new true answer and closes a changed case");

const FILE_FIELD: &str = "\t";
because!(FILE_FIELD, ChangeFile, "the mark between the mark of a line and its fields, and between the named values of the first line");

const FILE_LINE: &str = "\n";
because!(FILE_LINE, ChangeFile, "the end of a line of the file; a case ends with an empty line");

const FILE_NO_LESSON: &str = "-";
because!(FILE_NO_LESSON, ChangeFile, "what the line of the lesson says for a case that was taken from the set as it is and changes no lesson");

const FILE_TEXT_END: &str = ".txt";
because!(FILE_TEXT_END, ChangeFile, "the end of the name of a file of changed cases, after the kind and the control");

const FILE_PARTS: &str = ".";
because!(FILE_PARTS, ChangeFile, "the mark between the kind and the control in the name of a file of changed cases");

const NAME_KIND: &str = "kind=";
because!(NAME_KIND, ChangeFile, "the name of the value of the first line that gives the kind");

const NAME_SOURCE: &str = "source=";
because!(NAME_SOURCE, ChangeFile, "the name of the value of the first line that gives the set of the kind");

const NAME_CONTROL: &str = "control=";
because!(NAME_CONTROL, ChangeFile, "the name of the value of the first line that gives the control that made the changes");

const NAME_WHOLE: &str = "whole=";
because!(NAME_WHOLE, ChangeFile, "the name of the value of the first line that says whether the changes were made from all lessons, so that a model of all lessons reads them, or from the part that is held back");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChangedCase {
    pub place: Option<usize>,
    pub sort: String,
    pub sentences: Vec<String>,
    pub question: String,
    pub truth: String,
}
because!(ChangedCase, ChangeFile, "one changed case: the place of its lesson in the learned part of the kind, none for a case that is read as it is, the sort of the change, the changed told sentences, the question, and the new true answer as words with spaces between them");

pub fn changes_text(control: &str, source: &str, kind: &str, changes: &[ChangedCase], whole: bool) -> String {
    let mut lines: Vec<String> = vec![[FILE_HEAD, &format!("{NAME_KIND}{kind}"), &format!("{NAME_SOURCE}{source}"), &format!("{NAME_CONTROL}{control}"), &format!("{NAME_WHOLE}{}", u8::from(whole))].join(FILE_FIELD)];
    for change in changes {
        lines.extend(change.sentences.iter().map(|sentence| [FILE_SENTENCE, &one_line(sentence)].join(FILE_FIELD)));
        lines.push([FILE_QUESTION, &one_line(&change.question)].join(FILE_FIELD));
        let place = change.place.map_or_else(|| FILE_NO_LESSON.to_string(), |place| place.to_string());
        lines.push([FILE_ORIGIN, &place, &one_line(&change.sort)].join(FILE_FIELD));
        lines.push([FILE_ANSWER, &one_line(&change.truth)].join(FILE_FIELD));
        lines.push(String::new());
    }
    let mut text = lines.join(FILE_LINE);
    text.push_str(FILE_LINE);
    text
}
because!(changes_text, ChangeFile, "the text of a file of changed cases: the first line, then each case with its lines and an empty line after it");

pub fn write_changes(root: &Path, control: &str, source: &str, kind: &str, changes: &[ChangedCase], whole: bool) -> std::io::Result<PathBuf> {
    let folder = root.join(source);
    std::fs::create_dir_all(&folder)?;
    let path = folder.join(format!("{kind}{FILE_PARTS}{control}{FILE_TEXT_END}"));
    std::fs::write(&path, changes_text(control, source, kind, changes, whole))?;
    Ok(path)
}
because!(write_changes, ChangeFile, "the file of changed cases of one kind and one control, in the folder of its set under the given root");
