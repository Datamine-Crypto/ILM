use crate::base::words::words;
use crate::tools::pattern::classes::{ABSENT, KIND_MARK};
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::names::WordTexts;
use patterns::{because, source};

pub struct RepeatedParts;
source!(RepeatedParts, "a sentence that tells several steps in one row, as a list of moves, is read as one part for each repeat of the run of words that starts each step; the run that repeats most often is taken, then the longest, then the one that starts first");

pub type StatePiece = (Vec<u32>, Vec<u32>);
because!(StatePiece, RepeatedParts, "a piece of a sentence: its words as written, and the same words in their normal form");

#[derive(Clone, Debug)]
pub struct Cut {
    pub whole: StatePiece,
    pub lead: Option<StatePiece>,
    pub parts: Vec<StatePiece>,
}
because!(Cut, RepeatedParts, "a sentence as it is cut: the whole sentence, the text before the first repeat when there is one, and the parts, each without the word that stands between repeats");

pub struct Split {
    pub raw: Vec<u32>,
    pub norm: Vec<u32>,
    pub spans: Vec<(usize, usize)>,
    pub tokens: Vec<u32>,
    pub parts: Vec<(usize, usize)>,
}
because!(Split, RepeatedParts, "the first step of a sentence: its words, their normal forms, the word ranges of its tokens, the tokens, and the token ranges of its parts, with no parts when no run repeats");

pub fn join_ranges(norm: &[u32], has_run: &dyn Fn(&[u32]) -> bool, most: usize) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::with_capacity(norm.len());
    let mut at = 0;
    while at < norm.len() {
        let mut size = most.min(norm.len() - at);
        while size > 1 && !has_run(&norm[at..at + size]) {
            size -= 1;
        }
        let size = size.max(1);
        out.push((at, at + size));
        at += size;
    }
    out
}
because!(join_ranges, RepeatedParts, "the word ranges of a text with each known run of words as one range, the longest run first at each place");

fn periodic(gram: &[u32]) -> bool {
    let size = gram.len();
    (1..=size / STATE_PAIR_SIZE).any(|period| size % period == 0 && gram.chunks(period).all(|piece| piece == &gram[..period]))
}
because!(periodic, RepeatedParts, "a run that is a shorter run said again and again, which is no run of its own");

pub const STATE_PAIR_SIZE: usize = 2;
because!(STATE_PAIR_SIZE, RepeatedParts, "a run that repeats has at least this many tokens, and a sentence holds a run at most once for each share of its length of this size");

pub fn repeat_starts(shape: &[u32]) -> Vec<usize> {
    let count = shape.len();
    let mut best: Option<((usize, usize, isize), usize, usize)> = None;
    let mut seen: Vec<RunStarts> = Vec::new();
    for size in STATE_PAIR_SIZE..=count / STATE_PAIR_SIZE {
        seen.clear();
        for at in 0..=count - size {
            let gram = &shape[at..at + size];
            match seen.iter_mut().find(|held| &shape[held.first..held.first + size] == gram) {
                None => {
                    let plain = gram.iter().any(|entry| entry & KIND_MARK != 0) && !periodic(gram);
                    seen.push(RunStarts { first: at, last: at, starts: usize::from(plain), plain });
                }
                Some(held) => {
                    if held.plain && at >= held.last + size {
                        held.starts += 1;
                        held.last = at;
                    }
                }
            }
        }
        for held in &seen {
            if held.starts >= STATE_PAIR_SIZE {
                let key = (held.starts, size, -(held.first as isize));
                if best.as_ref().is_none_or(|(top, _, _)| key > *top) {
                    best = Some((key, size, held.first));
                }
            }
        }
    }
    let Some((_, size, first)) = best else { return Vec::new() };
    let gram = &shape[first..first + size];
    let mut starts: Vec<usize> = vec![first];
    for at in first + 1..=count - size {
        if at >= starts[starts.len() - 1] + size && &shape[at..at + size] == gram {
            starts.push(at);
        }
    }
    starts
}

struct RunStarts {
    first: usize,
    last: usize,
    starts: usize,
    plain: bool,
}
because!(RunStarts, RepeatedParts, "one run of tokens of a form while its starts are counted: where it stands first, its last start that was counted, how many starts were counted, and whether it is a run that can start a part; the list of the starts is made only for the run that is taken");
because!(repeat_starts, RepeatedParts, "the starts of the run of tokens, one of them of a kind and no repeat of a shorter run, that stands most often in a form without overlap; of these the longest, then the first; nothing when no run repeats");

pub fn token_of<N: WordTexts>(names: &mut N, norm: &[u32]) -> u32 {
    if norm.len() == 1 {
        return norm[0];
    }
    let text: Vec<&str> = norm.iter().map(|word| names.name(*word)).collect();
    let joined = text.join(crate::tools::pattern::kinds::JOINT);
    names.number(&joined)
}
because!(token_of, RepeatedParts, "the token of a range of words: the word itself, or the words with the mark of a run between them");

pub fn split<N: WordTexts>(first: &PatternWords, names: &mut N, sentence: &str, extra: &[(u32, u32)]) -> Split {
    let raw: Vec<u32> = words(sentence).iter().map(|word| names.number(word)).collect();
    let norm: Vec<u32> = raw.iter().map(|word| first.norm(*word)).collect();
    let spans = join_ranges(&norm, &|run: &[u32]| first.runs.contains_key(run), crate::tools::pattern::kinds::MOST_RUN);
    let tokens: Vec<u32> = spans.iter().map(|(from, to)| token_of(names, &norm[*from..*to])).collect();
    let shape: Vec<u32> = tokens
        .iter()
        .map(|token| {
            let kind = first.kind_with(*token, extra);
            if kind == ABSENT { *token } else { KIND_MARK | kind }
        })
        .collect();
    let starts = repeat_starts(&shape);
    let parts: Vec<(usize, usize)> = starts.iter().enumerate().map(|(place, start)| (*start, starts.get(place + 1).copied().unwrap_or(tokens.len()))).collect();
    Split { raw, norm, spans, tokens, parts }
}
because!(split, RepeatedParts, "the first step of a sentence: the words, their normal forms by the first step of the pattern learner, its tokens with the runs of that learner, and the parts where a run repeats");

pub fn cut<N: WordTexts>(first: &PatternWords, names: &mut N, between: &dyn Fn(u32) -> bool, sentence: &str, extra: &[(u32, u32)]) -> Cut {
    let Split { raw, norm, spans, tokens, parts } = split(first, names, sentence, extra);
    let lead = match parts.first() {
        Some((start, _)) if *start > 0 => {
            let until = spans[*start].0;
            Some((raw[..until].to_vec(), norm[..until].to_vec()))
        }
        _ => None,
    };
    let mut out: Vec<StatePiece> = Vec::with_capacity(parts.len());
    for (place, (from, to)) in parts.iter().enumerate() {
        let mut end = *to;
        if place + 1 < parts.len() && end - from > 1 && between(tokens[end - 1]) {
            end -= 1;
        }
        let (start, stop) = (spans[*from].0, spans[end - 1].1);
        out.push((raw[start..stop].to_vec(), norm[start..stop].to_vec()));
    }
    Cut { whole: (raw, norm), lead, parts: out }
}
because!(cut, RepeatedParts, "a sentence as its whole, the text before the first repeat, and its parts; a part before the last one loses its last token when that token is a word that stands between repeats");
