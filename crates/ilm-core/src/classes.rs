use patterns::{because, source};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct WordClassTable;
source!(WordClassTable, "classes of words counted from plain text with no label: words that stand in the same places of a text fall into one class, and the table gives each word the name of its class; with it a tool can read a word that no lesson holds as a word of a known sort");

pub type WordClasses = HashMap<String, String>;
because!(WordClasses, WordClassTable, "the class of each word of a table, found by the word");

pub fn read_classes(path: &Path) -> std::io::Result<WordClasses> {
    let mut classes: WordClasses = HashMap::new();
    for line in BufReader::new(std::fs::File::open(path)?).lines() {
        let line = line?;
        let mut parts = line.split_whitespace();
        if let (Some(word), Some(class)) = (parts.next(), parts.next()) {
            classes.insert(word.to_string(), class.to_string());
        }
    }
    Ok(classes)
}
because!(read_classes, WordClassTable, "the table of a file: a word and its class at the start of each line, with space between them; more fields of a line are not read, and a line with fewer is passed over");
