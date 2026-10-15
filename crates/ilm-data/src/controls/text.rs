use ilm_core::know::signs::is_space;
use patterns::{because, source};

pub struct StoryPieces;
source!(StoryPieces, "a told text read by its form alone: a name is a capital letter of the Latin alphabet with small letters after it; a number is a run of digits, with a minus sign before it where a number may be below nought; a list is cut into pieces at a comma with the spaces after it and the joining word if it comes next, or at the joining word between spaces; space is the white space of the Unicode character database with the separators of the first code points; a word is in small letters when it has a letter with case and every such letter is small");

const PIECE_COMMA: char = ',';
because!(PIECE_COMMA, StoryPieces, "the mark that ends a piece of a list");

const PIECE_AND: &str = "and";
because!(PIECE_AND, StoryPieces, "the word that joins the last piece of a list to the others, after a comma or between spaces");

const NUMBER_MINUS: char = '-';
because!(NUMBER_MINUS, StoryPieces, "the sign before a number below nought");


pub fn byte_runs(text: &str, keep: fn(&u8) -> bool) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if !keep(&bytes[at]) {
            at += 1;
            continue;
        }
        let start = at;
        while at < bytes.len() && keep(&bytes[at]) {
            at += 1;
        }
        out.push((start, at));
    }
    out
}
because!(byte_runs, StoryPieces, "the places of the runs of a text whose signs all pass a test: the words of a text by the signs that a word is made of");

pub fn capital_name_runs(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at].is_ascii_uppercase() {
            let mut end = at + 1;
            while end < bytes.len() && bytes[end].is_ascii_lowercase() {
                end += 1;
            }
            if end > at + 1 {
                out.push((at, end));
                at = end;
                continue;
            }
        }
        at += 1;
    }
    out
}
because!(capital_name_runs, StoryPieces, "the places of the names of a text in their order: each run of a capital letter and the small letters after it, read from the left with no overlap");

pub fn paired(table: &[(&str, &'static str)], word: &str) -> Option<&'static str> {
    table.iter().find(|(held, _)| *held == word).map(|(_, other)| *other)
}
because!(paired, StoryPieces, "the second word of the pair of a table whose first word is the given word, or nothing");

pub fn run_texts<'a>(text: &'a str, runs: &[(usize, usize)]) -> Vec<&'a str> {
    runs.iter().map(|(start, end)| &text[*start..*end]).collect()
}
because!(run_texts, StoryPieces, "the pieces of a text at some places, in their order");

pub fn capital_names(text: &str) -> Vec<&str> {
    run_texts(text, &capital_name_runs(text))
}
because!(capital_names, StoryPieces, "the names of a text in their order");

fn joining_word(signs: &[(usize, char)], at: usize) -> Option<usize> {
    let mut end = at;
    for sign in PIECE_AND.chars() {
        if signs.get(end).map(|(_, found)| *found) != Some(sign) {
            return None;
        }
        end += 1;
    }
    if !signs.get(end).is_some_and(|(_, found)| is_space(*found)) {
        return None;
    }
    while end < signs.len() && is_space(signs[end].1) {
        end += 1;
    }
    Some(end)
}
because!(joining_word, StoryPieces, "the end of the joining word and of the spaces after it, when the joining word stands at a place and at least one space follows it");

fn piece_mark(signs: &[(usize, char)], at: usize) -> Option<usize> {
    let mut end = at;
    if signs[at].1 == PIECE_COMMA {
        end += 1;
        while end < signs.len() && is_space(signs[end].1) {
            end += 1;
        }
        return Some(joining_word(signs, end).unwrap_or(end));
    }
    while end < signs.len() && is_space(signs[end].1) {
        end += 1;
    }
    if end == at {
        return None;
    }
    joining_word(signs, end)
}
because!(piece_mark, StoryPieces, "the end of the mark between two pieces of a list that starts at a place: a comma with the spaces after it and a joining word that may follow, or spaces with a joining word and spaces; nothing when no such mark starts there");

pub fn list_pieces(text: &str) -> Vec<&str> {
    let signs: Vec<(usize, char)> = text.char_indices().collect();
    let place = |at: usize| signs.get(at).map_or(text.len(), |(byte, _)| *byte);
    let mut out: Vec<&str> = Vec::new();
    let (mut start, mut at) = (0usize, 0usize);
    while at < signs.len() {
        match piece_mark(&signs, at) {
            Some(end) => {
                out.push(&text[place(start)..place(at)]);
                start = end;
                at = end;
            }
            None => at += 1,
        }
    }
    out.push(&text[place(start)..]);
    out
}
because!(list_pieces, StoryPieces, "the pieces of a list between its marks, read from the left, with an empty piece where two marks meet or a mark stands at an end");

pub fn number_runs(text: &str, signed: bool) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let minus = signed && char::from(bytes[at]) == NUMBER_MINUS && bytes.get(at + 1).is_some_and(u8::is_ascii_digit);
        if minus || bytes[at].is_ascii_digit() {
            let mut end = at + usize::from(minus);
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            out.push((at, end));
            at = end;
            continue;
        }
        at += 1;
    }
    out
}
because!(number_runs, StoryPieces, "the places of the numbers of a text, each a run of digits and, where asked, the minus sign before it, read from the left with no overlap");

pub fn with_numbers(text: &str, runs: &[(usize, usize)], values: &[String]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for ((start, end), value) in runs.iter().zip(values) {
        out.push_str(&text[last..*start]);
        out.push_str(value);
        last = *end;
    }
    out.push_str(&text[last..]);
    out
}
because!(with_numbers, StoryPieces, "a text whose first numbers are written anew, in their order, and whose other signs stay");
