use crate::base::lanes::{hardware_lanes, on_lanes};
use crate::base::symbols::Symbols;
use crate::cases::{Kind, read_kind};
use crate::tools::pattern::classes::{KIND_MARK, ABSENT};
use crate::tools::pattern::kinds::{ENDS_NAMES, IN_LESSONS, PatternWords, STARTS_NAMES};
use crate::tools::pattern::learn::PatternLearner;
use crate::tools::pattern::model::PatternModel;
use crate::tools::pattern::texts::asked_text;
use patterns::{because, source};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub struct StageCheck;
source!(StageCheck, "a learner is checked stage by stage: each of its tables is printed as lines in one fixed order, so that two programs that must make the same table are compared line by line, and a difference is found at the first table that has it and not in an answer at the end");

pub const CELL: &str = "\t";
because!(CELL, StageCheck, "the mark between the cells of a printed line: no word holds it, so a line is cut into its cells again without doubt");

pub const WORD_GAP: &str = " ";
because!(WORD_GAP, StageCheck, "the mark between the words of a text in a printed line");

pub fn row_text(words: &PatternWords, symbols: &Symbols, row: &[u32]) -> String {
    let parts: Vec<&str> = row.iter().map(|code| if code & KIND_MARK != 0 && *code != ABSENT { words.kind_names[(code & !KIND_MARK) as usize].as_str() } else { symbols.name(*code) }).collect();
    parts.join(WORD_GAP)
}
because!(row_text, StageCheck, "the text of a row of words or of a form: a word as its text, a place of a kind as the name of the kind");

pub fn words_lines(words: &PatternWords, symbols: &Symbols) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let line = |cells: &[&str]| cells.join(CELL);
    for (word, base) in words.read_as.iter().enumerate() {
        if *base != ABSENT {
            lines.push(line(&["same", symbols.name(word as u32), symbols.name(*base)]));
        }
    }
    for run in words.runs.keys() {
        lines.push(line(&["run", &row_text(words, symbols, run)]));
    }
    for (word, kind) in words.kinds.iter().enumerate() {
        if *kind != ABSENT {
            lines.push(line(&["kind", symbols.name(word as u32), &words.kind_names[*kind as usize]]));
        }
    }
    for (place, kind) in words.answer_kinds.iter().enumerate() {
        lines.push(line(&["answer_kind", &place.to_string(), &words.kind_names[*kind as usize]]));
    }
    for word in 0..words.flags.len() as u32 {
        for (flag, mark) in [(IN_LESSONS, "vocab"), (STARTS_NAMES, "run_first"), (ENDS_NAMES, "run_last")] {
            if words.has(word, flag) {
                lines.push(line(&[mark, symbols.name(word)]));
            }
        }
    }
    for (kind, members) in words.members.iter().enumerate() {
        for (place, word) in members.iter().enumerate() {
            lines.push(line(&["member", &words.kind_names[kind], &place.to_string(), symbols.name(*word)]));
        }
    }
    for (size, shapes) in &words.shapes {
        for (shape, count) in shapes.rows() {
            lines.push(line(&["shape", &size.to_string(), &count.to_string(), &row_text(words, symbols, shape)]));
        }
    }
    for (size, shapes) in &words.sent_shapes {
        for shape in shapes {
            lines.push(line(&["sent_shape", &size.to_string(), &row_text(words, symbols, shape)]));
        }
    }
    for (word, forms) in &words.answer_forms {
        for (raw, count) in forms.rows() {
            lines.push(line(&["answer_form", symbols.name(*word), symbols.name(*raw), &count.to_string()]));
        }
    }
    for ((old, new), count) in &words.answer_changes {
        lines.push(line(&["answer_change", old, new, &count.to_string()]));
    }
    lines.sort();
    lines
}
because!(words_lines, StageCheck, "the tables of the words and kinds of a set of lessons as sorted lines, one entry on a line with the name of its table first");

pub fn cases_of(path: &str) -> Option<Kind> {
    match read_kind(Path::new(path)) {
        Ok(kind) => Some(kind),
        Err(error) => {
            eprintln!("{path}: {error}");
            None
        }
    }
}
because!(cases_of, StageCheck, "the cases of a file, or nothing with the cause on the error stream when the file can not be read");

pub fn show_words(path: &str) -> ExitCode {
    let Some(kind) = cases_of(path) else { return ExitCode::FAILURE };
    let mut symbols = Symbols::new();
    let words = PatternWords::learn(&kind.cases, &mut symbols);
    println!("{}", words_lines(&words, &symbols).join("\n"));
    ExitCode::SUCCESS
}
because!(show_words, StageCheck, "the words and kinds that the pattern learner makes from the cases of one file, printed as lines");

pub fn model_lines(learner: &PatternLearner, model: &PatternModel, mark: &str, rules: bool, lines: &mut Vec<String>) {
    let (words, names) = (&learner.words, &learner.symbols);
    let line = |cells: &[&str]| format!("{mark}{}", cells.join(CELL));
    lines.push(line(&["patterns", &model.table.parent.len().to_string()]));
    lines.push(line(&["left", &model.left.to_string()]));
    lines.push(line(&["default", &row_text(words, names, &model.default)]));
    for kind in &model.point_kinds {
        lines.push(line(&["point_kind", &words.kind_names[*kind as usize]]));
    }
    for (place, rule) in model.order.iter().enumerate() {
        lines.push(line(&["order", &place.to_string(), &model.text(words, names, rule.pattern), &model.place_name(names, rule.place), &rule.readings.to_string(), &rule.hit.to_string()]));
    }
    for part in &model.parts {
        let name = format!("{mark}split{CELL}{} | {}{CELL}", asked_text(names, model.asked.held(part.asked)), words.kind_names[part.kind as usize]);
        for label in &part.null {
            lines.push(format!("{name}null{CELL}{}", names.name(*label)));
        }
        model_lines(learner, &part.model, &name, false, lines);
    }
    if rules {
        let mut found: Vec<String> = (0..model.table.parent.len() as u32)
            .map(|pattern| {
                let at = pattern as usize;
                line(&["rule", &model.text(words, names, pattern), &model.table.count[at].to_string(), &model.hit[at].to_string(), &model.place_name(names, model.place[at])])
            })
            .collect();
        found.sort();
        lines.append(&mut found);
    }
}
because!(model_lines, StageCheck, "one counted model as lines: the number of its patterns, the lessons that its list leaves open, its most frequent answer, the kinds of its pointings, the rules of its list in their order, its splits with what they leave out and with their own models, and on demand every pattern with its readings, hits and place, sorted by the text of the pattern");

pub fn show_model(path: &str, rules: bool) -> ExitCode {
    let Some(kind) = cases_of(path) else { return ExitCode::FAILURE };
    let learner = PatternLearner::learn(&kind.cases, hardware_lanes());
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("plain{CELL}{}", learner.plain.is_some()));
    lines.push(format!("affirm{CELL}{}", row_text(&learner.words, &learner.symbols, &learner.affirm)));
    for form in &learner.writes {
        lines.push(format!("write{CELL}{}", asked_text(&learner.symbols, &form.template)));
    }
    model_lines(&learner, &learner.model, "", rules, &mut lines);
    if let Some(choice) = &learner.choice {
        lines.push(format!("choice{CELL}{}{CELL}{}", learner.symbols.name(choice.mark), row_text(&learner.words, &learner.symbols, &choice.tail)));
        model_lines(&learner, &choice.model, &format!("choice{CELL}"), false, &mut lines);
    }
    println!("{}", lines.join("\n"));
    ExitCode::SUCCESS
}
because!(show_model, StageCheck, "the model that the pattern learner makes from the cases of one file, printed as lines");

pub fn show_answers(lessons: &str, asked: &str) -> ExitCode {
    let (Some(learned), Some(read)) = (cases_of(lessons), cases_of(asked)) else { return ExitCode::FAILURE };
    let lanes = hardware_lanes();
    let started = Instant::now();
    let learner = PatternLearner::learn(&learned.cases, lanes);
    let learn_seconds = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let said: Vec<String> = on_lanes(read.cases.len(), lanes, &|number| learner.said(&read.cases[number].sentences, &read.cases[number].question).0.join(WORD_GAP));
    let answer_seconds = started.elapsed().as_secs_f64();
    for line in &said {
        println!("{line}");
    }
    eprintln!("lessons {} patterns {} rules {} learned in {learn_seconds:.3} s on {lanes} threads | {} cases answered in {answer_seconds:.3} s", learned.cases.len(), learner.model.table.parent.len(), learner.model.order.len(), read.cases.len());
    ExitCode::SUCCESS
}
because!(show_answers, StageCheck, "the answers of the pattern learner for the cases of one file after the lessons of another file, one answer on a line with its words, and on the error stream the size of the model and the seconds of learning and of answering, which is the speed test of the learner");

pub fn pattern_main() -> ExitCode {
    let given: Vec<String> = std::env::args().skip(1).collect();
    pattern_command(&given)
}
because!(pattern_main, StageCheck, "the entry of the check program of the pattern learner: the words of its command line after the name of the program");

pub fn pattern_command(given: &[String]) -> ExitCode {
    match given.iter().map(String::as_str).collect::<Vec<&str>>().as_slice() {
        ["long-story", ..] => crate::tools::pattern::long_story::long_story_command(&given[1..]),
        ["words", path] => show_words(path),
        ["model", path] => show_model(path, false),
        ["rules", path] => show_model(path, true),
        ["answers", lessons, asked] => show_answers(lessons, asked),
        _ => {
            eprintln!("usage: ilm patterns words <case file>");
            eprintln!("   or: ilm patterns model <case file>");
            eprintln!("   or: ilm patterns rules <case file>");
            eprintln!("   or: ilm patterns answers <case file of lessons> <case file of cases to answer>");
            eprintln!("   or: ilm patterns long-story <far|many> <case file> <file of the words of people and things>");
            ExitCode::FAILURE
        }
    }
}
because!(pattern_command, StageCheck, "the entry of the checks of the pattern learner: the first word names the table that is printed, the words after it are the files that it is made from");
