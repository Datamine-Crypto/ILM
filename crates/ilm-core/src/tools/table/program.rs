use crate::tools::table::rows::{ToldRows, amount};
use patterns::{because, source};

pub struct TablePrograms;
source!(TablePrograms, "a program over a told table, found from the lessons for each form of question: it keeps the rows whose fields are values of the question; it keeps the rows whose span of two numeric fields holds a number of the question, meets a span that the question gives, or holds a number of another row that the question names; it picks rows by the order of a numeric field (the least, the greatest, the n-th, or the one right after or right before a row that the question names); and it says one field, the count of the values of a field, the difference of two numeric fields, or the values of a field in the order of another. The parts are tried from the simplest, and a program is kept for a form when it gives the answers of the lessons of that form");

const MOST_BINDS: usize = 3;
because!(MOST_BINDS, TablePrograms, "the most fields that a program ties to values of the question: a row of a told fact has a few named things, and more ties would only fit the lessons by chance");

const JOIN_LEAST: usize = 2;
because!(JOIN_LEAST, TablePrograms, "the fewest ties of the row that another row is timed by: a single tie names a field and not a fact, so it would time by any row of that field");

pub(super) const SAMPLE: usize = 20;
because!(SAMPLE, TablePrograms, "the lessons of a form that a candidate program must answer right before it is counted on all of them: enough that a program right by chance is rare, few enough that the search stays quick");

pub(super) const MOST_FOUND: usize = 512;
because!(MOST_FOUND, TablePrograms, "the most programs that pass the sample and are counted on all lessons of a form, the simplest first");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Bind {
    pub(super) field: usize,
    pub(super) value: usize,
}
because!(Bind, TablePrograms, "a tie of a field of a row to a value of the question by their places");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Keep {
    All,
    Inside { low: usize, high: usize, value: usize },
    Meets { low: usize, high: usize, value: usize },
    Joined { other: Vec<Bind>, at: usize, low: usize, high: usize },
}
because!(Keep, TablePrograms, "which tied rows are kept by a number: all of them; those whose span holds a number of the question; those whose span meets the span of two numbers of the question; those whose span holds a numeric field of the one other row that some values of the question name");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pick {
    All,
    Least(usize),
    Greatest(usize),
    Nth { order: usize, value: usize },
    After { order: usize, field: usize, value: usize },
    Before { order: usize, field: usize, value: usize },
}
because!(Pick, TablePrograms, "which kept rows are picked_rows_of by the order of a numeric field: all; the least; the greatest; the n-th, with n from a value of the question; the one right after or right before the row whose field is a value of the question");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Say {
    Field(usize),
    Count(usize),
    Difference(usize, usize),
    Sorted { field: usize, order: usize },
}
because!(Say, TablePrograms, "what the picked_rows_of rows say: the one value of a field, the count of the values of a field, the difference of two numeric fields of the one picked_rows_of row, or the values of a field in the order of a numeric field");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TableProgram {
    pub(super) binds: Vec<Bind>,
    pub(super) keep: Keep,
    pub(super) pick: Pick,
    pub(super) say: Say,
}
because!(TableProgram, TablePrograms, "a whole program: its ties, what it keeps, what it picks and what it says");

pub(super) type TableLesson = (ToldRows, Vec<String>, Vec<String>);
because!(TableLesson, TablePrograms, "a lesson as the search reads it: the table of its story, the values of its question, and the words of its answer");

fn tied(table: &ToldRows, values: &[String], binds: &[Bind]) -> Vec<usize> {
    (0..table.rows.len()).filter(|row| binds.iter().all(|bind| values.get(bind.value).is_some_and(|value| table.rows[*row][bind.field] == *value))).collect()
}
because!(tied, TablePrograms, "the rows whose tied fields hold the values of the question, in the order of the story");

fn number_at(table: &ToldRows, row: usize, field: usize) -> Option<i64> {
    amount(&table.rows[row][field])
}
because!(number_at, TablePrograms, "the number of a field of a row, and none for a field with no digit");

fn kept(table: &ToldRows, values: &[String], rows: &[usize], keep: &Keep) -> Vec<usize> {
    let spans = |row: &usize, low: usize, high: usize| number_at(table, *row, low).zip(number_at(table, *row, high));
    match keep {
        Keep::All => rows.to_vec(),
        Keep::Inside { low, high, value } => {
            let Some(at) = values.get(*value).and_then(|held| amount(held)) else { return Vec::new() };
            rows.iter().copied().filter(|row| spans(row, *low, *high).is_some_and(|(start, end)| start <= at && at <= end)).collect()
        }
        Keep::Meets { low, high, value } => {
            let (Some(first), Some(last)) = (values.get(*value).and_then(|held| amount(held)), values.get(value + 1).and_then(|held| amount(held))) else { return Vec::new() };
            rows.iter().copied().filter(|row| spans(row, *low, *high).is_some_and(|(start, end)| start <= last && end >= first)).collect()
        }
        Keep::Joined { other, at, low, high } => {
            let named = tied(table, values, other);
            let [one] = named.as_slice() else { return Vec::new() };
            let Some(time) = number_at(table, *one, *at) else { return Vec::new() };
            rows.iter().copied().filter(|row| row != one && spans(row, *low, *high).is_some_and(|(start, end)| start <= time && time <= end)).collect()
        }
    }
}
because!(kept, TablePrograms, "the tied rows that a program keeps by a number; none when the number that it needs is not there");

fn ordered(table: &ToldRows, rows: &[usize], order: usize) -> Vec<usize> {
    let mut sorted = rows.to_vec();
    sorted.sort_by_key(|row| (number_at(table, *row, order).unwrap_or_default(), *row));
    sorted
}
because!(ordered, TablePrograms, "rows in the order of a numeric field, rows with the same number in the order of the story");

fn picked_rows_of(table: &ToldRows, values: &[String], rows: &[usize], pick: Pick) -> Vec<usize> {
    match pick {
        Pick::All => rows.to_vec(),
        Pick::Least(order) => ordered(table, rows, order).into_iter().take(1).collect(),
        Pick::Greatest(order) => ordered(table, rows, order).into_iter().last().into_iter().collect(),
        Pick::Nth { order, value } => {
            let sorted = ordered(table, rows, order);
            let Some(place) = values.get(value).and_then(|held| amount(held)).and_then(|number| usize::try_from(number).ok()).filter(|number| *number > 0) else { return Vec::new() };
            sorted.get(place - 1).copied().into_iter().collect()
        }
        Pick::After { order, field, value } | Pick::Before { order, field, value } => {
            let sorted = ordered(table, rows, order);
            let Some(held) = values.get(value) else { return Vec::new() };
            let named: Vec<usize> = (0..sorted.len()).filter(|place| table.rows[sorted[*place]][field] == *held).collect();
            let [place] = named.as_slice() else { return Vec::new() };
            let next = if matches!(pick, Pick::After { .. }) { place.checked_add(1) } else { place.checked_sub(1) };
            next.and_then(|at| sorted.get(at)).copied().into_iter().collect()
        }
    }
}
because!(picked_rows_of, TablePrograms, "the kept rows that a program picks by the order of a numeric field; none when the order does not give one");

fn words_said(table: &ToldRows, rows: &[usize], say: Say) -> Option<Vec<String>> {
    match say {
        Say::Field(field) => {
            let first = &table.rows[*rows.first()?][field];
            rows.iter().all(|row| table.rows[*row][field] == *first).then(|| vec![first.clone()])
        }
        Say::Count(field) => {
            let mut held: Vec<&String> = rows.iter().map(|row| &table.rows[*row][field]).collect();
            held.sort();
            held.dedup();
            Some(vec![held.len().to_string()])
        }
        Say::Difference(minuend, subtrahend) => {
            let [row] = rows else { return None };
            Some(vec![(number_at(table, *row, minuend)? - number_at(table, *row, subtrahend)?).to_string()])
        }
        Say::Sorted { field, order } => Some(ordered(table, rows, order).into_iter().map(|row| table.rows[row][field].clone()).collect()),
    }
}
because!(words_said, TablePrograms, "the words that the picked_rows_of rows say; none when a program asks for one value and the rows hold several, or for one row and there are more");

fn is_zero(answer: &[String]) -> bool {
    answer.len() == 1 && amount(&answer[0]) == Some(0)
}
because!(is_zero, TablePrograms, "whether an answer is the count none: a program that keeps no row may then still be right, so the search does not drop it");

pub(super) fn run(program: &TableProgram, table: &ToldRows, values: &[String]) -> Option<Vec<String>> {
    let rows = tied(table, values, &program.binds);
    let rows = kept(table, values, &rows, &program.keep);
    let rows = picked_rows_of(table, values, &rows, program.pick);
    if rows.is_empty() && !matches!(program.say, Say::Count(_)) {
        return None;
    }
    words_said(table, &rows, program.say)
}
because!(run, TablePrograms, "the answer of a program for one table and the values of its question");

fn bind_sets(width: usize, count: usize) -> Vec<Vec<Bind>> {
    let mut sets: Vec<Vec<Bind>> = vec![Vec::new()];
    let mut frontier: Vec<Vec<Bind>> = vec![Vec::new()];
    for _ in 0..MOST_BINDS {
        let mut next: Vec<Vec<Bind>> = Vec::new();
        for set in &frontier {
            let last_value = set.last().map(|bind| bind.value + 1).unwrap_or(0);
            for value in last_value..count {
                for field in 0..width {
                    if set.iter().any(|bind| bind.field == field) {
                        continue;
                    }
                    let mut grown = set.clone();
                    grown.push(Bind { field, value });
                    next.push(grown);
                }
            }
        }
        sets.extend(next.iter().cloned());
        frontier = next;
    }
    sets
}
because!(bind_sets, TablePrograms, "every set of ties of a few fields to a few values, the values in their order and each field once, the smallest sets first");

fn spans(width: usize) -> Vec<(usize, usize)> {
    (0..width).flat_map(|low| (0..width).filter(move |high| *high != low).map(move |high| (low, high))).collect()
}
because!(spans, TablePrograms, "every pair of two fields that may be the start and the end of a span");

fn keeps(width: usize, count: usize, others: &[Vec<Bind>]) -> Vec<Keep> {
    let mut out = vec![Keep::All];
    for (low, high) in spans(width) {
        for value in 0..count {
            out.push(Keep::Inside { low, high, value });
            if value + 1 < count {
                out.push(Keep::Meets { low, high, value });
            }
        }
    }
    for other in others {
        for at in 0..width {
            for (low, high) in spans(width) {
                out.push(Keep::Joined { other: other.clone(), at, low, high });
            }
        }
    }
    out
}
because!(keeps, TablePrograms, "every way to keep rows by a number, the simplest first, with the rows that some values of the question name as one other row");

fn picks(width: usize, count: usize) -> Vec<Pick> {
    let mut out = vec![Pick::All];
    out.extend((0..width).map(Pick::Least));
    out.extend((0..width).map(Pick::Greatest));
    for order in 0..width {
        for value in 0..count {
            out.push(Pick::Nth { order, value });
            for field in 0..width {
                out.push(Pick::After { order, field, value });
                out.push(Pick::Before { order, field, value });
            }
        }
    }
    out
}
because!(picks, TablePrograms, "every way to pick rows by the order of a field, the simplest first");

fn says(width: usize) -> Vec<Say> {
    let mut out: Vec<Say> = (0..width).map(Say::Field).collect();
    out.extend((0..width).map(Say::Count));
    for (minuend, subtrahend) in spans(width) {
        out.push(Say::Difference(minuend, subtrahend));
    }
    for field in 0..width {
        out.extend((0..width).map(|order| Say::Sorted { field, order }));
    }
    out
}
because!(says, TablePrograms, "every way for the picked_rows_of rows to say an answer");

fn values_used(program: &TableProgram) -> usize {
    let mut used: Vec<usize> = program.binds.iter().map(|bind| bind.value).collect();
    match &program.keep {
        Keep::All => {}
        Keep::Inside { value, .. } => used.push(*value),
        Keep::Meets { value, .. } => used.extend([*value, value + 1]),
        Keep::Joined { other, .. } => used.extend(other.iter().map(|bind| bind.value)),
    }
    match program.pick {
        Pick::Nth { value, .. } | Pick::After { value, .. } | Pick::Before { value, .. } => used.push(value),
        _ => {}
    }
    used.sort_unstable();
    used.dedup();
    used.len()
}
because!(values_used, TablePrograms, "how many values of the question a program uses: of programs that read the lessons of a form equally well, the one that uses more of what the question names is taken, since a program that leaves a named value out fits the lessons by chance");

pub(super) fn search(lessons: &[TableLesson], width: usize, count: usize) -> Option<(TableProgram, usize)> {
    let sample = &lessons[..lessons.len().min(SAMPLE)];
    let all_sets = bind_sets(width, count);
    let mains: Vec<&Vec<Bind>> = all_sets.iter().filter(|binds| sample.iter().all(|(table, values, _)| !tied(table, values, binds).is_empty())).collect();
    let others: Vec<Vec<Bind>> = all_sets.iter().filter(|binds| binds.len() >= JOIN_LEAST && sample.iter().all(|(table, values, _)| tied(table, values, binds).len() == 1)).cloned().collect();
    let keep_ways = keeps(width, count, &others);
    let pick_ways = picks(width, count);
    let say_ways = says(width);
    let mut found: Vec<TableProgram> = Vec::new();
    'search: for binds in mains {
        let base: Vec<Vec<usize>> = sample.iter().map(|(table, values, _)| tied(table, values, binds)).collect();
        for keep in &keep_ways {
            let kept_rows: Vec<Vec<usize>> = sample.iter().zip(&base).map(|((table, values, _), rows)| kept(table, values, rows, keep)).collect();
            if kept_rows.iter().zip(sample).any(|(rows, (_, _, answer))| rows.is_empty() && !is_zero(answer)) {
                continue;
            }
            for pick in &pick_ways {
                let picked_rows: Vec<Vec<usize>> = sample.iter().zip(&kept_rows).map(|((table, values, _), rows)| picked_rows_of(table, values, rows, *pick)).collect();
                if picked_rows.iter().zip(sample).any(|(rows, (_, _, answer))| rows.is_empty() && !is_zero(answer)) {
                    continue;
                }
                for say in &say_ways {
                    if sample.iter().zip(&picked_rows).all(|((table, _, answer), rows)| words_said(table, rows, *say).as_ref() == Some(answer)) {
                        found.push(TableProgram { binds: binds.clone(), keep: keep.clone(), pick: *pick, say: *say });
                        if found.len() >= MOST_FOUND {
                            break 'search;
                        }
                    }
                }
            }
        }
    }
    let mut best: Option<(TableProgram, usize, usize)> = None;
    for program in found {
        let right = lessons.iter().filter(|(table, values, answer)| run(&program, table, values).as_ref() == Some(answer)).count();
        let used = values_used(&program);
        if best.as_ref().is_none_or(|(_, top, top_used)| right > *top || (right == *top && used > *top_used)) {
            best = Some((program, right, used));
        }
    }
    best.map(|(program, right, _)| (program, right))
}
because!(search, TablePrograms, "the program of one form of question: the candidates that answer the sample of its lessons right, from the simplest, counted on all its lessons; the one that answers most of them, of equals the one that uses most values of the question, then the simplest");
