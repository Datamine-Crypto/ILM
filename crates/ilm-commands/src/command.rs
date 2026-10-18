use ilm_eval::measure::board::{board, compare_boards, earlier_lessons, final_part, grown_jobs, grown_ways, known_times};
use ilm_core::cases::{Case, read_kind};
use ilm_core::classes::{WordClasses, read_classes};
use ilm_core::choice::{inner_parts, picked, picked_labels, take};
use ilm_eval::measure::control::{changed_cases, control};
use ilm_core::base::digest::Sha256;
use ilm_core::base::lanes::{hardware_lanes, on_lanes};
use ilm_core::tools::explain::story_disk::{close_story_disk, open_story_disk};
use ilm_core::base::shares::PERCENT;
use ilm_core::tools::call::{Learner, Lessons, Said, Tool};
use ilm_eval::measure::kind::{ABOVE, Asked, Ground, LEARNED, Row, learned, line, measure, table_head};
use ilm_data::sets::folder::{Source, file_sizes, load, load_file, new_words, read_source};
use ilm_core::split::parts;
use ilm_core::tools::{all_tools, named_tools, tool_names};
use ilm_core::base::words::{answer_words, words};
use patterns::{because, source};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub struct LearnerCommand;
source!(LearnerCommand, "one program for the whole system: each part of the work is one word after the name of the program, so that every result can be made again with one line and no part needs a setup of its own");

const BETWEEN_WORDS: &str = " ";
because!(BETWEEN_WORDS, LearnerCommand, "the mark between the words of a line that the program prints, and between the words of an answer in a file of answers");

const OPTION_NAMED: char = '=';
because!(OPTION_NAMED, LearnerCommand, "the mark between the name and the value of an option of a run");

const BETWEEN_KINDS: char = ',';
because!(BETWEEN_KINDS, LearnerCommand, "the mark between the names of kinds or of sets in a list that a run gives");

const FIT_OPTION: &str = "fit";
because!(FIT_OPTION, LearnerCommand, "the option with the most lessons of a kind");

const HELD_OPTION: &str = "held";
because!(HELD_OPTION, LearnerCommand, "the option with the most read cases of a kind");

const CURVE_OPTION: &str = "curve";
because!(CURVE_OPTION, LearnerCommand, "the option that asks for the share right by the number of lessons");

const WORDS_OPTION: &str = "newwords";
because!(WORDS_OPTION, LearnerCommand, "the option that asks for the read cases again with new words");

const SHOW_OPTION: &str = "show";
because!(SHOW_OPTION, LearnerCommand, "the option with the number of wrong lesson cases to print");

const SMALL_OPTION: &str = "small";
because!(SMALL_OPTION, LearnerCommand, "the option that reads the read part of a kind whatever its size, as the official test split of a benchmark is read whole however few cases a kind of it has");

const SEALED_OPTION: &str = "sealed";
because!(SEALED_OPTION, LearnerCommand, "the option that reads the final part in place of the read part: for the one final run with frozen code, never while a tool is designed");

const TOOL_OPTION: &str = "tool";
because!(TOOL_OPTION, LearnerCommand, "the option that names one tool to measure alone in place of the choice among all");

const TOOLS_OPTION: &str = "tools";
because!(TOPIC_OPTION, LearnerCommand, "the option that names the text of the library whose rules a learned wording uses");
const TOPIC_OPTION: &str = "topic";

const RULES_OPTION: &str = "rules";
because!(RULES_OPTION, LearnerCommand, "the option that asks a learned wording to learn the rules of its concept first, from the facts of the stories and the facts that answer the lessons");

const TOLD_OPTION: &str = "told";
because!(TOLD_OPTION, LearnerCommand, "the option that asks a learned wording to learn the rules that read the told sentences too");
because!(TOOLS_OPTION, LearnerCommand, "the option that names the tools of the choice, for a run with a part of them");

const LANES_OPTION: &str = "lanes";
because!(LANES_OPTION, LearnerCommand, "the option with the number of threads of the run");

const OUT_OPTION: &str = "out";
because!(OUT_OPTION, LearnerCommand, "the option with the file that the board is also written to");

const JOBS_OPTION: &str = "jobs";
because!(JOBS_OPTION, LearnerCommand, "the option with the file that the times of the jobs of the board are written to, to see which jobs hold the whole run");

const GROWN_LINE: &str = "jobs whose ways grew against the order file: ";
because!(GROWN_LINE, LearnerCommand, "the words before the jobs of a board whose ways of learning grew against the order file, printed after the board and not written into it, as the order file is a run of another day");

const GROWN_BETWEEN: &str = "; ";
because!(GROWN_BETWEEN, LearnerCommand, "the marks between the jobs whose ways grew, in the line after the board");

const ORDER_OPTION: &str = "order";
because!(ORDER_OPTION, LearnerCommand, "the option with a file of the times of the jobs that an earlier board wrote: the board starts the jobs that took longest there first, so that no long job starts near the end");

const KEPT_OPTION: &str = "kept";
because!(KEPT_OPTION, LearnerCommand, "the option with the folder of the store that lasts between runs: a board or a curriculum with it reads back the chains of an earlier run whose engine is the same, and what the learners discovered in an earlier run of the same core, and writes the new ones, so that nothing is made twice while nothing that it rests on changed; a board without it reads and writes no such file, as a timed run must");

const BOARD_KEYS: &[&str] = &[FIT_OPTION, TOOLS_OPTION, TOOL_OPTION, CLASSES_OPTION, LANES_OPTION, OUT_OPTION, JOBS_OPTION, ORDER_OPTION, SEALED_OPTION, KEPT_OPTION, ENGINE_OPTION, COMPARE_OPTION];
because!(BOARD_KEYS, LearnerCommand, "the options that the board reads itself; it refuses any other that is not an option of every command, so that a run never believes that an option it named was used");

pub const COMMAND_WIDE_OPTIONS: &[&str] = &[PATHS_OPTION];
because!(COMMAND_WIDE_OPTIONS, LearnerCommand, "the options that every command takes, read before the command runs: no command that refuses the options it does not read may refuse these");

pub const REFUSING_COMMANDS: &[(&str, &[&str])] = &[(BOARD_COMMAND, BOARD_KEYS)];
because!(REFUSING_COMMANDS, LearnerCommand, "the commands that refuse the options that they do not read, each with its own options");

pub fn refused_options(command: &str, named: &[&str]) -> Vec<String> {
    let Some((_, own)) = REFUSING_COMMANDS.iter().find(|(name, _)| *name == command) else { return Vec::new() };
    named.iter().filter(|key| !own.contains(key) && !COMMAND_WIDE_OPTIONS.contains(key)).map(|key| key.to_string()).collect()
}
because!(refused_options, LearnerCommand, "the options of a run that its command refuses: for a command that refuses the options that it does not read, those that are neither its own nor options of every command; none for a command that refuses none");

const BOARD_COMMAND: &str = "board";
because!(BOARD_COMMAND, LearnerCommand, "the name of the command of the board, by which the program runs it and the check of its options finds its own options");

const NOT_A_BOARD_KEY: &str = "the board does not use this option; its options are: ";
because!(NOT_A_BOARD_KEY, LearnerCommand, "what the board says of an option that it does not read, before the options that it reads");

const BETWEEN_KEYS: &str = ", ";
because!(BETWEEN_KEYS, LearnerCommand, "the marks between the names of the options that the board reads");

const COMPARE_OPTION: &str = "compare";
because!(COMPARE_OPTION, LearnerCommand, "the option of the board that names a reference board file: after its table the board prints what changed from the reference");

const ENGINE_OPTION: &str = "engine";
because!(ENGINE_OPTION, LearnerCommand, "the option with the fingerprint of the code of the engine that a script hashed from the tree it built; the program finds the fingerprint of its own engine from the sources of its build and refuses the store when the two differ, so that a run never reads chains that another engine made");

const ON: usize = 1;
because!(ON, LearnerCommand, "the value that turns an option on: the least number that is not nothing");

const SECONDS_PLACES: usize = 2;
because!(SECONDS_PLACES, LearnerCommand, "the places after the point of the seconds of a tool: a tool that needs a hundredth of a second is still told from one that needs none");

const CLASSES_OPTION: &str = "classes";
because!(CLASSES_OPTION, LearnerCommand, "the option with the file of a table of word classes; the program reads it once and gives it to every tool with its lessons");

const MOST_OPTION: &str = "most";
because!(MOST_OPTION, LearnerCommand, "the option with the most lessons that a control uses, in the order of its file");

const WHOLE_OPTION: &str = "whole";
because!(WHOLE_OPTION, LearnerCommand, "the option of a control whose changed cases were made from all lessons, for a model of all lessons as the board makes it");

const RIGHT_COUNTS: &str = "right answers of the read part, by kind:";
because!(RIGHT_COUNTS, LearnerCommand, "the words before the counts of right answers under the table of a measure: the table gives shares, and a report of a final read gives the counts");

const NO_FILE: &str = "the cases of the kind could not be read";
because!(NO_FILE, LearnerCommand, "the note of a kind of a set whose case file is not there or cannot be read");

const HELP_WORDS: &[&str] = &["help", "--help", "-h"];
because!(HELP_WORDS, LearnerCommand, "the words that ask for the forms of the commands: the plain word and the two forms that most programs take");

const VERSION_WORDS: &[&str] = &["version", "--version", "-V"];
because!(VERSION_WORDS, LearnerCommand, "the words that ask for the version of the program: the plain word and the two forms that most programs take");

pub const PROGRAM_VERSION: [u32; 3] = [2, 0, 0];
because!(PROGRAM_VERSION, LearnerCommand, "the major, minor and patch numbers of this release; a test checks them against the version that the workspace gives");

const VERSION_MARK: &str = ".";
because!(VERSION_MARK, LearnerCommand, "the sign between the numbers of a version");

pub fn version_text() -> String {
    PROGRAM_VERSION.map(|part| part.to_string()).join(VERSION_MARK)
}
because!(version_text, LearnerCommand, "the version of the release as its numbers joined by the sign of a version, as the workspace writes it");

const PROGRAM_NAME: &str = "ilm";
because!(PROGRAM_NAME, LearnerCommand, "the name of the program in the line of its version and in the line that names a word that is no command");

fn usage_lines() -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("usage: ilm board <folder of the sets> [set,set,...] [tools=<names>] [classes=<file>] [lanes=<n>] [out=<file>] [jobs=<file>] [order=<file>] [sealed={ON}] [kept=<folder> [engine=<fingerprint>]] [compare=<reference board file>] [fit=<n>, for every set, the controls still from the set's lessons]"));
    lines.push(format!("   or: ilm compare <reference board file> <board file>"));
    lines.push(format!("   or: ilm measure <folder of a set, or a case file> [kind,kind,...] [tool=<name>] [tools=<names>] [classes=<file>] [fit=<n>] [held=<n>] [sealed={ON}] [curve={ON}] [newwords={ON}] [show=<n>] [lanes=<n>] [small={ON}]"));
    lines.push(format!("   or: ilm choice <case file> [tools=<names>] [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm prove \"<told sentences> <question>\" [cases=<case file>] [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm places <text of the library, or a folder of them>..."));
    lines.push(format!("   or: ilm concepts [<folder of the sets>] [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm wording <case file> topic=<name of a text of the library> [told={ON}]"));
    lines.push(format!("   or: ilm store-lines <case file> topic=<name of a text of the library> [classes=<file>]"));
    lines.push(format!("   or: ilm facts <folder of a large text> topic=<name of a text of the library> kinds=<case file>[,<case file>] [out=<file>] [dump=<file>] [files=<n>] [classes=<file>] [fit=<n>] [lanes=<n>]"));
    lines.push(format!("   or: ilm joining <case file> [fit=<n>] [held=<n>]"));
    lines.push(format!("   or: ilm why <case file> [show=<n>] [all={ON}] [before=<case file>,...] [classes=<file>] [fit=<n>] [held=<n>] [lanes=<n>]"));
    lines.push(format!("   or: ilm meanings <case file> [fit=<n>] [held=<n>]"));
    lines.push(format!("   or: ilm explain-cost <case file> [show=<n>] [fit=<n>] [held=<n>] [lanes=<n>]"));
    lines.push(format!("   or: ilm control <case file of the kind> <file of changed cases> [tool=<name>] [whole={ON}] [most=<n>] [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm cases <case file>"));
    lines.push(format!("   or: ilm words <text>"));
    lines.push(format!("   or: ilm answer-words <text>"));
    lines.push(format!("   or: ilm fetch <list of sources> [sets=<set>,<set>] [store=<folder of the encyclopedia>] [folder=<folder of a set>] [check=yes]"));
    lines.push(format!("   or: ilm export <set> <description file> <folder of the case files>"));
    lines.push(format!("   or: ilm store <folder of the copy of the encyclopedia> <folder of the store> <description of the probe> <description of the questions> [others=<signs>] [lanes=<n>]"));
    lines.push(format!("   or: ilm classes <folder of the copy of the encyclopedia> <folder of the table> [files=<n>] [bytes=<n>] [classes=<n>] [lanes=<n>]"));
    lines.push(format!("   or: ilm changes <control> <case file of a kind> [whole=1] [root=<folder>] [part=read] [support=<folder>] [forms=<description>] [about=<description>]   (ilm changes alone prints the controls)"));
    lines.push(format!("   or: ilm parquet <file> [column,column,...]"));
    lines.push(format!("   or: ilm shared-text <folder of data>[,<folder of data>] [run=<tokens>] [words=<tokens of letters>] [only=<part of a path>]"));
    lines.push(format!("   or: ilm --version"));
    lines.push(format!("board: every kind of every set through the choice among the tools, and the table of the tasks"));
    lines.push(format!("compare: what changed from a reference board to a board: each changed field of a task, the tasks that are slower, the changed lines after the table"));
    lines.push(format!("measure: the kinds of one set with their controls; a model is made by the choice among the tools, or by one tool alone"));
    lines.push(format!("choice: the share of every tool on the lessons that it did not learn from, and the tool that is taken"));
    lines.push(format!("prove: the answer of the library to told sentences and a question (the last sentence), with its proof tree back to the told sentences; with a case file, the answer of the model that the explain tool learns from its lessons, with the rules that it discovered"));
    lines.push(format!("places: each line of the texts of the library with the open places that the reader takes in it, for an audit of names read as places"));
    lines.push(format!("concepts: the topics of the library of told logic, what each builds on and its number of sentences; with a folder of sets, the kinds where each topic answers most of its sample, with its share, and the tool that the choice takes there"));
    lines.push(format!("explain-cost: the rules of the library that took the most time when the explain tool learns from a case file and reads its read part"));
    lines.push(format!("facts: the facts of a text of the library found in a large text by shapes learned from seed facts, judged by the proofs of the lessons of the kinds, written as a learned text with the files and lines of their sentences"));
    lines.push(format!("joining: the ways of joining of a language of commands learned from the lessons of a case file, written as lines of the library, and the shares of the engine's reading of them"));
    lines.push(format!("why: for each lesson that a model of the explain tool answers wrong, the proof of the answer that it gave and the first step that the facts of the story miss for the wanted answer, then the most common missing steps"));
    lines.push(format!("control: a model reads changed forms of the lessons that it answers right; how often it gives the new true answer, where the truth turns and where it stays"));
    lines.push(format!("cases: the number of cases of a case file and the named values of its first line"));
    lines.push(format!("words: the words of a text as every learner counts them; answer-words: the words of an answer as the measure compares them"));
    lines.push(format!("fetch: the raw files of the sets from their public sources, each kept only with the right SHA-256"));
    lines.push(format!("export: the case files of a set from its raw files, as its description in data/sets names them"));
    lines.push(format!("store: the text store of the fact reader from a copy of an encyclopedia"));
    lines.push(format!("classes: the table of word classes from a copy of an encyclopedia"));
    lines.push(format!("changes: the changed cases of a kind, for a control"));
    lines.push(format!("parquet: the rows of a parquet file; shared-text: runs of tokens that a tracked file shares with a row of the data"));
    lines.push(format!("tools: {}", tool_names().join(", ")));
    lines
}
because!(usage_lines, LearnerCommand, "the forms of every command, what each does, and the names of the tools");

fn forms(asked: bool) -> ExitCode {
    for line in usage_lines() {
        if asked {
            println!("{line}");
        } else {
            eprintln!("{line}");
        }
    }
    if asked { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
because!(forms, LearnerCommand, "the forms of every command: on the output and as a success when a run asks for help, on the error stream and as a fault when the words of a run name no part of the work");

fn usage() -> ExitCode {
    forms(false)
}
because!(usage, LearnerCommand, "what the program prints, as a fault, when its words do not name a part of the work");

fn version() -> ExitCode {
    println!("{PROGRAM_NAME} {}", version_text());
    ExitCode::SUCCESS
}
because!(version, LearnerCommand, "what a run that asks which release it is prints: the name of the program and the numbers of its release");

fn unknown(part: &str) -> ExitCode {
    eprintln!("{PROGRAM_NAME}: no command {part}");
    usage()
}
because!(unknown, LearnerCommand, "a word that names no command, said before the forms of the commands");

fn failed(what: &str, why: &dyn std::fmt::Display) -> ExitCode {
    eprintln!("{what}: {why}");
    ExitCode::FAILURE
}
because!(failed, LearnerCommand, "the end of a run that cannot go on, with the thing that failed and the cause");

fn shown(found: &[String]) -> ExitCode {
    println!("{}", found.join(BETWEEN_WORDS));
    ExitCode::SUCCESS
}
because!(shown, LearnerCommand, "the words of a text on one line with a space between them, so that a person or a check sees how a text is cut");

fn counted(path: &str) -> ExitCode {
    match read_kind(Path::new(path)) {
        Ok(kind) => {
            println!("{} cases", kind.cases.len());
            for (name, value) in &kind.head {
                println!("{name} = {value}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => failed(path, &error),
    }
}
because!(counted, LearnerCommand, "the size of a case file and what its first line says, as a check that a file was written whole");

struct Given {
    plain: Vec<String>,
    named: HashMap<String, String>,
}
because!(Given, LearnerCommand, "the words of a run after the name of its part: the plain ones in their order, and the options by name");

fn given(args: &[String]) -> Given {
    let mut found = Given { plain: Vec::new(), named: HashMap::new() };
    for arg in args {
        match arg.split_once(OPTION_NAMED).filter(|(name, _)| !name.is_empty() && name.bytes().all(|sign| sign.is_ascii_lowercase())) {
            Some((name, value)) => {
                found.named.insert(name.to_string(), value.to_string());
            }
            None => found.plain.push(arg.clone()),
        }
    }
    found
}
because!(given, LearnerCommand, "the words of a run cut into plain words and options");

const PATHS_OPTION: &str = "paths";
because!(PATHS_OPTION, LearnerCommand, "the option that asks any command to say on the error output which branches of the search for a learned wording ran for each kind and how long each took");

const NUMBER_OPTIONS: &[&str] = &[PATHS_OPTION, TOLD_OPTION, RULES_OPTION, FIT_OPTION, HELD_OPTION, CURVE_OPTION, WORDS_OPTION, SHOW_OPTION, SMALL_OPTION, SEALED_OPTION, LANES_OPTION, MOST_OPTION, WHOLE_OPTION, ALL_OPTION, "families", "bytes", "files", "others", "run", "words"];

const TEXT_COMMANDS: &[&str] = &["words", "answer-words"];
because!(TEXT_COMMANDS, LearnerCommand, "the commands whose words after the name are a text to show, which no option check reads");
because!(NUMBER_OPTIONS, LearnerCommand, "the options of every command whose value is a whole number: those of the learner commands, and the counts that the commands for data take");

fn bad_number(run: &Given) -> Option<String> {
    NUMBER_OPTIONS.iter().find_map(|name| run.named.get(*name).filter(|value| value.parse::<usize>().is_err()).map(|value| format!("{PROGRAM_NAME}: the option {name} takes a whole number, not {value}")))
}
because!(bad_number, LearnerCommand, "the first option of a run, in the order of the list, whose value should be a whole number and is not, said as a fault; so a mistyped number does not quietly fall back to the usual value");

fn number(run: &Given, name: &str) -> Option<usize> {
    run.named.get(name).and_then(|text| text.parse::<usize>().ok())
}
because!(number, LearnerCommand, "an option as a whole number, when the run gives it");

fn is_on(run: &Given, name: &str) -> bool {
    number(run, name) == Some(ON)
}
because!(is_on, LearnerCommand, "whether a run turns an option on");

fn lanes_of(run: &Given) -> usize {
    number(run, LANES_OPTION).filter(|lanes| *lanes > 0).unwrap_or_else(hardware_lanes)
}
because!(lanes_of, LearnerCommand, "the threads of a run: what the run asks for, else all of the machine");

fn tools_of(run: &Given) -> Result<(Vec<Tool>, Option<usize>), String> {
    match (run.named.get(TOOL_OPTION), run.named.get(TOOLS_OPTION)) {
        (Some(name), _) => {
            let mut list = all_tools();
            let one = named_tools(name)?;
            let place = match list.iter().position(|tool| tool.name == one[0].name) {
                Some(place) => place,
                None => {
                    list.push(one[0]);
                    list.len() - 1
                }
            };
            Ok((list, Some(place)))
        }
        (None, Some(names)) => Ok((named_tools(names)?, None)),
        (None, None) => Ok((all_tools(), None)),
    }
}
because!(tools_of, LearnerCommand, "the tools of a run: all of them for the choice, a named part of them, or one tool to measure alone (which stays beside the others, so that the control by the form of the question is made as in the choice)");

fn classes_of(run: &Given) -> Result<Option<WordClasses>, ExitCode> {
    match run.named.get(CLASSES_OPTION).map(|path| read_classes(Path::new(path))) {
        Some(Ok(classes)) => Ok(Some(classes)),
        Some(Err(error)) => Err(failed(CLASSES_OPTION, &error)),
        None => Ok(None),
    }
}
because!(classes_of, LearnerCommand, "the table of word classes that a run names, read once for all its kinds and tools; nothing when the run names none");

fn asked_of(run: &Given, fit: usize, held: usize, tool: Option<usize>) -> Asked {
    Asked {
        fit: number(run, FIT_OPTION).unwrap_or(fit),
        held: number(run, HELD_OPTION).unwrap_or(held),
        sealed: is_on(run, SEALED_OPTION),
        new_words: is_on(run, WORDS_OPTION),
        curve: is_on(run, CURVE_OPTION),
        show: number(run, SHOW_OPTION).unwrap_or(0),
        questions: 0,
        tool,
        lanes: lanes_of(run),
        small: is_on(run, SMALL_OPTION),
        control_fit: number(run, FIT_OPTION).filter(|asked| *asked != fit).map(|_| fit),
    }
}
because!(asked_of, LearnerCommand, "what a run asks of the measure: its options, and for what it does not name the values of the set; a run that names another number of lessons than the set gives learns the controls from the lessons of the set");

fn print_table(rows: &[Row], curve: bool) {
    println!("{}", table_head(curve));
    for row in rows {
        println!("{}", line(row));
        for wrong in &row.wrong {
            println!("{wrong}");
        }
    }
    let done = rows.iter().filter(|row| row.learner.is_some()).count();
    let got = rows.iter().filter(|row| row.learner.is_some() && learned(row)).count();
    println!("kinds with a result: {done}; learned ({LEARNED}% or more, and {ABOVE} points above one answer and above the question alone): {got}");
    let counts: Vec<String> = rows.iter().filter_map(|row| row.learner.map(|got| format!("{} {} of {}", row.kind, (got * row.read as f64 / PERCENT).round(), row.read))).collect();
    println!("{RIGHT_COUNTS} {}", counts.join(", "));
}
because!(print_table, LearnerCommand, "the table of a measure: one line for each kind in the order of the kinds, the wrong lesson cases that were asked for, how many kinds are learned, and the count of right answers of each kind");

fn measure_kind(from: &Source, kind: &str, tools: &[Tool], asked: &Asked, classes: Option<&WordClasses>) -> Row {
    let Ok(loaded) = load(from, kind) else { return Row { kind: kind.to_string(), note: NO_FILE.to_string(), ..Row::default() } };
    let table = if asked.new_words { new_words(from, kind) } else { None };
    let last_part = if asked.sealed { Some(final_part(from, kind, asked.fit, asked.held)) } else { None };
    let earlier = || earlier_lessons(from, kind, asked.fit, asked.held);
    measure(&Ground { name: kind, cases: &loaded.kind.cases, labels: &loaded.kind.labels, start: loaded.start, stories: loaded.stories, final_part: last_part.as_deref(), sealed: None, new_words: table.as_deref(), answerer: None, classes, before: Some(&earlier), stored: true }, tools, asked)
}
because!(measure_kind, LearnerCommand, "one kind of a set through the measure, with its table of new words and its final part when the run asks for them, and the lessons of the kinds before it in its set, as the board gives them");

fn measure_set(folder: &Path, run: &Given) -> ExitCode {
    let from = match read_source(folder) {
        Ok(from) => from,
        Err(error) => return failed(&folder.to_string_lossy(), &error),
    };
    let (tools, tool) = match tools_of(run) {
        Ok(found) => found,
        Err(why) => return failed(TOOL_OPTION, &why),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let asked = asked_of(run, from.fit, from.held, tool);
    let kinds: Vec<String> = if run.plain.len() > 1 { run.plain[1..].iter().flat_map(|names| names.split(BETWEEN_KINDS)).map(str::to_string).collect() } else { from.kinds.clone() };
    println!("set {} ({}), at most {} cases learned from and {} read, {} threads", from.set, from.name, asked.fit, asked.held, asked.lanes);
    let alone = Asked { lanes: if kinds.len() > 1 { 1 } else { asked.lanes }, ..asked };
    let rows = on_lanes(kinds.len(), asked.lanes, &|kind| measure_kind(&from, &kinds[kind], &tools, &alone, classes.as_ref()));
    print_table(&rows, asked.curve);
    ExitCode::SUCCESS
}
because!(measure_set, LearnerCommand, "the kinds of one set through the measure, side by side on the threads when there are several, with the table in the order of the kinds");

fn measure_file(path: &Path, run: &Given) -> ExitCode {
    let loaded = match load_file(path) {
        Ok(loaded) => loaded,
        Err(error) => return failed(&path.to_string_lossy(), &error),
    };
    let (tools, tool) = match tools_of(run) {
        Ok(found) => found,
        Err(why) => return failed(TOOL_OPTION, &why),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let (fit, held) = file_sizes(path).unwrap_or((None, None));
    let asked = Asked { sealed: false, new_words: false, ..asked_of(run, fit.unwrap_or(loaded.kind.cases.len()), held.unwrap_or(loaded.kind.cases.len()), tool) };
    let name = path.file_stem().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let row = measure(&Ground { name: &name, cases: &loaded.kind.cases, labels: &loaded.kind.labels, start: loaded.start, stories: loaded.stories, final_part: None, sealed: None, new_words: None, answerer: None, classes: classes.as_ref(), before: None, stored: true }, &tools, &asked);
    print_table(&[row], asked.curve);
    ExitCode::SUCCESS
}
because!(measure_file, LearnerCommand, "one case file through the measure with no folder of a set around it: the read part alone, with no new words");

fn measured(run: &Given) -> ExitCode {
    let Some(place) = run.plain.first() else { return usage() };
    let path = Path::new(place);
    if path.is_dir() { measure_set(path, run) } else { measure_file(path, run) }
}
because!(measured, LearnerCommand, "the measure of a set or of one case file, by what the first word of the run names");

fn boarded(run: &Given) -> ExitCode {
    let Some(root) = run.plain.first() else { return usage() };
    let named = run.plain.len() > 1;
    let mut names: Vec<String> = if named {
        run.plain[1..].iter().flat_map(|names| names.split(BETWEEN_KINDS)).map(str::to_string).collect()
    } else {
        match std::fs::read_dir(root) {
            Ok(entries) => entries.filter_map(Result::ok).filter(|entry| entry.path().is_dir()).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect(),
            Err(error) => return failed(root, &error),
        }
    };
    if !named {
        names.sort();
    }
    let mut sources: Vec<Source> = Vec::new();
    for name in &names {
        match read_source(&Path::new(root).join(name)) {
            Ok(from) => sources.push(from),
            Err(error) if named => return failed(name, &error),
            Err(_) => {}
        }
    }
    if sources.is_empty() {
        return failed(root, &"no folder of a set here (a folder of a set holds the source.txt that ilm export writes)");
    }
    let tools = match tools_of(run) {
        Ok((tools, None)) => tools,
        Ok((_, Some(_))) => return failed(TOOL_OPTION, &"the board is made by the choice among tools; name them with the other option"),
        Err(why) => return failed(TOOLS_OPTION, &why),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    if let Some(unused) = refused_options(BOARD_COMMAND, &run.named.keys().map(String::as_str).collect::<Vec<&str>>()).first() {
        return failed(unused, &format!("{NOT_A_BOARD_KEY}{}", BOARD_KEYS.iter().chain(COMMAND_WIDE_OPTIONS).copied().collect::<Vec<&str>>().join(BETWEEN_KEYS)));
    }
    let asked = Asked { new_words: true, curve: false, show: 0, control_fit: None, ..asked_of(run, 0, 0, None) };
    let reference = match run.named.get(COMPARE_OPTION).map(|path| (path, std::fs::read_to_string(path))) {
        Some((_, Ok(text))) => Some(text),
        Some((path, Err(error))) => return failed(path, &error),
        None => None,
    };
    let order = match run.named.get(ORDER_OPTION).map(|path| (path, std::fs::read_to_string(path))) {
        Some((_, Ok(text))) => Some(text),
        Some((path, Err(error))) => return failed(path, &error),
        None => None,
    };
    let known = order.as_deref().map(known_times).unwrap_or_default();
    if let Some(folder) = run.named.get(KEPT_OPTION)
        && let Err(why) = open_story_disk(folder, run.named.get(ENGINE_OPTION).map(String::as_str))
    {
        return failed(folder, &why);
    }
    let made = board(&sources, &tools, &asked, classes.as_ref(), &known);
    close_story_disk();
    println!("{}", made.text.join("\n"));
    let jobs = order.as_deref().map_or_else(|| made.jobs.clone(), |order| grown_ways(&made.jobs, order));
    let grown = grown_jobs(&jobs);
    if !grown.is_empty() {
        println!("{GROWN_LINE}{}", grown.join(GROWN_BETWEEN));
    }
    for (option, lines) in [(OUT_OPTION, &made.text), (JOBS_OPTION, &jobs)] {
        if let Some(out) = run.named.get(option) {
            let written = std::fs::File::create(out).and_then(|mut file| file.write_all((lines.join("\n") + "\n").as_bytes()));
            if let Err(error) = written {
                return failed(out, &error);
            }
        }
    }
    if let Some(reference) = reference {
        println!("{}", compare_boards(&reference, &made.text.join("\n")).join("\n"));
    }
    ExitCode::SUCCESS
}
because!(boarded, LearnerCommand, "the board of all sets of a folder, or of the named ones in the order of their names; a folder that says nothing of a set is no set and is passed over; the board and the times of its jobs are also written to the files that the options name, and what changed from a reference board is printed after it");

fn compared(run: &Given) -> ExitCode {
    let [reference, board] = run.plain.as_slice() else { return usage() };
    let reference_text = match std::fs::read_to_string(reference) {
        Ok(text) => text,
        Err(error) => return failed(reference, &error),
    };
    match std::fs::read_to_string(board) {
        Ok(board_text) => {
            println!("{}", compare_boards(&reference_text, &board_text).join("\n"));
            ExitCode::SUCCESS
        }
        Err(error) => failed(board, &error),
    }
}
because!(compared, LearnerCommand, "what changed from a reference board file to a board file, each read from its file");

struct Opened {
    learn: Vec<Case>,
    labels: Vec<Vec<String>>,
    read: Vec<Case>,
}
because!(Opened, LearnerCommand, "the learned part of a case file with its label lines, and its read part, as the measure cuts them");

fn cut_file(path: &Path, fit: Option<usize>, held: Option<usize>) -> std::io::Result<Opened> {
    let loaded = load_file(path)?;
    let (own_fit, own_held) = file_sizes(path)?;
    let all = loaded.kind.cases.len();
    let cut = parts(&loaded.kind.cases, fit.or(own_fit).unwrap_or(all), held.or(own_held).unwrap_or(all), loaded.start, loaded.stories);
    Ok(Opened { learn: picked(&loaded.kind.cases, &cut.learn), labels: picked_labels(&loaded.kind.labels, &cut.learn), read: picked(&loaded.kind.cases, &cut.read) })
}
because!(cut_file, LearnerCommand, "the parts of one case file for a run that looks at tools: the sizes that are given, else the sizes of the first line of the file, else all cases");

fn opened(place: &str, run: &Given) -> std::io::Result<Opened> {
    cut_file(Path::new(place), number(run, FIT_OPTION), number(run, HELD_OPTION))
}
because!(opened, LearnerCommand, "the parts of one case file with the sizes of the run, when it gives them");


struct ConceptUse {
    kind: String,
    right: usize,
    sample: usize,
    taken: String,
    taken_share: f64,
}
because!(ConceptUse, LearnerCommand, "one kind where a topic of the library answers most of its sample: the kind, the lessons that the topic answers right and the size of its sample, and the tool that the choice takes for the kind with its share on the lessons that it did not learn from");

const NOTHING_NAMED: &str = "nothing";
because!(NOTHING_NAMED, LearnerCommand, "what the list of concepts says for a topic that builds on no other topic");

const CYCLE_CAUSE: &str = "cycle: ";
because!(CYCLE_CAUSE, LearnerCommand, "the words before each wait of a cycle in the list of concepts");

const KNOWLEDGE_CUT: &str = "; WHAT IT KNOWS BEFORE A STORY STOPPED AT A BOUND, so some of it is missing";
because!(KNOWLEDGE_CUT, LearnerCommand, "what the list of concepts says for a topic whose knowledge a bound cut, so that a writer of the library sees why a kind falls");

const NOT_CYCLE: &str = "; its rules with 'not' or a count wait for each other in a cycle, so they are read together";
because!(NOT_CYCLE, LearnerCommand, "what the list of concepts says for a topic whose rules that read what is absent or how many have no order of strata");

fn concepts(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::chain::{not_cycle_causes, not_strata};
    use ilm_core::tools::explain::learn::{LIBRARY_FOLDER, library, topic_shares};
    let library = match library() {
        Ok(library) => library,
        Err(fault) => return failed(LIBRARY_FOLDER, &fault),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let mut uses: Vec<Vec<ConceptUse>> = (0..library.topics.len()).map(|_| Vec::new()).collect();
    if let Some(root) = run.plain.first() {
        let mut folders: Vec<PathBuf> = match std::fs::read_dir(root) {
            Ok(entries) => entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).filter(|path| path.is_dir()).collect(),
            Err(error) => return failed(root, &error),
        };
        folders.sort();
        let tools = all_tools();
        for folder in folders {
            let Ok(from) = read_source(&folder) else { continue };
            for kind in &from.kinds {
                let Ok(loaded) = load(&from, kind) else { continue };
                let cut = parts(&loaded.kind.cases, from.fit, from.held, loaded.start, loaded.stories);
                let learn = picked(&loaded.kind.cases, &cut.learn);
                let labels = picked_labels(&loaded.kind.labels, &cut.learn);
                let lessons = Lessons { cases: &learn, labels: &labels, ask: None, classes: classes.as_ref(), before: None };
                let shares = topic_shares(&library, &lessons, lanes_of(run));
                if shares.iter().all(Option::is_none) {
                    continue;
                }
                let (taken, taken_share) = match take(&tools, lessons, lanes_of(run), true) {
                    Ok(taken) => (taken.named.clone(), taken.shares[taken.tool]),
                    Err(fault) => (fault, f64::default()),
                };
                for (place, share) in shares.iter().enumerate() {
                    if let Some((right, sample)) = share {
                        uses[place].push(ConceptUse { kind: format!("{} ({})", kind, from.set), right: *right, sample: *sample, taken: taken.clone(), taken_share });
                    }
                }
            }
        }
    }
    println!("concepts of the library {LIBRARY_FOLDER}: {} topics", library.texts.len());
    for (place, ((topic, built), used)) in library.texts.iter().zip(&library.topics).zip(&uses).enumerate() {
        let base = if topic.builds_on.is_empty() { NOTHING_NAMED.to_string() } else { topic.builds_on.join(", ") };
        let cycle = if not_strata(&built.rules).1 { NOT_CYCLE } else { "" };
        let cut = if library.knowledge_stopped(place) { KNOWLEDGE_CUT } else { "" };
        println!("{}: builds on {base}; sentences {} (rules {}, question forms {}, reply sentences {}, told facts {}){cycle}{cut}", topic.name, topic.lines, topic.rules.len(), topic.questions.len(), topic.reply_lines.len(), topic.facts.len());
        if !cycle.is_empty() {
            for cause in not_cycle_causes(&built.rules) {
                println!("  {CYCLE_CAUSE}{cause}");
            }
        }
        for one in used {
            println!("  {}: answers {} of the {} lessons of its sample; the choice takes {} ({:.1}% of the last lessons)", one.kind, one.right, one.sample, one.taken, one.taken_share * PERCENT);
        }
    }
    ExitCode::SUCCESS
}
because!(concepts, LearnerCommand, "the master list of the concepts that the library teaches: each topic with what it builds on and its number of sentences, and a mark when what it knows before a story stopped at a bound, and with a folder of sets, every kind where the topic answers most of its sample, with that share and the tool that the choice takes for the kind");

fn inputs_line(run: &Given) -> Result<String, String> {
    use ilm_core::tools::explain::learn::{LIBRARY_FOLDER, library};
    let library = library()?;
    let classes = match run.named.get(CLASSES_OPTION) {
        Some(path) => files_print(&[path.as_str()])?,
        None => NO_CLASSES.to_string(),
    };
    let cases = files_print(&run.plain.iter().map(String::as_str).collect::<Vec<&str>>())?;
    Ok(format!("{INPUTS_HEAD}{} ({} texts of {LIBRARY_FOLDER}), case files {cases}, classes {classes}", library.print, library.texts.len()))
}
because!(inputs_line, LearnerCommand, "the first line of a learning command: the fingerprints of the library, of the case files and of the table of classes that the run read, so that two outputs that differ with other inputs are told from a fault of the program");

fn files_print(paths: &[&str]) -> Result<String, String> {
    let mut print = Sha256::new();
    for path in paths {
        let bytes = std::fs::read(path).map_err(|fault| format!("{path}: {fault}"))?;
        print.add(&(bytes.len() as u64).to_le_bytes());
        print.add(&bytes);
    }
    Ok(print.hex().chars().take(INPUT_PRINT_SIGNS).collect())
}
because!(files_print, LearnerCommand, "the fingerprint of some files in the order given, each by its length and its bytes, so that a file that changed between two runs shows");

const INPUTS_HEAD: &str = "inputs: library ";
because!(INPUTS_HEAD, LearnerCommand, "the start of the line of the fingerprints of the inputs of a learning command, a line that a person finds by its first word");

const NO_CLASSES: &str = "none";
because!(NO_CLASSES, LearnerCommand, "what the line of the inputs says for a run with no table of classes");

const INPUT_PRINT_SIGNS: usize = 16;
because!(INPUT_PRINT_SIGNS, LearnerCommand, "the signs of the fingerprint of the files of a run: as many as the fingerprint of the library, enough that files that differ never print the same");

fn worded(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::wording::{form_line, learn_rule_wording, learn_wording, named_forms};
    let (Some(place), Some(name)) = (run.plain.first(), run.named.get(TOPIC_OPTION)) else { return usage() };
    match inputs_line(run) {
        Ok(line) => println!("{line}"),
        Err(fault) => return failed(place, &fault),
    }
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    let (first, last) = inner_parts(&file.learn);
    let fit: Vec<&Case> = first.iter().map(|at| &file.learn[*at]).collect();
    let check: Vec<&Case> = last.iter().map(|at| &file.learn[*at]).collect();
    let with_told = is_on(run, TOLD_OPTION);
    let learned = if is_on(run, RULES_OPTION) { learn_rule_wording(name, &fit, &check) } else { learn_wording(name, &fit, &check, with_told) };
    match learned {
        Ok(report) => {
            for line in &report.told {
                println!("{line}");
            }
            println!("{} forms learned from {} lessons for the text {name}; on the {} last lessons the learned forms answer {} right, the written forms of the text {}", report.forms.len(), fit.len(), report.checked, report.learned_right, report.written_right);
            let named = named_forms(&report.forms, &fit);
            println!("{} of {} learned forms name a word that a learned form with fewer fixed words reads too{NAMED_MARK}", named.iter().filter(|named| **named).count(), report.forms.len());
            for (form, named) in report.forms.iter().zip(&named) {
                println!("{} ({} of {} lessons){}", form_line(form), form.right, form.lessons, if *named { NAMED_MARK } else { "" });
            }
            ExitCode::SUCCESS
        }
        Err(fault) => failed(name, &fault),
    }
}
because!(worded, LearnerCommand, "the wording of a kind learned from its first lessons for one text of the library, written as lines of the library, and its share on the last lessons beside the written forms of the text; the forms that name a word that a form with fewer fixed words reads too are counted and marked");

const NAMED_MARK: &str = " (named words)";
because!(NAMED_MARK, LearnerCommand, "the mark of a learned form that names a word that a learned form with fewer fixed words reads too, and the words after the count of such forms, so that a person finds them by the same words");

fn taught_in_turn(run: &Given) -> ExitCode {
    use ilm_core::base::lanes::BUDGET_STOPPED;
    use ilm_core::tools::explain::discovery::{BELOW_BAR_GAP, BELOW_BAR_HEAD};
    use ilm_core::tools::explain::wording::{form_line, learn_curriculum};
    let Some(name) = run.named.get(TOPIC_OPTION) else { return usage() };
    match inputs_line(run) {
        Ok(line) => println!("{line}"),
        Err(fault) => return failed(name, &fault),
    }
    let mut files: Vec<(String, Vec<Case>, Vec<usize>, Vec<usize>)> = Vec::new();
    for place in &run.plain {
        let file = match opened(place, run) {
            Ok(file) => file,
            Err(error) => return failed(place, &error),
        };
        let (first, last) = inner_parts(&file.learn);
        files.push((place.clone(), file.learn, first, last));
    }
    let picked: Vec<(Vec<&Case>, Vec<&Case>)> = files.iter().map(|(_, learn, first, last)| (first.iter().map(|at| &learn[*at]).collect(), last.iter().map(|at| &learn[*at]).collect())).collect();
    let kinds: Vec<(&[&Case], &[&Case])> = picked.iter().map(|(fit, check)| (fit.as_slice(), check.as_slice())).collect();
    if let Some(folder) = run.named.get(KEPT_OPTION)
        && let Err(why) = open_story_disk(folder, run.named.get(ENGINE_OPTION).map(String::as_str))
    {
        return failed(folder, &why);
    }
    let taught = learn_curriculum(name, &kinds);
    close_story_disk();
    match taught {
        Ok(reports) => {
            for ((place, _, first, _), worked) in files.iter().zip(&reports) {
                let report = &worked.value;
                println!("{place}: {} rules learned on top of the kinds before it (seconds: rules {:.1}, readings {:.1}); {} forms learned from {} lessons; on the {} last lessons the learned forms answer {} right; work of the chains of the searches for rules and readings: {}", report.told.len(), report.seconds.0, report.seconds.1, report.forms.len(), first.len(), report.checked, report.learned_right, worked.work);
                if worked.stopped {
                    println!("  {BUDGET_STOPPED} {}: the searches stopped at their next step, and the rules below are those that they found before it", worked.work);
                }
                if !report.below_bar.is_empty() {
                    println!("  {BELOW_BAR_HEAD}{}", report.below_bar.iter().map(|(form, count, of)| format!("{form} ({count} of {of})")).collect::<Vec<String>>().join(BELOW_BAR_GAP));
                }
                for line in &report.told {
                    println!("  {line}");
                }
                for form in &report.forms {
                    println!("  {} ({} of {} lessons)", form_line(form), form.right, form.lessons);
                }
            }
            println!("rules of the concept {name} learned in turn:");
            for report in &reports {
                for line in &report.value.told {
                    println!("{line}");
                }
            }
            ExitCode::SUCCESS
        }
        Err(fault) => failed(name, &fault),
    }
}
because!(taught_in_turn, LearnerCommand, "the rules of one text learned from the lessons of several case files in the order given, each on top of the rules that the files before it taught, with each file's new rules, its wording and its share on its last lessons, and at the end all the rules learned, as lines of the library; with a store between runs, the rules that an earlier run of the same code learned for a file are read back");

fn kept_in_store(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::known_good::{STORE_FOLDER, store_text, stored_of};
    use ilm_core::tools::explain::wording::store_discovery;
    let (Some(place), Some(name)) = (run.plain.first(), run.named.get(TOPIC_OPTION)) else { return usage() };
    match inputs_line(run) {
        Ok(line) => println!("{line}"),
        Err(fault) => return failed(place, &fault),
    }
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    let (first, _) = inner_parts(&file.learn);
    let fit: Vec<&Case> = first.iter().map(|at| &file.learn[*at]).collect();
    let worked = match store_discovery(name, &fit) {
        Ok(worked) => worked,
        Err(fault) => return failed(name, &fault),
    };
    if worked.stopped {
        return failed(name, &format!("{STORE_STOPPED}{}", worked.work));
    }
    let case_file = Path::new(place);
    let named = |part: Option<&std::ffi::OsStr>| part.map_or_else(String::new, |part| part.to_string_lossy().into_owned());
    let source = format!("{} ({}), {} lessons", named(case_file.file_stem()), named(case_file.parent().and_then(Path::file_name)), fit.len());
    let out = Path::new(STORE_FOLDER).join(format!("{name}{STORE_FILE_END}"));
    let known = std::fs::read_to_string(&out).map(|text| stored_of(&text)).unwrap_or_default();
    let new = worked.value.iter().filter(|line| !known.iter().any(|stored| stored.line == **line)).count();
    let again = worked.value.iter().filter(|line| known.iter().any(|stored| stored.line == **line) && !known.iter().any(|stored| stored.line == **line && stored.source == source)).count();
    let written = std::fs::create_dir_all(STORE_FOLDER).and_then(|()| std::fs::write(&out, store_text(&known, &worked.value, &source)));
    if let Err(error) = written {
        return failed(&out.display().to_string(), &error);
    }
    println!("{} lines discovered for the text {name} on {source} (work of their chains {}); {new} new in {}, {again} stored again under this kind", worked.value.len(), worked.work, out.display());
    ExitCode::SUCCESS
}
because!(kept_in_store, LearnerCommand, "the known-good store written by discovery and by nothing else: the lines that one kind's lessons teach a text, found with the store of the text set aside, added to the store file of the text after a note of their source (the kind, its set and the number of lessons); a line that the store holds already under another kind is stored again under this one, so that each kind's lines stay whole as one group, and a discovery cut by the bound of its work writes nothing");

const STORE_STOPPED: &str = "the discovery stopped at the bound of its work, so nothing is stored: work ";
because!(STORE_STOPPED, LearnerCommand, "why the store command writes nothing for a discovery cut by the bound of its work: its lines may be only a part of what the lessons teach");

const STORE_FILE_END: &str = ".txt";
because!(STORE_FILE_END, LearnerCommand, "the ending of the file of the store of a text, as its texts end in the library");

fn discovered_why(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::discovery::discovery_report;
    let Some(name) = run.named.get(TOPIC_OPTION) else { return usage() };
    match inputs_line(run) {
        Ok(line) => println!("{line}"),
        Err(fault) => return failed(name, &fault),
    }
    let mut files: Vec<(String, Vec<Case>, Vec<usize>, Vec<usize>, Vec<Vec<String>>)> = Vec::new();
    for place in &run.plain {
        let file = match opened(place, run) {
            Ok(file) => file,
            Err(error) => return failed(place, &error),
        };
        let (first, last) = inner_parts(&file.learn);
        files.push((place.clone(), file.learn, first, last, file.labels));
    }
    if files.is_empty() {
        return usage();
    }
    let mut given: Vec<(String, String)> = Vec::new();
    for path in run.named.get(GIVEN_OPTION).map(|paths| paths.split(BETWEEN_KINDS).collect::<Vec<&str>>()).unwrap_or_default() {
        match std::fs::read_to_string(path) {
            Ok(text) => given.push((Path::new(path).file_stem().map_or_else(|| path.to_string(), |stem| stem.to_string_lossy().into_owned()), text)),
            Err(error) => return failed(path, &error.to_string()),
        }
    }
    let kinds: Vec<(String, Vec<&Case>, Vec<&Case>)> = files.iter().map(|(place, learn, first, last, _)| (place.clone(), first.iter().map(|at| &learn[*at]).collect(), last.iter().map(|at| &learn[*at]).collect())).collect();
    let library = match ilm_core::tools::explain::learn::library() {
        Ok(library) => library,
        Err(fault) => return failed(name, &fault),
    };
    let sources: Vec<Vec<String>> = files.iter().map(|(_, _, first, _, labels)| first.iter().map(|at| labels.get(*at).and_then(|marks| marks.iter().find_map(|mark| mark.strip_prefix(SOURCE_MARK))).unwrap_or_default().to_string()).collect()).collect();
    match discovery_report(&library, name, &kinds, &given, (number(run, SHOW_OPTION).unwrap_or(DISCOVER_SHOWN), &sources)) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(fault) => failed(name, &fault),
    }
}
because!(discovered_why, LearnerCommand, "the report of the discovery of the rules of one text over the case files given, in turn, as a tree: for each kind the asked fact, each step of the search with the rules that it judged and why each lost, the rules discovered with what they build on and a proof under them, and with the given text of the concept, each given rule with whether it was discovered and why not, and a summary line for each");

const SOURCE_MARK: &str = "K	";
because!(SOURCE_MARK, LearnerCommand, "the mark of the label line of a case that names the kind it comes from, in a case file that pools the lessons of several kinds, so that the report of the discovery counts a rule's answers by source kind");

const GIVEN_OPTION: &str = "given";
because!(GIVEN_OPTION, LearnerCommand, "the option that names the files of the given texts of one or more concepts (from the whole library, between commas), whose rules the report of the discovery compares with the rules discovered");

const DISCOVER_SHOWN: usize = 5;
because!(DISCOVER_SHOWN, LearnerCommand, "the rules beaten at each step that the report of the discovery shows unless asked for more: the first few tell how close the search came");

const CORPUS_FILES_OPTION: &str = "files";
because!(CORPUS_FILES_OPTION, LearnerCommand, "the option with the number of the first files of the large text that the step that reads facts from it reads, in the order of their names, to see how the facts grow with the text");

const DUMP_OPTION: &str = "dump";
because!(DUMP_OPTION, LearnerCommand, "the option with the file that the step that reads facts from a large text writes the pairs of each round to, with their sentences and the answers that rest on them");

const KINDS_OPTION: &str = "kinds";
because!(KINDS_OPTION, LearnerCommand, "the option that names the case files of the kinds on whose lessons the step that reads facts from a large text judges them, between commas");

fn text_facts(run: &Given) -> ExitCode {
    use ilm_data::sets::knowledge::text_facts::{KindLessons, learn_text_facts, learned_text};
    let (Some(corpus), Some(topic), Some(listed)) = (run.plain.first(), run.named.get(TOPIC_OPTION), run.named.get(KINDS_OPTION)) else { return usage() };
    let mut files: Vec<(String, Opened)> = Vec::new();
    for place in listed.split(BETWEEN_KINDS) {
        match opened(place, run) {
            Ok(file) => files.push((place.to_string(), file)),
            Err(error) => return failed(place, &error),
        }
    }
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let kinds: Vec<KindLessons> = files.iter().map(|(place, file)| KindLessons { name: place.clone(), cases: file.learn.iter().collect(), labels: file.labels.clone() }).collect();
    match learn_text_facts((Path::new(corpus), number(run, CORPUS_FILES_OPTION).unwrap_or(usize::MAX)), topic, &kinds, (classes.as_ref(), lanes_of(run), run.named.get(DUMP_OPTION).map(Path::new))) {
        Ok(report) => {
            for line in &report.lines {
                println!("{line}");
            }
            if let Some(out) = run.named.get(OUT_OPTION) {
                let names: Vec<String> = files.iter().map(|(place, _)| Path::new(place).file_stem().map_or_else(|| place.clone(), |stem| stem.to_string_lossy().into_owned())).collect();
                if let Err(error) = std::fs::write(out, learned_text(&report, topic, &names)) {
                    return failed(out, &error);
                }
                println!("written: {out}");
            }
            ExitCode::SUCCESS
        }
        Err(fault) => failed(topic, &fault),
    }
}
because!(text_facts, LearnerCommand, "the facts of one text of the library read from a large text: the kinds' lessons from their case files, the step's report, and the learned text written to a file when the run names one");

fn costed_rules(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::EXPLAIN_TOOL;
    use ilm_core::tools::explain::chain::{COSTING, rule_costs};
    let Some(place) = run.plain.first() else { return usage() };
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    COSTING.store(true, std::sync::atomic::Ordering::Relaxed);
    let model = match (EXPLAIN_TOOL.learn)(Lessons::of(&file.learn), lanes_of(run)) {
        Ok(model) => model,
        Err(fault) => return failed(place, &fault),
    };
    let right = on_lanes(file.read.len(), lanes_of(run), &|at| model.answer(&file.read[at].sentences, &file.read[at].question).words == answer_words(&file.read[at].answer)).into_iter().filter(|right| *right).count();
    COSTING.store(false, std::sync::atomic::Ordering::Relaxed);
    let costs = rule_costs();
    let total: f64 = costs.iter().map(|cost| cost.seconds).sum();
    println!("rules of the library by the time of their readings, learning from {} lessons and reading {} cases of {place} ({right} right); {total:.2} s of rule readings over all threads", file.learn.len(), file.read.len());
    for cost in costs.iter().take(number(run, SHOW_OPTION).unwrap_or(COSTS_SHOWN)) {
        println!("{:>8.2} s {:>6.1}% {:>12} facts read {:>9} readings | {}", cost.seconds, PERCENT * cost.seconds / total.max(f64::MIN_POSITIVE), cost.reads, cost.readings, cost.text);
    }
    ExitCode::SUCCESS
}
because!(costed_rules, LearnerCommand, "the profile of the explain tool on one case file: the tool learns from the lessons and reads the read part with the engine timing each rule and counting the facts that it reads, and the rules that took the most time are printed with their share, so that a writer of the library sees which rules to write in a cheaper order");

const COSTS_SHOWN: usize = 25;
because!(COSTS_SHOWN, LearnerCommand, "the rules that the profile prints when the command names no number: enough to hold the costly rules of a text, few enough to read");

const BEFORE_OPTION: &str = "before";
because!(BEFORE_OPTION, LearnerCommand, "the option of the why command with the case files of the kinds that come before the kind, whose learn parts the model may learn earlier lines from, as the board gives a kind the lessons of the kinds before it in its set");

const ALL_OPTION: &str = "all";
because!(ALL_OPTION, LearnerCommand, "the option of why that learns from all lessons and reads all of them again, in place of learning from the first two thirds of the lessons and reading the last third, as the choice judges a tool");

const WHY_COUNTS_HEAD: &str = "the answers of the learned form that reads it, on the first lessons of the form (the lessons where each condition holds, by their answers):";
because!(WHY_COUNTS_HEAD, LearnerCommand, "the words before the answers of the learned form of a wrong lesson with their counts by answer, in the report of why");

const WHY_SHOWN: usize = 10;
because!(WHY_SHOWN, LearnerCommand, "the wrong lessons that why prints in full when the run names no number: enough to see the sorts of faults, few enough to read");

const WHY_COUNTED: usize = 10;
because!(WHY_COUNTED, LearnerCommand, "the most common missing steps that the last lines of why count");

const WHY_NONE: &str = "no answer of the model's texts that fits the question can give the wanted words";
because!(WHY_NONE, LearnerCommand, "what why says of a wrong lesson for which no answer of the texts of the model can be diagnosed: no question form fits, or no answer can give the wanted words");

fn whyed(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::EXPLAIN_TOOL;
    use ilm_core::tools::explain::learn::reading_judgments;
    use ilm_core::tools::explain::why::MISSING_AS_WRITTEN;
    let [place] = run.plain.as_slice() else { return usage() };
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let lanes = lanes_of(run);
    let all = is_on(run, ALL_OPTION);
    let (first, last) = if all { ((0..file.learn.len()).collect(), (0..file.learn.len()).collect()) } else { inner_parts(&file.learn) };
    let (fit, labels, asked) = (picked(&file.learn, &first), picked_labels(&file.labels, &first), picked(&file.learn, &last));
    let mut earlier: Vec<Vec<Case>> = Vec::new();
    for path in run.named.get(BEFORE_OPTION).map(|paths| paths.split(BETWEEN_KINDS).collect::<Vec<&str>>()).unwrap_or_default() {
        match opened(path, run) {
            Ok(file) => earlier.push(file.learn),
            Err(error) => return failed(path, &error),
        }
    }
    let earlier_lessons = || earlier.clone();
    let before: Option<&(dyn Fn() -> Vec<Vec<Case>> + Sync)> = if earlier.is_empty() { None } else { Some(&earlier_lessons) };
    let model = match (EXPLAIN_TOOL.learn)(Lessons { cases: &fit, labels: &labels, ask: None, classes: classes.as_ref(), before }, lanes) {
        Ok(model) => model,
        Err(why) => return failed(place, &why),
    };
    let said: Vec<Said> = on_lanes(asked.len(), lanes, &|at| model.answer(&asked[at].sentences, &asked[at].question));
    let wrong: Vec<usize> = (0..asked.len()).filter(|at| said[*at].words != answer_words(&asked[*at].answer)).collect();
    let silent = wrong.iter().filter(|at| said[**at].words.is_empty()).count();
    let missing: Vec<Vec<String>> = on_lanes(wrong.len(), lanes, &|at| model.missing(&asked[wrong[at]].sentences, &asked[wrong[at]].question, &answer_words(&asked[wrong[at]].answer)));
    let read = if all { "the same lessons" } else { "the last third of the lessons, which it did not learn from" };
    println!("why: a model of {place} learned from {} lessons reads {} lessons ({read}): {} wrong, {silent} of them with no answer", fit.len(), asked.len(), wrong.len());
    if let Ok(table) = reading_judgments(&Lessons { cases: &fit, labels: &labels, ask: None, classes: classes.as_ref(), before }, lanes) {
        table.iter().for_each(|line| println!("{line}"));
    }
    for (shown, at) in wrong.iter().enumerate().take(number(run, SHOW_OPTION).unwrap_or(WHY_SHOWN)) {
        let case = &asked[*at];
        println!();
        println!("lesson {} of the lessons:", last[*at]);
        for sentence in &case.sentences {
            println!("  told: {sentence}");
        }
        println!("  question: {}", case.question);
        println!("  wanted: {} | said: {} ({})", case.answer, said[*at].words.join(" "), said[*at].note);
        if !said[*at].words.is_empty() {
            model.proof(&case.sentences, &case.question).iter().for_each(|line| println!("  {line}"));
        }
        if missing[shown].is_empty() {
            println!("  {WHY_NONE}");
        }
        missing[shown].iter().for_each(|line| println!("  {line}"));
        let counts = model.answer_counts(&case.question, &fit);
        if !counts.is_empty() {
            println!("  {WHY_COUNTS_HEAD}");
            counts.iter().for_each(|line| println!("    {line}"));
        }
    }
    let mut counted: HashMap<String, usize> = HashMap::new();
    for lines in &missing {
        let key = lines.iter().find_map(|line| line.trim_start().strip_prefix(MISSING_AS_WRITTEN)).map_or(WHY_NONE, str::trim);
        *counted.entry(key.to_string()).or_default() += 1;
    }
    let mut ranked: Vec<(String, usize)> = counted.into_iter().collect();
    ranked.sort_by(|one, other| other.1.cmp(&one.1).then_with(|| one.0.cmp(&other.0)));
    println!();
    println!("the most common missing steps of the {} wrong lessons:", wrong.len());
    ranked.iter().take(WHY_COUNTED).for_each(|(key, count)| println!("  {count} | {key}"));
    ExitCode::SUCCESS
}
because!(whyed, LearnerCommand, "why a model of the explain tool answers lessons wrong: it learns from the first two thirds of the lessons of a case file (all lessons with the option all), with the lessons of the kinds before it when the run names their case files, as the board gives them, and reads the rest; for each wrong lesson, the story, the wanted and the given answer, the proof of the answer that it gave, and the first step that the facts of the story miss for the wanted answer; then the missing steps counted, the most common first, so that the writer of a text or a learner sees which step to add; only lessons are read, never the read part");

fn joined(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::wording_joining::learn_joining;
    let Some(place) = run.plain.first() else { return usage() };
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    let joining = match learn_joining(&file.learn) {
        Ok(joining) => joining,
        Err(fault) => return failed(place, &fault),
    };
    let right = |cases: &[Case]| cases.iter().filter(|case| joining.model.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count();
    println!("{} lines learned from {} lessons; the search reads {} of them right; the engine reads the written lines {} of {} lessons right and {} of {} of the read part", joining.lines.len(), file.learn.len(), joining.proposed_right, right(&file.learn), file.learn.len(), right(&file.read), file.read.len());
    for line in &joining.lines {
        println!("{line}");
    }
    ExitCode::SUCCESS
}
because!(joined, LearnerCommand, "the ways of joining of a language of commands learned from the lessons of a case file, written as lines of the library, with the lessons that the search reads right and the shares of the engine's reading of the lines on the lessons and on the read part");

fn meant(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::learn::{library, model_of};
    use ilm_core::tools::explain::wording_meanings::{meanings_lines, meanings_topic};
    let Some(place) = run.plain.first() else { return usage() };
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    let library = match library() {
        Ok(library) => library,
        Err(fault) => return failed(place, &fault),
    };
    let lessons: Vec<&Case> = file.learn.iter().collect();
    let (Some((_, lines)), Some(topic)) = (meanings_lines(&library.topics, &lessons), meanings_topic(&library.topics, &lessons)) else { return failed(place, &NO_MEANINGS) };
    let model = model_of(vec![topic]);
    let right = |cases: &[Case]| cases.iter().filter(|case| model.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count();
    println!("meanings learned from {} lessons; the engine reads the learned lines {} of {} lessons right and {} of {} of the read part", file.learn.len(), right(&file.learn), file.learn.len(), right(&file.read), file.read.len());
    println!("{lines}");
    ExitCode::SUCCESS
}
because!(meant, LearnerCommand, "the meanings of the function words of a case file learned over the text of functions on lists of the library, written as lines of the library, with the shares of the engine's reading of them on the lessons and on the read part");

const NO_MEANINGS: &str = "no meanings: the library has no text of functions on lists, the lessons tell sentences, no word of the questions is missing from the answers, or no word gains a meaning";
because!(NO_MEANINGS, LearnerCommand, "what the command of meanings says when the lessons give no lines");

const PROOF_INDENT: &str = "   ";
because!(PROOF_INDENT, LearnerCommand, "the space before each line of a proof under the answer that it proves");

const CASES_OPTION: &str = "cases";
because!(CASES_OPTION, LearnerCommand, "the option of the command of a proof that names a case file: the model that the explain tool learns from its lessons answers, so a rule that was discovered and is in no text proves too");

const NO_PROOF: &str = "no text of the library answers the question from the told sentences";
because!(NO_PROOF, LearnerCommand, "what the command of a proof says when no topic answers the question from the told sentences, as an answer that holds otherwise proves nothing");

fn proved(run: &Given) -> ExitCode {
    use ilm_core::base::signs::sentences_of;
    use ilm_core::tools::explain::learn::{LIBRARY_FOLDER, library, library_proof, marked_proof};
    let sentences = sentences_of(&run.plain.join(BETWEEN_WORDS));
    let Some((question, told)) = sentences.split_last() else { return usage() };
    if let Some(place) = run.named.get(CASES_OPTION) {
        use ilm_core::tools::explain::EXPLAIN_TOOL;
        let file = match opened(place, run) {
            Ok(file) => file,
            Err(error) => return failed(place, &error),
        };
        let classes = match classes_of(run) {
            Ok(classes) => classes,
            Err(code) => return code,
        };
        let model = match (EXPLAIN_TOOL.learn)(Lessons { cases: &file.learn, labels: &file.labels, ask: None, classes: classes.as_ref(), before: None }, lanes_of(run)) {
            Ok(model) => model,
            Err(why) => return failed(place, &why),
        };
        let said = model.answer(told, question);
        if said.words.is_empty() {
            return failed(question, &NO_PROOF);
        }
        println!("{}", said.words.join(BETWEEN_WORDS));
        let kind = Path::new(place).file_stem().map_or_else(|| place.clone(), |stem| stem.to_string_lossy().into_owned());
        for line in marked_proof(&model.proof(told, question), &model.rules_used(told, question), &model.lines_from_lessons(), &kind) {
            println!("{PROOF_INDENT}{line}");
        }
        return ExitCode::SUCCESS;
    }
    let library = match library() {
        Ok(library) => library,
        Err(why) => return failed(LIBRARY_FOLDER, &why),
    };
    match library_proof(&library, told, question) {
        Some((words, proof)) => {
            println!("{}", words.join(BETWEEN_WORDS));
            for line in marked_proof(&proof, &[], &[], LIBRARY_FOLDER) {
                println!("{PROOF_INDENT}{line}");
            }
            ExitCode::SUCCESS
        }
        None => failed(question, &NO_PROOF),
    }
}
because!(proved, LearnerCommand, "the answer to told sentences and a question, the last sentence of the text, with its proof tree back to the told sentences, each rule marked as discovered on the kind or given, and each known fact that was learned marked as discovered: by the topics of the library alone, with no lessons, or by the model that the explain tool learns from the lessons of a case file");

const TEXT_ENDING: &str = "txt";
because!(TEXT_ENDING, LearnerCommand, "the ending of the files of the library that the command of places reads in a folder");

const NOTE_START: &str = "Note:";
because!(NOTE_START, LearnerCommand, "the start of a note line of a text of the library, which holds no places");

const PLACE_BY_MARK: &str = "; a place stands before a point that a letter follows, as in a name with points";
because!(PLACE_BY_MARK, LearnerCommand, "the words after a line whose places include a letter that a point follows with a letter right after it, the sign of a name read as places (Washington, D.C.); a point that ends an answer and a comma of a list of places are not flagged");

const NOT_READ: &str = "the reader refuses this line";
because!(NOT_READ, LearnerCommand, "what the command of places says of a line that the reader of the library refuses");

fn placed(run: &Given) -> ExitCode {
    use ilm_core::tools::explain::text::{place_letter, read_topic, topic_places};
    let mut files: Vec<PathBuf> = Vec::new();
    for place in &run.plain {
        let path = Path::new(place);
        if path.is_dir() {
            let mut found: Vec<PathBuf> = match std::fs::read_dir(path) {
                Ok(entries) => entries.filter_map(Result::ok).map(|entry| entry.path()).filter(|file| file.extension().is_some_and(|ending| ending == TEXT_ENDING)).collect(),
                Err(error) => return failed(place, &error),
            };
            found.sort();
            files.extend(found);
        } else {
            files.push(path.to_path_buf());
        }
    }
    if files.is_empty() {
        return usage();
    }
    for file in &files {
        let shown = file.display().to_string();
        let text = match std::fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) => return failed(&shown, &error),
        };
        for (at, line) in text.lines().map(str::trim).enumerate() {
            if line.is_empty() || line.starts_with(NOTE_START) {
                continue;
            }
            let places = match read_topic(&shown, line) {
                Ok(read) => topic_places(&read),
                Err(_) => {
                    println!("{shown}:{}: {NOT_READ} | {line}", at + 1);
                    continue;
                }
            };
            if places.is_empty() {
                continue;
            }
            let letters: Vec<String> = places.iter().map(|place| place_letter(*place)).collect();
            let by_mark = letters.iter().any(|letter| [format!(" {letter}."), format!(".{letter}.")].iter().any(|marked| line.match_indices(marked.as_str()).any(|(found, _)| line[found + marked.len()..].chars().next().is_some_and(char::is_alphanumeric))));
            println!("{shown}:{}: places {}{} | {line}", at + 1, letters.join(BETWEEN_WORDS), if by_mark { PLACE_BY_MARK } else { "" });
        }
    }
    ExitCode::SUCCESS
}
because!(placed, LearnerCommand, "each line of the texts of the library that holds open places, with the places that the reader takes in it, and a flag where a point with a letter right after it follows a place, the sign of a name read as places; a file, or every text of a folder");

fn chosen(run: &Given) -> ExitCode {
    let Some(place) = run.plain.first() else { return usage() };
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    let tools = match tools_of(run) {
        Ok((tools, _)) => tools,
        Err(why) => return failed(TOOLS_OPTION, &why),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let (first, last) = inner_parts(&file.learn);
    println!("lessons {} first {} last {}", file.learn.len(), first.len(), last.len());
    match take(&tools, Lessons { cases: &file.learn, labels: &file.labels, ask: None, classes: classes.as_ref(), before: None }, lanes_of(run), true) {
        Ok(taken) => {
            for (place, tool) in tools.iter().enumerate() {
                if taken.said[place].is_empty() {
                    println!("tool {} right {} seconds {:.places$}", tool.name, (taken.shares[place] * (last.len().max(1) as f64)).round(), taken.seconds[place], places = SECONDS_PLACES);
                } else {
                    println!("tool {} no model: {}", tool.name, taken.said[place]);
                }
            }
            for (tool, why) in &taken.stopped {
                println!("stopped on all lessons: {} ({why})", tools[*tool].name);
            }
            println!("taken {}", tools[taken.tool].name);
            ExitCode::SUCCESS
        }
        Err(why) => failed(place, &why),
    }
}
because!(chosen, LearnerCommand, "the choice among the tools for one case file, in the open: what each tool reads right of the lessons that it did not learn from, what a tool said that gave no model, and the tool that is taken");

fn controlled(run: &Given) -> ExitCode {
    let [place, changes] = run.plain.as_slice() else { return usage() };
    let path = Path::new(place);
    let loaded = match load_file(path) {
        Ok(loaded) => loaded,
        Err(error) => return failed(place, &error),
    };
    let changed_file = match read_kind(Path::new(changes)) {
        Ok(kind) => kind,
        Err(error) => return failed(changes, &error),
    };
    let (tools, tool) = match tools_of(run) {
        Ok(found) => found,
        Err(why) => return failed(TOOL_OPTION, &why),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let (fit, held) = file_sizes(path).unwrap_or((None, None));
    let all = loaded.kind.cases.len();
    let changed = changed_cases(&changed_file.cases, &changed_file.labels);
    let name = path.file_stem().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    match control(&loaded, number(run, FIT_OPTION).or(fit).unwrap_or(all), number(run, HELD_OPTION).or(held).unwrap_or(all), &changed, &tools, tool, is_on(run, WHOLE_OPTION), number(run, MOST_OPTION), classes.as_ref(), lanes_of(run)) {
        Ok(count) => {
            println!("{}", count.line(&name));
            ExitCode::SUCCESS
        }
        Err(why) => failed(place, &why),
    }
}
because!(controlled, LearnerCommand, "one control of changed cases: the kind from its case file, the changed cases from theirs, a model by the choice among the tools or by one tool, and one line of the result");

pub fn run_command() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((part, rest)) = args.split_first() else { return usage() };
    let run = given(rest);
    if let Some(fault) = bad_number(&run).filter(|_| !TEXT_COMMANDS.contains(&part.as_str())) {
        eprintln!("{fault}");
        return ExitCode::FAILURE;
    }
    if is_on(&run, PATHS_OPTION) {
        ilm_core::tools::explain::wording::SEARCH_PATHS.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    match part.as_str() {
        BOARD_COMMAND => boarded(&run),
        "compare" => compared(&run),
        "measure" => measured(&run),
        "choice" => chosen(&run),
        "prove" => proved(&run),
        "places" => placed(&run),
        "concepts" => concepts(&run),
        "wording" => worded(&run),
        "curriculum" => taught_in_turn(&run),
        "discover-why" => discovered_why(&run),
        "store-lines" => kept_in_store(&run),
        "facts" => text_facts(&run),
        "joining" => joined(&run),
        "why" => whyed(&run),
        "meanings" => meant(&run),
        "explain-cost" => costed_rules(&run),
        "control" => controlled(&run),
        "changes" => ilm_data::controls::command::changes_command(rest),
        "parquet" => ilm_data::sets::parquet::show::parquet_command(rest),
        "export" => ilm_data::sets::readers::export_command(rest),
        "fetch" => ilm_data::sets::fetch::fetch_command(rest),
        "store" => ilm_data::sets::knowledge::store::store_command(rest),
        "classes" => ilm_data::sets::knowledge::classes::classes_command(rest),
        "shared-text" => ilm_data::sets::shared::shared_command(rest),
        "cases" => rest.first().map_or_else(usage, |path| counted(path)),
        "words" => rest.first().map_or_else(usage, |text| shown(&words(text))),
        "answer-words" => rest.first().map_or_else(usage, |text| shown(&answer_words(text))),
        word if HELP_WORDS.contains(&word) => forms(true),
        word if VERSION_WORDS.contains(&word) => version(),
        word => unknown(word),
    }
}
because!(run_command, LearnerCommand, "the one entry of the program: the first word names the part of the work, the words after it are what that part reads");
