use crate::base::lanes::store_offered;
use patterns::{because, source};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::OnceLock;

pub struct KnownGoodStore;
source!(KnownGoodStore, "the known-good store of discovered rules (the user's design): the lines that the board discovered from the lessons of one kind are kept beside the library, each with the kind and the lessons it came from, and offered to every later kind as candidates that it judges again on its own lessons; a stored line is never a written line, so a proof that rests on it is discovered, and only discovery writes the store");

pub const STORE_FOLDER: &str = "data/store";
because!(STORE_FOLDER, KnownGoodStore, "the folder of the store beside the library of the run, one file for each text, named as the text");

const STORE_FILE_ENDING: &str = "txt";
because!(STORE_FILE_ENDING, KnownGoodStore, "the ending of the file of the stored lines of a text, as the texts of the library end");

const NOTE_START: &str = "Note:";
because!(NOTE_START, KnownGoodStore, "the start of a line of the store that says where the next line comes from, or what the store is, and is no candidate");

const SOURCE_START: &str = "Note: discovered on ";
because!(SOURCE_START, KnownGoodStore, "the start of the note before each stored line, before the kind, its set and the number of lessons that it was discovered from");

const STORE_HEAD: &str = "Note: The known-good store of a text: each line below was discovered by the board's learner from the lessons of the kind that the note before it names, and is offered to later kinds as a candidate that each judges on its own lessons. Only the command store-lines writes this file.";
because!(STORE_HEAD, KnownGoodStore, "the first line of a file of the store, which says what it is and that no person writes it");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredLine {
    pub line: String,
    pub source: String,
}
because!(StoredLine, KnownGoodStore, "one line of the store with what its note says of where it comes from: the kind, its set and the lessons");

pub fn stored_of(text: &str) -> Vec<StoredLine> {
    let mut out: Vec<StoredLine> = Vec::new();
    let mut source = String::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(said) = line.strip_prefix(SOURCE_START) {
            source = said.to_string();
        } else if !line.starts_with(NOTE_START) {
            out.push(StoredLine { line: line.to_string(), source: std::mem::take(&mut source) });
        }
    }
    out
}
because!(stored_of, KnownGoodStore, "the stored lines of the text of a file of the store, each with the source that its note names");

pub fn stored_groups(stored: &[StoredLine]) -> Vec<(&str, Vec<String>)> {
    let mut groups: Vec<(&str, Vec<String>)> = Vec::new();
    for one in stored {
        match groups.iter_mut().find(|(source, _)| *source == one.source.as_str()) {
            Some((_, lines)) => lines.push(one.line.clone()),
            None => groups.push((one.source.as_str(), vec![one.line.clone()])),
        }
    }
    groups
}
because!(stored_groups, KnownGoodStore, "the stored lines of a text in groups, one for each source in the order of the store: the lines that one kind's lessons taught were found together and hold together, so a learner takes a group whole or not at all");

pub fn source_kind(source: &str) -> &str {
    source.split_whitespace().next().unwrap_or(source)
}
because!(source_kind, KnownGoodStore, "the kind that the source of a stored line names, the first word of its note, as the board names the kinds whose stored lines a learning took");

pub fn store_text(known: &[StoredLine], found: &[String], source: &str) -> String {
    let mut lines: Vec<String> = vec![STORE_HEAD.to_string()];
    let mut written: Vec<(&str, &str)> = Vec::new();
    for stored in known.iter().map(|stored| (stored.line.as_str(), stored.source.as_str())).chain(found.iter().map(|line| (line.as_str(), source))) {
        if written.contains(&stored) {
            continue;
        }
        written.push(stored);
        lines.push(format!("{SOURCE_START}{}", stored.1));
        lines.push(stored.0.to_string());
    }
    lines.join(STORE_LINE_END) + STORE_LINE_END
}
because!(store_text, KnownGoodStore, "the text of a file of the store with the lines it held, then each newly found line that it does not hold yet under its source, each after a note of its source: a line that a later kind discovers again is stored again under that kind, so that the lines of each kind stay whole as one group (stored_groups), and the first kind that discovered a line still comes first");

const STORE_LINE_END: &str = "\n";
because!(STORE_LINE_END, KnownGoodStore, "the mark between the lines of a file of the store, one note or one stored line each, as a person reads them");

static STORED: OnceLock<HashMap<String, Vec<StoredLine>>> = OnceLock::new();
because!(STORED, KnownGoodStore, "the store of the folder of the run, read once, by the name of each text");

fn read_store(folder: &Path) -> HashMap<String, Vec<StoredLine>> {
    let mut out: HashMap<String, Vec<StoredLine>> = HashMap::new();
    for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ending| ending != STORE_FILE_ENDING) {
            continue;
        }
        let (Some(name), Ok(text)) = (path.file_stem().map(|stem| stem.to_string_lossy().into_owned()), std::fs::read_to_string(&path)) else { continue };
        out.insert(name, stored_of(&text));
    }
    out
}
because!(read_store, KnownGoodStore, "the files of a folder of the store, each read into its stored lines by the name of its text; a folder that does not exist is an empty store");

pub fn stored_lines(name: &str) -> &'static [StoredLine] {
    if !store_offered() {
        return &[];
    }
    STORED.get_or_init(|| read_store(Path::new(STORE_FOLDER))).get(name).map_or(&[], Vec::as_slice)
}
because!(stored_lines, KnownGoodStore, "the stored lines of a text in the store of the run that the learning of this thread may take: none when the store holds none for the text, and none for a learning that must stand on its own lessons alone");

pub fn store_key(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    for stored in stored_lines(name) {
        stored.line.hash(&mut hasher);
    }
    hasher.finish()
}
because!(store_key, KnownGoodStore, "a hash of the stored lines of a text, for the keys of what a learner learns for it: the store changes what the rule search of a text takes");

pub fn stored_source(line: &str) -> Option<&'static str> {
    let store = STORED.get_or_init(|| read_store(Path::new(STORE_FOLDER)));
    let mut names: Vec<&String> = store.keys().collect();
    names.sort();
    names.into_iter().find_map(|name| store[name].iter().find(|stored| stored.line == line)).map(|stored| stored.source.as_str())
}
because!(stored_source, KnownGoodStore, "where a stored line comes from, as its note says, for the proofs that rest on it: the first text in the order of the names that stores it");
