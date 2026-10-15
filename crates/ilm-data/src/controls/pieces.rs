use crate::controls::file::ChangedCase;
use crate::controls::text::list_pieces;
use crate::controls::things::{choice_segments, choices_mark};
use ilm_core::base::words::{answer_words, words};
use ilm_core::cases::Case;
use ilm_core::choice::inner_parts;
use patterns::{because, source};

pub struct PieceChanges;
source!(PieceChanges, "changed stories of things in places: the question lists the places as its choices, and a told sentence lists as many pieces as there are places, each piece starting with the same word, and so names a kind of things with one thing in each place; the first and the last piece of every such list change places. A list says which things there are and not where they stand, so the truth stays; a learner that answers by the place of a thing in its list does not follow the change");

const PIECES_SORT: &str = "the first and the last piece of every told list change places";
because!(PIECES_SORT, PieceChanges, "the sort of the change, where the truth stays");

const PIECES_MARK: char = ':';
because!(PIECES_MARK, PieceChanges, "the mark after which a told sentence gives its list");

const PIECES_LEAST: usize = 2;
because!(PIECES_LEAST, PieceChanges, "the fewest pieces of a list where a first and a last piece differ");

const PIECES_GAP: &str = " ";
because!(PIECES_GAP, PieceChanges, "the mark between the words of the truth");

fn piece_span(listed: &str, piece: &str) -> (usize, usize) {
    let start = piece.as_ptr() as usize - listed.as_ptr() as usize;
    let kept = piece.trim_start();
    let from = start + piece.len() - kept.len();
    let word_end = kept.trim_end_matches(|sign: char| !sign.is_alphanumeric());
    (from, from + word_end.len())
}
because!(piece_span, PieceChanges, "the place of a piece in the text of its list, without the spaces before it and the signs that are not letters or digits after it");

fn ends_exchanged(sentence: &str, places: usize) -> Option<String> {
    let (head, listed) = sentence.rsplit_once(PIECES_MARK)?;
    let pieces = list_pieces(listed);
    if places < PIECES_LEAST || pieces.len() != places {
        return None;
    }
    let starts: Vec<&str> = pieces.iter().map(|piece| piece.split_whitespace().next().unwrap_or_default()).collect();
    if starts.iter().any(|word| word.is_empty() || *word != starts[0]) {
        return None;
    }
    let (first, last) = (piece_span(listed, pieces[0]), piece_span(listed, pieces[places - 1]));
    if first.1 > last.0 {
        return None;
    }
    let changed = [&listed[..first.0], &listed[last.0..last.1], &listed[first.1..last.0], &listed[first.0..first.1], &listed[last.1..]].concat();
    Some(format!("{head}{PIECES_MARK}{changed}"))
}
because!(ends_exchanged, PieceChanges, "a told sentence with the first and the last piece of its list changed for each other, when it lists as many pieces as there are places and each piece starts with the same word; nothing for any other sentence");

pub fn piece_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let Some(mark) = choices_mark(lessons) else { return Vec::new() };
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        let places = choice_segments(&words(&case.question), &mark).len();
        let mut listed = false;
        let sentences: Vec<String> = case
            .sentences
            .iter()
            .map(|sentence| match ends_exchanged(sentence, places) {
                Some(changed) => {
                    listed = true;
                    changed
                }
                None => sentence.clone(),
            })
            .collect();
        if listed {
            out.push(ChangedCase { place: Some(place), sort: PIECES_SORT.to_string(), sentences, question: case.question.clone(), truth: answer_words(&case.answer).join(PIECES_GAP) });
        }
    }
    out
}
because!(piece_changes, PieceChanges, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons: every story with a list of things, with the first and the last piece of each list changed, and the old truth");
