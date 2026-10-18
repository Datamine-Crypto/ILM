use crate::cases::Case;
use crate::tools::call::AnswerMark;
use crate::tools::explain::text::{Condition, LogicPiece, Pattern, Topic, fact_numbers};
use crate::tools::explain::words::WordId;
use patterns::{because, source};
use std::collections::HashSet;

pub struct ReadFrames;
source!(ReadFrames, "the reject option of a classifier (Chow): a reader that answers only where its evidence holds and marks the rest makes no wrong answer that claims to be sure. A line of the engine fits a sentence by the words that it names and by the open places between them, each of which takes a run of any words, so sentences with the same named words in the same order and runs of the same lengths between them are read in the same way, whatever the names and places in the runs. Lines that read parts of a sentence (a list, a sum, a command of commands) meet new sentences that are new rows of parts the lessons showed, so a sentence is judged by its pieces two at a time, as a model of word pairs judges a new text (Shannon): a pair of pieces in a row that no lesson showed, such as a run of words where the lessons had one word before a named word, may be read as something else");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FramePiece {
    Word(WordId),
    Run(usize),
}
because!(FramePiece, ReadFrames, "one piece of the frame of a sentence: a word that a line of the model names, or a run of other words with its length, since an open place takes any words and only their count changes how a line fits");

#[derive(Debug, Default)]
pub struct SeenFrames {
    named: HashSet<WordId>,
    told: HashSet<PiecePair>,
    asked: HashSet<PiecePair>,
}
because!(SeenFrames, ReadFrames, "the pairs of pieces in a row that the lessons of a model showed, in their told sentences and in their questions apart, over the words that the lines of the model name");

type PiecePair = (Option<FramePiece>, Option<FramePiece>);
because!(PiecePair, ReadFrames, "two pieces in a row of the frame of a sentence, nothing before its first piece and after its last, so that a run at the start or at the end of a sentence is a pair of its own");

fn frame(words: &[WordId], named: &HashSet<WordId>) -> Vec<FramePiece> {
    let mut pieces: Vec<FramePiece> = Vec::new();
    for word in words {
        if named.contains(word) {
            pieces.push(FramePiece::Word(*word));
        } else if let Some(FramePiece::Run(length)) = pieces.last_mut() {
            *length += 1;
        } else {
            pieces.push(FramePiece::Run(1));
        }
    }
    pieces
}
because!(frame, ReadFrames, "the frame of a sentence: each word that a line names, and each run of the other words as its length, so that new names and places leave the frame as it was");

fn piece_pairs(words: &[WordId], named: &HashSet<WordId>) -> Vec<PiecePair> {
    let pieces = frame(words, named);
    let mut pairs: Vec<PiecePair> = Vec::with_capacity(pieces.len() + 1);
    let mut before = None;
    for piece in pieces {
        pairs.push((before, Some(piece)));
        before = Some(piece);
    }
    pairs.push((before, None));
    pairs
}
because!(piece_pairs, ReadFrames, "the pairs of pieces in a row of the frame of a sentence, from its start to its end");

fn pattern_words(pattern: &Pattern) -> impl Iterator<Item = WordId> + '_ {
    pattern.iter().filter_map(|logic_piece| match logic_piece {
        LogicPiece::Word(word) => Some(*word),
        LogicPiece::Open(_) | LogicPiece::OpenWithEnd(..) => None,
    })
}
because!(pattern_words, ReadFrames, "the words that a sentence with open places names");

fn condition_words(condition: &Condition) -> Vec<WordId> {
    match condition {
        Condition::Fact { pattern, .. } | Condition::Count { pattern, .. } | Condition::Proves { statement: pattern, .. } => pattern_words(pattern).collect(),
        Condition::Alike { first, second } | Condition::Follows { statement: first, said: second } => pattern_words(first).chain(pattern_words(second)).collect(),
        Condition::Holds { word: LogicPiece::Word(word), .. } => vec![*word],
        Condition::HoldsKind { kind, .. } => kind.clone(),
        _ => Vec::new(),
    }
}
because!(condition_words, ReadFrames, "the words of a condition that a told sentence may hold for it to fit: those of its sentences and the word or the words of a kind that it asks a sentence to hold");

fn named_words(topics: &[&Topic]) -> HashSet<WordId> {
    let mut named: HashSet<WordId> = HashSet::new();
    for topic in topics {
        for condition in topic.rules.iter().flat_map(|rule| rule.conditions.iter().chain(&rule.absent)) {
            named.extend(condition_words(condition));
        }
        for form in &topic.questions {
            named.extend(pattern_words(&form.question));
            for condition in form.replies.iter().flat_map(|reply| reply.conditions.iter().chain(&reply.absent)) {
                named.extend(condition_words(condition));
            }
        }
    }
    named
}
because!(named_words, ReadFrames, "the words that the rules and the question forms of the topics of a model name, those by which its lines fit a sentence");

impl SeenFrames {
    pub(super) fn of(topics: &[&Topic], cases: &[Case]) -> SeenFrames {
        let named = named_words(topics);
        let sentences: HashSet<&String> = cases.iter().flat_map(|case| &case.sentences).collect();
        let questions: HashSet<&String> = cases.iter().map(|case| &case.question).collect();
        let told = sentences.into_iter().flat_map(|sentence| piece_pairs(&fact_numbers(sentence), &named)).collect();
        let asked = questions.into_iter().flat_map(|question| piece_pairs(&fact_numbers(question), &named)).collect();
        SeenFrames { named, told, asked }
    }

    fn unseen(&self, sentences: &[String], question: &str) -> bool {
        piece_pairs(&fact_numbers(question), &self.named).iter().any(|pair| !self.asked.contains(pair)) || sentences.iter().any(|sentence| piece_pairs(&fact_numbers(sentence), &self.named).iter().any(|pair| !self.told.contains(pair)))
    }
}

pub(super) fn answer_mark(words: &[String], by_default: bool, (frames, sentences, question): (Option<&SeenFrames>, &[String], &str)) -> AnswerMark {
    if words.is_empty() {
        AnswerMark::NoAnswer
    } else if by_default {
        AnswerMark::Default
    } else if frames.is_some_and(|frames| frames.unseen(sentences, question)) {
        AnswerMark::Guess
    } else {
        AnswerMark::Proof
    }
}
because!(answer_mark, ReadFrames, "the mark of an answer of a model of the explain tool: none with no words, by default when it holds otherwise, a guess when a told sentence or the question of the case holds a pair of pieces in a row that the lessons of the model did not show, else by proof; a model that keeps no frames of its lessons marks no guess");
