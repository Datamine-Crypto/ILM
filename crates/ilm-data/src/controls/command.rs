use crate::controls::file::{ChangedCase, write_changes};
use crate::controls::arith::arithmetic_changes;
use crate::controls::chain::chain_changes;
use crate::controls::deduction::deduction_changes;
use crate::controls::flows::flow_changes;
use crate::controls::held::{held_changes, held_from_all};
use crate::controls::list::list_changes;
use crate::controls::loops::loop_changes;
use crate::controls::lonli::lonli_changes;
use crate::controls::more::more_changes;
use crate::controls::numbers::number_changes;
use crate::controls::parts::parts_changes;
use crate::controls::pieces::piece_changes;
use crate::controls::requests::request_changes;
use crate::controls::ruletaker::ruletaker_changes;
use crate::controls::support::support_changes;
use crate::controls::swaps::swap_changes;
use crate::controls::things::thing_changes;
use crate::controls::tomi::{tomi_changes, tomqa_changes};
use crate::controls::walk::walk_changes;
use ilm_core::choice::{picked, picked_labels};
use ilm_core::split::parts;
use ilm_core::cases::{Case, head_value};
use crate::sets::folder::{file_sizes, load_file};
use patterns::{because, source};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub struct ChangeCommand;
source!(ChangeCommand, "one part of the program for every control of changed cases: the control is named, the case file of a kind is read and cut as the measure cuts it, and the changed cases of its lessons are written as a file that the control of the program reads; so every file of changes can be made again with one line");

const CONTROL_REQUESTS: &str = "requests_control";
because!(CONTROL_REQUESTS, ChangeCommand, "the name of the control of changed requests, in the command and in the name of its files");


const CONTROL_SWAPS: &str = "swaps_control";
because!(CONTROL_SWAPS, ChangeCommand, "the name of the control of changed stories of swaps");

const CONTROL_ARITH: &str = "arith_control";
because!(CONTROL_ARITH, ChangeCommand, "the name of the control of changed sums with choices");

const CONTROL_WALK: &str = "walk_control";
because!(CONTROL_WALK, ChangeCommand, "the name of the control of changed stories of steps");

const CONTROL_LIST: &str = "list_control";
because!(CONTROL_LIST, ChangeCommand, "the name of the control of changed counts over a told list");

const CONTROL_MORE: &str = "more_controls";
because!(CONTROL_MORE, ChangeCommand, "the name of the control of changed cases of the kinds that a tool counts and that have no control of their own");

const CONTROL_PARTS: &str = "parts_control";
because!(CONTROL_PARTS, ChangeCommand, "the name of the control of changed texts of the families of rows");

const CONTROL_NUMBERS: &str = "numbers_control";
because!(CONTROL_NUMBERS, ChangeCommand, "the name of the control of changed problems that tell numbers and ask for a number");

const CONTROL_LONLI: &str = "lonli_control";
because!(CONTROL_LONLI, ChangeCommand, "the name of the control of the groups of a set of fixed forms whose cases are read as they are");

const CONTROL_TOMI: &str = "tomi_control";
because!(CONTROL_TOMI, ChangeCommand, "the name of the control of changed stories of a thing that is placed and moved");

const OPTION_PART: &str = "part=";
because!(OPTION_PART, ChangeCommand, "the option that names the part whose cases a control of groups writes: the read part for a model of all lessons, or else the lessons that the model does not learn");

const PART_READ: &str = "read";
because!(PART_READ, ChangeCommand, "the value of the option of the part that names the read part");

const CONTROL_SUPPORT: &str = "babi_support";
because!(CONTROL_SUPPORT, ChangeCommand, "the name of the control of stories of the twenty tasks with a supporting sentence taken out");

const OPTION_SUPPORT: &str = "support=";
because!(OPTION_SUPPORT, ChangeCommand, "the option that names the folder of the training files of the first form of the twenty tasks, which name the supporting sentences");

const CONTROL_CHAIN: &str = "chain_control";
because!(CONTROL_CHAIN, ChangeCommand, "the name of the control of stories of a chain with a needed link taken out, whose changes are always made from the lessons that the model does not learn");

const CONTROL_DEDUCTION: &str = "deduction_control";
because!(CONTROL_DEDUCTION, ChangeCommand, "the name of the control of theories of told facts and rules with a needed unit taken out");

const CONTROL_TOMQA: &str = "tomqa_control";
because!(CONTROL_TOMQA, ChangeCommand, "the name of the control of changed stories of the belief set, where a person who did not see the move now sees it");
const CONTROL_HELD: &str = "held_control";
because!(CONTROL_HELD, ChangeCommand, "the name of the control of changed texts of the tables used one after the other, whose changes are always made from all lessons, since the tables stand whole only there");

const CONTROL_RULETAKER: &str = "ruletaker_control";
because!(CONTROL_RULETAKER, ChangeCommand, "the name of the control of theories of a set with no formal forms, read by tables counted from another set");

const CONTROL_THINGS: &str = "things_control";
because!(CONTROL_THINGS, ChangeCommand, "the name of the control of changed stories of a choice among things, where two things change names");

const CONTROL_LOOPS: &str = "loops_control";
because!(CONTROL_LOOPS, ChangeCommand, "the name of the control of changed graphs of a question about a loop, where an edge is taken out or added");

const CONTROL_FLOWS: &str = "flows_control";
because!(CONTROL_FLOWS, ChangeCommand, "the name of the control of changed graphs of a question about the largest flow, where an edge is taken out");

const CONTROL_PIECES: &str = "pieces_control";
because!(CONTROL_PIECES, ChangeCommand, "the name of the control of changed stories of things in places, where two pieces of each told list change places");

const OPTION_FORMS: &str = "forms=";
because!(OPTION_FORMS, ChangeCommand, "the option that names the description of the set whose train files give the pairs of sentences and formal forms");

const OPTION_ABOUT: &str = "about=";
because!(OPTION_ABOUT, ChangeCommand, "the option that names the description of the set of a kind, whose raw files a control reads beside the case file");

const CONTROL_NAMES: &[&str] = &[CONTROL_REQUESTS, CONTROL_SWAPS, CONTROL_ARITH, CONTROL_WALK, CONTROL_LIST, CONTROL_MORE, CONTROL_PARTS, CONTROL_NUMBERS, CONTROL_LONLI, CONTROL_TOMI, CONTROL_TOMQA, CONTROL_SUPPORT, CONTROL_CHAIN, CONTROL_DEDUCTION, CONTROL_HELD, CONTROL_RULETAKER, CONTROL_THINGS, CONTROL_LOOPS, CONTROL_FLOWS, CONTROL_PIECES];
because!(CONTROL_NAMES, ChangeCommand, "the controls that the part knows, for the line that says how it is used");

const CONTROL_BETWEEN: &str = "|";
because!(CONTROL_BETWEEN, ChangeCommand, "the mark between the names of the controls in the line that says how the part is used");

const OPTION_MARK: char = '=';
because!(OPTION_MARK, ChangeCommand, "the mark between the name of an option and its value; a word without it names a control or a file");

const OPTION_WHOLE: &str = "whole=";
because!(OPTION_WHOLE, ChangeCommand, "the option that makes the changes from all lessons, for a model of all lessons, in place of the part that is held back");

const OPTION_ROOT: &str = "root=";
because!(OPTION_ROOT, ChangeCommand, "the option that names the folder under which the files of changes are written, one folder for each set");

const ROOT_USUAL: &str = "data/port/controls";
because!(ROOT_USUAL, ChangeCommand, "the folder of the files of changes that the control of the program reads when nothing else is named");

const HEAD_KIND: &str = "kind";
because!(HEAD_KIND, ChangeCommand, "the named value of the first line of a case file that gives its kind");

const HEAD_SOURCE: &str = "source";
because!(HEAD_SOURCE, ChangeCommand, "the named value of the first line of a case file that gives its set");

const CHANGE_WORDS_END: &str = ".words.txt";
because!(CHANGE_WORDS_END, ChangeCommand, "the end of the name of the file of new words that stands beside the case file of a kind");

const TEXT_END: &str = ".txt";
because!(TEXT_END, ChangeCommand, "the end of the name of a case file");

const WORDS_FIELD: char = '\t';
because!(WORDS_FIELD, ChangeCommand, "the mark between a word and its new word in a file of new words");

fn change_usage() -> ExitCode {
    eprintln!("usage: ilm changes {} <case file of a kind> [{OPTION_WHOLE}<nought or one>] [{OPTION_ROOT}<folder>] [{OPTION_PART}{PART_READ}] [{OPTION_SUPPORT}<folder>] [{OPTION_FORMS}<description>] [{OPTION_ABOUT}<description>]", CONTROL_NAMES.join(CONTROL_BETWEEN));
    eprintln!("writes the changed cases of the lessons of a kind as the file <folder>/<set>/<kind>.<control>.txt, which `ilm control` reads");
    ExitCode::FAILURE
}
because!(change_usage, ChangeCommand, "what the part prints when its words name no control that it knows");

fn change_words(path: &Path) -> HashMap<String, String> {
    let text = path.to_string_lossy();
    let words_path = PathBuf::from(format!("{}{CHANGE_WORDS_END}", text.strip_suffix(TEXT_END).unwrap_or(&text)));
    std::fs::read_to_string(words_path).map(|found| found.lines().filter_map(|line| line.split_once(WORDS_FIELD)).map(|(old, new)| (old.to_string(), new.to_string())).collect()).unwrap_or_default()
}
because!(change_words, ChangeCommand, "the table of new words of a kind from the file beside its case file, and no word for a kind that has none");

struct ChangeInput {
    lessons: Vec<Case>,
    labels: Vec<Vec<String>>,
    read: Vec<Case>,
    kind: String,
    set: String,
}
because!(ChangeInput, ChangeCommand, "what a control takes from a case file: the learned part of the kind with the label lines of its cases and the read part, cut as the measure cuts them, and the names of the kind and of its set");

fn change_lessons(path: &Path) -> std::io::Result<ChangeInput> {
    let loaded = load_file(path)?;
    let (fit, held) = file_sizes(path)?;
    let all = loaded.kind.cases.len();
    let cut = parts(&loaded.kind.cases, fit.unwrap_or(all), held.unwrap_or(all), loaded.start, loaded.stories);
    let kind = head_value(&loaded.kind, HEAD_KIND).unwrap_or_default().to_string();
    let set = head_value(&loaded.kind, HEAD_SOURCE).unwrap_or_default().to_string();
    Ok(ChangeInput { lessons: picked(&loaded.kind.cases, &cut.learn), labels: picked_labels(&loaded.kind.labels, &cut.learn), read: picked(&loaded.kind.cases, &cut.read), kind, set })
}
because!(change_lessons, ChangeCommand, "the parts of the kind of a case file that the controls take");

pub fn changes_command(args: &[String]) -> ExitCode {
    let plain: Vec<&String> = args.iter().filter(|arg| !arg.contains(OPTION_MARK)).collect();
    let option = |name: &str| args.iter().find_map(|arg| arg.strip_prefix(name));
    let whole = option(OPTION_WHOLE).is_some_and(|value| value.parse::<u8>().is_ok_and(|on| on == 1));
    let root = PathBuf::from(option(OPTION_ROOT).unwrap_or(ROOT_USUAL));
    let [control, file] = plain.as_slice() else { return change_usage() };
    let path = Path::new(file.as_str());
    let read_part = option(OPTION_PART) == Some(PART_READ);
    let ChangeInput { lessons, labels, read, kind, set } = match change_lessons(path) {
        Ok(found) => found,
        Err(error) => {
            eprintln!("{file}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let made: Result<Vec<ChangedCase>, String> = match control.as_str() {
        CONTROL_REQUESTS => Ok(request_changes(&lessons, whole, &change_words(path))),
        CONTROL_SWAPS => Ok(swap_changes(&lessons, whole)),
        CONTROL_ARITH => Ok(arithmetic_changes(&lessons, whole, &kind)),
        CONTROL_WALK => walk_changes(&lessons, whole),
        CONTROL_LIST => Ok(list_changes(&lessons, whole)),
        CONTROL_MORE => more_changes(&lessons, whole, &kind),
        CONTROL_PARTS => parts_changes(&lessons, whole, &kind),
        CONTROL_NUMBERS => Ok(number_changes(&lessons, whole)),
        CONTROL_LONLI => Ok(lonli_changes(&lessons, &read, read_part)),
        CONTROL_TOMI => Ok(tomi_changes(&lessons, whole, &kind)),
        CONTROL_TOMQA => Ok(tomqa_changes(&lessons, whole)),
        CONTROL_CHAIN => Ok(chain_changes(&lessons, &set)),
        CONTROL_DEDUCTION => Ok(deduction_changes(&lessons, &labels, whole)),
        CONTROL_HELD => held_changes(&lessons, &set, &kind, option(OPTION_ABOUT).map(Path::new)),
        CONTROL_RULETAKER => ruletaker_changes(&lessons, whole, option(OPTION_FORMS).map(Path::new)),
        CONTROL_SUPPORT => support_changes(&lessons, whole, &kind, option(OPTION_SUPPORT).map(Path::new)),
        CONTROL_THINGS => Ok(thing_changes(&lessons, whole)),
        CONTROL_LOOPS => Ok(loop_changes(&lessons, whole)),
        CONTROL_FLOWS => Ok(flow_changes(&lessons, whole)),
        CONTROL_PIECES => Ok(piece_changes(&lessons, whole)),
        _ => return change_usage(),
    };
    let changes = match made {
        Ok(changes) => changes,
        Err(error) => {
            eprintln!("{kind}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let whole = match control.as_str() {
        CONTROL_LONLI => read_part,
        CONTROL_CHAIN => false,
        CONTROL_HELD => held_from_all(&set),
        _ => whole,
    };
    match write_changes(&root, control, &set, &kind, &changes, whole) {
        Ok(written) => {
            println!("{kind}: {} changed cases written to {}", changes.len(), written.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: {error}", root.display());
            ExitCode::FAILURE
        }
    }
}
because!(changes_command, ChangeCommand, "the part of the program that writes the changed cases of one control for one kind");
