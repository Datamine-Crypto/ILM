use ilm_core::base::lanes::{hardware_lanes, on_lanes};
use ilm_core::class_count::{COUNTED_WORDS, CONTEXT_WORDS, MOST_RUNS, all_rows, count_tokens, frequent_words, neighbour_classes, row_width, word_codes};
use crate::sets::knowledge::store::dump_files;
use crate::sets::parquet::open;
use crate::sets::parquet::records::Value;
use patterns::{because, source};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub struct ClassesFromText;
source!(ClassesFromText, "the table of word classes made from a copy of an encyclopedia in parquet files: the plain text of every article of the first files of the copy, in the order of their names, is read twice, once to count the words and once to count the words beside each frequent word; the classes come from those counts, and the table holds a word, its class and its count on each line, the words by frequency, beside a file that lists the first words of each class for a person to read");

const ARTICLE_TEXT: &str = "text";
because!(ARTICLE_TEXT, ClassesFromText, "the column of the plain text of an article in a file of the copy");

const FILES_OPTION: &str = "files";
because!(FILES_OPTION, ClassesFromText, "the option with the number of the first files of the copy whose text is read; with none, every file");

const FILE_BYTES: usize = 100_000_000;
because!(FILE_BYTES, ClassesFromText, "the bytes of text read from each file when a run names no number: a part of every file, so that the text is spread over the whole encyclopedia, and over all files some billions of signs, enough to count the places of forty thousand words with little memory");

const BYTES_OPTION: &str = "bytes";
because!(BYTES_OPTION, ClassesFromText, "the option that sets how much text is read from each file, so that a run can take a part of every file and spread its text over the whole encyclopedia");

const CLASSES_OPTION: &str = "classes";
because!(CLASSES_OPTION, ClassesFromText, "the option that sets how many groups the words are cut into, for a finer or a coarser table");

const MADE_CLASSES: usize = 1_000;
because!(MADE_CLASSES, ClassesFromText, "the number of classes when a run names none: fine enough to keep names, places and kinds of things apart, with about forty words in a class");

const LANES_OPTION: &str = "lanes";
because!(LANES_OPTION, ClassesFromText, "the option that sets how many threads read the files and count, for a machine that is shared; the table is the same for every number");

const CLASS_OPTION_MARK: char = '=';
because!(CLASS_OPTION_MARK, ClassesFromText, "the mark between the name and the value of an option");

const TABLE_NAME: &str = "classes_";
because!(TABLE_NAME, ClassesFromText, "the start of the name of the table, before the number of classes");

const READ_NAME: &str = "_read";
because!(READ_NAME, ClassesFromText, "the part of the name of the file for a person, after the number of classes");

const TABLE_END: &str = ".txt";
because!(TABLE_END, ClassesFromText, "the end of the names of both files");

const SHOWN_WORDS: usize = 40;
because!(SHOWN_WORDS, ClassesFromText, "the first words of a class that the file for a person shows");

const FIELD: char = '\t';
because!(FIELD, ClassesFromText, "the mark between the word, its class and its count in a line of the table");

const SHOWN_SECONDS: usize = 1;
because!(SHOWN_SECONDS, ClassesFromText, "the places after the point of a printed time in seconds");

fn file_texts(place: &Path, most: usize, each: &mut dyn FnMut(&str)) -> Result<(), String> {
    let mut table = open(place)?;
    let mut read = 0usize;
    for group in 0..table.groups() {
        if read >= most {
            break;
        }
        let columns = table.read_group(group, &[ARTICLE_TEXT])?;
        let Some(texts) = columns.into_iter().next() else { return Err(format!("{}: a row group with no texts", place.display())) };
        for text in texts {
            if read >= most {
                break;
            }
            let Value::Text(text) = text else { return Err(format!("{}: an article with no text", place.display())) };
            read += text.len();
            each(&text);
        }
    }
    Ok(())
}
because!(file_texts, ClassesFromText, "the plain text of the articles of one file of the copy, in the order of the file, whole articles until the bytes of text read reach the most that the run asks for");

fn file_counts(place: &Path, most: usize) -> Result<HashMap<String, u64>, String> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    file_texts(place, most, &mut |text| count_tokens(text, &mut counts))?;
    Ok(counts)
}
because!(file_counts, ClassesFromText, "the count of each token of the text of one file");

fn file_codes(place: &Path, most: usize, numbers: &HashMap<String, u32>) -> Result<Vec<u32>, String> {
    let mut codes: Vec<u32> = Vec::new();
    file_texts(place, most, &mut |text| word_codes(text, numbers, &mut codes))?;
    Ok(codes)
}
because!(file_codes, ClassesFromText, "the codes of the tokens of the text of one file, line by line");

fn table_lines(words: &[(String, u64)], classes: &[Option<usize>]) -> String {
    let mut text = String::new();
    for ((word, count), class) in words.iter().zip(classes) {
        if let Some(class) = class {
            text.push_str(&format!("{word}{FIELD}{class}{FIELD}{count}\n"));
        }
    }
    text
}
because!(table_lines, ClassesFromText, "the table: for each word with a class, the word, its class and its count, the words by frequency");

fn class_list_lines(words: &[(String, u64)], classes: &[Option<usize>]) -> String {
    let mut members: Vec<(usize, Vec<&str>)> = Vec::new();
    let mut place: HashMap<usize, usize> = HashMap::new();
    for ((word, _), class) in words.iter().zip(classes) {
        let Some(class) = class else { continue };
        let at = *place.entry(*class).or_insert_with(|| {
            members.push((*class, Vec::new()));
            members.len() - 1
        });
        members[at].1.push(word);
    }
    members.sort_by(|left, right| right.1.len().cmp(&left.1.len()));
    let mut text = String::new();
    for (class, words) in &members {
        let shown: Vec<&str> = words.iter().take(SHOWN_WORDS).copied().collect();
        text.push_str(&format!("class {class} ({} words): {}\n", words.len(), shown.join(" ")));
    }
    text
}
because!(class_list_lines, ClassesFromText, "the file for a person: each class with its number of words and its first words by frequency, the largest class first and classes of equal size in the order of their most frequent word");

fn written(place: &Path, text: &str) -> Result<(), String> {
    let fault = |error: std::io::Error| format!("{}: {error}", place.display());
    let mut out = BufWriter::new(std::fs::File::create(place).map_err(fault)?);
    out.write_all(text.as_bytes()).map_err(fault)?;
    out.flush().map_err(fault)
}
because!(written, ClassesFromText, "a file written whole, or the fault that stopped it");

pub fn make_classes(copy: &Path, folder: &Path, files: usize, most: usize, classes: usize, lanes: usize) -> Result<Vec<String>, String> {
    let started = Instant::now();
    let mut places = dump_files(copy)?;
    places.truncate(files);
    let counted: Vec<Result<HashMap<String, u64>, String>> = on_lanes(places.len(), lanes, &|file| file_counts(&places[file], most));
    let mut all: HashMap<String, u64> = HashMap::new();
    for counts in counted {
        for (word, count) in counts? {
            *all.entry(word).or_default() += count;
        }
    }
    let tokens: u64 = all.values().sum();
    let different = all.len();
    let words = frequent_words(all, COUNTED_WORDS);
    let numbers: HashMap<String, u32> = words.iter().enumerate().filter_map(|(at, (word, _))| u32::try_from(at).ok().map(|at| (word.clone(), at))).collect();
    let counted_at = started.elapsed().as_secs_f64();
    let coded: Vec<Result<Vec<u32>, String>> = on_lanes(places.len(), lanes, &|file| file_codes(&places[file], most, &numbers));
    let streams: Vec<Vec<u32>> = coded.into_iter().collect::<Result<Vec<Vec<u32>>, String>>()?;
    let context = CONTEXT_WORDS.min(words.len());
    let rows = all_rows(&streams, words.len(), context, lanes);
    drop(streams);
    let rows_at = started.elapsed().as_secs_f64();
    let classing = neighbour_classes(&rows, row_width(context), classes, MOST_RUNS, lanes);
    let classed = classing.classes.iter().filter(|class| class.is_some()).count();
    std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    let table = folder.join(format!("{TABLE_NAME}{classes}{TABLE_END}"));
    written(&table, &table_lines(&words, &classing.classes))?;
    written(&folder.join(format!("{TABLE_NAME}{classes}{READ_NAME}{TABLE_END}")), &class_list_lines(&words, &classing.classes))?;
    Ok(vec![
        format!("{} files of the copy: {tokens} tokens, {different} different; rows of the {} most frequent words over {context} context words", places.len(), words.len()),
        format!("{classed} words in {classes} classes after {} rounds: {}", classing.runs, table.display()),
        format!("seconds: counts {counted_at:.*}, rows {rows_at:.*}, all {:.*}", SHOWN_SECONDS, SHOWN_SECONDS, SHOWN_SECONDS, started.elapsed().as_secs_f64()),
    ])
}
because!(make_classes, ClassesFromText, "the table of word classes from the first files of a copy: the counts of the tokens of each file on its own thread, summed in the order of the files; the codes of each file on its own thread; the rows of the counted words; the classes; the two files in the folder; the lines that report the run");

pub fn classes_command(rest: &[String]) -> ExitCode {
    let (named, plain): (Vec<&String>, Vec<&String>) = rest.iter().partition(|argument| argument.contains(CLASS_OPTION_MARK));
    let option = |name: &str| named.iter().find_map(|argument| argument.split_once(CLASS_OPTION_MARK).filter(|(key, _)| *key == name).and_then(|(_, value)| value.parse::<usize>().ok()));
    if let Some(bad) = named.iter().filter_map(|argument| argument.split_once(CLASS_OPTION_MARK)).find(|(name, value)| [FILES_OPTION, BYTES_OPTION, CLASSES_OPTION, LANES_OPTION].contains(name) && value.parse::<usize>().is_err()) {
        eprintln!("the option {} takes a whole number, not {}", bad.0, bad.1);
        return ExitCode::FAILURE;
    }
    let [copy, folder] = plain.as_slice() else {
        eprintln!("usage: ilm classes <folder of the copy of the encyclopedia> <folder of the table> [{FILES_OPTION}=<n>] [{BYTES_OPTION}=<n>] [{CLASSES_OPTION}=<n>] [{LANES_OPTION}=<n>]");
        return ExitCode::FAILURE;
    };
    let lanes = option(LANES_OPTION).filter(|lanes| *lanes > 0).unwrap_or_else(hardware_lanes);
    match make_classes(Path::new(copy.as_str()), Path::new(folder.as_str()), option(FILES_OPTION).unwrap_or(usize::MAX), option(BYTES_OPTION).unwrap_or(FILE_BYTES), option(CLASSES_OPTION).filter(|classes| *classes > 0).unwrap_or(MADE_CLASSES), lanes) {
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
because!(classes_command, ClassesFromText, "the table of word classes written from a copy of an encyclopedia into a folder, with the lines that report the run");
