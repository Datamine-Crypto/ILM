use ilm_core::base::lanes::{hardware_lanes, on_lanes, shards};
use crate::sets::export::Described;
use crate::sets::json::json_text;
use crate::sets::knowledge::asked::asked_cases;
use crate::sets::knowledge::facts::probe_facts;
use crate::sets::knowledge::letters::letters;
use crate::sets::knowledge::titles::names;
use crate::sets::parquet::open;
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{lines_of, stripped};
use patterns::{because, source};
use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub struct EncyclopediaStore;
source!(EncyclopediaStore, "the text store of the reader of facts, made from a copy of an encyclopedia in parquet files with the title and the plain text of each article, read file by file in the order of their names: the articles that answer to the name of a subject of the probe (by the names of their title in small letters), other articles of the first files up to a number of signs, the articles that answer to the name of a subject of the questions and stand in neither of those two files, all titles in the order of their signs, and the known names (the letters of each name of a title, each once, in rising order); an article is one line, a JSON object with its title and its text");

const SUBJECTS_FILE: &str = "subjects.jsonl";
because!(SUBJECTS_FILE, EncyclopediaStore, "the file of the store with the articles that answer to the name of a subject of the probe");

const QUESTIONS_FILE: &str = "questions.jsonl";
because!(QUESTIONS_FILE, EncyclopediaStore, "the file of the store with the articles that answer to the name of a subject of the questions that people wrote");

const OTHERS_FILE: &str = "others.jsonl";
because!(OTHERS_FILE, EncyclopediaStore, "the file of the store with other articles, for a run at the full size of the text");

const OTHER_SIGNS: usize = 1_000_000_000;
because!(OTHER_SIGNS, EncyclopediaStore, "the signs of other articles that the store holds when a run names no number: text for a model at full size beside the articles of the subjects");

const OTHER_FILES: usize = 4;
because!(OTHER_FILES, EncyclopediaStore, "the first files of the copy from which the other articles come, an equal share of the signs from each");

const TITLES_FILE: &str = "titles.txt";
because!(TITLES_FILE, EncyclopediaStore, "the file of the store with the titles of all articles, the names that the encyclopedia has");

const KNOWN_NAMES_FILE: &str = "titles.txt.plain";
because!(KNOWN_NAMES_FILE, EncyclopediaStore, "the file of the store with the known names, the letters of each name of a title, read by the reader of questions");

const DUMP_END: &str = ".parquet";
because!(DUMP_END, EncyclopediaStore, "the end of the name of a file of the copy");

const HIDDEN_START: char = '.';
because!(HIDDEN_START, EncyclopediaStore, "the start of the name of a hidden file, which is no file of the copy");

const TITLE_COLUMN: &str = "title";
because!(TITLE_COLUMN, EncyclopediaStore, "the column of the title of an article, and its key in a line of the store");

const TEXT_COLUMN: &str = "text";
because!(TEXT_COLUMN, EncyclopediaStore, "the column of the plain text of an article, and its key in a line of the store");

const ARTICLE_OPEN: &str = "{";
because!(ARTICLE_OPEN, EncyclopediaStore, "the sign that opens the JSON object of an article");

const ARTICLE_CLOSE: &str = "}";
because!(ARTICLE_CLOSE, EncyclopediaStore, "the sign that closes the JSON object of an article");

const KEY_GAP: &str = ": ";
because!(KEY_GAP, EncyclopediaStore, "the signs between a key and its value in a line of the store");

const MEMBER_GAP: &str = ", ";
because!(MEMBER_GAP, EncyclopediaStore, "the signs between the title and the text in a line of the store");

const STORE_LINE: char = '\n';
because!(STORE_LINE, EncyclopediaStore, "the end of each line of a file of the store; a title that holds it is left out of the file of titles");

const NAMES_GAP: &str = "\n";
because!(NAMES_GAP, EncyclopediaStore, "the mark between two known names, with none after the last");

const OTHERS_OPTION: &str = "others";
because!(OTHERS_OPTION, EncyclopediaStore, "the option with the number of signs of other articles");

const STORE_LANES: &str = "lanes";
because!(STORE_LANES, EncyclopediaStore, "the option with the number of threads, each of which reads one file of the copy at a time");

const STORE_OPTION_MARK: char = '=';
because!(STORE_OPTION_MARK, EncyclopediaStore, "the mark between the name and the value of an option");

type StoreArticle = (String, String);
because!(StoreArticle, EncyclopediaStore, "what a line of the store holds of an entry of the encyclopedia: its title and its plain text");

#[derive(Default)]
struct FileArticles {
    subjects: Vec<StoreArticle>,
    others: Vec<StoreArticle>,
    asked: Vec<StoreArticle>,
    titles: Vec<String>,
}
because!(FileArticles, EncyclopediaStore, "what one file of the copy gives: the articles of the subjects of the probe, the other articles, the articles of the subjects of the questions that are neither, and every title, each in the order of the file");

fn file_articles(place: &Path, probe: &HashSet<String>, asked: &HashSet<String>, most: usize) -> Result<FileArticles, String> {
    let mut table = open(place)?;
    let mut found = FileArticles::default();
    let mut size = 0usize;
    for group in 0..table.groups() {
        let mut columns = table.read_group(group, &[TITLE_COLUMN, TEXT_COLUMN])?.into_iter();
        let (Some(titles), Some(texts)) = (columns.next(), columns.next()) else { return Err(format!("{}: a row group with no titles or texts", place.display())) };
        for (title, text) in titles.into_iter().zip(texts) {
            let (Value::Text(title), Value::Text(text)) = (title, text) else { return Err(format!("{}: an article with no title or no text", place.display())) };
            let named = names(&title);
            found.titles.push(title.clone());
            if named.iter().any(|name| probe.contains(name)) {
                found.subjects.push((title, text));
            } else if size < most {
                size += text.chars().count();
                found.others.push((title, text));
            } else if named.iter().any(|name| asked.contains(name)) {
                found.asked.push((title, text));
            }
        }
    }
    Ok(found)
}
because!(file_articles, EncyclopediaStore, "the articles of one file: an article goes with the subjects of the probe when one of its names is the name of such a subject; else it is another article while the signs of the other articles of the file stay under the share of the file; else it goes with the subjects of the questions when one of its names is the name of such a subject");

fn article_line(article: &StoreArticle) -> String {
    [ARTICLE_OPEN, &json_text(TITLE_COLUMN), KEY_GAP, &json_text(&article.0), MEMBER_GAP, &json_text(TEXT_COLUMN), KEY_GAP, &json_text(&article.1), ARTICLE_CLOSE].concat()
}
because!(article_line, EncyclopediaStore, "an article as one line of the store: a JSON object with its title and its text");

fn write_articles<'a>(place: &Path, articles: impl Iterator<Item = &'a StoreArticle>) -> Result<usize, String> {
    let fault = |error: std::io::Error| format!("{}: {error}", place.display());
    let mut out = BufWriter::new(std::fs::File::create(place).map_err(fault)?);
    let mut count = 0usize;
    for article in articles {
        out.write_all(article_line(article).as_bytes()).map_err(fault)?;
        out.write_all(&[STORE_LINE as u8]).map_err(fault)?;
        count += 1;
    }
    out.flush().map_err(fault)?;
    Ok(count)
}
because!(write_articles, EncyclopediaStore, "a file of the store written from articles, one line each; the number of them");

pub(crate) fn dump_files(dump: &Path) -> Result<Vec<PathBuf>, String> {
    let listed = std::fs::read_dir(dump).map_err(|error| format!("{}: {error}", dump.display()))?;
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for entry in listed {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(DUMP_END) && !name.starts_with(HIDDEN_START) {
            files.push((name, entry.path()));
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("{}: no file of a copy of the encyclopedia (no name ends with {DUMP_END})", dump.display()));
    }
    Ok(files.into_iter().map(|(_, place)| place).collect())
}
because!(dump_files, EncyclopediaStore, "the files of the copy in a folder, in the order of their names; a folder with none is a fault, so that a wrong folder does not make an empty store or table");

pub fn make_store(dump: &Path, store: &Path, probe: &Described, questions: &Described, others: usize, lanes: usize) -> Result<Vec<String>, String> {
    let probe_names: HashSet<String> = probe_facts(probe)?.iter().map(|fact| fact.subject.to_lowercase()).collect();
    let asked_names: HashSet<String> = asked_cases(questions)?.iter().map(|case| case.subject().to_lowercase()).collect();
    let files = dump_files(dump)?;
    let share = others / files.len().clamp(1, OTHER_FILES) + 1;
    let read: Vec<Result<FileArticles, String>> = on_lanes(files.len(), lanes, &|place| file_articles(&files[place], &probe_names, &asked_names, if place < OTHER_FILES { share } else { 0 }));
    let found: Vec<FileArticles> = read.into_iter().collect::<Result<Vec<FileArticles>, String>>()?;
    std::fs::create_dir_all(store).map_err(|error| format!("{}: {error}", store.display()))?;
    let subjects = write_articles(&store.join(SUBJECTS_FILE), found.iter().flat_map(|file| file.subjects.iter()))?;
    let other = write_articles(&store.join(OTHERS_FILE), found.iter().flat_map(|file| file.others.iter()))?;
    let have: HashSet<&str> = found.iter().flat_map(|file| file.subjects.iter().chain(&file.others)).map(|(title, _)| title.as_str()).collect();
    let asked = write_articles(&store.join(QUESTIONS_FILE), found.iter().flat_map(|file| file.asked.iter()).filter(|(title, _)| !have.contains(title.as_str())))?;
    let mut all: Vec<&str> = found.iter().flat_map(|file| file.titles.iter().map(String::as_str)).collect();
    all.sort_unstable();
    all.dedup();
    let mut titles = String::new();
    for title in all.iter().filter(|title| !title.contains(STORE_LINE)) {
        titles.push_str(title);
        titles.push(STORE_LINE);
    }
    std::fs::write(store.join(TITLES_FILE), &titles).map_err(|error| error.to_string())?;
    let lines = lines_of(&titles);
    let runs = shards(lines.len(), lanes);
    let mut known: Vec<String> = on_lanes(runs.len(), lanes, &|run| {
        let mut made: Vec<String> = lines[runs[run].clone()].iter().flat_map(|line| names(stripped(line)).into_iter().map(|name| letters(&name))).collect();
        made.sort_unstable();
        made.dedup();
        made
    })
    .concat();
    known.sort_unstable();
    known.dedup();
    known.retain(|name| !name.is_empty());
    std::fs::write(store.join(KNOWN_NAMES_FILE), known.iter().map(String::as_str).collect::<Vec<&str>>().join(NAMES_GAP)).map_err(|error| error.to_string())?;
    Ok(vec![
        format!("{} files of the copy; {} names of subjects of the probe, {} of the questions", files.len(), probe_names.len(), asked_names.len()),
        format!("{SUBJECTS_FILE}: {subjects} articles; {OTHERS_FILE}: {other} articles; {QUESTIONS_FILE}: {asked} articles"),
        format!("{TITLES_FILE}: {} titles; {KNOWN_NAMES_FILE}: {} names", all.len(), known.len()),
    ])
}
because!(make_store, EncyclopediaStore, "the files of the text store from the files of the copy, each file of the copy read on a thread of its own and taken in the order of their names; the titles and the known names are sorted lists with each entry once, the order of their signs, and the known names of the titles are made in runs on all threads");

pub fn store_command(rest: &[String]) -> ExitCode {
    let (named, plain): (Vec<&String>, Vec<&String>) = rest.iter().partition(|argument| argument.contains(STORE_OPTION_MARK));
    let option = |name: &str| named.iter().find_map(|argument| argument.split_once(STORE_OPTION_MARK).filter(|(key, _)| *key == name).and_then(|(_, value)| value.parse::<usize>().ok()));
    let [dump, store, probe, questions] = plain.as_slice() else {
        eprintln!("usage: ilm store <folder of the copy of the encyclopedia> <folder of the store> <description of the probe> <description of the questions> [{OTHERS_OPTION}=<signs>] [{STORE_LANES}=<n>]");
        return ExitCode::FAILURE;
    };
    let described = |place: &str| Described::read(Path::new(place));
    let made = described(probe).and_then(|probe| described(questions).and_then(|questions| {
        make_store(Path::new(dump), Path::new(store), &probe, &questions, option(OTHERS_OPTION).unwrap_or(OTHER_SIGNS), option(STORE_LANES).filter(|lanes| *lanes > 0).unwrap_or_else(hardware_lanes))
    }));
    match made {
        Ok(said) => {
            said.iter().for_each(|line| println!("{line}"));
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}
because!(store_command, EncyclopediaStore, "the text store written from a copy of an encyclopedia, with the descriptions of the probe and of the questions whose subjects it holds the articles of");
