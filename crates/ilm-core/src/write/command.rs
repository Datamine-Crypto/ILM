use crate::base::lanes::{hardware_lanes, on_lanes};
use crate::write::check::{FactCheck, FactFault};
use crate::write::levels::Writer;
use crate::write::lookup::NearestLesson;
use crate::write::measure::{FactList, at_even_steps, grouped_lists, learn_and_dev};
use crate::write::measured::{MeasuredTexts, measure_texts};
use crate::write::overlap::person_overlap;
use crate::write::rows::{FactRow, cut_of_lists, read_rows};
use crate::write::show::{ShownLines, bits_shown};
use crate::write::show_forms::show_forms;
use crate::write::show_lessons::show_lessons;
use crate::write::show_reader::show_reader;
use patterns::{because, source};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub struct WriterProgram;
source!(WriterProgram, "the writer as a part of the one program: a model is made from rows of facts and texts, a part is written on all threads, and the texts and the numbers of the measure are compared with a file of wanted texts, so that a change of the writer is proved before it is kept");

const WRITE_DEV_PART: &str = "dev";
because!(WRITE_DEV_PART, WriterProgram, "the name of the part of the train rows on which a writer is developed: the rows of the lists of facts that give no lesson; every other name of a part is the path of a file of rows");

const WRITE_SHOWN_SECONDS: usize = 3;
because!(WRITE_SHOWN_SECONDS, WriterProgram, "the places after the point of a printed time in seconds: a thousandth of a second is the least that a run of a part on this kind of machine shows");

fn said_failure(what: &str) -> ExitCode {
    eprintln!("the writer stops: {what}");
    ExitCode::FAILURE
}
because!(said_failure, WriterProgram, "the one place where the writer part says why it stops: a file that cannot be read or a model that cannot be made, in the words of the fault");

fn written_file(out: &str, text: &str) -> std::io::Result<()> {
    if let Some(folder) = Path::new(out).parent().filter(|folder| !folder.as_os_str().is_empty()) {
        std::fs::create_dir_all(folder)?;
    }
    std::fs::write(out, text)
}
because!(written_file, WriterProgram, "a file of the writer written whole, with the folders that its place needs");

fn lessons_and_dev(train: &str, fit: &str) -> Result<(Vec<FactRow>, Vec<FactRow>), String> {
    let Ok(wanted) = fit.parse::<usize>() else { return Err(format!("the number of lessons is a whole number, not {fit}")) };
    let rows = read_rows(Path::new(train)).map_err(|error| format!("{train}: {error}"))?;
    let (learn, dev) = learn_and_dev(&rows.rows);
    Ok((at_even_steps(&learn, wanted), dev))
}
because!(lessons_and_dev, WriterProgram, "the lessons of a model, so many at even steps through the rows of the learn lists (nought for all), and the rows of the part for development; a number of lessons that is no whole number is a fault, so that a mistyped number does not quietly learn from nothing");

fn show_writer_model(train: &str, fit: &str, out: &str) -> ExitCode {
    let (picked, _) = match lessons_and_dev(train, fit) {
        Ok(found) => found,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let writer = match Writer::learn(&picked) {
        Ok(writer) => writer,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let mut shown = ShownLines::default();
    show_lessons(&writer.lessons, &mut shown);
    show_reader(&writer.lessons, &writer.reader, &mut shown);
    show_forms(&writer.lessons, &writer.reader, &writer.forms, &mut shown);
    let mut text = shown.lines.join("\n");
    text.push_str("\n");
    match written_file(out, &text) {
        Ok(()) => {
            println!("{out} {} lines", shown.lines.len());
            ExitCode::SUCCESS
        }
        Err(error) => said_failure(&format!("{out}: {error}")),
    }
}
because!(show_writer_model, WriterProgram, "writes what a model of so many lessons counted into a file of lines, so that two builds of the model can be compared line by line");

fn lists_of(part: &str, dev: &[FactRow]) -> Result<Vec<FactList>, String> {
    if part == WRITE_DEV_PART {
        return Ok(grouped_lists(dev));
    }
    let rows = read_rows(Path::new(part)).map_err(|error| format!("{part}: {error}"))?;
    Ok(grouped_lists(&rows.rows))
}
because!(lists_of, WriterProgram, "the lists of facts of a part with their references: the part for development from the train rows, or the rows of a file");

fn write_texts(train: &str, part: &str, fit: &str, wanted_file: &str, shown: &str, traced: bool) -> ExitCode {
    let (picked, dev) = match lessons_and_dev(train, fit) {
        Ok(found) => found,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let lists = match lists_of(part, &dev) {
        Ok(lists) => lists,
        Err(error) => return said_failure(&format!("{part}: {error}")),
    };
    let started = Instant::now();
    let writer = match Writer::learn(&picked) {
        Ok(writer) => writer,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let made = started.elapsed().as_secs_f64();
    let lanes = hardware_lanes();
    let started = Instant::now();
    let texts: Vec<String> = on_lanes(lists.len(), lanes, &|at| if traced { writer.desk().trace(&lists[at].facts) } else { writer.write(&lists[at].facts) });
    let wrote = started.elapsed().as_secs_f64();
    println!("lessons {} lists {} threads {lanes} model {made:.*} s texts {wrote:.*} s", picked.len(), lists.len(), WRITE_SHOWN_SECONDS, WRITE_SHOWN_SECONDS);
    let wanted = match read_rows(Path::new(wanted_file)) {
        Ok(rows) => rows,
        Err(error) => return said_failure(&format!("{wanted_file}: {error}")),
    };
    if wanted.rows.len() != texts.len() {
        println!("the file of wanted texts has {} texts", wanted.rows.len());
        return ExitCode::FAILURE;
    }
    let mut other = 0usize;
    let show = if part == WRITE_DEV_PART { shown.parse::<usize>().unwrap_or(0) } else { 0 };
    for (at, (text, want)) in texts.iter().zip(&wanted.rows).enumerate() {
        if *text == want.text {
            continue;
        }
        other += 1;
        if other <= show {
            let facts: Vec<String> = lists[at].facts.iter().map(|(slot, value)| format!("{slot} = {value}")).collect();
            println!("list {at}: {}", facts.join("; "));
            println!("    wanted: {}", want.text);
            println!("    given:  {text}");
        }
    }
    println!("texts {} the same {} other {other}", texts.len(), texts.len() - other);
    if other == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
because!(write_texts, WriterProgram, "writes a text, or its trace, for every list of facts of a part on all threads and compares each with the wanted text, sign for sign; it prints how many are the same, and for the part for development the first lists whose texts differ, with their facts");

const WRITTEN_LINE: &str = "W\t";
because!(WRITTEN_LINE, WriterProgram, "the start of the line of a written text in a file of texts, as the files of wanted texts hold them");

const WRITE_EVERY_ROW: &str = "all";
because!(WRITE_EVERY_ROW, WriterProgram, "the word after the out file that asks for lessons from every train row, also the rows of the part for development, as a final read of a part outside the train file takes them");

fn write_out(train: &str, part: &str, fit: &str, out: &str, every: bool) -> ExitCode {
    let (picked, dev) = match lessons_and_dev(train, fit) {
        Ok(found) => found,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let picked = if every {
        match read_rows(Path::new(train)) {
            Ok(rows) => at_even_steps(&rows.rows, fit.parse::<usize>().unwrap_or_default()),
            Err(error) => return said_failure(&format!("{train}: {error}")),
        }
    } else {
        picked
    };
    let lists = match lists_of(part, &dev) {
        Ok(lists) => lists,
        Err(error) => return said_failure(&format!("{part}: {error}")),
    };
    let writer = match Writer::learn(&picked) {
        Ok(writer) => writer,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let texts: Vec<String> = on_lanes(lists.len(), hardware_lanes(), &|at| writer.write(&lists[at].facts));
    let lines: Vec<String> = texts.iter().map(|text| format!("{WRITTEN_LINE}{text}")).collect();
    match written_file(out, &format!("{}\n", lines.join("\n"))) {
        Ok(()) => {
            println!("{out}: {} texts from {} lessons", texts.len(), picked.len());
            ExitCode::SUCCESS
        }
        Err(error) => said_failure(&format!("{out}: {error}")),
    }
}
because!(write_out, WriterProgram, "writes the text of every list of facts of a part into a file, one line for each list in the order of the lists, so that a measure outside the program can read them; the lessons come from the learn lists, or from every train row when the run asks for all");

const REFERENCE_GAP: &str = "

";
because!(REFERENCE_GAP, WriterProgram, "the empty line between the references of two lists of facts in a file of references, as the official metrics of the E2E challenge read it");

fn metric_files(part: &str, written: &str, references_out: &str, texts_out: &str) -> ExitCode {
    let rows = match read_rows(Path::new(part)) {
        Ok(rows) => rows,
        Err(error) => return said_failure(&format!("{part}: {error}")),
    };
    let lists = grouped_lists(&rows.rows);
    let written_text = match std::fs::read_to_string(written) {
        Ok(text) => text,
        Err(error) => return said_failure(&format!("{written}: {error}")),
    };
    let texts: Vec<&str> = written_text.lines().filter_map(|line| line.strip_prefix(WRITTEN_LINE)).collect();
    if texts.len() != lists.len() {
        return said_failure(&format!("{written}: {} written texts against {} lists of facts", texts.len(), lists.len()));
    }
    let groups: Vec<String> = lists.iter().map(|list| list.references.join("\n")).collect();
    if let Err(error) = written_file(references_out, &format!("{}\n", groups.join(REFERENCE_GAP))) {
        return said_failure(&format!("{references_out}: {error}"));
    }
    match written_file(texts_out, &format!("{}\n", texts.join("\n"))) {
        Ok(()) => {
            println!("{} rows, {} lists of facts: {references_out}, {texts_out}", rows.rows.len(), lists.len());
            ExitCode::SUCCESS
        }
        Err(error) => said_failure(&format!("{texts_out}: {error}")),
    }
}
because!(metric_files, WriterProgram, "the two files that the official metrics of the E2E challenge read: the references of each list of facts of a part, one text on a line with an empty line between two lists, in the order of the lists; and the written texts of a file of the writer, one on a line in the same order");

fn numbers_of(label: &str, found: &MeasuredTexts) -> Vec<(String, String)> {
    vec![
        (format!("{label}_bleu"), bits_shown(found.run_overlap)),
        (format!("{label}_beside"), bits_shown(found.beside)),
        (format!("{label}_right"), found.right.to_string()),
        (format!("{label}_lost"), found.lost.to_string()),
        (format!("{label}_wrong"), found.wrong.to_string()),
        (format!("{label}_added"), found.added.to_string()),
        (format!("{label}_tokens"), bits_shown(found.tokens)),
        (format!("{label}_different"), found.different.to_string()),
    ]
}
because!(numbers_of, WriterProgram, "the numbers of the measure for one writer as named values, a number with a fraction as the signs of its bits, so that it is compared to the last bit");

fn measure_part(train: &str, part: &str, fit: &str, wanted_file: &str) -> ExitCode {
    let rows = match read_rows(Path::new(train)) {
        Ok(rows) => rows,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let (learn, dev) = learn_and_dev(&rows.rows);
    let picked = at_even_steps(&learn, fit.parse::<usize>().unwrap_or(0));
    let lists = match lists_of(part, &dev) {
        Ok(lists) => lists,
        Err(error) => return said_failure(&format!("{part}: {error}")),
    };
    let wanted = match read_rows(Path::new(wanted_file)) {
        Ok(rows) => rows,
        Err(error) => return said_failure(&format!("{wanted_file}: {error}")),
    };
    let started = Instant::now();
    let cut = cut_of_lists(&rows);
    let check = FactCheck::learn(&learn, cut.as_deref());
    let checked = started.elapsed().as_secs_f64();
    let writer = match Writer::learn(&picked) {
        Ok(writer) => writer,
        Err(error) => return said_failure(&format!("{train}: {error}")),
    };
    let lanes = hardware_lanes();
    let texts: Vec<String> = on_lanes(lists.len(), lanes, &|at| writer.write(&lists[at].facts));
    let nearest = NearestLesson::learn(&picked);
    let near_texts: Vec<String> = on_lanes(lists.len(), lanes, &|at| nearest.write(&lists[at].facts));
    let mut given = numbers_of("writer", &measure_texts(&check, &lists, &texts));
    given.extend(numbers_of("nearest", &measure_texts(&check, &lists, &near_texts)));
    given.push(("person_bleu".to_string(), bits_shown(person_overlap(&lists))));
    let mut people = MeasuredTexts::default();
    let mut people_texts = 0usize;
    let mut cannot = (0usize, 0usize);
    for list in &lists {
        let (blind, all) = check.cannot(&list.facts);
        cannot = (cannot.0 + blind, cannot.1 + all);
        for text in &list.references {
            let faults = check.faults(&list.facts, text);
            people_texts += 1;
            people.right += usize::from(faults.is_empty());
            people.lost += faults.iter().filter(|fault| **fault == FactFault::Lost).count();
            people.wrong += faults.iter().filter(|fault| **fault == FactFault::Wrong).count();
            people.added += faults.iter().filter(|fault| **fault == FactFault::Added).count();
        }
    }
    for (name, value) in [
        ("check_names", check.names.len()),
        ("check_marks", check.mark_count()),
        ("check_cannot", cannot.0),
        ("check_facts", cannot.1),
        ("people_right", people.right),
        ("people_texts", people_texts),
        ("people_lost", people.lost),
        ("people_wrong", people.wrong),
        ("people_added", people.added),
    ] {
        given.push((name.to_string(), value.to_string()));
    }
    let mut other = 0usize;
    for (name, value) in &given {
        let want = wanted.head.get(name).map_or("", String::as_str);
        let same = want == value;
        other += usize::from(!same);
        println!("{name} {value} {}", if same { "the same" } else { want });
    }
    println!("check {checked:.*} s; numbers {} other {other}", WRITE_SHOWN_SECONDS, given.len());
    if other == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
because!(measure_part, WriterProgram, "measures the texts of the writer and of the nearest lesson for a part, with the check of facts counted from all lessons of the learn lists, and compares every number with the named values of the first line of a file of wanted texts; a number that differs is printed with the wanted value");

pub fn writer_command(given: &[&str]) -> ExitCode {
    match given {
        ["model", train, fit, out] => show_writer_model(train, fit, out),
        ["texts", train, part, fit, wanted_file] => write_texts(train, part, fit, wanted_file, "", false),
        ["texts", train, part, fit, wanted_file, shown] => write_texts(train, part, fit, wanted_file, shown, false),
        ["traces", train, part, fit, wanted_file] => write_texts(train, part, fit, wanted_file, "", true),
        ["traces", train, part, fit, wanted_file, shown] => write_texts(train, part, fit, wanted_file, shown, true),
        ["measure", train, part, fit, wanted_file] => measure_part(train, part, fit, wanted_file),
        ["out", train, part, fit, out] => write_out(train, part, fit, out, false),
        ["out", train, part, fit, out, every] if *every == WRITE_EVERY_ROW => write_out(train, part, fit, out, true),
        ["metric-files", part, written, references_out, texts_out] => metric_files(part, written, references_out, texts_out),
        _ => {
            eprintln!("usage: ilm write model <train rows> <lessons> <out file>");
            eprintln!("   or: ilm write texts <train rows> <dev, or a file of rows> <lessons> <file of wanted texts> [lists to show]");
            eprintln!("   or: ilm write traces <train rows> <dev, or a file of rows> <lessons> <file of wanted traces> [lists to show]");
            eprintln!("   or: ilm write measure <train rows> <dev, or a file of rows> <lessons> <file of wanted texts>");
            eprintln!("   or: ilm write out <train rows> <dev, or a file of rows> <lessons> <out file> [all]");
            eprintln!("   or: ilm write metric-files <file of rows of a part> <file of written texts> <references out> <texts out>");
            ExitCode::FAILURE
        }
    }
}
because!(writer_command, WriterProgram, "the part of the program for the writer: the word after its name says what is done with the rows of a set");
