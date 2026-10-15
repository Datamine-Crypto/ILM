use ilm_eval::measure::board::{board, final_part};
use ilm_core::cases::{Case, read_kind};
use ilm_core::classes::{WordClasses, read_classes};
use ilm_core::choice::{NO_LABELS, inner_parts, picked, picked_labels, take};
use ilm_eval::measure::control::{changed_cases, control};
use crate::ask::facts::{build_facts, held_answers, read_questions};
use crate::ask::request::{Whole, build_router};
use ilm_core::base::lines::one_line;
use ilm_core::base::lanes::{hardware_lanes, on_lanes};
use ilm_core::base::shares::{PERCENT, share};
use ilm_core::know::probe::{Numbers, Which, development, probe, references, whole};
use ilm_core::know::questions::{Asking, Names, Questions, SortedNames, held_shares};
use ilm_core::know::reader::Entry;
use ilm_core::know::store::{Texts, read_facts};
use ilm_core::know::sure::{Knowledge, OFTEN_NAMES, level_of};
use ilm_core::tools::call::{Learner, Lessons, Tool};
use ilm_eval::measure::kind::{ABOVE, Asked, Ground, LEARNED, Row, learned, line, measure, renamed, table_head};
use ilm_data::sets::folder::{Source, file_sizes, load, load_file, new_words, read_source};
use ilm_core::split::parts;
use ilm_core::tools::{all_tools, named_tools, tool_names};
use ilm_core::base::words::{answer_words, words};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
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
because!(TOOLS_OPTION, LearnerCommand, "the option that names the tools of the choice, for a run with a part of them");

const LANES_OPTION: &str = "lanes";
because!(LANES_OPTION, LearnerCommand, "the option with the number of threads of the run");

const OUT_OPTION: &str = "out";
because!(OUT_OPTION, LearnerCommand, "the option with the file that the board is also written to");

const ON: usize = 1;
because!(ON, LearnerCommand, "the value that turns an option on: the least number that is not nothing");

const SHOWN_DIFFERENT: usize = 5;
because!(SHOWN_DIFFERENT, LearnerCommand, "the lesson cases that a check prints when its answers are not the answers of the file: enough to see the sort of the difference");

const SECONDS_PLACES: usize = 2;
because!(SECONDS_PLACES, LearnerCommand, "the places after the point of the seconds of a tool: a tool that needs a hundredth of a second is still told from one that needs none");

const PART_OPTION: &str = "part";
because!(PART_OPTION, LearnerCommand, "the option that names the part of the facts that is asked: the development part, or the asked facts of the probe");

const PROBE_PART: &str = "probe";
because!(PROBE_PART, LearnerCommand, "the value that asks the facts of the probe; without it the development part is asked, so that no run reads the probe by oversight");

const TEXT_OPTION: &str = "text";
because!(TEXT_OPTION, LearnerCommand, "the option that says how much of the store a run reads");

const TEXT_ASKED: &str = "asked";
because!(TEXT_ASKED, LearnerCommand, "the value for the articles that answer to a name of a lesson fact or of an asked fact: the same answers as with every article of the subjects, in less time");

const TEXT_ALL: &str = "all";
because!(TEXT_ALL, LearnerCommand, "the value for every article of the store: the run that counts for the size of the text");

const CLASSES_OPTION: &str = "classes";
because!(CLASSES_OPTION, LearnerCommand, "the option with the file of a table of word classes; the program reads it once and gives it to every tool with its lessons");

const NAMES_OPTION: &str = "names";
because!(NAMES_OPTION, LearnerCommand, "the option with a file of known names, one name on a line as its letters, in rising order");

const HELD_PART: &str = "H";
because!(HELD_PART, LearnerCommand, "the mark of a held question in a file of questions; every other line is a lesson");

const QUESTION_FIELDS: usize = 4;
because!(QUESTION_FIELDS, LearnerCommand, "the fields of a line of a file of questions that the reader of questions needs: the part, the question, the subject and the relation");

const DUMP_OPTION: &str = "dump";
because!(DUMP_OPTION, LearnerCommand, "the option with a folder that the tables of a model are written to as plain lines, so that two models can be compared line by line");

const SHOWN_RELATIONS: usize = 6;
because!(SHOWN_RELATIONS, LearnerCommand, "the relations with the lowest and with the highest share that a run of the probe names");

const MILLION: f64 = 1e6;
because!(MILLION, LearnerCommand, "the size of a text is printed in millions of signs");

const MOST_OPTION: &str = "most";
because!(MOST_OPTION, LearnerCommand, "the option with the most lessons that a control uses, in the order of its file");

const WHOLE_OPTION: &str = "whole";
because!(WHOLE_OPTION, LearnerCommand, "the option of a control whose changed cases were made from all lessons, for a model of all lessons as the board makes it");

const TABLE_OPTION: &str = "table";
because!(TABLE_OPTION, LearnerCommand, "the option with the folder that the table of the one command is written to");

const TABLE_FOLDER: &str = "table";
because!(TABLE_FOLDER, LearnerCommand, "the folder of the table of the one command inside the folder of the text store, when the run names no other");

const MILLISECONDS: f64 = 1000.0;
because!(MILLISECONDS, LearnerCommand, "the thousandths of a second in a second, to print the time of one answer");

const RIGHT_COUNTS: &str = "right answers of the read part, by kind:";
because!(RIGHT_COUNTS, LearnerCommand, "the words before the counts of right answers under the table of a measure: the table gives shares, and a report of a final read gives the counts");

const NO_FILE: &str = "the cases of the kind could not be read";
because!(NO_FILE, LearnerCommand, "the note of a kind of a set whose case file is not there or cannot be read");

const HELP_WORDS: &[&str] = &["help", "--help", "-h"];
because!(HELP_WORDS, LearnerCommand, "the words that ask for the forms of the commands: the plain word and the two forms that most programs take");

const VERSION_WORDS: &[&str] = &["version", "--version", "-V"];
because!(VERSION_WORDS, LearnerCommand, "the words that ask for the version of the program: the plain word and the two forms that most programs take");

pub const PROGRAM_VERSION: [u32; 3] = [1, 1, 0];
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
    lines.push(format!("usage: ilm board <folder of the sets> [set,set,...] [tools=<names>] [classes=<file>] [lanes=<n>] [out=<file>] [sealed={ON}]"));
    lines.push(format!("   or: ilm measure <folder of a set, or a case file> [kind,kind,...] [tool=<name>] [tools=<names>] [classes=<file>] [fit=<n>] [held=<n>] [curve={ON}] [newwords={ON}] [show=<n>] [lanes=<n>] [small={ON}]"));
    lines.push(format!("   or: ilm choice <case file> [tools=<names>] [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm check <case file> <tool> <file of answers for the last lessons> <file of answers for the read part> [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm check-all <folder of answers> <name of the files of answers> <tool> cases=<folder>,<folder> [sources=<set>,<set>] [write=1] [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm know <folder of the text store> <file of facts> [part={PROBE_PART}] [text={TEXT_ASKED}|{TEXT_ALL}] [fit=<n>] [classes=<file>] [lanes=<n>] [dump=<folder>]"));
    lines.push(format!("   or: ilm questions <file of questions> [names=<file of known names>] [fit=<n>] [dump=<folder>]"));
    lines.push(format!("   or: ilm ask build <folder of the text store> <file of facts> <file of questions> <file of known names> <folder of the sets> <board file> [classes=<file>] [table=<folder>] [lanes=<n>]"));
    lines.push(format!("   or: ilm ask measure <folder of the table> <file of questions> [dump=<file>]"));
    lines.push(format!("   or: ilm ask kinds <folder of the table> <file of questions>"));
    lines.push(format!("   or: ilm ask <folder of the table> \"<request>\" [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm ask <folder of the table> [classes=<file>] [lanes=<n>]   (a session: one request on each line of the input)"));
    lines.push(format!("   or: ilm patterns <words|model|rules|answers> <case file> ..."));
    lines.push(format!("   or: ilm control <case file of the kind> <file of changed cases> [tool=<name>] [whole={ON}] [most=<n>] [classes=<file>] [lanes=<n>]"));
    lines.push(format!("   or: ilm cases <case file>"));
    lines.push(format!("   or: ilm words <text>"));
    lines.push(format!("   or: ilm answer-words <text>"));
    lines.push(format!("   or: ilm fetch <list of sources> [sets=<set>,<set>] [store=<folder of the encyclopedia>] [folder=<folder of a set>] [check=yes]"));
    lines.push(format!("   or: ilm export <set> <description file> <folder of the case files>"));
    lines.push(format!("   or: ilm store <folder of the copy of the encyclopedia> <folder of the store> <description of the probe> <description of the questions> [others=<signs>] [lanes=<n>]"));
    lines.push(format!("   or: ilm classes <folder of the copy of the encyclopedia> <folder of the table> [files=<n>] [bytes=<n>] [classes=<n>] [lanes=<n>]"));
    lines.push(format!("   or: ilm write <out|metric-files|measure|texts|traces|model> ...   (ilm write alone prints its forms)"));
    lines.push(format!("   or: ilm changes <control> <case file of a kind> [whole=1] [root=<folder>] [part=read] [support=<folder>] [forms=<description>] [about=<description>]   (ilm changes alone prints the controls)"));
    lines.push(format!("   or: ilm made <folder> [families=<n>] [held=1]"));
    lines.push(format!("   or: ilm parquet <file> [column,column,...]"));
    lines.push(format!("   or: ilm shared-text <folder of data>[,<folder of data>] [run=<tokens>] [words=<tokens of letters>] [only=<part of a path>]"));
    lines.push(format!("   or: ilm --version"));
    lines.push(format!("board: every kind of every set through the choice among the tools, and the table of the tasks"));
    lines.push(format!("measure: the kinds of one set with their controls; a model is made by the choice among the tools, or by one tool alone"));
    lines.push(format!("choice: the share of every tool on the lessons that it did not learn from, and the tool that is taken"));
    lines.push(format!("check: the answers of one tool beside the answers of a file, for a model of the first lessons on the last ones and for a model of all lessons on the read part"));
    lines.push(format!("know: facts read from the text of an encyclopedia and asked back: the model from the lesson facts, the text read into its table, the asked facts answered from the table"));
    lines.push(format!("questions: the reader of questions that people wrote about facts: learned from the lesson questions of a file, read on its held questions (the subject, the relation, both)"));
    lines.push(format!("ask: one command for a request (a told story with a question, a question about a fact, a request to a service), with a trace of every step; build makes its table, measure answers the held questions about facts, kinds measures where requests go"));
    lines.push(format!("patterns: what the pattern learner counted for a kind, stage by stage: its words and kinds, its model, its rules, its answers"));
    lines.push(format!("control: a model reads changed forms of the lessons that it answers right; how often it gives the new true answer, where the truth turns and where it stays"));
    lines.push(format!("cases: the number of cases of a case file and the named values of its first line"));
    lines.push(format!("words: the words of a text as every learner counts them; answer-words: the words of an answer as the measure compares them"));
    lines.push(format!("fetch: the raw files of the sets from their public sources, each kept only with the right SHA-256"));
    lines.push(format!("export: the case files of a set from its raw files, as its description in data/sets names them"));
    lines.push(format!("store: the text store of the fact reader from a copy of an encyclopedia"));
    lines.push(format!("classes: the table of word classes from a copy of an encyclopedia"));
    lines.push(format!("write: the writer from lists of facts to texts, and its measures"));
    lines.push(format!("changes: the changed cases of a kind, for a control; made: the case files of the made families"));
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

const NUMBER_OPTIONS: &[&str] = &[FIT_OPTION, HELD_OPTION, CURVE_OPTION, WORDS_OPTION, SHOW_OPTION, SMALL_OPTION, SEALED_OPTION, LANES_OPTION, MOST_OPTION, WHOLE_OPTION, WRITE_OPTION, "families", "bytes", "files", "others", "run", "words"];

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
    }
}
because!(asked_of, LearnerCommand, "what a run asks of the measure: its options, and for what it does not name the values of the set");

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
    measure(&Ground { name: kind, cases: &loaded.kind.cases, labels: &loaded.kind.labels, start: loaded.start, stories: loaded.stories, final_part: last_part.as_deref(), sealed: None, new_words: table.as_deref(), answerer: None, classes }, tools, asked)
}
because!(measure_kind, LearnerCommand, "one kind of a set through the measure, with its table of new words and its final part when the run asks for them");

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
    let row = measure(&Ground { name: &name, cases: &loaded.kind.cases, labels: &loaded.kind.labels, start: loaded.start, stories: loaded.stories, final_part: None, sealed: None, new_words: None, answerer: None, classes: classes.as_ref() }, &tools, &asked);
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
    let asked = Asked { new_words: true, curve: false, show: 0, ..asked_of(run, 0, 0, None) };
    let made = board(&sources, &tools, &asked, classes.as_ref());
    println!("{}", made.text.join("\n"));
    if let Some(out) = run.named.get(OUT_OPTION) {
        let written = std::fs::File::create(out).and_then(|mut file| file.write_all((made.text.join("\n") + "\n").as_bytes()));
        if let Err(error) = written {
            return failed(out, &error);
        }
    }
    ExitCode::SUCCESS
}
because!(boarded, LearnerCommand, "the board of all sets of a folder, or of the named ones in the order of their names; a folder that says nothing of a set is no set and is passed over");

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

const NOT_TRIED: &str = "not tried: it reads texts for the learner of another tool, and that learner does not lead";
because!(NOT_TRIED, LearnerCommand, "what the choice prints for a tool that the choice leaves out, because it runs the learner of another tool that another tool reads better");

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
    match take(&tools, Lessons { cases: &file.learn, labels: &file.labels, ask: None, classes: classes.as_ref() }, lanes_of(run), true) {
        Ok(taken) => {
            for (place, tool) in tools.iter().enumerate() {
                if taken.said[place].is_empty() && taken.shares[place] < f64::default() {
                    println!("tool {} {NOT_TRIED}", tool.name);
                } else if taken.said[place].is_empty() {
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

fn file_lines(path: &str) -> std::io::Result<Vec<String>> {
    BufReader::new(std::fs::File::open(path)?).lines().collect()
}
because!(file_lines, LearnerCommand, "the lines of a file of answers: one answer on a line, its words with one space between them, an empty line for no answer");

fn same_answers(model: &dyn Learner, cases: &[Case], theirs: &[String], lanes: usize, show: usize) -> (usize, Vec<String>) {
    let said: Vec<(String, String)> = on_lanes(cases.len(), lanes, &|place| {
        let said = model.answer(&cases[place].sentences, &cases[place].question);
        (said.words.join(BETWEEN_WORDS), said.note)
    });
    let mut same = 0usize;
    let mut different: Vec<String> = Vec::new();
    for (place, (mine, note)) in said.iter().enumerate() {
        if theirs.get(place) == Some(mine) {
            same += 1;
        } else if different.len() < show {
            let case = &cases[place];
            different.push(format!("      DIFFERENT (lesson case {place}): {} | {} -> {}; the file says: {}; said: {mine} ({note})", case.sentences.join(" / "), case.question, case.answer, theirs.get(place).map_or("", String::as_str)));
        }
    }
    (same, different)
}
because!(same_answers, LearnerCommand, "the answers of a model beside the answers of a file, case by case: how many are the same, and the first cases that differ when the run may print them, each with the note of the model that tells how it found its answer");

fn beside(part: &str, model: &Result<Box<dyn Learner>, String>, cases: &[Case], file: &str, lanes: usize, show: usize) -> (bool, Vec<String>) {
    match (model, file_lines(file)) {
        (Ok(model), Ok(theirs)) => {
            let (same, different) = same_answers(model.as_ref(), cases, &theirs, lanes, show);
            let mut lines = vec![format!("{part}: the same answer in {same} of {} cases (the file holds {} answers)", cases.len(), theirs.len())];
            lines.extend(different);
            (same == cases.len() && theirs.len() == cases.len(), lines)
        }
        (Err(why), Ok(theirs)) => (false, vec![format!("{part}: no model ({why}); the file holds {} answers", theirs.len())]),
        (Ok(_), Err(_)) => (false, vec![format!("{part}: a model, and no file of answers")]),
        (Err(why), Err(_)) => (true, vec![format!("{part}: no model ({why}), and no file of answers")]),
    }
}
because!(beside, LearnerCommand, "one model beside one file of answers, with the lines that say what was found; a tool that gives no model is the same as a file that is not there");

struct AnswerFiles {
    inner: String,
    read: String,
    new_inner: Option<String>,
    new_read: Option<String>,
    table: Option<Vec<(String, String)>>,
}
because!(AnswerFiles, LearnerCommand, "the files of answers that a tool is checked against for one kind: of the model of the first lessons on the last lessons, of the model of all lessons on the read part, the same two on the cases renamed with the new words of the kind when they are asked, and the table of new words of the kind");

type ToolPart<'a> = (&'static str, &'a Result<Box<dyn Learner>, String>, Vec<Case>, Option<String>, usize);
because!(ToolPart, LearnerCommand, "one part of the check of a tool on a kind: its name, the model that reads it, its cases, the file of answers that it is held against, and how many differing cases may be shown");

fn tool_models(file: &Opened, tool: &Tool, classes: Option<&WordClasses>, lanes: usize) -> (Result<Box<dyn Learner>, String>, Result<Box<dyn Learner>, String>, Vec<Case>) {
    let (first, last) = inner_parts(&file.learn);
    let lessons = Lessons { cases: &file.learn, labels: &file.labels, ask: None, classes };
    let first_cases = picked(&file.learn, &first);
    let first_labels = picked_labels(&file.labels, &first);
    let needs = tool.needs_labels && !lessons.labelled();
    let inner = if needs { Err(NO_LABELS.to_string()) } else { (tool.learn)(lessons.part(&first_cases, &first_labels), lanes) };
    let whole = if needs { Err(NO_LABELS.to_string()) } else { (tool.learn)(lessons, lanes) };
    (inner, whole, picked(&file.learn, &last))
}
because!(tool_models, LearnerCommand, "the two models of the check of a tool on a kind: one of the first lessons, one of all lessons, with the last lessons that the first model reads");

fn tool_parts<'a>(file: &Opened, models: &'a (Result<Box<dyn Learner>, String>, Result<Box<dyn Learner>, String>, Vec<Case>), answers: &AnswerFiles, show: usize) -> Vec<ToolPart<'a>> {
    let (inner, whole, last_cases) = models;
    let mut parts: Vec<ToolPart<'a>> = vec![("last lessons", inner, last_cases.clone(), Some(answers.inner.clone()), show), ("read part", whole, file.read.clone(), Some(answers.read.clone()), 0)];
    for (part, model, cases, their_file, shown) in [("last lessons with new words", inner, last_cases, &answers.new_inner, show), ("read part with new words", whole, &file.read, &answers.new_read, 0)] {
        if their_file.is_some() {
            let renamed = answers.table.as_ref().map(|table| renamed_all(cases, table)).unwrap_or_default();
            parts.push((part, model, renamed, their_file.clone().filter(|_| answers.table.is_some()), shown));
        }
    }
    parts
}
because!(tool_parts, LearnerCommand, "the parts of the check of a tool on a kind in their order: the model of the first lessons on the last lessons, the model of all lessons on the read part, and the same two on the cases renamed with the new words of the kind when their files are named; a renamed part of a kind with no table of new words has no file");

fn tool_beside(file: &Opened, tool: &Tool, classes: Option<&WordClasses>, answers: &AnswerFiles, lanes: usize, show: usize) -> (bool, Vec<String>) {
    let models = tool_models(file, tool, classes, lanes);
    let mut same = true;
    let mut lines: Vec<String> = Vec::new();
    for (part, model, cases, their_file, shown) in tool_parts(file, &models, answers, show) {
        let (same_part, part_lines) = match their_file {
            Some(their_file) => beside(part, model, &cases, &their_file, lanes, shown),
            None => (false, vec![format!("{part}: the kind has no table of new words")]),
        };
        same &= same_part;
        lines.extend(part_lines);
    }
    (same, lines)
}
because!(tool_beside, LearnerCommand, "the answers of one tool on one kind beside its files of answers: a model of the first lessons on the last lessons, a model of all lessons on the read part, and when they are asked the same two models on the cases renamed with the new words of the kind; differing cases are shown for the lessons only, because no case of a read part is shown to the one who designs a tool");

fn tool_written(file: &Opened, tool: &Tool, classes: Option<&WordClasses>, answers: &AnswerFiles, lanes: usize) -> (bool, Vec<String>) {
    let models = tool_models(file, tool, classes, lanes);
    let mut lines: Vec<String> = Vec::new();
    let mut done = true;
    for (part, model, cases, their_file, _) in tool_parts(file, &models, answers, 0) {
        let (Ok(model), Some(their_file)) = (model, their_file) else {
            lines.push(format!("{part}: no model or no file, nothing written"));
            continue;
        };
        let said: Vec<String> = on_lanes(cases.len(), lanes, &|place| model.answer(&cases[place].sentences, &cases[place].question).words.join(BETWEEN_WORDS));
        let written = std::fs::write(&their_file, said.iter().map(|line| format!("{line}\n")).collect::<String>());
        done &= written.is_ok();
        lines.push(format!("{part}: {} answers written", said.len()));
    }
    (done, lines)
}
because!(tool_written, LearnerCommand, "the files of answers of one tool on one kind written by the present program, part by part as the check reads them, for a kind whose cases changed on purpose; a part with no model gets no file, as the check holds no model and no file the same");

fn renamed_table(place: &Path) -> Option<Vec<(String, String)>> {
    let from = read_source(place.parent()?).ok()?;
    new_words(&from, &place.file_stem()?.to_string_lossy())
}
because!(renamed_table, LearnerCommand, "the table of new words of the kind of a case file, from the folder of its set; nothing when the kind has none");

fn renamed_all(cases: &[Case], pairs: &[(String, String)]) -> Vec<Case> {
    let table: HashMap<&str, &str> = pairs.iter().map(|(old, new)| (old.as_str(), new.as_str())).collect();
    cases.iter().map(|case| Case { sentences: case.sentences.iter().map(|sentence| renamed(sentence, &table)).collect(), question: renamed(&case.question, &table), answer: renamed(&case.answer, &table) }).collect()
}
because!(renamed_all, LearnerCommand, "cases with every word of the table replaced by its new word in the story, the question and the answer, as the measure renames the read cases");

fn checked(run: &Given) -> ExitCode {
    let [place, name, inner_file, read_file] = run.plain.as_slice() else { return usage() };
    let file = match opened(place, run) {
        Ok(file) => file,
        Err(error) => return failed(place, &error),
    };
    let tool = match named_tools(name) {
        Ok(found) => found[0],
        Err(why) => return failed(name, &why),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let new_inner = run.named.get(NEW_INNER_OPTION).cloned();
    let new_read = run.named.get(NEW_READ_OPTION).cloned();
    let table = if new_inner.is_some() || new_read.is_some() { renamed_table(Path::new(place)) } else { None };
    let answers = AnswerFiles { inner: inner_file.clone(), read: read_file.clone(), new_inner, new_read, table };
    let (same, lines) = tool_beside(&file, &tool, classes.as_ref(), &answers, lanes_of(run), number(run, SHOW_OPTION).unwrap_or(SHOWN_DIFFERENT));
    lines.iter().for_each(|line| println!("{line}"));
    if same {
        println!("THE SAME");
        ExitCode::SUCCESS
    } else {
        println!("NOT THE SAME");
        ExitCode::FAILURE
    }
}
because!(checked, LearnerCommand, "the answers of one tool on one case file beside two files of answers, with what was found and a last line that says whether all are the same");

const ABOUT_KIND: &str = "about.txt";
because!(ABOUT_KIND, LearnerCommand, "the file that marks a folder of a folder of answers as the answers of one kind");

const INNER_END: &str = ".inner.txt";
because!(INNER_END, LearnerCommand, "the end of the name of a file of the answers of a model of the first lessons on the last lessons");

const READ_END: &str = ".read.txt";
because!(READ_END, LearnerCommand, "the end of the name of a file of the answers of a model of all lessons on the read part");

const NEW_INNER_OPTION: &str = "newinner";
because!(NEW_INNER_OPTION, LearnerCommand, "the option of the check that names a file of answers of the model of the first lessons on the last lessons renamed with the new words of the kind, and the name of that part in a file of the parts with new words");

const NEW_READ_OPTION: &str = "newread";
because!(NEW_READ_OPTION, LearnerCommand, "the option of the check that names a file of answers of the model of all lessons on the read part renamed with the new words of the kind, and the name of that part in a file of the parts with new words");

const NEW_INNER_END: &str = ".newinner.txt";
because!(NEW_INNER_END, LearnerCommand, "the end of the name of a file of answers of the model of the first lessons on the last lessons renamed with new words");

const NEW_READ_END: &str = ".newread.txt";
because!(NEW_READ_END, LearnerCommand, "the end of the name of a file of answers of the model of all lessons on the read part renamed with new words");

const NEW_WORDS_ABOUT: &str = "newwords.txt";
because!(NEW_WORDS_ABOUT, LearnerCommand, "the file of a folder of answers of one kind that names the tools and parts whose answers with new words were written");

const TOOL_LINE: &str = "tool";
because!(TOOL_LINE, LearnerCommand, "the first field of a line of the file of parts with new words that names a tool and one of its parts");

fn new_word_parts(folder: &Path, name: &str) -> HashSet<String> {
    let Ok(text) = std::fs::read_to_string(folder.join(NEW_WORDS_ABOUT)) else { return HashSet::new() };
    text.lines().filter_map(|line| match line.split(FIELD_MARK).collect::<Vec<&str>>().as_slice() {
        [mark, tool, part, ..] if *mark == TOOL_LINE && *tool == name => Some(part.to_string()),
        _ => None,
    }).collect()
}
because!(new_word_parts, LearnerCommand, "the parts with new words whose answers were written for one tool and one kind: a part is checked only where its answers were written");

const CASE_FILE_END: &str = ".txt";
because!(CASE_FILE_END, LearnerCommand, "a case file is plain text named after its kind, so the folder of a set is searched for the kind with this end and no other file is taken for it");

const CASES_OPTION: &str = "cases";
because!(CASES_OPTION, LearnerCommand, "the option that names the folders of sets where the case files of the kinds lie; the first folder that holds the file of a kind is taken");

const WRITE_OPTION: &str = "write";
because!(WRITE_OPTION, LearnerCommand, "the option of check-all that writes the files of answers of the present program in place of comparing them, for the sets that the option sources names");

const WRITTEN_WORD: &str = "WRITTEN";
because!(WRITTEN_WORD, LearnerCommand, "the word of the count line of check-all when the files of answers were written");

const SAME_WORD: &str = "THE SAME";
because!(SAME_WORD, LearnerCommand, "the word of the count line of check-all when the answers of every part of a kind are the answers of its files");

const SOURCES_OPTION: &str = "sources";
because!(SOURCES_OPTION, LearnerCommand, "the option that names the sets whose kinds are checked; with none, every set of the folder of answers");

fn folders_in(folder: &Path) -> std::io::Result<Vec<String>> {
    let mut names: Vec<String> = std::fs::read_dir(folder)?.filter_map(Result::ok).filter(|entry| entry.path().is_dir()).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
    names.sort_unstable();
    Ok(names)
}
because!(folders_in, LearnerCommand, "the names of the folders in a folder, in the order of their bytes, so that a run goes through them in one order on every machine");

fn checked_all(run: &Given) -> ExitCode {
    let [golden, name, tool_name] = run.plain.as_slice() else { return usage() };
    let tool = match named_tools(tool_name) {
        Ok(found) => found[0],
        Err(why) => return failed(tool_name, &why),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let roots: Vec<PathBuf> = run.named.get(CASES_OPTION).map(|text| text.split(BETWEEN_KINDS).map(PathBuf::from).collect()).unwrap_or_default();
    let wanted: Option<HashSet<&str>> = run.named.get(SOURCES_OPTION).map(|text| text.split(BETWEEN_KINDS).collect());
    let write = is_on(run, WRITE_OPTION);
    if write && wanted.is_none() {
        return failed(WRITE_OPTION, &"the files of answers are written only for the sets that the option sources names");
    }
    let golden = Path::new(golden);
    let sources = match folders_in(golden) {
        Ok(sources) => sources,
        Err(error) => return failed(&golden.to_string_lossy(), &error),
    };
    let mut items: Vec<(String, String)> = Vec::new();
    for source in sources.into_iter().filter(|source| wanted.as_ref().is_none_or(|wanted| wanted.contains(source.as_str()))) {
        let kinds = folders_in(&golden.join(&source)).unwrap_or_default();
        items.extend(kinds.into_iter().filter(|kind| golden.join(&source).join(kind).join(ABOUT_KIND).exists()).map(|kind| (source.clone(), kind)));
    }
    let results: Vec<(bool, Vec<String>)> = on_lanes(items.len(), lanes_of(run), &|at| {
        let (source, kind) = &items[at];
        let Some(place) = roots.iter().map(|root| root.join(source).join(format!("{kind}{CASE_FILE_END}"))).find(|path| path.exists()) else {
            return (false, vec![format!("no case file of {kind} in the folders of the option {CASES_OPTION}")]);
        };
        let folder = golden.join(source).join(kind);
        let file_of = |end: &str| folder.join(format!("{name}{end}")).to_string_lossy().into_owned();
        let parts = new_word_parts(&folder, name);
        let (new_inner, new_read) = (parts.contains(NEW_INNER_OPTION).then(|| file_of(NEW_INNER_END)), parts.contains(NEW_READ_OPTION).then(|| file_of(NEW_READ_END)));
        let table = if new_inner.is_some() || new_read.is_some() { renamed_table(&place) } else { None };
        let answers = AnswerFiles { inner: file_of(INNER_END), read: file_of(READ_END), new_inner, new_read, table };
        match cut_file(&place, None, None) {
            Ok(file) if write => tool_written(&file, &tool, classes.as_ref(), &answers, 1),
            Ok(file) => tool_beside(&file, &tool, classes.as_ref(), &answers, 1, SHOWN_DIFFERENT),
            Err(error) => (false, vec![error.to_string()]),
        }
    });
    let same = results.iter().filter(|(same, _)| *same).count();
    println!("tool {name} as {}: {same} of {} kinds {}", tool.name, items.len(), if write { WRITTEN_WORD } else { SAME_WORD });
    for ((source, kind), (_, lines)) in items.iter().zip(&results).filter(|(_, (same, _))| !*same) {
        println!("  {source} {kind}");
        lines.iter().take(SHOWN_DIFFERENT).for_each(|line| println!("      {line}"));
    }
    if same == items.len() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
because!(checked_all, LearnerCommand, "the answers of one tool beside the files of answers of every kind of a folder of answers, the kinds side by side on the threads and each on one thread, with a line for the count of kinds that are the same and the first lines of each kind that is not");

const FIELD_MARK: char = '\t';
because!(FIELD_MARK, LearnerCommand, "the sign between the fields of a line of a file of plain lines");


fn put_lines(folder: &Path, name: &str, mut lines: Vec<String>, sorted: bool) -> std::io::Result<()> {
    if sorted {
        lines.sort_unstable();
    }
    lines.push(String::new());
    std::fs::write(folder.join(name), lines.join("\n"))
}
because!(put_lines, LearnerCommand, "some lines written to a file of a folder, in their order or sorted, so that the file does not depend on the order of a table in memory");

fn entry_line(model: &Knowledge, title: &str, relation: u32, entry: &Entry) -> String {
    format!("{}\t{}\t{}\t{:x}\t{}\t{}", one_line(title), model.frames.relations.name(relation), one_line(&entry.answer), entry.chance.to_bits(), model.frames.frame_text(&entry.frame), one_line(&entry.sentence))
}
because!(entry_line, LearnerCommand, "one entry of the table as a line: the title, the relation, the answer, the bits of the chance, the frame and the sentence");

fn dump_model(folder: &Path, model: &Knowledge, asked: &[&ilm_core::know::store::Fact]) -> std::io::Result<()> {
    std::fs::create_dir_all(folder)?;
    put_lines(folder, "share.txt", model.frames.kept_lines(), true)?;
    let sure = model.sure.iter().flat_map(|((word, step), rows)| rows.iter().map(|row| format!("{}\t{}\t{}\t{}\t{}\t{}", model.frames.words.name(*word), OFTEN_NAMES[*step as usize], model.frames.relations.name(row.relation), one_line(&row.object), row.hit, row.seen))).collect();
    put_lines(folder, "sure.txt", sure, true)?;
    let routes = model
        .words_first
        .iter()
        .map(|(relation, first)| {
            let listed = model.listed_first.get(relation).copied().unwrap_or(false);
            let (after_sentence, after_words) = model.listed_counts.get(relation).copied().unwrap_or((0, 0));
            format!("{relation}\t{}\t{}\t{}\t{}\t{after_sentence}\t{after_words}", u32::from(*first), model.route_counts[relation].0, model.route_counts[relation].1, u32::from(listed))
        })
        .collect();
    put_lines(folder, "routes.txt", routes, true)?;
    let levels = model.levels.iter().map(|(level, right, answers)| format!("{}\t{}\t{right}\t{answers}", if level.listed { "listed" } else if level.words { "words" } else { "read" }, level.level)).collect();
    put_lines(folder, "levels.txt", levels, true)?;
    let listed = model.listed.iter().flat_map(|(relation, objects)| objects.iter().map(move |object| format!("{}\t{}\t{}\t{}\t{}", model.frames.relations.name(*relation), one_line(&object.written), object.lessons, object.hit, object.seen))).collect();
    put_lines(folder, "listed.txt", listed, true)?;
    let table = model.by_title.iter().flat_map(|(title, entries)| entries.iter().map(|(relation, entry)| entry_line(model, title, *relation, entry))).collect();
    put_lines(folder, "table.txt", table, true)?;
    let answers = asked
        .iter()
        .map(|fact| {
            let got = model.answer(&fact.relation, &fact.subject);
            format!("{}\t{}\t{}\t{}", fact.relation, one_line(&fact.subject), got.map_or(String::new(), |(entry, _)| one_line(&entry.answer)), got.map_or(String::new(), |(_, title)| one_line(title)))
        })
        .collect();
    put_lines(folder, "answers.txt", answers, false)?;
    let numbers = level_of_all(model);
    put_lines(folder, "entries.txt", vec![numbers], false)
}
because!(dump_model, LearnerCommand, "the tables of a model of knowledge as files of plain lines: the kept frames, the sure words, the choice of the routes, the levels of the chance, the table of what was read, and the answer of each asked fact");

fn level_of_all(model: &Knowledge) -> String {
    let entries: usize = model.by_title.values().map(Vec::len).sum();
    let high = model.by_title.values().flatten().filter(|(_, entry)| level_of(entry).level > 0).count();
    format!("{} articles, {entries} entries, {high} of them above the lowest level of the chance", model.by_title.len())
}
because!(level_of_all, LearnerCommand, "the size of the table of a model in one line");

fn share_of(count: usize, of: usize) -> f64 {
    share(count, of.max(1))
}
because!(share_of, LearnerCommand, "a count as hundredths of a whole, and nothing of an empty whole");

fn print_numbers(numbers: &Numbers) {
    println!("lesson facts | right, % of all asked facts | answered, % | answers with their source | forget | text read | entries | seconds: model, read, answers");
    println!(
        "{:>count$} | {:>part$.places$} | {:>part$.places$} | {} of {} | {} of {} | {} articles, {:.none$} M signs | {} | {:.places$}, {:.places$}, {:.places$}",
        numbers.lessons,
        share_of(numbers.right, numbers.asked),
        share_of(numbers.answered, numbers.asked),
        numbers.sourced,
        numbers.answered,
        numbers.gone,
        numbers.forgotten,
        numbers.articles,
        (numbers.signs as f64) / MILLION,
        numbers.entries,
        numbers.model_seconds,
        numbers.read_seconds,
        numbers.answer_seconds,
        count = KNOW_COUNT,
        part = KNOW_SHARE,
        places = 1,
        none = 0
    );
    if numbers.inferred > 0 {
        println!("    answers that no sentence states (given by sure words of the article): {}", numbers.inferred);
    }
    if numbers.guessed > 0 {
        println!(
            "    facts with no answer in the text, guessed from the subject's name: {}, right {}; right with the guesses, % of all asked facts: {:.places$}",
            numbers.guessed,
            numbers.guessed_right,
            share_of(numbers.right + numbers.guessed_right, numbers.asked),
            places = 1
        );
    }
    let mut worst: Vec<&(String, usize, usize)> = numbers.by_relation.iter().collect();
    worst.sort_by(|one, other| (one.1 as f64 / one.2.max(1) as f64).total_cmp(&(other.1 as f64 / other.2.max(1) as f64)));
    let named = |rows: &[&(String, usize, usize)]| rows.iter().map(|(relation, right, of)| format!("{relation} {:.none$}", share_of(*right, *of), none = 0)).collect::<Vec<String>>().join(", ");
    println!("    lowest relations: {} | highest: {}", named(&worst[..worst.len().min(SHOWN_RELATIONS)]), named(&worst[worst.len().saturating_sub(SHOWN_RELATIONS)..]));
}
because!(print_numbers, LearnerCommand, "the numbers of a run of the probe: the line of the shares, and the relations with the lowest and the highest share");

const KNOW_COUNT: usize = 6;
because!(KNOW_COUNT, LearnerCommand, "the signs of the column of the lesson facts in the line of a probe");

const KNOW_SHARE: usize = 5;
because!(KNOW_SHARE, LearnerCommand, "the signs of a column of a share in the line of a probe");

fn known(run: &Given) -> ExitCode {
    let [store, facts_file] = run.plain.as_slice() else { return usage() };
    let store = Path::new(store);
    let facts = match read_facts(Path::new(facts_file)) {
        Ok(facts) => facts,
        Err(error) => return failed(facts_file, &error),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes.unwrap_or_default(),
        Err(code) => return code,
    };
    let on_probe = run.named.get(PART_OPTION).is_some_and(|part| part == PROBE_PART);
    let (lessons, asked) = if on_probe { whole(&facts) } else { development(&facts) };
    let which = match run.named.get(TEXT_OPTION).map(String::as_str) {
        Some(TEXT_ASKED) => Which::Asked,
        Some(TEXT_ALL) => Which::All,
        _ => Which::Subjects,
    };
    let texts = match Texts::read(store) {
        Ok(texts) => texts,
        Err(error) => return failed(&store.to_string_lossy(), &error),
    };
    let reference = references(&lessons, &asked, &texts);
    let relations: std::collections::HashSet<&str> = lessons.iter().map(|fact| fact.relation.as_str()).collect();
    println!(
        "part {}: {} lesson facts, {} facts asked, {} relations; the subject of {:.places$}% of the asked facts has one article in the store (by its title, or the only one of its name)",
        if on_probe { PROBE_PART } else { "development" },
        lessons.len(),
        asked.len(),
        relations.len(),
        reference.with_article,
        places = 1
    );
    println!(
        "reference points (share of all asked facts): one answer for a relation {:.places$} | the first word of the article that is an object of the relation {:.places$} | the object stands in the article {:.places$}",
        reference.one_answer,
        reference.first_word,
        reference.in_article,
        places = 1
    );
    let dump = run.named.get(DUMP_OPTION).map(Path::new);
    let mut written: std::io::Result<()> = Ok(());
    let mut look = |model: &Knowledge| {
        if let Some(folder) = dump {
            written = dump_model(folder, model, &asked);
        }
    };
    match probe(&lessons, &asked, &texts, store, which, number(run, FIT_OPTION).unwrap_or(0), &classes, lanes_of(run), &mut look) {
        Ok((_, numbers)) => {
            print_numbers(&numbers);
            match written {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => failed(DUMP_OPTION, &error),
            }
        }
        Err(error) => failed(&store.to_string_lossy(), &error),
    }
}
because!(known, LearnerCommand, "the probe of knowledge: the lesson facts and the asked facts of a file, the text of a store, one model, and the numbers of the run; the development part is asked unless the run names the probe");

fn asked_questions(run: &Given) -> ExitCode {
    let [file] = run.plain.as_slice() else { return usage() };
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) => return failed(file, &error),
    };
    let rows: Vec<Vec<&str>> = text.lines().map(|line| line.split(FIELD_MARK).collect::<Vec<&str>>()).filter(|fields| fields.len() >= QUESTION_FIELDS).collect();
    let asking = |held: bool| rows.iter().filter(|fields| (fields[0] == HELD_PART) == held).map(|fields| Asking { question: fields[1], subject: fields[QUESTION_FIELDS - 1 - 1], relation: fields[QUESTION_FIELDS - 1] }).collect::<Vec<Asking>>();
    let (mut lessons, held) = (asking(false), asking(true));
    if let Some(fit) = number(run, FIT_OPTION).filter(|fit| *fit > 0 && *fit < lessons.len()) {
        lessons.truncate(fit);
    }
    let known = match run.named.get(NAMES_OPTION).map(|path| SortedNames::read(Path::new(path))) {
        Some(Ok(known)) => Some(known),
        Some(Err(error)) => return failed(NAMES_OPTION, &error),
        None => None,
    };
    let names: Option<&dyn Names> = known.as_ref().map(|known| known as &dyn Names);
    let relations: std::collections::HashSet<&str> = lessons.iter().map(|lesson| lesson.relation).collect();
    println!("{} lesson questions, {} held questions, {} relations{}", lessons.len(), held.len(), relations.len(), known.as_ref().map_or(String::new(), |known| format!("; {} known names", known.count)));
    let model = Questions::learn(&lessons, names, true, lanes_of(run));
    let shares = held_shares(&model, &held, names);
    println!("lessons | lessons with the subject in the question | subject right, % | relation right, % | both right, %");
    println!("{:>wide$} | {:>wide$} | {:>part$.places$} | {:>part$.places$} | {:>part$.places$}", lessons.len(), model.used, shares.subject, shares.relation, shares.both, wide = KNOW_COUNT + 1, part = KNOW_SHARE, places = 1);
    if let Some(folder) = run.named.get(DUMP_OPTION).map(Path::new) {
        let read = held
            .iter()
            .map(|case| match model.read(case.question, names) {
                None => String::new(),
                Some(got) => format!("{}\t{}\t{}\t{}\t{}", one_line(&got.subject), model.relation_of(&got).unwrap_or("None"), Questions::way_text(got.way), got.sure.0, got.sure.1),
            })
            .collect();
        let ways = model.ways.iter().map(|(way, (right, all))| format!("{}\t{right}\t{all}", Questions::way_text(Some(*way)))).collect();
        let written = std::fs::create_dir_all(folder).and_then(|()| put_lines(folder, "read.txt", read, false)).and_then(|()| put_lines(folder, "ways.txt", ways, true));
        if let Err(error) = written {
            return failed(DUMP_OPTION, &error);
        }
    }
    ExitCode::SUCCESS
}
because!(asked_questions, LearnerCommand, "the reader of questions on the held questions of a file: learned from the lesson questions, with known names when a file of them is given");

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

fn ask_build(run: &Given) -> ExitCode {
    let [_, store, facts, questions, names, sets, board] = run.plain.as_slice() else { return usage() };
    let classes = match classes_of(run) {
        Ok(classes) => classes.unwrap_or_default(),
        Err(code) => return code,
    };
    let folder = run.named.get(TABLE_OPTION).map_or_else(|| Path::new(store).join(TABLE_FOLDER), std::path::PathBuf::from);
    let cases = match read_questions(Path::new(questions)) {
        Ok(cases) => cases,
        Err(error) => return failed(questions, &error),
    };
    let started = std::time::Instant::now();
    let built = match build_facts(Path::new(store), Path::new(facts), &cases, Path::new(names), &classes, lanes_of(run), &folder) {
        Ok(built) => built,
        Err(error) => return failed(store, &error),
    };
    println!(
        "the table of facts: {} lesson facts, {} articles read ({:.none$} M signs), {} articles with {} facts in the table; {:.none$} s",
        built.numbers.lessons,
        built.numbers.articles,
        (built.numbers.signs as f64) / MILLION,
        built.articles,
        built.facts,
        started.elapsed().as_secs_f64(),
        none = 0
    );
    match build_router(Path::new(sets), Path::new(board), &cases, &folder) {
        Ok((kinds, counted)) => {
            println!("the router: {kinds} kinds, {counted} of them kinds of the counted tasks; written to {}", folder.to_string_lossy());
            ExitCode::SUCCESS
        }
        Err(error) => failed(sets, &error),
    }
}
because!(ask_build, LearnerCommand, "the table of the one command: the table of facts from the whole store, and the router from the counted kinds of a board, the lesson questions and the sets of requests");

fn ask_measure(run: &Given) -> ExitCode {
    let [_, folder, questions] = run.plain.as_slice() else { return usage() };
    let cases = match read_questions(Path::new(questions)) {
        Ok(cases) => cases,
        Err(error) => return failed(questions, &error),
    };
    let whole = match Whole::open(Path::new(folder), None, lanes_of(run)) {
        Ok(whole) => whole,
        Err(error) => return failed(folder, &error),
    };
    let (counts, lines) = match held_answers(&whole.facts, &cases) {
        Ok(found) => found,
        Err(error) => return failed(folder, &error),
    };
    let rows = [
        (counts.held, "held questions"),
        (counts.read_right, "read right (subject and relation)"),
        (counts.given, "an answer is given"),
        (counts.right, "the answer is the answer of the set"),
        (counts.one_word, "questions whose answer is one word"),
        (counts.one_read_right, "of these: read right"),
        (counts.one_given, "of these: an answer is given"),
        (counts.one_right, "of these: the answer is right"),
        (counts.one_both, "of these: read right and the answer is right"),
    ];
    println!("held questions about facts");
    for (count, name) in rows {
        println!("   {count:>wide$}  {name}", wide = KNOW_SHARE);
    }
    println!(
        "answer right: {:.places$}% of all held questions; {:.places$}% of those with an answer of one word; {:.places$} ms for a question",
        share_of(counts.right, counts.held),
        share_of(counts.one_right, counts.one_word),
        counts.seconds * MILLISECONDS / (counts.held.max(1) as f64),
        places = 1
    );
    if let Some(out) = run.named.get(DUMP_OPTION) {
        if let Err(error) = std::fs::write(out, lines.join("\n") + "\n") {
            return failed(out, &error);
        }
    }
    ExitCode::SUCCESS
}
because!(ask_measure, LearnerCommand, "the held questions about facts through the one command: how many are read right, how many get an answer, how many get the answer of the set; with dump, the answer of each question as a line");

fn ask_kinds(run: &Given) -> ExitCode {
    let [_, folder, questions] = run.plain.as_slice() else { return usage() };
    let cases = match read_questions(Path::new(questions)) {
        Ok(cases) => cases,
        Err(error) => return failed(questions, &error),
    };
    let whole = match Whole::open(Path::new(folder), None, lanes_of(run)) {
        Ok(whole) => whole,
        Err(error) => return failed(folder, &error),
    };
    match whole.routes(&cases) {
        Ok(lines) => {
            println!("{}", lines.join("\n"));
            ExitCode::SUCCESS
        }
        Err(error) => failed(folder, &error),
    }
}
because!(ask_kinds, LearnerCommand, "where the requests of known kinds go: the held questions about facts, the read cases of the counted kinds, and the held requests of the sets of requests");

fn answered(whole: &Whole, folder: &str, request: &str) -> Result<(), ExitCode> {
    match whole.answer(request) {
        Ok((said, trace)) => {
            println!("{}", said.unwrap_or_else(|| "No answer: see the trace.".to_string()));
            for line in trace {
                println!("   {line}");
            }
            Ok(())
        }
        Err(error) => Err(failed(folder, &error)),
    }
}
because!(answered, LearnerCommand, "one request through the one command: the answer, or the words that there is none, and the trace of every step");

fn asked_once(run: &Given) -> ExitCode {
    let (folder, request) = match run.plain.as_slice() {
        [folder, request] => (folder, Some(request)),
        [folder] => (folder, None),
        _ => return usage(),
    };
    let classes = match classes_of(run) {
        Ok(classes) => classes,
        Err(code) => return code,
    };
    let whole = match Whole::open(Path::new(folder), classes, lanes_of(run)) {
        Ok(whole) => whole,
        Err(error) => return failed(folder, &error),
    };
    if let Some(request) = request {
        return answered(&whole, folder, request).map_or_else(|code| code, |()| ExitCode::SUCCESS);
    }
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Err(code) = answered(&whole, folder, line.trim()) {
            return code;
        }
        println!();
        if std::io::stdout().flush().is_err() {
            break;
        }
    }
    ExitCode::SUCCESS
}
because!(asked_once, LearnerCommand, "the one command on its table: one request given with the command, or with no request a session that opens the table once and answers each line of the input, an empty line after each answer");

fn asked(run: &Given) -> ExitCode {
    match run.plain.first().map(String::as_str) {
        Some("build") => ask_build(run),
        Some("measure") => ask_measure(run),
        Some("kinds") => ask_kinds(run),
        _ => asked_once(run),
    }
}
because!(asked, LearnerCommand, "the parts of the one command: the building of its table, its two measures, and one request");

pub fn run_command() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((part, rest)) = args.split_first() else { return usage() };
    let run = given(rest);
    if let Some(fault) = bad_number(&run).filter(|_| !TEXT_COMMANDS.contains(&part.as_str())) {
        eprintln!("{fault}");
        return ExitCode::FAILURE;
    }
    match part.as_str() {
        "board" => boarded(&run),
        "measure" => measured(&run),
        "choice" => chosen(&run),
        "check" => checked(&run),
        "check-all" => checked_all(&run),
        "know" => known(&run),
        "ask" => asked(&run),
        "control" => controlled(&run),
        "changes" => ilm_data::controls::command::changes_command(rest),
        "made" => ilm_data::sets::made::files::made_command(rest),
        "patterns" => ilm_core::tools::pattern::show::pattern_command(rest),
        "write" => ilm_core::write::command::writer_command(&rest.iter().map(String::as_str).collect::<Vec<&str>>()),
        "questions" => asked_questions(&run),
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
