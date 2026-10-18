use crate::base::lanes::{on_lanes, shards};
use patterns::{because, source};
use std::collections::HashMap;
use std::fmt::Write;
use std::hash::BuildHasher;
use std::ops::Range;

pub struct NeighbourClasses;
source!(NeighbourClasses, "classes of words from the words beside them in plain text, as Brown and others, Schuetze and Mintz count them: for each frequent word, how often each of the most frequent words stands right before it and right after it, so that two words with alike rows are of one class. The rows are made comparable (the square root of each count, then length one) and grouped by the spherical k-means of Dhillon and Modha: each row goes to the centre with the largest dot product, and each centre moves to the mean of its rows made of length one again, until no centre moves; the first centres are the rows at even steps of the list of words by frequency, so no chance enters and the classes are the same in every run and for every number of threads");

pub const COUNTED_WORDS: usize = 40_000;
because!(COUNTED_WORDS, NeighbourClasses, "the most frequent words that get a row and can get a class: enough for the words that a lesson of a set uses, small enough that the rows fit in memory");

pub const CONTEXT_WORDS: usize = 2_000;
because!(CONTEXT_WORDS, NeighbourClasses, "the most frequent words whose places right before and right after a word are counted apart; every other word is counted in one shared column");

pub const MOST_RUNS: usize = 25;
because!(MOST_RUNS, NeighbourClasses, "the most rounds of the k-means: after them the few rows that still change their class do not change what the table is good for");

const LEAST_SEEN: u64 = 40;
because!(LEAST_SEEN, NeighbourClasses, "a word gets a class only when it stood beside other words this often or more: with fewer, its row says too little about its places");

const NORM_FLOOR: f64 = 1e-9;
because!(NORM_FLOOR, NeighbourClasses, "the least length that a row or a centre is divided by, so that an empty one does not divide by nought");

const MOVE_BOUND: f64 = 1e-4;
because!(MOVE_BOUND, NeighbourClasses, "a centre has moved when the sum of the changes of its places is above this: a smaller change moves no row to another class");

const APOSTROPHE: u8 = b'\'';
because!(APOSTROPHE, NeighbourClasses, "the sign that joins two runs of letters into one word, as in the short forms of English");

const LINE_BREAK: char = '\n';
because!(LINE_BREAK, NeighbourClasses, "the sign that ends a line of a text; a word has no neighbour across it");

const ROW_HALVES: usize = 2;
because!(ROW_HALVES, NeighbourClasses, "the halves of a row: the words before a word and the words after it");

const NO_SUM: f64 = 0.0;
because!(NO_SUM, NeighbourClasses, "the start of a sum of values, and a place of a centre with no value");

const NUMBER_WORD: u8 = 0;
because!(NUMBER_WORD, NeighbourClasses, "the digit whose token stands for every run of digits, so that all numbers share their places in the rows");

pub const LINE_GAP: u32 = u32::MAX;
because!(LINE_GAP, NeighbourClasses, "the code between two lines of a text: a word has no neighbour in another line");

pub const NOT_COUNTED: u32 = u32::MAX - 1;
because!(NOT_COUNTED, NeighbourClasses, "the code of a token that is none of the counted words; as a neighbour it counts in the shared column of the other words");

pub fn each_token(line: &str, token: &mut String, seen: &mut dyn FnMut(&str)) {
    let bytes = line.as_bytes();
    let mut at = 0usize;
    while at < bytes.len() {
        let sign = bytes[at];
        if sign.is_ascii_alphabetic() {
            token.clear();
            while at < bytes.len() && bytes[at].is_ascii_alphabetic() {
                token.push(char::from(bytes[at].to_ascii_lowercase()));
                at += 1;
            }
            if at + 1 < bytes.len() && bytes[at] == APOSTROPHE && bytes[at + 1].is_ascii_alphabetic() {
                token.push(char::from(APOSTROPHE));
                at += 1;
                while at < bytes.len() && bytes[at].is_ascii_alphabetic() {
                    token.push(char::from(bytes[at].to_ascii_lowercase()));
                    at += 1;
                }
            }
            seen(token);
        } else if sign.is_ascii_digit() {
            while at < bytes.len() && bytes[at].is_ascii_digit() {
                at += 1;
            }
            token.clear();
            let _ = write!(token, "{NUMBER_WORD}");
            seen(token);
        } else {
            at += 1;
        }
    }
}
because!(each_token, NeighbourClasses, "the tokens of a line, one at a time: a run of letters of the Latin alphabet in small letters, with one more run after an apostrophe inside it, or a run of digits as the number token; every other sign parts tokens");

pub fn count_tokens<S: BuildHasher>(text: &str, counts: &mut HashMap<String, u64, S>) {
    let mut token = String::new();
    each_token(text, &mut token, &mut |word| match counts.get_mut(word) {
        Some(count) => *count += 1,
        None => {
            counts.insert(word.to_string(), 1);
        }
    });
}
because!(count_tokens, NeighbourClasses, "the count of each token of a text added to a table of counts");

pub fn frequent_words<S: BuildHasher>(counts: HashMap<String, u64, S>, most: usize) -> Vec<(String, u64)> {
    let mut words: Vec<(String, u64)> = counts.into_iter().collect();
    words.sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    words.truncate(most);
    words
}
because!(frequent_words, NeighbourClasses, "the most frequent words with their counts, the most frequent first and words of equal count in the order of their signs, so that the list is the same in every run");

pub fn word_codes<S: BuildHasher>(text: &str, numbers: &HashMap<String, u32, S>, codes: &mut Vec<u32>) {
    let mut token = String::new();
    for line in text.split(LINE_BREAK) {
        each_token(line, &mut token, &mut |word| codes.push(numbers.get(word).copied().unwrap_or(NOT_COUNTED)));
        codes.push(LINE_GAP);
    }
}
because!(word_codes, NeighbourClasses, "the codes of the tokens of a text, line by line, each line closed by the gap: the number of a counted word, or the code of no word");

pub fn row_width(context: usize) -> usize {
    ROW_HALVES * (context + 1)
}
because!(row_width, NeighbourClasses, "the places of a row: for the word before and for the word after, one place for each context word and one shared place for every other word");

fn context_place(code: u32, context: usize) -> usize {
    usize::try_from(code).ok().filter(|place| *place < context).unwrap_or(context)
}
because!(context_place, NeighbourClasses, "the place of a neighbour in a half of a row: its own place for a context word, else the shared place");

pub fn neighbour_rows(streams: &[Vec<u32>], words: Range<usize>, context: usize) -> Vec<u32> {
    let width = row_width(context);
    let after = context + 1;
    let mut rows = vec![0u32; words.len() * width];
    for codes in streams {
        for (at, code) in codes.iter().enumerate() {
            let Some(word) = usize::try_from(*code).ok().filter(|word| words.contains(word)) else { continue };
            let row = (word - words.start) * width;
            if let Some(before) = at.checked_sub(1).map(|place| codes[place]).filter(|before| *before != LINE_GAP) {
                rows[row + context_place(before, context)] += 1;
            }
            if let Some(next) = codes.get(at + 1).copied().filter(|next| *next != LINE_GAP) {
                rows[row + after + context_place(next, context)] += 1;
            }
        }
    }
    rows
}
because!(neighbour_rows, NeighbourClasses, "the rows of the counted words whose numbers lie in a range: for each place of such a word in the codes, its neighbour before and its neighbour after in the same line, each counted in its place of the row");

pub fn all_rows(streams: &[Vec<u32>], words: usize, context: usize, lanes: usize) -> Vec<u32> {
    let parts = shards(words, lanes);
    on_lanes(parts.len(), lanes, &|part| neighbour_rows(streams, parts[part].clone(), context)).concat()
}
because!(all_rows, NeighbourClasses, "the rows of all counted words, each thread counting the words of its own range over all codes, so that no count is shared between threads; the rows in the order of the words");

#[derive(Clone, Debug, PartialEq)]
pub struct WordClassing {
    pub classes: Vec<Option<usize>>,
    pub runs: usize,
}
because!(WordClassing, NeighbourClasses, "the class of each counted word, nothing for a word that stood beside too few words, and the number of rounds that the k-means made");

type SparseRow = Vec<(usize, f64)>;
because!(SparseRow, NeighbourClasses, "a row made comparable, as its places with a count and the value there, in the order of the places");

fn comparable_rows(rows: &[u32], width: usize) -> (Vec<usize>, Vec<SparseRow>) {
    let mut kept = Vec::new();
    let mut made = Vec::new();
    for (word, row) in rows.chunks(width).enumerate() {
        let seen: u64 = row.iter().map(|count| u64::from(*count)).sum();
        if seen < LEAST_SEEN {
            continue;
        }
        let length = (seen as f64).sqrt().max(NORM_FLOOR);
        kept.push(word);
        made.push(row.iter().enumerate().filter(|(_, count)| **count > 0).map(|(place, count)| (place, f64::from(*count).sqrt() / length)).collect());
    }
    (kept, made)
}
because!(comparable_rows, NeighbourClasses, "the rows of the words that stood beside enough words, each made comparable: the square root of each count, divided by the length of the row, which is the square root of the sum of the counts");

fn nearest_centres(rows: &[SparseRow], columns: &[f64], classes: usize, width: usize, out: &mut Vec<usize>) {
    let mut sums = vec![NO_SUM; rows.len() * classes];
    let mut cursors = vec![0usize; rows.len()];
    for place in 0..width {
        let column = &columns[place * classes..(place + 1) * classes];
        for (at, row) in rows.iter().enumerate() {
            let Some((held, value)) = row.get(cursors[at]) else { continue };
            if *held != place {
                continue;
            }
            cursors[at] += 1;
            sums[at * classes..(at + 1) * classes].iter_mut().zip(column).for_each(|(sum, centre)| *sum += value * centre);
        }
    }
    for found in sums.chunks(classes) {
        let mut best = 0usize;
        for (centre, sum) in found.iter().enumerate() {
            if *sum > found[best] {
                best = centre;
            }
        }
        out.push(best);
    }
}
because!(nearest_centres, NeighbourClasses, "for each row of a batch the centre with the largest dot product, the first of equals; the batch walks the places in their order and each place meets the rows of the batch that have a value there, so that the values of the centres at a place are read from memory once for the whole batch; each sum still adds the products of its row in the order of its places, so every sum is the sum of the row alone");

const NEAREST_BATCH: usize = 32;
because!(NEAREST_BATCH, NeighbourClasses, "the rows that share one walk over the places: their sums stay in the second cache of a core while the values of the centres at each place are read once for all of them");

fn centre_columns(centres: &[Vec<f64>], width: usize) -> Vec<f64> {
    let classes = centres.len();
    let mut columns = vec![NO_SUM; width * classes];
    for (centre, values) in centres.iter().enumerate() {
        for (place, value) in values.iter().enumerate() {
            columns[place * classes + centre] = *value;
        }
    }
    columns
}
because!(centre_columns, NeighbourClasses, "the centres held place by place: for each place of a row, the values of all centres there");

fn moved_centre(members: &[usize], rows: &[SparseRow], old: &[f64]) -> Option<(Vec<f64>, bool)> {
    if members.is_empty() {
        return None;
    }
    let mut mean = vec![NO_SUM; old.len()];
    for member in members {
        for (place, value) in &rows[*member] {
            mean[*place] += value;
        }
    }
    let count = members.len() as f64;
    mean.iter_mut().for_each(|value| *value /= count);
    let length = mean.iter().map(|value| value * value).sum::<f64>().sqrt().max(NORM_FLOOR);
    mean.iter_mut().for_each(|value| *value /= length);
    let change: f64 = mean.iter().zip(old).map(|(new, was)| (new - was).abs()).sum();
    Some((mean, change > MOVE_BOUND))
}
because!(moved_centre, NeighbourClasses, "the new place of a centre, the mean of its rows made of length one again, and whether it moved by more than the bound; nothing for a centre with no row, which stays where it is");

pub fn neighbour_classes(rows: &[u32], width: usize, classes: usize, most_runs: usize, lanes: usize) -> WordClassing {
    let words = rows.len() / width.max(1);
    let (kept, made) = comparable_rows(rows, width);
    let mut classing = WordClassing { classes: vec![None; words], runs: 0 };
    if made.is_empty() || classes == 0 {
        return classing;
    }
    let mut centres: Vec<Vec<f64>> = (0..classes)
        .map(|centre| {
            let row = if classes == 1 { 0 } else { centre * (made.len() - 1) / (classes - 1) };
            let mut dense = vec![NO_SUM; width];
            made[row].iter().for_each(|(place, value)| dense[*place] = *value);
            dense
        })
        .collect();
    let parts = shards(made.len(), lanes);
    let mut nearest: Vec<usize> = Vec::new();
    for _ in 0..most_runs {
        let columns = centre_columns(&centres, width);
        nearest = on_lanes(parts.len(), lanes, &|part| {
            let mut found: Vec<usize> = Vec::with_capacity(parts[part].len());
            for batch in made[parts[part].clone()].chunks(NEAREST_BATCH) {
                nearest_centres(batch, &columns, classes, width, &mut found);
            }
            found
        })
        .concat();
        classing.runs += 1;
        let mut members: Vec<Vec<usize>> = vec![Vec::new(); classes];
        nearest.iter().enumerate().for_each(|(row, centre)| members[*centre].push(row));
        let moved: Vec<Option<(Vec<f64>, bool)>> = on_lanes(classes, lanes, &|centre| moved_centre(&members[centre], &made, &centres[centre]));
        let mut any = false;
        for (centre, new) in moved.into_iter().enumerate() {
            if let Some((new, far)) = new {
                centres[centre] = new;
                any |= far;
            }
        }
        if !any {
            break;
        }
    }
    for (row, word) in kept.iter().enumerate() {
        classing.classes[*word] = Some(nearest[row]);
    }
    classing
}
because!(neighbour_classes, NeighbourClasses, "the classes of the counted words: the rows made comparable, the first centres at even steps of the list of kept words, then rounds of the k-means until no centre moves or the most rounds are done; each kept word gets the centre that the last round gave it");
