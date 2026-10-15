use ilm_core::base::lanes::{hardware_lanes, on_lanes};
use crate::sets::made::family::{Family, MadeCase};
use crate::sets::made::texts::{MADE_LESSONS_COUNT, MADE_READ_COUNT};
use crate::sets::made::{MADE_DEEP, MADE_PREFIX, made_family};
use patterns::{because, source};
use std::path::Path;
use std::process::ExitCode;

pub struct MadeFiles;
source!(MadeFiles, "the families as a set of plain files that the measure reads as it reads a public set: one case file for each kind, a file that says what the set gives beside its cases, and the cut of each kind; the cases of a kind are its lessons and then its read part, and no case tells a sentence");

const MADE_SOURCE: &str = "src_made";
because!(MADE_SOURCE, MadeFiles, "the name of the set of the families in the files of the measure");

const MADE_SET_NAME: &str = "families of texts that a program makes";
because!(MADE_SET_NAME, MadeFiles, "the name of the set in the file that says what the set gives");

const MADE_BY: &str = "a program";
because!(MADE_BY, MadeFiles, "who wrote the text of the set: a program, not people");

const MADE_READ_DEPTHS: (usize, usize) = (1, 4);
because!(MADE_READ_DEPTHS, MadeFiles, "the least and the most functions inside each other of a text of the read part, in the words that name a kind");

const MADE_DEEP_DEPTHS: (usize, usize) = (5, 6);
because!(MADE_DEEP_DEPTHS, MadeFiles, "the depths of a deep text, in the words that name a deep kind");

const FILE_SPLIT: &str = "split.txt";
because!(FILE_SPLIT, MadeFiles, "the file with the cut of each kind into its parts");

const FILE_SOURCE: &str = "source.txt";
because!(FILE_SOURCE, MadeFiles, "the file that says what the set gives beside its cases");

const FILE_END: &str = ".txt";
because!(FILE_END, MadeFiles, "the end of the name of the case file of a kind");

const MADE_FIELD: &str = "\t";
because!(MADE_FIELD, MadeFiles, "the mark between the fields of a line of the files of the set");

const MADE_LINE: &str = "\n";
because!(MADE_LINE, MadeFiles, "the mark that ends each line of the files of the set, the last one too");

const MADE_OPTION_MARK: char = '=';
because!(MADE_OPTION_MARK, MadeFiles, "the mark between the name of an option and its value; a word without it names the folder");

const OPTION_FAMILIES: &str = "families=";
because!(OPTION_FAMILIES, MadeFiles, "the option that says how many families the files hold");

const OPTION_HELD: &str = "held=";
because!(OPTION_HELD, MadeFiles, "the option that opens the held families, for their one test");

fn made_names(number: usize) -> (String, String) {
    (format!("{MADE_PREFIX}{number}"), format!("{MADE_PREFIX}{number}{MADE_DEEP}"))
}
because!(made_names, MadeFiles, "the names of the two kinds of a family: the kind with the read part and the kind with the deep read part");

fn made_case_file(family: &Family, kind: &str, deep: bool) -> String {
    let read: &[MadeCase] = if deep { &family.deep } else { &family.read };
    let cases = family.lessons.len() + read.len();
    let head = [format!("#{MADE_FIELD}kind={kind}"), format!("source={MADE_SOURCE}"), format!("cases={cases}"), format!("fit={MADE_LESSONS_COUNT}"), format!("held={MADE_READ_COUNT}"), format!("fit_stories={}", u8::from(false)), format!("read_from={}", family.lessons.len())];
    let mut lines: Vec<String> = vec![head.join(MADE_FIELD)];
    for case in family.lessons.iter().chain(read) {
        lines.push(format!("Q{MADE_FIELD}{}", case.text));
        lines.push(format!("A{MADE_FIELD}{}", case.answer));
        lines.push(String::new());
    }
    let mut text = lines.join(MADE_LINE);
    text.push_str(MADE_LINE);
    text
}
because!(made_case_file, MadeFiles, "the case file of a kind of a family: the first line with its named values, then each case with its question, its answer and an empty line");

pub fn made_files(folder: &Path, count: usize, held_open: bool, lanes: usize) -> Result<usize, String> {
    let families: Vec<Result<Family, String>> = on_lanes(count, lanes, &|number| made_family(number, held_open));
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    let mut kinds: Vec<(String, String)> = Vec::new();
    let mut deep_kinds: Vec<(String, String)> = Vec::new();
    for (number, family) in families.iter().enumerate() {
        let family = family.as_ref().map_err(String::clone)?;
        let (kind, deep) = made_names(number);
        std::fs::write(folder.join(format!("{kind}{FILE_END}")), made_case_file(family, &kind, false)).map_err(|error| error.to_string())?;
        std::fs::write(folder.join(format!("{deep}{FILE_END}")), made_case_file(family, &deep, true)).map_err(|error| error.to_string())?;
        kinds.push((kind, format!("the family of seed {number}: texts with {} to {} functions inside each other", MADE_READ_DEPTHS.0, MADE_READ_DEPTHS.1)));
        deep_kinds.push((deep, format!("the family of seed {number}: the same lessons, texts with {} or {} functions inside each other", MADE_DEEP_DEPTHS.0, MADE_DEEP_DEPTHS.1)));
    }
    let every: Vec<&(String, String)> = kinds.iter().chain(&deep_kinds).collect();
    let mut about: Vec<String> = vec![format!("set{MADE_FIELD}{MADE_SET_NAME}"), format!("by{MADE_FIELD}{MADE_BY}"), format!("fit{MADE_FIELD}{MADE_LESSONS_COUNT}"), format!("held{MADE_FIELD}{MADE_READ_COUNT}")];
    about.extend(every.iter().map(|(kind, what)| format!("kind{MADE_FIELD}{kind}{MADE_FIELD}{what}")));
    about.extend(every.iter().map(|(kind, _)| format!("task{MADE_FIELD}{kind}{MADE_FIELD}{kind}")));
    about.extend(every.iter().map(|(kind, _)| format!("all{MADE_FIELD}{kind}")));
    std::fs::write(folder.join(FILE_SOURCE), about.join(MADE_LINE) + MADE_LINE).map_err(|error| error.to_string())?;
    let split: Vec<String> = every.iter().map(|(kind, _)| format!("{kind}{MADE_FIELD}{}{MADE_FIELD}{MADE_LESSONS_COUNT}{MADE_FIELD}{MADE_READ_COUNT}{MADE_FIELD}{}", MADE_LESSONS_COUNT + MADE_READ_COUNT, usize::default())).collect();
    std::fs::write(folder.join(FILE_SPLIT), split.join(MADE_LINE) + MADE_LINE).map_err(|error| error.to_string())?;
    Ok(every.len())
}
because!(made_files, MadeFiles, "the files of the families of the first seeds in a folder: the families are made side by side on the threads, the files are written in the order of the seeds, the kinds with a read part first and the deep kinds after them");

pub fn made_command(args: &[String]) -> ExitCode {
    let option = |name: &str| args.iter().find_map(|arg| arg.strip_prefix(name));
    let count = option(OPTION_FAMILIES).and_then(|value| value.parse::<usize>().ok()).unwrap_or(crate::sets::made::MADE_FAMILIES);
    let held_open = option(OPTION_HELD).is_some_and(|value| value.parse::<u8>().is_ok_and(|on| on == 1));
    let Some(folder) = args.iter().find(|arg| !arg.contains(MADE_OPTION_MARK)) else {
        eprintln!("usage: ilm made <folder> [{OPTION_FAMILIES}<count>] [{OPTION_HELD}<nought or one>]");
        return ExitCode::FAILURE;
    };
    match made_files(Path::new(folder), count, held_open, hardware_lanes()) {
        Ok(kinds) => {
            println!("{kinds} kinds of {count} families written to {folder}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
because!(made_command, MadeFiles, "the part of the program that writes the files of the families of the first seeds");
