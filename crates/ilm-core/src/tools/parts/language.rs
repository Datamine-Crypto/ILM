use patterns::{because, source};
use std::collections::HashSet;

pub struct PartsLanguage;
source!(PartsLanguage, "the principle of Frege that the meaning of a text is built from the meanings of its parts, as a small language of operations on rows of answer words: the answer of a part, two answers one after the other in either order, a row some times, an answer turned, an answer without the word at one end, the word at one end alone, and a word that a table gives for one word of a part; a program is a row of such pieces, and which program a form of the text means is found from lessons");

pub type Word = u32;
because!(Word, PartsLanguage, "a word as the number that it got when it was first seen, so that rows are compared as numbers");

pub type WordRow = Vec<Word>;
because!(WordRow, PartsLanguage, "a row of answer words: the answer of a part or of a whole text");

pub const NO_OPERATION: u8 = 0;
because!(NO_OPERATION, PartsLanguage, "the place of an operation that is not there: a view holds fewer operations than it has room for");

pub const TURNED: u8 = 1;
because!(TURNED, PartsLanguage, "the operation that gives an answer with its words in the opposite order; it comes first in the list of the operations, and the numbers of the operations are the order in which two programs of the same size are compared");

pub const LESS_FIRST: u8 = 2;
because!(LESS_FIRST, PartsLanguage, "the operation that gives an answer without its first word, after the turn in the list of the operations");

pub const LESS_LAST: u8 = 3;
because!(LESS_LAST, PartsLanguage, "the operation that gives an answer without its last word, after the cut at the start in the list of the operations");

pub const FIRST_OF: u8 = 4;
because!(FIRST_OF, PartsLanguage, "the operation that gives the first word of an answer alone, after the cuts in the list of the operations");

pub const LAST_OF: u8 = 5;
because!(LAST_OF, PartsLanguage, "the operation that gives the last word of an answer alone, the last in the list of the operations");

pub const MOST_PIECES: usize = 4;
because!(MOST_PIECES, PartsLanguage, "the pieces of a program at most: a form has few parts, and a longer row of pieces fits any answer by chance");

pub const PIECES_WITH_LOOK: usize = 8;
because!(PIECES_WITH_LOOK, PartsLanguage, "the pieces at most of a program that holds a looked-up piece: such a program is found from an answer with words that no part says, and those words stand between the pieces");

pub const ROW_MOST: usize = 2000;
because!(ROW_MOST, PartsLanguage, "the answer words of a row at most: a longer row is not built and its text is not read, so a program that doubles a row again and again ends");

pub const GAPS_MOST: usize = 6;
because!(GAPS_MOST, PartsLanguage, "the words of one answer at most that no part says: with more of them the answer is not made from the parts");

pub const GUESSES: usize = 48;
because!(GUESSES, PartsLanguage, "the guessed programs of a form that are tried, the simplest first: a program beyond these is too large to be told from chance by few lessons");

pub const LOOKED: u8 = 1;
because!(LOOKED, PartsLanguage, "the number of the first table of a looked-up piece, the least number that is not the mark of no operation; a piece whose words do not agree on a single table gets the next numbers");

pub const PARTS_OF_FORM: usize = 2;
because!(PARTS_OF_FORM, PartsLanguage, "a form of the list of shapes has this many parts at most; the places after them in the answers that a program is run on hold what the fixed words of the form say");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnswerView {
    pub first: u8,
    pub then: u8,
}
because!(AnswerView, PartsLanguage, "the operations that an answer is under, in their order: none is the plain answer. on gives the answer of a row under the view; size is the number of its operations; a view is compared by its operations in their order, so the plain view is the least");

impl AnswerView {
    pub fn plain(&self) -> bool {
        self.first == NO_OPERATION
    }

    pub fn operations(&self) -> impl Iterator<Item = u8> {
        [self.first, self.then].into_iter().filter(|operation| *operation != NO_OPERATION)
    }

    pub fn size(&self) -> usize {
        self.operations().count()
    }

    pub fn on(&self, row: &[Word]) -> WordRow {
        let mut out: WordRow = row.to_vec();
        for operation in self.operations() {
            match operation {
                TURNED => out.reverse(),
                LESS_FIRST => {
                    if !out.is_empty() {
                        out.remove(0);
                    }
                }
                LESS_LAST => {
                    out.pop();
                }
                FIRST_OF => out.truncate(1),
                _ => {
                    let dropped = out.len().saturating_sub(1);
                    out.drain(..dropped);
                }
            }
        }
        out
    }
}

pub const PLAIN: AnswerView = AnswerView { first: NO_OPERATION, then: NO_OPERATION };
because!(PLAIN, PartsLanguage, "the view with no operation: the answer of a part as it is");

pub const VIEWS: &[AnswerView] = &[
    PLAIN,
    AnswerView { first: TURNED, then: NO_OPERATION },
    AnswerView { first: LESS_FIRST, then: NO_OPERATION },
    AnswerView { first: LESS_LAST, then: NO_OPERATION },
    AnswerView { first: FIRST_OF, then: NO_OPERATION },
    AnswerView { first: LAST_OF, then: NO_OPERATION },
    AnswerView { first: LESS_FIRST, then: LESS_LAST },
    AnswerView { first: LESS_FIRST, then: LESS_FIRST },
    AnswerView { first: LESS_LAST, then: LESS_LAST },
    AnswerView { first: LESS_FIRST, then: TURNED },
    AnswerView { first: LESS_LAST, then: TURNED },
    AnswerView { first: LESS_FIRST, then: FIRST_OF },
    AnswerView { first: LESS_LAST, then: LAST_OF },
    AnswerView { first: LESS_LAST, then: FIRST_OF },
    AnswerView { first: LESS_FIRST, then: LAST_OF },
];
because!(VIEWS, PartsLanguage, "an answer under no operation, under one, or under a pair of them: the pairs that differ from every shorter view and from every other pair for some row; the order is the order in which the views are tried, the simpler first");

pub const ONLY_PLAIN: &[AnswerView] = &[PLAIN];
because!(ONLY_PLAIN, PartsLanguage, "the views of a search that uses the answers of the parts as they are");

pub const KEY_VIEWS: &[AnswerView] = &[AnswerView { first: FIRST_OF, then: NO_OPERATION }, AnswerView { first: LAST_OF, then: NO_OPERATION }, AnswerView { first: LESS_FIRST, then: FIRST_OF }, AnswerView { first: LESS_LAST, then: LAST_OF }];
because!(KEY_VIEWS, PartsLanguage, "the views that give one word of a part as the key of a table: the word at an end of the part, or the word next to it");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Piece {
    pub slot: u8,
    pub view: AnswerView,
    pub look: u8,
}
because!(Piece, PartsLanguage, "one piece of a program: the part whose answer it takes (the parts of the form first, then what its fixed words say), the view of that answer, and the number of the table that the piece stands under, or none. Pieces are compared by part, view and table in this order, a piece with no table before one with a table");

impl Piece {
    pub fn of(slot: usize, view: AnswerView) -> Piece {
        Piece { slot: slot as u8, view, look: NO_OPERATION }
    }

    pub fn looked(&self) -> bool {
        self.look != NO_OPERATION
    }

    pub fn under(&self, table: u8) -> Piece {
        Piece { slot: self.slot, view: self.view, look: table }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Program {
    pub pieces: Vec<Piece>,
    pub times: usize,
}
because!(Program, PartsLanguage, "a program of the language: a row of pieces, and how many times the row that they make is said. nothing is the program with no piece; looks says whether a piece stands under a table; two programs are compared by their pieces from the left, then by the times");

impl Program {
    pub fn nothing() -> Program {
        Program { pieces: Vec::new(), times: 1 }
    }

    pub fn looks(&self) -> bool {
        self.pieces.iter().any(Piece::looked)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PartAnswers {
    pub rows: Vec<Option<WordRow>>,
}
because!(PartAnswers, PartsLanguage, "the answers that the program of a form is run on: the answers of its parts, and after them what its fixed words say when one of them says something; a place with no answer is empty. row gives the answer of a place, and nothing for an empty place or a place that is not there");

impl PartAnswers {
    pub fn of(rows: Vec<Option<WordRow>>) -> PartAnswers {
        PartAnswers { rows }
    }

    pub fn row(&self, slot: usize) -> Option<&WordRow> {
        self.rows.get(slot).and_then(Option::as_ref)
    }
}

pub fn shortest_program(pieces: &[Piece]) -> Program {
    let size = pieces.len();
    for unit in 1..=size {
        if size % unit == 0 && pieces.chunks(unit).all(|chunk| chunk == &pieces[..unit]) {
            return Program { pieces: pieces[..unit].to_vec(), times: size / unit };
        }
    }
    Program::nothing()
}
because!(shortest_program, PartsLanguage, "a row of pieces as the shortest row that it repeats and the number of times, so that one program has one form");

fn ways_into(unit: &[Word], values: &[(Piece, WordRow)], most: usize, so_far: &mut Vec<Piece>, found: &mut Vec<Vec<Piece>>, one: bool) -> bool {
    if unit.is_empty() {
        found.push(so_far.clone());
        return one;
    }
    if most == 0 {
        return false;
    }
    for (piece, value) in values {
        if unit.starts_with(value) {
            so_far.push(*piece);
            let stop = ways_into(&unit[value.len()..], values, most - 1, so_far, found, one);
            so_far.pop();
            if stop {
                return true;
            }
        }
    }
    false
}
because!(ways_into, PartsLanguage, "the rows of at most so many pieces whose values in a row are the unit, each tried in the order of the values, the first piece first; with one it stops at the first row found");

fn values_of(given: &PartAnswers, views: &[AnswerView]) -> Vec<(Piece, WordRow)> {
    let mut values: Vec<(Piece, WordRow)> = Vec::new();
    for (slot, part) in given.rows.iter().enumerate() {
        let Some(part) = part.as_ref().filter(|part| !part.is_empty()) else { continue };
        let mut seen: Vec<WordRow> = Vec::new();
        for view in views {
            let value = view.on(part);
            if !value.is_empty() && !seen.contains(&value) {
                seen.push(value.clone());
                values.push((Piece::of(slot, *view), value));
            }
        }
    }
    values
}
because!(values_of, PartsLanguage, "one piece for each different row that the views of a part give, the parts in their order and the simpler view of equal rows first; a part with no answer word gives no piece");

pub fn cut_programs(row: &[Word], given: &PartAnswers, views: &[AnswerView]) -> Vec<Program> {
    if given.rows.iter().take(PARTS_OF_FORM).any(|part| part.as_ref().is_some_and(Vec::is_empty)) {
        return Vec::new();
    }
    if row.is_empty() {
        return vec![Program::nothing()];
    }
    let values = values_of(given, views);
    let mut found: Vec<Program> = Vec::new();
    let mut seen: HashSet<Program> = HashSet::new();
    let size = row.len();
    for times in 1..=size {
        if size % times != 0 {
            continue;
        }
        let unit = &row[..size / times];
        if !row.chunks(unit.len()).all(|chunk| chunk == unit) {
            continue;
        }
        let mut rows: Vec<Vec<Piece>> = Vec::new();
        ways_into(unit, &values, MOST_PIECES, &mut Vec::new(), &mut rows, false);
        for pieces in rows {
            let program = shortest_program(&pieces.repeat(times));
            if seen.insert(program.clone()) {
                found.push(program);
            }
        }
    }
    found
}
because!(cut_programs, PartsLanguage, "every program of the language that makes a row from the answers of some parts, each once; a part that says nothing tells nothing about its place in a program, so with such a part there is no program");

pub fn gap_programs(row: &[Word], given: &PartAnswers) -> Vec<Vec<Option<Piece>>> {
    let said: HashSet<Word> = given.rows.iter().flatten().flatten().copied().collect();
    let gaps: Vec<usize> = (0..row.len()).filter(|place| !said.contains(&row[*place])).collect();
    if gaps.is_empty() || gaps.len() > GAPS_MOST {
        return Vec::new();
    }
    let values = values_of(given, VIEWS);
    let mut pieces: Vec<Option<Piece>> = Vec::new();
    let mut at = 0;
    for stop in gaps.iter().copied().chain([row.len()]) {
        if stop > at {
            let mut found: Vec<Vec<Piece>> = Vec::new();
            for most in 1..=MOST_PIECES {
                ways_into(&row[at..stop], &values, most, &mut Vec::new(), &mut found, true);
                if !found.is_empty() {
                    break;
                }
            }
            let Some(run) = found.into_iter().next() else { return Vec::new() };
            pieces.extend(run.into_iter().map(Some));
        }
        if stop < row.len() {
            pieces.push(None);
        }
        at = stop + 1;
    }
    if pieces.len() <= PIECES_WITH_LOOK { vec![pieces] } else { Vec::new() }
}
because!(gap_programs, PartsLanguage, "the program for a row that holds words that no part and no fixed word of the form says: each such word is a gap of one word, and each run between two gaps is cut into the fewest pieces; no program when the row has no such word, too many of them, or a run that cannot be cut");

pub type Lay = Vec<(usize, Option<WordRow>, WordRow)>;
because!(Lay, PartsLanguage, "how a program with gaps or looked-up pieces lies on a row: for each gap and each looked-up piece the number of the piece, the key (nothing for a gap) and the word of the row at that place");

pub fn laid_on(pieces: &[Option<Piece>], given: &PartAnswers, row: &[Word]) -> Option<Lay> {
    let mut found: Lay = Vec::new();
    let mut at = 0;
    for (number, piece) in pieces.iter().enumerate() {
        let Some(piece) = piece else {
            if at >= row.len() {
                return None;
            }
            found.push((number, None, row[at..=at].to_vec()));
            at += 1;
            continue;
        };
        let part = given.row(piece.slot as usize)?;
        let value = piece.view.on(part);
        if piece.looked() {
            if at >= row.len() || value.is_empty() {
                return None;
            }
            found.push((number, Some(value), row[at..=at].to_vec()));
            at += 1;
        } else if !row[at.min(row.len())..].starts_with(&value) {
            return None;
        } else {
            at += value.len();
        }
    }
    (at == row.len()).then_some(found)
}
because!(laid_on, PartsLanguage, "the gaps and the looked-up pieces of a program on a row, or nothing when the row is not made in this way");

pub fn whole_pieces(program: &Program) -> Vec<Option<Piece>> {
    program.pieces.iter().copied().map(Some).collect()
}
because!(whole_pieces, PartsLanguage, "the pieces of a finished program in the form of a program with gaps, so that one walk lays both on a row");
