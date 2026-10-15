use ilm_core::cases::{Case, Kind, head_value, read_head, read_kind};
use ilm_core::cases::START_NAME;
use patterns::{because, source};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct SetFolder;
source!(SetFolder, "a public set as a folder of plain files, written once by the loader of the set: one case file for each kind, and one file that says what the authors of the set give beside the cases (the tasks of the set and their kinds, who wrote the text, the most lessons, the pairs of a small and a large form); the program reads these files and opens no file of the set itself, so every set is measured in one way");

const ABOUT_FILE: &str = "source.txt";
because!(ABOUT_FILE, SetFolder, "the name of the file of a folder that says what the set gives beside its cases");

const CASES_END: &str = ".txt";
because!(CASES_END, SetFolder, "the end of the name of the case file of a kind");

const WORDS_END: &str = ".words.txt";
because!(WORDS_END, SetFolder, "the end of the name of the file with the new words of a kind: a word, a tab, its new word on each line");

const FINAL_END: &str = ".sealed.txt";
because!(FINAL_END, SetFolder, "the end of the name of the case file with the final part of a kind, for a set that keeps a file of its own for it");

const BETWEEN: char = '\t';
because!(BETWEEN, SetFolder, "the mark between the values of a line of the file of a folder");

const SET_LINE: &str = "set";
because!(SET_LINE, SetFolder, "the line that gives the name of the public set");

const BY_LINE: &str = "by";
because!(BY_LINE, SetFolder, "the line that says who wrote the text of the set");

const BY_PEOPLE: &str = "people";
because!(BY_PEOPLE, SetFolder, "the value for a set whose text people wrote; any other value is a set whose text a program wrote");

const FIT_LINE: &str = "fit";
because!(FIT_LINE, SetFolder, "the line with the most lessons of a kind of the set, and the name of that value in a case file");

const HELD_LINE: &str = "held";
because!(HELD_LINE, SetFolder, "the line with the most read cases of a kind of the set, and the name of that value in a case file");

const KIND_LINE: &str = "kind";
because!(KIND_LINE, SetFolder, "a line that names one kind of the set");

const TASK_LINE: &str = "task";
because!(TASK_LINE, SetFolder, "a line that names a task of the set and then its kinds; a task is learned only when all its kinds are");

const PAIR_LINE: &str = "pair";
because!(PAIR_LINE, SetFolder, "a line that names a task and then a small kind and a large kind: lessons of the small kind, cases to read of the large kind");

const NO_NAMES_LINE: &str = "no_names";
because!(NO_NAMES_LINE, SetFolder, "a line that names a task that holds no name, place or thing, so that new words do not apply to it");

const BAR_ONLY_LINE: &str = "bar_only";
because!(BAR_ONLY_LINE, SetFolder, "a line that names a kind where a control is so high that no model can be clearly above it; such a kind must be at the bar, and an even kind of its task carries the controls");

const STRONGER_LINE: &str = "stronger";
because!(STRONGER_LINE, SetFolder, "the line that says what the lessons of the set hold more than a story with its answer");

const NOT_COUNTED_LINE: &str = "not_counted";
because!(NOT_COUNTED_LINE, SetFolder, "a line that names a task which passes by a regularity of its set, with the reason; it is printed and counted for nothing");

const REQUESTS_LINE: &str = "requests";
because!(REQUESTS_LINE, SetFolder, "the line of a set of requests to a service: the kind of its slots and the kind of its intents; a request of such a set is read into its facts and not answered as a question");

const ALL_LINE: &str = "all";
because!(ALL_LINE, SetFolder, "a line that names one of all the kinds that runs measure on the set, also those that are in no task; the final part of a kind is cleared of the stories that any of them has touched");


const STORIES_NAME: &str = "fit_stories";
because!(STORIES_NAME, SetFolder, "the name of the value of a case file that says that the lessons are counted in stories and not in cases");

const FIT_USUAL: usize = 600;
because!(FIT_USUAL, SetFolder, "the most lessons of a kind when the set says nothing: the size that the kinds of the first sets were measured with");

const HELD_USUAL: usize = 400;
because!(HELD_USUAL, SetFolder, "the most read cases of a kind when the set says nothing: with the usual lessons a kind of a thousand cases is used whole");

#[derive(Clone, Debug, Default)]
pub struct Source {
    pub name: String,
    pub folder: PathBuf,
    pub set: String,
    pub people: bool,
    pub fit: usize,
    pub held: usize,
    pub kinds: Vec<String>,
    pub tasks: Vec<(String, Vec<String>)>,
    pub pairs: Vec<(String, String, String)>,
    pub no_names: Vec<String>,
    pub bar_only: Vec<String>,
    pub stronger: String,
    pub not_counted: Vec<(String, String)>,
    pub all: Vec<String>,
    pub requests: Option<(String, String)>,
}
because!(Source, SetFolder, "what the folder of a set says: the name of the set, who wrote its text, the most lessons and read cases, its kinds, its tasks with their kinds, its pairs of a small and a large kind, the marks of single tasks and kinds, every kind that runs measure on the set, and for a set of requests the kind of its slots and the kind of its intents");

pub fn read_source(folder: &Path) -> std::io::Result<Source> {
    let name = folder.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let mut found = Source { set: name.clone(), name, folder: folder.to_path_buf(), fit: FIT_USUAL, held: HELD_USUAL, ..Source::default() };
    let about = folder.join(ABOUT_FILE);
    let file = std::fs::File::open(&about).map_err(|error| std::io::Error::new(error.kind(), format!("{}: {error} (a folder of a set holds the file that ilm export writes)", about.display())))?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(BETWEEN).collect();
        match fields.as_slice() {
            [SET_LINE, set] => found.set = set.to_string(),
            [BY_LINE, by] => found.people = *by == BY_PEOPLE,
            [FIT_LINE, fit] => found.fit = fit.parse().unwrap_or(FIT_USUAL),
            [HELD_LINE, held] => found.held = held.parse().unwrap_or(HELD_USUAL),
            [KIND_LINE, kind, ..] => found.kinds.push(kind.to_string()),
            [TASK_LINE, task, kinds @ ..] => found.tasks.push((task.to_string(), kinds.iter().map(|kind| kind.to_string()).collect())),
            [PAIR_LINE, task, small, large] => found.pairs.push((task.to_string(), small.to_string(), large.to_string())),
            [NO_NAMES_LINE, task] => found.no_names.push(task.to_string()),
            [BAR_ONLY_LINE, kind] => found.bar_only.push(kind.to_string()),
            [STRONGER_LINE, what] => found.stronger = what.to_string(),
            [NOT_COUNTED_LINE, task, why] => found.not_counted.push((task.to_string(), why.to_string())),
            [ALL_LINE, kind] => found.all.push(kind.to_string()),
            [REQUESTS_LINE, slots, intent] => found.requests = Some((slots.to_string(), intent.to_string())),
            _ => {}
        }
    }
    if found.tasks.is_empty() {
        found.tasks = found.kinds.iter().map(|kind| (kind.clone(), vec![kind.clone()])).collect();
    }
    Ok(found)
}
because!(read_source, SetFolder, "what the folder of a set says; a set that names no task has one task for each kind");

#[derive(Clone, Debug, Default)]
pub struct Loaded {
    pub kind: Kind,
    pub start: Option<usize>,
    pub stories: bool,
}
because!(Loaded, SetFolder, "the cases of a kind with what its file says about the cut: the place of the first case to read, and whether lessons are counted in stories");

fn head_number(kind: &Kind, name: &str) -> Option<usize> {
    head_value(kind, name).and_then(|text| text.parse::<usize>().ok())
}
because!(head_number, SetFolder, "a named value of the first line of a case file as a whole number, when it is there");

const CASES_NAME: &str = "cases";
because!(CASES_NAME, SetFolder, "the name of the number of the cases of a kind on the first line of its case file");

pub fn kind_weight(from: &Source, kind: &str, fit: usize) -> u64 {
    let path = from.folder.join(format!("{kind}{CASES_END}"));
    let Ok(bytes) = std::fs::metadata(&path).map(|meta| meta.len()) else { return u64::default() };
    let cases = read_head(&path).ok().and_then(|head| head_number(&head, CASES_NAME)).unwrap_or(1).max(1) as u64;
    bytes / cases * cases.min((fit + from.held) as u64)
}
because!(kind_weight, SetFolder, "a guess of the work of one run of the measure on a kind before it runs: the bytes of a case on the mean times the cases that the run takes; enough to start the largest runs first");

pub fn load_file(path: &Path) -> std::io::Result<Loaded> {
    let kind = read_kind(path)?;
    let start = head_number(&kind, START_NAME);
    let stories = head_number(&kind, STORIES_NAME) == Some(1);
    Ok(Loaded { kind, start, stories })
}
because!(load_file, SetFolder, "the cases of one case file with what its first line says about the cut");

pub fn file_sizes(path: &Path) -> std::io::Result<(Option<usize>, Option<usize>)> {
    let kind = read_kind(path)?;
    Ok((head_number(&kind, FIT_LINE), head_number(&kind, HELD_LINE)))
}
because!(file_sizes, SetFolder, "the most lessons and the most read cases that a case file names for its kind, for a file that is measured with no folder of a set around it");

pub fn load(from: &Source, kind: &str) -> std::io::Result<Loaded> {
    load_file(&from.folder.join(format!("{kind}{CASES_END}")))
}
because!(load, SetFolder, "the cases of a kind of a set");

pub fn load_pair(from: &Source, small: &str, large: &str) -> std::io::Result<Loaded> {
    let mut first = load(from, small)?;
    let second = load(from, large)?;
    let start = first.kind.cases.len();
    first.kind.cases.extend(second.kind.cases);
    first.kind.labels.extend(second.kind.labels);
    Ok(Loaded { kind: first.kind, start: Some(start), stories: first.stories })
}
because!(load_pair, SetFolder, "a kind made of two kinds of a set: the cases of the small kind, then the cases of the large kind, with the place where the large kind starts; the measure then learns from the small form and reads the large one");

pub fn new_words(from: &Source, kind: &str) -> Option<Vec<(String, String)>> {
    let file = std::fs::File::open(from.folder.join(format!("{kind}{WORDS_END}"))).ok()?;
    Some(BufReader::new(file).lines().map_while(Result::ok).filter_map(|line| line.split_once(BETWEEN).map(|(old, new)| (old.to_string(), new.to_string()))).collect())
}
because!(new_words, SetFolder, "the table of new words of a kind, and nothing for a kind that has no such file; a file with no line is a table that changes no case");

pub fn final_file(from: &Source, kind: &str) -> Option<Vec<Case>> {
    read_kind(&from.folder.join(format!("{kind}{FINAL_END}"))).ok().map(|kind| kind.cases)
}
because!(final_file, SetFolder, "the final part of a kind from a file of its own, when the folder holds one");
