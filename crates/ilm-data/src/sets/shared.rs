use ilm_core::base::lanes::{hardware_lanes, on_lanes};
use ilm_core::base::symbols::Symbols;
use crate::sets::parquet::open;
use crate::sets::parquet::records::Value;
use patterns::{because, source};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub struct SharedRuns;
source!(SharedRuns, "a check before files are made public: a run of tokens that stands both in a tracked file and in a row of a data set that asks not to be copied to the open web shows a copied case; tokens are runs of small letters or of digits, and a run counts only with enough tokens of letters, so that rows of numbers do not count");

const TEXT_ENDS: &[&str] = &[".py", ".txt", ".md", ".rs", ".sh", ".json", ".jsonl", ".log", ".toml", ".checks", ".zones", ".html"];
because!(TEXT_ENDS, SharedRuns, "the ends of the names of the tracked files that hold text; other files are not read");

const PARQUET_END: &str = ".parquet";
because!(PARQUET_END, SharedRuns, "the end of the name of a data file of the set; other files of its folders are not read");

const RUN_TOKENS: usize = 6;
because!(RUN_TOKENS, SharedRuns, "the tokens of a run when no other number is given: fewer give common phrases of the language, more miss a case that was copied with small changes");

const RUN_WORDS: usize = 4;
because!(RUN_WORDS, SharedRuns, "the tokens of letters that a run needs when no other number is given, so that a run of numbers or of single letters does not count");

const FIRST_LINES: usize = 12;
because!(FIRST_LINES, SharedRuns, "the lines of a file that the report names; the person who cleans a file finds the rest from them");

const NAMED_KINDS: usize = 8;
because!(NAMED_KINDS, SharedRuns, "the folders of data that the report names for a file");

const SHARED_OPTION_MARK: char = '=';
because!(SHARED_OPTION_MARK, SharedRuns, "the sign between the name and the value of an option");

const SHARED_LIST_MARK: char = ',';
because!(SHARED_LIST_MARK, SharedRuns, "the sign between the items of a list in an option or an argument");

fn shared_tokens(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut run = String::new();
    let mut digits = false;
    for sign in text.chars().flat_map(char::to_lowercase) {
        let sort = if sign.is_ascii_lowercase() { Some(false) } else if sign.is_ascii_digit() { Some(true) } else { None };
        match sort {
            Some(digit) if !run.is_empty() && digit == digits => run.push(sign),
            Some(digit) => {
                if !run.is_empty() {
                    out.push(std::mem::take(&mut run));
                }
                run.push(sign);
                digits = digit;
            }
            None => {
                if !run.is_empty() {
                    out.push(std::mem::take(&mut run));
                }
            }
        }
    }
    if !run.is_empty() {
        out.push(run);
    }
    out
}
because!(shared_tokens, SharedRuns, "the tokens of a text in small letters: each run of letters of the plain alphabet and each run of digits");

fn strings_of<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::Text(text) => out.push(text),
        Value::List(items) => items.iter().for_each(|item| strings_of(item, out)),
        Value::Struct(fields) => fields.iter().for_each(|(_, item)| strings_of(item, out)),
        _ => {}
    }
}
because!(strings_of, SharedRuns, "every text inside a value of a data row, in lists and groups too");

fn data_files(folder: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    let mut paths: Vec<PathBuf> = entries.filter_map(Result::ok).map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            data_files(&path, out);
        } else if path.to_string_lossy().ends_with(PARQUET_END) {
            out.push(path);
        }
    }
}
because!(data_files, SharedRuns, "the data files under a folder, in the order of their names at every level");

struct FileRuns {
    files: Vec<String>,
    symbols: Symbols,
    places: HashMap<Vec<u32>, Vec<(usize, usize)>>,
}
because!(FileRuns, SharedRuns, "the tracked text files, the tokens that they hold as numbers, and for each run of tokens with enough words the files and lines where it stands");

fn file_runs(files: Vec<String>, run: usize, words: usize) -> FileRuns {
    let mut symbols = Symbols::new();
    let mut places: HashMap<Vec<u32>, Vec<(usize, usize)>> = HashMap::new();
    for (at, name) in files.iter().enumerate() {
        let Ok(bytes) = std::fs::read(name) else { continue };
        for (line, text) in String::from_utf8_lossy(&bytes).lines().enumerate() {
            let tokens = shared_tokens(text);
            let numbers: Vec<u32> = tokens.iter().map(|token| symbols.number(token)).collect();
            for start in 0..(tokens.len() + 1).saturating_sub(run) {
                if tokens[start..start + run].iter().filter(|token| token.starts_with(|sign: char| sign.is_ascii_lowercase())).count() >= words {
                    places.entry(numbers[start..start + run].to_vec()).or_default().push((at, line + 1));
                }
            }
        }
    }
    FileRuns { files, symbols, places }
}
because!(file_runs, SharedRuns, "the runs of the tracked files: each line cut into tokens, and each run of tokens with enough words kept with its file and its line");

fn data_hits(path: &Path, runs: &FileRuns, run: usize) -> (usize, Vec<(usize, usize)>) {
    let Ok(mut table) = open(path) else { return (0, Vec::new()) };
    let names = table.names();
    let asked: Vec<&str> = names.iter().map(String::as_str).collect();
    let Ok(columns) = table.read(&asked) else { return (0, Vec::new()) };
    let rows = columns.first().map_or(0, Vec::len);
    let mut hits: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<&str> = Vec::new();
    for value in columns.iter().flatten() {
        texts.clear();
        strings_of(value, &mut texts);
        for text in &texts {
            let numbers: Vec<Option<u32>> = shared_tokens(text).iter().map(|token| runs.symbols.find(token)).collect();
            for start in 0..(numbers.len() + 1).saturating_sub(run) {
                let Some(key) = numbers[start..start + run].iter().copied().collect::<Option<Vec<u32>>>() else { continue };
                if let Some(places) = runs.places.get(&key) {
                    hits.extend(places.iter().copied());
                }
            }
        }
    }
    (rows, hits)
}
because!(data_hits, SharedRuns, "the rows of one data file and the places of the tracked files whose runs stand in a text of its rows; a run with a token that no tracked file holds is passed over");

fn tracked_text_files(only: &[&str]) -> Result<Vec<String>, String> {
    let listed = std::process::Command::new("git").arg("ls-files").output().map_err(|error| error.to_string())?;
    let text = String::from_utf8_lossy(&listed.stdout).into_owned();
    Ok(text
        .lines()
        .filter(|name| TEXT_ENDS.iter().any(|end| name.ends_with(end)) && Path::new(name).exists())
        .filter(|name| only.is_empty() || only.iter().any(|part| name.contains(part)))
        .map(str::to_string)
        .collect())
}
because!(tracked_text_files, SharedRuns, "the text files that git tracks in the folder of the run, or those of them whose names hold one of the given parts");

pub fn shared_command(rest: &[String]) -> ExitCode {
    let named: HashMap<&str, &str> = rest.iter().filter_map(|word| word.split_once(SHARED_OPTION_MARK)).collect();
    let plain: Vec<&String> = rest.iter().filter(|word| !word.contains(SHARED_OPTION_MARK)).collect();
    let [folders] = plain.as_slice() else {
        eprintln!("usage: ilm shared-text <folder of data>[,<folder of data>] [run=<tokens>] [words=<tokens of letters>] [only=<part of a path>,<part>]");
        return ExitCode::FAILURE;
    };
    let run = named.get("run").and_then(|text| text.parse().ok()).unwrap_or(RUN_TOKENS);
    let words = named.get("words").and_then(|text| text.parse().ok()).unwrap_or(RUN_WORDS);
    let only: Vec<&str> = named.get("only").map(|text| text.split(SHARED_LIST_MARK).collect()).unwrap_or_default();
    let files = match tracked_text_files(&only) {
        Ok(files) => files,
        Err(why) => {
            eprintln!("git ls-files: {why}");
            return ExitCode::FAILURE;
        }
    };
    let runs = file_runs(files, run, words);
    println!("{} tracked text files, {} different runs of {run} tokens with {words} words or more", runs.files.len(), runs.places.len());
    let mut paths: Vec<PathBuf> = Vec::new();
    folders.split(SHARED_LIST_MARK).for_each(|folder| data_files(Path::new(folder), &mut paths));
    let done = on_lanes(paths.len(), hardware_lanes(), &|at| data_hits(&paths[at], &runs, run));
    let mut found: BTreeMap<(usize, usize), BTreeSet<String>> = BTreeMap::new();
    let mut rows = 0usize;
    for (path, (count, hits)) in paths.iter().zip(done) {
        rows += count;
        let kind = path.parent().and_then(Path::file_name).map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        for place in hits {
            found.entry(place).or_default().insert(kind.clone());
        }
    }
    let mut by_file: BTreeMap<usize, Vec<(usize, &BTreeSet<String>)>> = BTreeMap::new();
    for ((file, line), kinds) in &found {
        by_file.entry(*file).or_default().push((*line, kinds));
    }
    println!("{rows} data rows read; {} lines in {} tracked files hold a run of a data row", found.len(), by_file.len());
    let mut order: Vec<(&usize, &Vec<(usize, &BTreeSet<String>)>)> = by_file.iter().collect();
    order.sort_by_key(|(file, lines)| (std::cmp::Reverse(lines.len()), **file));
    for (file, lines) in order {
        let kinds: BTreeSet<&String> = lines.iter().flat_map(|(_, kinds)| kinds.iter()).collect();
        let named_kinds: Vec<&str> = kinds.iter().take(NAMED_KINDS).map(|kind| kind.as_str()).collect();
        let first: Vec<String> = lines.iter().take(FIRST_LINES).map(|(line, _)| line.to_string()).collect();
        println!("{:>5} lines | {} | kinds: {} | first lines: {}", lines.len(), runs.files[*file], named_kinds.join(", "), first.join(", "));
    }
    ExitCode::SUCCESS
}
because!(shared_command, SharedRuns, "the check of the tracked files against the data files of some folders: the counts of runs and rows, then each file that shares a run with a data row, with its number of lines, the folders of data and its first lines; no text of the data is printed");
