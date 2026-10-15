use crate::write::quotes::escaped_back;
use patterns::{because, source};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct RowFile;
source!(RowFile, "the rows of a set for the writer as plain lines, written once from the public set by its loader: a line with a mark, a tab, a slot, a tab and a value for each fact in the order of the set, then a line with a mark, a tab and the text that a person wrote from these facts, then an empty line; the first line names the set and, for a set that writes lists inside one value, the signs between the items");

pub struct WantedTexts;
source!(WantedTexts, "the texts that a writer must give for the lists of facts of one part, as plain lines: a first line with the numbers of the measure for these texts as named values, then one line with a mark, a tab and the text for each list in the order of the lists; a build of the writer is proved against such a file");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactRow {
    pub facts: Vec<(String, String)>,
    pub text: String,
}
because!(FactRow, RowFile, "one row of a set: its facts as slot and value in the order of the set, and the text of a person");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactRows {
    pub head: HashMap<String, String>,
    pub rows: Vec<FactRow>,
}
because!(FactRows, RowFile, WantedTexts, "the rows of one file in their order with the named values of the first line; a file of wanted texts is read as rows with no facts");

const WRITE_ROW_HEAD: char = '#';
because!(WRITE_ROW_HEAD, RowFile, "the mark of the first line of a file of rows, which holds named values");

const WRITE_ROW_FACT: char = 'F';
because!(WRITE_ROW_FACT, RowFile, "the mark of a line that holds one fact of a row");

const WRITE_ROW_TEXT: char = 'T';
because!(WRITE_ROW_TEXT, RowFile, "the mark of the line that holds the text of a row and closes the row");

const WRITE_ROW_WRITTEN: char = 'W';
because!(WRITE_ROW_WRITTEN, WantedTexts, "the mark of a line of a file of wanted texts: the text for one list of facts, as it is, with nothing changed");

const WRITE_ROW_FIELD: char = '\t';
because!(WRITE_ROW_FIELD, RowFile, "the mark between the parts of a line of a file of rows");

const WRITE_ROW_NAMED: char = '=';
because!(WRITE_ROW_NAMED, RowFile, "the mark between a name and its value in the first line of a file of rows");

const WRITE_CUT_NAME: &str = "cut";
because!(WRITE_CUT_NAME, RowFile, "the name of the value of the first line that gives the signs between the items of a list, each as its code");

const WRITE_CUT_BETWEEN: char = '.';
because!(WRITE_CUT_BETWEEN, RowFile, "the mark between the codes of the signs of the cut of lists");

pub fn read_rows(path: &Path) -> std::io::Result<FactRows> {
    let mut out = FactRows::default();
    let mut facts: Vec<(String, String)> = Vec::new();
    for line in BufReader::new(std::fs::File::open(path)?).lines() {
        let line = line?;
        let mut marks = line.chars();
        let Some(mark) = marks.next() else { continue };
        let rest = marks.as_str().strip_prefix(WRITE_ROW_FIELD).unwrap_or(marks.as_str());
        if mark == WRITE_ROW_HEAD {
            out.head = rest.split(WRITE_ROW_FIELD).filter_map(|part| part.split_once(WRITE_ROW_NAMED)).map(|(name, value)| (name.to_string(), value.to_string())).collect();
        } else if mark == WRITE_ROW_FACT {
            let (slot, value) = rest.split_once(WRITE_ROW_FIELD).unwrap_or((rest, ""));
            facts.push((escaped_back(slot), escaped_back(value)));
        } else if mark == WRITE_ROW_TEXT {
            out.rows.push(FactRow { facts: std::mem::take(&mut facts), text: escaped_back(rest) });
        } else if mark == WRITE_ROW_WRITTEN {
            out.rows.push(FactRow { facts: Vec::new(), text: rest.to_string() });
        }
    }
    Ok(out)
}
because!(read_rows, RowFile, WantedTexts, "the rows of one file in their order; a line with another mark is left out");

pub fn cut_of_lists(rows: &FactRows) -> Option<String> {
    let codes = rows.head.get(WRITE_CUT_NAME)?;
    codes.split(WRITE_CUT_BETWEEN).map(|code| code.parse::<u32>().ok().and_then(char::from_u32)).collect()
}
because!(cut_of_lists, RowFile, "the signs that the set writes between the items of a list inside one value, when the first line of the file gives them; the check of the measure reads lists by them");
