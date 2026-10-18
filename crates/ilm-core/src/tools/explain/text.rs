use crate::tools::explain::chain::rules_key;
use crate::tools::explain::words::{WordId, word_number, word_text};
use patterns::{because, source};
use std::collections::HashMap;

pub struct ToldLogic;
source!(ToldLogic, "the logic of a kind of question told in English, as a book states a rule for any things: a rule is a sentence 'If <sentence> and <sentence> then <sentence>', and a question form is a question with its answers, each answer with the sentences under which it holds; a capital letter from A to P, but the pronoun I, is an open place for one or more words, a capital letter from R to Z is an open place for exactly one word, as a name or a verb, and such a letter before an ending inside one word (of a plural, a comparison, a form of a verb or an owner) is the stem of a word with that ending; a hyphen or an apostrophe between two letters joins one word (self-assured, Port-au-Prince, Mary's), and a low line parts words as a space does (green_bottle is green bottle); a condition that starts with 'not' holds when no fact fits its sentence, or when the comparison, step of arithmetic or count after it does not hold, as negation as failure reads a closed world (Clark), and a condition that starts with 'latest' is held only by the latest fact that fits it, as the state of a thing is the last that the story told of it (with every place filled, the last word or place of the sentence is that state); a condition that starts with 'at that time' is held by the latest fact that fits it up to the place in the story of the fact that the condition before it read, and one that starts with 'before that time' by the latest that fits it before that place, as English 'at that time' points to the clause before it");

pub(crate) const NOT_START: &str = "not ";
because!(NOT_START, ToldLogic, "the word that opens a condition that holds when no fact fits the rest of it");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Moment {
    Any,
    Latest,
    AtThatTime,
    BeforeThatTime,
}
because!(Moment, ToldLogic, "the facts that may hold the sentence of a condition: any fact that fits it, the latest that fits it, the latest up to the place in the story of the fact that the condition before it read, or the latest before that place");

const TIMED_STARTS: &[(&str, Moment)] = &[("latest ", Moment::Latest), ("at that time ", Moment::AtThatTime), ("before that time ", Moment::BeforeThatTime)];
because!(TIMED_STARTS, ToldLogic, "the words that open a condition that only one fact of the story may hold, by its place in the story: the latest fitting fact, the latest at the time of the clause before it, or the latest before that time");

const OPEN_FIRST: char = 'A';
because!(OPEN_FIRST, ToldLogic, "open places are letters from the start of the alphabet, as the rules of a book of logic write them (A, B, C)");

const OPEN_LAST: char = 'P';
because!(OPEN_LAST, ToldLogic, "the last letter that is an open place: a rule of the library about a list of five names and five places needs ten places, and a capital letter after P, such as the Q of a question, stays a word");

const NOT_OPEN: char = 'I';
because!(NOT_OPEN, ToldLogic, "the one capital letter among the open places that stays a word, since English writes the pronoun I with it");

pub const PLAIN_PLACES: usize = 15;
because!(PLAIN_PLACES, ToldLogic, "the number of open places of one or more words: the letters from A to P without I");

pub const OPEN_PLACES: usize = ANY_FIRST + ANY_PLACES;
because!(OPEN_PLACES, ToldLogic, "the number of open places: the letters from A to P without I, then the places of one word, from R to Z, then the places that may hold marks, the letters from A to P without I written twice, then the places that may hold whole groups in brackets, the same letters written three times, then the places of one sign of any kind, the first letters of the places of one word written twice, then the places of any signs, the first letters written four times");

pub const ONE_SIGN_FIRST: usize = GROUPED_FIRST + PLAIN_PLACES;
because!(ONE_SIGN_FIRST, ToldLogic, "the number of the first open place of one sign of any kind, after the places that may hold whole groups in brackets");

const ONE_SIGN_PLACES: usize = 4;
because!(ONE_SIGN_PLACES, ToldLogic, "the places of one sign of any kind, named by the first letters of the places of one word: a line that reads a sequence sign by sign reads a few signs next to each other (a bracket and the bracket that closes it), and every place more makes each filling of each rule longer, which every rule of every kind pays for");

pub const ANY_FIRST: usize = ONE_SIGN_FIRST + ONE_SIGN_PLACES;
because!(ANY_FIRST, ToldLogic, "the number of the first open place that may hold any signs, after the places of one sign");

pub const ANY_PLACES: usize = 4;
because!(ANY_PLACES, ToldLogic, "the places of any signs, named by the first letters: a line reads a sequence as the run before and the run after the signs that it reads, and a rule joins a few such runs, and every place more makes each filling of each rule longer, which every rule of every kind pays for");

const ANY_TIMES: usize = 4;
because!(ANY_TIMES, ToldLogic, "a place that may hold any signs is named by its letter written once more than a place that may hold whole groups in brackets, so that each kind of place reads apart by the length of its name");

pub const GROUPED_FIRST: usize = 39;
because!(GROUPED_FIRST, ToldLogic, "the number of the first open place that may hold whole groups in brackets, after the places that may hold marks");

const GROUPED_TIMES: usize = 3;
because!(GROUPED_TIMES, ToldLogic, "a place that may hold whole groups in brackets is named by its letter written once more than a place that may hold marks, so that each kind of place reads apart by the length of its name");

pub const MARKED_FIRST: usize = 24;
because!(MARKED_FIRST, ToldLogic, "the number of the first open place that may hold marks, after the places of one or more words and the places of one word");

const OPENING_BRACKETS: &[&str] = &["(", "["];
because!(OPENING_BRACKETS, ToldLogic, "the marks that open a group or a name in brackets, round or square, as a set of stories marks each name of a person with square brackets: an open place that may hold marks never holds one, so that such a place never takes a name apart");

const CLOSING_BRACKETS: &[&str] = &[")", "]"];
because!(CLOSING_BRACKETS, ToldLogic, "the marks that close a group or a name in brackets, in the order of the marks that open them");

const ONE_WORD_PLACES: &[char] = &['R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
because!(ONE_WORD_PLACES, ToldLogic, "the letters of the open places that take exactly one word, as a name, a verb or a word of a list does, so that a rule about two names cannot read a phrase as a name, and one rule can name several such words (the verb and the head nouns of a passive); Q between them stays a word, as the Q of a question; such a letter takes no ending, so that a word such as Yes or Us stays a word");

const JOINING_MARKS: &[(char, char)] = &[('-', '-'), ('\'', '\''), ('\u{2019}', '\'')];
because!(JOINING_MARKS, ToldLogic, "the marks that join two parts of one word when they stand between two letters with no space, each with the mark that the word keeps: a hyphen (self-assured), and an apostrophe, plain or typed, kept plain (Mary's, don't), so that the two ways of typing it are one word");

pub const WORD_ENDINGS: &[&str] = &["est", "ing", "es", "er", "ed", "'s", "s"];
because!(WORD_ENDINGS, ToldLogic, "the endings that may follow an open place inside one word: of a plural, of a comparison (Ber, Best), of the forms of a verb (Bing, Bed), and of an owner (A's); the longer first among those that one ending holds, so that 'Aes' takes the stem before 'es' and 'Best' the stem before 'est'");

const WORD_JOIN: char = '_';
because!(WORD_JOIN, ToldLogic, "a mark that joins the parts of one word, as a name with parts does in a told text");

const WRITE_START: &str = "Write ";
because!(WRITE_START, ToldLogic, "a line that writes a sentence in a new form in place of the old one opens with this word, so that a line is such a rewriting by its first word");

const WRITE_INSIDE: &str = "inside ";
because!(WRITE_INSIDE, ToldLogic, "the word after the opening word of a line that writes again, for a line that fits also the words inside a pair of brackets: a line that only fits whole sentences keeps its meaning, and a text asks for the reading of parts where it needs it");

const WRITE_AS: &str = " as ";
because!(WRITE_AS, ToldLogic, "the word between the old form of a sentence and its new form in a line that writes it again; the first such word ends the old form");

const READ_TOLD_RULES: &str = "Read told rules with ";
because!(READ_TOLD_RULES, ToldLogic, "the words that open a line by which a topic reads each fact of a story that is a rule ('if ... then ...') as a rule of that story, as a reader takes a rule that the text states, with the words that stand for any thing in such a rule");

const ANY_THING_END: &str = " for any thing";
because!(ANY_THING_END, ToldLogic, "the words after the words that stand for any thing in a line that reads told rules");

const READ_LESSONS: &str = "Read the lessons as the answers of their questions.";
because!(READ_LESSONS, ToldLogic, "the line by which a topic reads each lesson that the tool learns from as the fact that the answer of its question is its answer");

const TOLD_RULE_START: &str = "If ";
because!(TOLD_RULE_START, ToldLogic, "a rule of the library opens with this word, as a rule in a book does, so that a line is a rule by its first word");

pub const USUALLY: &str = "usually ";
because!(USUALLY, ToldLogic, "the word after the 'then' of a rule that holds usually, not always (If A is a male name then usually A is male): a default, which a fact of the story overrules");

const RULE_THEN: &str = " then ";
because!(RULE_THEN, ToldLogic, "the word between the conditions of a rule and its outcome");

const BETWEEN_CONDITIONS: &str = " and ";
because!(BETWEEN_CONDITIONS, ToldLogic, "the word between two conditions of a rule or of an answer");

const QUOTED_START: &str = "To answer \"";
because!(QUOTED_START, ToldLogic, "the start of a question form whose question is in quotes, for a question that holds a question mark of its own");

const QUOTED_END: &str = "\":";
because!(QUOTED_END, ToldLogic, "the end of the quoted question of a question form, before its answers");

const QUESTION_MARK: char = '?';
because!(QUESTION_MARK, ToldLogic, "the end of the question of a question form that is not in quotes, before its answers");

const ANSWER_IF: &str = " if ";
because!(ANSWER_IF, ToldLogic, "the word between an answer and the sentences under which it holds");

pub const EVERY_START: &str = "every ";
because!(EVERY_START, ToldLogic, "the word that opens an answer which says the words of every filling under which its sentences hold, one after another in the order that the engine finds them, as a part of a request may be told more than once (trains from a to b or from c to d leave from a and c)");

const ANSWER_OTHERWISE: &str = " otherwise";
because!(ANSWER_OTHERWISE, ToldLogic, "the end of an answer that holds when no answer before it holds");

const SENTENCE_END: char = '.';
because!(SENTENCE_END, ToldLogic, "the mark at the end of a sentence, which also ends each answer of a question form");

const END_MARKS: &[&str] = &[".", "?", "!"];
because!(END_MARKS, ToldLogic, "the marks at the end of a sentence or a question, which are not words of it");

const BUILDS_ON: &str = "Builds on";
because!(BUILDS_ON, ToldLogic, "the words that open a line naming the topics whose rules and told facts a topic uses, as a chapter of a book builds on chapters before it; a colon may follow them");

const BUILDS_ON_COLON: char = ':';
because!(BUILDS_ON_COLON, ToldLogic, "the mark that may stand after the words that open a line of the topics that a topic builds on");

pub const QUESTION_TOLD: &str = "The question is ";
because!(QUESTION_TOLD, ToldLogic, "the words before the question of a case when the engine tells it as the last sentence of the story, for a topic whose rules read the question as a fact (\"If the question is Is there A , B then ...\"), as a reader keeps the question in mind beside the story");

pub(crate) const LIBRARY_NOTE: &str = "Note:";
because!(LIBRARY_NOTE, ToldLogic, "the word that opens a line for the reader of the library, which the engine skips, as a book has notes beside its rules");

const REPLY_START: &str = "Reply to \"";
because!(REPLY_START, ToldLogic, "the start of a reply line, which tells the sentence that says an answer to a question of a form, with the question in quotes");

const REPLY_WITH: &str = "\" with ";
because!(REPLY_WITH, ToldLogic, "the end of the quoted question of a reply line, before the answer that the line says");

const REPLY_SAYS: char = ':';
because!(REPLY_SAYS, ToldLogic, "the mark between the answer of a reply line and the sentence that says it");

const NAMES_BETWEEN: &[&str] = &[",", " and "];
because!(NAMES_BETWEEN, ToldLogic, "the marks between the names of the topics that a topic builds on");

const UNREAD_LINE: &str = "a line of the library that is no rule, no question form, no reply line and no told fact";
because!(UNREAD_LINE, ToldLogic, "the fault of a line that the reader of the library cannot read: a reply line without its answer or its sentence, or a line that is no rule and no question form and is not a sentence with a full stop, a told fact");

const CUT_THEN: &str = "the conditions of a rule end in 'and', so a 'then' inside a condition cut the rule; write 'And Then' inside a sentence of a rule";
because!(CUT_THEN, ToldLogic, "the fault of a rule whose first 'then' stands inside a condition ('Step and then stop is read'): the rule would be cut there and fit nothing, with no word; a capital Then inside a sentence keeps it a word");

const STRAY_LETTER: &str = "a capital letter that is no open place stands in a rule or an answer, which no fact can fit";
because!(STRAY_LETTER, ToldLogic, "the fault of a rule or an answer that holds a capital letter alone that is neither an open place nor the pronoun I, such as Q: a writer meant an open place, and the rule would fit nothing and lose its answers with no word; the Q of a question stays a word");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicPiece {
    Word(WordId),
    Open(usize),
    OpenWithEnd(usize, String),
}
because!(LogicPiece, ToldLogic, "one piece of a sentence with open places: a word that must stand there, an open place for one or more words, or an open place for one word, the stem of a word with the given ending (no ending for a place of one word)");

pub type Pattern = Vec<LogicPiece>;
because!(Pattern, ToldLogic, "a sentence with open places, piece by piece");

pub fn plain_piece(logic_piece: &LogicPiece) -> LogicPiece {
    match logic_piece {
        LogicPiece::OpenWithEnd(place, ending) if ending.is_empty() => LogicPiece::Open(*place),
        other => other.clone(),
    }
}
because!(plain_piece, ToldLogic, "a piece as a compare of the sentences of two rules sees it: an open place of one word as an open place, since the reader marks a place of one word (the letters from R on) by an empty ending, which the engine needs to bind one word there, while a learned rule names its places with the letters from A; so the same sentence compares the same whatever kind of letter names its places");

pub struct ToldNumbers;
source!(ToldNumbers, "conditions that the engine works out and no fact holds: a comparison of two numbers ('A is less than B', 'A is more than B', 'A is at least B', 'A is at most B'), the sameness of two runs of words ('A is the same as B'), a sum, a difference or a product ('A plus B is C', 'A minus B is C', 'A times B is C'), a count ('C is the number of A where <sentence>': how many different words the place A takes in the facts that fit the sentence) and a total ('C is the sum of A where <sentence>': the sum of the numbers that the place A holds in the different facts that fit the sentence) and a flow ('N is the flow from A to B where <sentence>': the largest flow from A to B over the facts that fit the sentence, its open places in their order the start, the end and the capacity of an edge), as the built-in predicates of Prolog and the aggregates of Datalog do it");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Comparison {
    Less,
    More,
    AtLeast,
    AtMost,
    Same,
    Earlier,
}
because!(Comparison, ToldNumbers, "a comparison of two places: of their numbers, for sameness of their words, or of their words in the order of the alphabet");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reckoning {
    Plus,
    Minus,
    Times,
    Over,
    Whole,
}
because!(Reckoning, ToldNumbers, "a step of arithmetic on two numbers whose result fills a third place or must equal it; a division gives the exact amount, and nothing when it divides by nought; the whole part of a number is a step on that number alone (written as a step whose second number is the same), rounding down, so that a remainder is a number minus the divisor times the whole part of their division");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tally {
    Number,
    Sum,
    Flow { source: LogicPiece, sink: LogicPiece, end: usize, capacity: usize },
}
because!(Tally, ToldNumbers, "what a condition over all the facts that fit a sentence works out: how many different words a place takes, the sum of the numbers of a place, or the largest flow from a start to an end over the edges that the facts tell, with the places of the end and of the capacity of an edge (the counted place is its start)");

#[derive(Clone, Debug, Hash)]
pub enum Condition {
    Fact { pattern: Pattern, when: Moment },
    Compare(Comparison, LogicPiece, LogicPiece),
    Alike { first: Pattern, second: Pattern },
    Reckon(Reckoning, LogicPiece, LogicPiece, LogicPiece),
    Count { result: LogicPiece, counted: usize, pattern: Pattern, tally: Tally },
    Follows { statement: Pattern, said: Pattern },
    Proves { statement: Pattern, holds: bool },
    Letter { word: LogicPiece, letter: LogicPiece, place: Option<LogicPiece> },
    Told(LogicPiece),
    Holds { sentence: LogicPiece, word: LogicPiece },
    HoldsKind { sentence: LogicPiece, kind: Vec<WordId> },
    Backward { result: LogicPiece, run: LogicPiece },
    After { sentence: LogicPiece, earlier: LogicPiece, later: LogicPiece },
}
because!(Condition, ToldLogic, "a condition of a rule or an answer: a sentence that must be a fact, and which facts may hold it by their place in the story; or a comparison, a step of arithmetic, or a count or a total of the facts that fit a sentence (summed for a total), or two runs of open places that hold the same words, or a statement that follows in the worlds that the announcements leave, that the engine works out, or that a fact of the same words says (a rule may make 'it follows that' a fact, and a condition with places that nothing fills yet reads those facts); or the first letter of a word, or the letter at a place of a word (a digit of a number counts as a letter); or that a run holds a word of a kind that facts name ('B holds a function word', a fact 'W is a function word' for one of its words), or that a run is another read backward");

const COMPARISON_WORDS: &[(&[&str], Comparison)] = &[(&["is", "less", "than"], Comparison::Less), (&["is", "more", "than"], Comparison::More), (&["is", "at", "least"], Comparison::AtLeast), (&["is", "at", "most"], Comparison::AtMost), (&["is", "the", "same", "as"], Comparison::Same)];
because!(COMPARISON_WORDS, ToldNumbers, "the words between the two places of each comparison");

const ALPHABET_WORDS: &[&str] = &["comes", "before"];
because!(ALPHABET_WORDS, ToldNumbers, "the words between two words that a condition compares in the order of the alphabet ('V comes before W in the alphabet')");

const ALPHABET_END: &[&str] = &["in", "the", "alphabet"];
because!(ALPHABET_END, ToldNumbers, "the words after the second word of a comparison in the order of the alphabet, which tell it from a sentence about time");

const RECKONING_WORDS: &[(&str, Reckoning)] = &[("plus", Reckoning::Plus), ("minus", Reckoning::Minus), ("times", Reckoning::Times)];
because!(RECKONING_WORDS, ToldNumbers, "the word between the two numbers of each step of arithmetic");

const WHOLE_PART_WORDS: &[&str] = &["the", "whole", "part", "of"];
because!(WHOLE_PART_WORDS, ToldNumbers, "the words before the number whose whole part a condition works out ('the whole part of A is B')");

const DIVIDED_WORDS: &[&str] = &["divided", "by"];
because!(DIVIDED_WORDS, ToldNumbers, "the words between the two numbers of a division ('A divided by B is C'), two words where the other steps take one");

const RESULT_WORD: &str = "is";
because!(RESULT_WORD, ToldNumbers, "the word before the place of the result of a step of arithmetic or of a count");

const COUNT_WORDS: &[&str] = &["is", "the", "number", "of"];
because!(COUNT_WORDS, ToldNumbers, "the words after the place of the result of a count and before the place that it counts");

const SUM_WORDS: &[&str] = &["is", "the", "sum", "of"];
because!(SUM_WORDS, ToldNumbers, "the words after the place of the result of a total and before the place whose numbers it adds");

const FLOW_WORDS: &[&str] = &["is", "the", "flow", "from"];
because!(FLOW_WORDS, ToldNumbers, "the words after the place of the result of a flow and before the place where it starts");

const FLOW_TO: &str = "to";
because!(FLOW_TO, ToldNumbers, "the word between the start and the end of a flow");

const FLOW_PLACES: usize = 3;
because!(FLOW_PLACES, ToldNumbers, "the open places of the sentence of a flow: the start, the end and the capacity of an edge");

const TOLD_WORDS: &[&str] = &["is", "told"];
because!(TOLD_WORDS, ToldNumbers, "the words after a place that holds a whole sentence that the story told, not one that the rules made");

const LETTER_WORDS: &[&str] = &["the", "first", "letter", "of"];
because!(LETTER_WORDS, ToldNumbers, "the words before a word whose first letter a condition reads");

const LETTER_IS: &str = "is";
because!(LETTER_IS, ToldNumbers, "the word between a word and its first letter, or a letter at a place");

const PLACED_LETTER_WORDS: &[&str] = &["letter", "of"];
because!(PLACED_LETTER_WORDS, ToldNumbers, "the words around the place of a letter in a word ('letter N of W is L'): the first stands before the place, the second between the place and the word");

pub(crate) const HOLDS_WORDS: &[&str] = &["holds", "the", "word"];
because!(HOLDS_WORDS, ToldNumbers, "the words between a place that holds a whole sentence and a word that stands in it");

const HOLDS_KIND: &str = "holds";
because!(HOLDS_KIND, ToldNumbers, "the word between a place that holds a run of words and the kind of word that one of them must be ('B holds a function word')");

const KIND_ARTICLES: &[&str] = &["a", "an"];
because!(KIND_ARTICLES, ToldNumbers, "the words that open the kind of word that a run must hold, as English writes a kind, and as the facts that name a word of that kind write it ('W is a function word')");

const KIND_IS: &str = "is";
because!(KIND_IS, ToldNumbers, "the word between a word and its kind in a fact that names the kind of a word");

const BACKWARD_WORDS: (&str, &str) = ("is", "backward");
because!(BACKWARD_WORDS, ToldNumbers, "the words around the run that a condition reads backward ('C is B backward'): the result first, as a step of arithmetic writes it");

const AFTER_WORDS: &[&str] = &["comes", "after"];
because!(AFTER_WORDS, ToldNumbers, "the words between a word of a sentence and a word that stands before it");

const AFTER_IN: &str = "in";
because!(AFTER_IN, ToldNumbers, "the word before the place that holds the sentence in which one word comes after another");

const PROVED_WORDS: &[(&[&str], bool)] = &[(&["it", "is", "proved", "that"], true), (&["it", "is", "disproved", "that"], false)];
because!(PROVED_WORDS, ToldNumbers, "the words before a statement that what is told proves, or proves false, by classical logic");

const FOLLOWS_WORDS: &[&str] = &["it", "follows", "that"];
because!(FOLLOWS_WORDS, ToldNumbers, "the words before a statement that must hold in every world that the announcements of the story leave");

const COUNT_WHERE: &str = "where";
because!(COUNT_WHERE, ToldNumbers, "the word between the place that a count counts and the sentence of the facts that it counts");

const COMPARED_PLACES: usize = 2;
because!(COMPARED_PLACES, ToldNumbers, "a comparison reads a place before its words and a place after them");

const COUNT_PIECES: usize = 2;
because!(COUNT_PIECES, ToldNumbers, "the pieces between the words of a count and its sentence: the place that it counts and the word that opens the sentence");

fn piece_word(logic_piece: &LogicPiece) -> Option<&'static str> {
    match logic_piece {
        LogicPiece::Word(word) => Some(word_text(*word)),
        _ => None,
    }
}
because!(piece_word, ToldNumbers, "the word of a piece that is a word, and nothing for an open place");

fn worked_out(pieces: &Pattern) -> Option<Condition> {
    for (words, comparison) in COMPARISON_WORDS {
        if pieces.len() == words.len() + COMPARED_PLACES && pieces[1..=words.len()].iter().zip(words.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word)) {
            return Some(Condition::Compare(*comparison, pieces[0].clone(), pieces[words.len() + 1].clone()));
        }
    }
    if let [first, comes, before, second, end @ ..] = pieces.as_slice()
        && [comes, before].iter().zip(ALPHABET_WORDS.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word))
        && end.len() == ALPHABET_END.len()
        && end.iter().zip(ALPHABET_END.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word))
    {
        return Some(Condition::Compare(Comparison::Earlier, first.clone(), second.clone()));
    }
    if let Some((same, _)) = COMPARISON_WORDS.iter().find(|(_, comparison)| *comparison == Comparison::Same) {
        let all_places = |run: &[LogicPiece]| !run.is_empty() && run.iter().all(|logic_piece| piece_word(logic_piece).is_none());
        for at in 1..pieces.len() {
            let (first, rest) = pieces.split_at(at);
            if rest.len() > same.len()
                && rest.iter().zip(same.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word))
                && all_places(first)
                && all_places(&rest[same.len()..])
            {
                return Some(Condition::Alike { first: first.to_vec(), second: rest[same.len()..].to_vec() });
            }
        }
    }
    if let [first, step, second, is, result] = pieces.as_slice()
        && piece_word(is) == Some(RESULT_WORD)
        && let Some((_, reckoning)) = RECKONING_WORDS.iter().find(|(word, _)| piece_word(step) == Some(*word))
    {
        return Some(Condition::Reckon(*reckoning, first.clone(), second.clone(), result.clone()));
    }
    if let [the, whole, part, of, number, is, result] = pieces.as_slice()
        && [the, whole, part, of].iter().zip(WHOLE_PART_WORDS.iter()).all(|(logic_piece, text)| piece_word(logic_piece) == Some(*text))
        && piece_word(is) == Some(RESULT_WORD)
    {
        return Some(Condition::Reckon(Reckoning::Whole, number.clone(), number.clone(), result.clone()));
    }
    if let [first, divided, by, second, is, result] = pieces.as_slice()
        && [divided, by].iter().zip(DIVIDED_WORDS.iter()).all(|(logic_piece, text)| piece_word(logic_piece) == Some(*text))
        && piece_word(is) == Some(RESULT_WORD)
    {
        return Some(Condition::Reckon(Reckoning::Over, first.clone(), second.clone(), result.clone()));
    }
    let is_place = |logic_piece: &LogicPiece| !matches!(logic_piece, LogicPiece::Word(_));
    if let [sentence, rest @ .., word] = pieces.as_slice()
        && is_place(sentence)
        && rest.len() == HOLDS_WORDS.len()
        && rest.iter().zip(HOLDS_WORDS.iter()).all(|(logic_piece, text)| piece_word(logic_piece) == Some(*text))
    {
        return Some(Condition::Holds { sentence: sentence.clone(), word: word.clone() });
    }
    if let [sentence, holds, article, kind @ ..] = pieces.as_slice()
        && is_place(sentence)
        && piece_word(holds) == Some(HOLDS_KIND)
        && piece_word(article).is_some_and(|word| KIND_ARTICLES.contains(&word))
        && !kind.is_empty()
        && kind.iter().all(|logic_piece| piece_word(logic_piece).is_some())
    {
        let mut named: Vec<WordId> = vec![word_number(KIND_IS)];
        named.extend([article].into_iter().chain(kind).filter_map(|logic_piece| piece_word(logic_piece)).map(word_number));
        return Some(Condition::HoldsKind { sentence: sentence.clone(), kind: named });
    }
    if let [result, is, run, backward] = pieces.as_slice()
        && is_place(result)
        && is_place(run)
        && piece_word(is) == Some(BACKWARD_WORDS.0)
        && piece_word(backward) == Some(BACKWARD_WORDS.1)
    {
        return Some(Condition::Backward { result: result.clone(), run: run.clone() });
    }
    if let [later, comes, after, earlier, within, sentence] = pieces.as_slice()
        && is_place(sentence)
        && [comes, after].iter().zip(AFTER_WORDS.iter()).all(|(logic_piece, text)| piece_word(logic_piece) == Some(*text))
        && piece_word(within) == Some(AFTER_IN)
    {
        return Some(Condition::After { sentence: sentence.clone(), earlier: earlier.clone(), later: later.clone() });
    }
    if let [sentence, is, told] = pieces.as_slice()
        && is_place(sentence)
        && [is, told].iter().zip(TOLD_WORDS.iter()).all(|(logic_piece, text)| piece_word(logic_piece) == Some(*text))
    {
        return Some(Condition::Told(sentence.clone()));
    }
    if let [the, first, letter, of, word, is, value] = pieces.as_slice()
        && [the, first, letter, of].iter().zip(LETTER_WORDS.iter()).all(|(logic_piece, text)| piece_word(logic_piece) == Some(*text))
        && piece_word(is) == Some(LETTER_IS)
    {
        return Some(Condition::Letter { word: word.clone(), letter: value.clone(), place: None });
    }
    if let [letter, place, of, word, is, value] = pieces.as_slice()
        && [letter, of].iter().zip(PLACED_LETTER_WORDS.iter()).all(|(logic_piece, text)| piece_word(logic_piece) == Some(*text))
        && piece_word(is) == Some(LETTER_IS)
    {
        return Some(Condition::Letter { word: word.clone(), letter: value.clone(), place: Some(place.clone()) });
    }
    for (words, holds) in PROVED_WORDS {
        if pieces.len() > words.len() && pieces.iter().zip(words.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word)) {
            return Some(Condition::Proves { statement: pieces[words.len()..].to_vec(), holds: *holds });
        }
    }
    if pieces.len() > FOLLOWS_WORDS.len() && pieces.iter().zip(FOLLOWS_WORDS.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word)) {
        return Some(Condition::Follows { statement: pieces[FOLLOWS_WORDS.len()..].to_vec(), said: pieces.clone() });
    }
    let flow_head = FLOW_WORDS.len() + 1;
    if let Some([source, to, sink, before]) = pieces.get(flow_head..flow_head + FLOW_PLACES + 1)
        && pieces[1..flow_head].iter().zip(FLOW_WORDS.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word))
        && piece_word(to) == Some(FLOW_TO)
        && piece_word(before) == Some(COUNT_WHERE)
    {
        let sentence = pieces[flow_head + FLOW_PLACES + 1..].to_vec();
        let mut places: Vec<usize> = Vec::new();
        for logic_piece in &sentence {
            if let LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) = logic_piece
                && !places.contains(place)
            {
                places.push(*place);
            }
        }
        if let [start, end, capacity] = places.as_slice() {
            return Some(Condition::Count { result: pieces[0].clone(), counted: *start, pattern: sentence, tally: Tally::Flow { source: source.clone(), sink: sink.clone(), end: *end, capacity: *capacity } });
        }
    }
    for (words, tally) in [(COUNT_WORDS, Tally::Number), (SUM_WORDS, Tally::Sum)] {
        let head = words.len() + 1;
        if pieces.len() > head + words.len()
            && pieces[1..head].iter().zip(words.iter()).all(|(logic_piece, word)| piece_word(logic_piece) == Some(*word))
            && let Some(counted) = counted_place(&pieces[head])
            && piece_word(&pieces[head + 1]) == Some(COUNT_WHERE)
        {
            return Some(Condition::Count { result: pieces[0].clone(), counted, pattern: pieces[head + COUNT_PIECES..].to_vec(), tally });
        }
    }
    None
}
because!(worked_out, ToldNumbers, "the condition that the engine works out for a sentence of one of its forms, and nothing for a sentence that must be a fact");

fn counted_place(logic_piece: &LogicPiece) -> Option<usize> {
    match logic_piece {
        LogicPiece::Open(place) => Some(*place),
        LogicPiece::OpenWithEnd(place, ending) if ending.is_empty() => Some(*place),
        _ => None,
    }
}
because!(counted_place, ToldNumbers, "the place that a count counts: a place of one or more words, or a place of one word, so that a count of words reads as a count and not as a sentence that no fact holds");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineKind {
    Adds,
    AddsByDefault,
    Replaces,
    ReplacesInside,
    ReadsTold,
    ReadsLessons,
}
because!(LineKind, ToldLogic, "what a line of rules does: a rule adds its outcome as a fact; a rule whose outcome holds usually adds it only where the story does not overrule it (a default); a line 'Write <old form> as <new form>' replaces a fact by its new form; a line 'Write inside <old form> as <new form>' does so also for the words inside a pair of brackets of the fact, so that a part of a formula nested in others can be written step by step; a line 'Read told rules with <words> for any thing' makes each fact of a story of the form 'if ... then ...' a rule of that story, the given words one open place; a line 'Read the lessons as the answers of their questions.' makes each lesson that the tool learns from a fact of the topic");

#[derive(Clone, Debug, Hash)]
pub struct ToldRule {
    pub conditions: Vec<Condition>,
    pub absent: Vec<Condition>,
    pub outcome: Pattern,
    pub kind: LineKind,
    pub name: Vec<WordId>,
    pub learned: bool,
}
because!(ToldRule, ToldLogic, "a told rule: when every condition holds with one filling of the open places, and no condition that must not hold holds under that filling, the outcome holds with the same filling; a rule that replaces is a line 'Write <old form> as <new form> if <conditions>': its first condition is the old form, and a fact of that form is written in the new form in place of it, the old form not kept; a line that reads told rules has no condition, and its outcome holds the words that stand for any thing; a rule that a learner made from lessons is marked, as each of its places stands for a thing of its own");

#[derive(Clone, Debug)]
pub struct Reply {
    pub words: Pattern,
    pub conditions: Vec<Condition>,
    pub absent: Vec<Condition>,
    pub text: String,
    pub every: bool,
}
because!(Reply, ToldLogic, "one answer of a question form: the words of the answer, which may hold open places that its sentences fill, the sentences that must hold for it (none for an answer that holds otherwise), the conditions that must not hold, its text as the library writes it, for the note of an answer, and whether it says the words of every filling under which it holds");

#[derive(Clone, Debug)]
pub struct AskedForm {
    pub question: Pattern,
    pub replies: Vec<Reply>,
}
because!(AskedForm, ToldLogic, "a question with open places and its answers in the order in which they are tried");

#[derive(Clone, Debug)]
pub struct ReplyLine {
    pub question: Pattern,
    pub answer: Pattern,
    pub said: Pattern,
    pub sentence: String,
}
because!(ReplyLine, ToldLogic, "a reply line: a question with open places, an answer, and the sentence that says the answer to the question, as the library writes it, with its open places also as a pattern; its letters are those of the question form that answered, so that a place holds the words that the answer gave it");

#[derive(Clone, Debug)]
pub struct Topic {
    pub name: String,
    pub rules: Vec<ToldRule>,
    pub questions: Vec<AskedForm>,
    pub reply_lines: Vec<ReplyLine>,
    pub facts: Vec<Vec<WordId>>,
    pub builds_on: Vec<String>,
    pub lines: usize,
    pub rules_key: u64,
}
because!(Topic, ToldLogic, "the told logic of one topic of the library: its name, its rules, its question forms, its reply lines, the facts that it tells, as a book tells a list of words in sentences (\"tall is the opposite of short\"), which hold in every story that the topic reads, the names of the topics whose rules, reply lines and facts it uses, the number of its lines, and the hash of its rules, by which the facts of its stories are kept");

pub fn word_not_mark(token: &str) -> bool {
    let rest = token.strip_prefix(NUMBER_BELOW_NOTHING).unwrap_or(token);
    rest.chars().next().is_some_and(|sign| if rest.len() < token.len() { sign.is_ascii_digit() } else { sign.is_alphanumeric() })
}
because!(word_not_mark, ToldLogic, "whether a piece of a told text is a word and not a mark (a number below nothing is a word, its sign a part of it): an open place stands for things and words about them, never for the marks between them, so that a rule does not read a name with its brackets or a clause with its commas as one thing");

pub fn text_words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut letters = text.chars().peekable();
    while let Some(letter) = letters.next() {
        let between = word.chars().last().is_some_and(char::is_alphabetic) && letters.peek().is_some_and(|next| next.is_alphabetic());
        if let Some((_, kept)) = JOINING_MARKS.iter().find(|(mark, _)| *mark == letter).filter(|_| between) {
            word.push(*kept);
            continue;
        }
        let digits = word.strip_prefix(NUMBER_BELOW_NOTHING).unwrap_or(&word);
        let in_number = !digits.is_empty() && digits.chars().all(|sign| sign.is_ascii_digit()) && letters.peek().is_some_and(char::is_ascii_digit);
        if letter == NUMBER_POINT && in_number {
            word.push(letter);
            continue;
        }
        if letter == NUMBER_BELOW_NOTHING && word.is_empty() && letters.peek().is_some_and(char::is_ascii_digit) {
            word.push(letter);
            continue;
        }
        if letter.is_alphanumeric() {
            word.push(letter);
            continue;
        }
        if !word.is_empty() {
            out.push(signed_nothing(std::mem::take(&mut word)));
        }
        if !letter.is_whitespace() && letter != WORD_JOIN {
            out.push(letter.to_string());
        }
    }
    if !word.is_empty() {
        out.push(signed_nothing(word));
    }
    out
}

fn signed_nothing(word: String) -> String {
    match word.strip_prefix(NUMBER_BELOW_NOTHING) {
        Some(rest) if rest.chars().all(|sign| sign == NOUGHT || sign == NUMBER_POINT) => rest.to_string(),
        _ => word,
    }
}
because!(signed_nothing, ToldLogic, "a number word as the rules read it: nothing written with a minus sign is nothing, so that the two writings of nothing are one word, as they are one number, and a fact or a sameness of words does not tell them apart");

const NOUGHT: char = '0';
because!(NOUGHT, ToldLogic, "the digit of nothing, of which a number below nothing with no other digit is written");
because!(text_words, ToldLogic, "the words and marks of a text in their order: a run of letters or digits is one word, a point between digits stays inside the number, as a quarter is written with a point, a minus sign at the start of a word before a digit starts the number, and every other mark that is not a space or a low line is a word of its own");

pub const NUMBER_BELOW_NOTHING: char = '-';
because!(NUMBER_BELOW_NOTHING, ToldLogic, "the sign before the digits of a number below nothing, which starts one word with them when it stands at the start of a word and a digit follows, so that a number below nothing is one word while a range between two numbers and a dash between words stay apart");

pub const NUMBER_OVER: char = '/';
because!(NUMBER_OVER, ToldLogic, "the mark between the two whole numbers of an amount whose decimal does not end, as a step of arithmetic writes it in one word (a third); a told text keeps the mark as a word of its own, so a date or a fraction in a story stays apart");

pub const NUMBER_POINT: char = '.';
because!(NUMBER_POINT, ToldLogic, "the mark between the whole part and the part below one of a number written with a point, which joins one word when it stands between digits and has only digits before it in the word");

fn without_end(mut words: Vec<String>) -> Vec<String> {
    while words.last().is_some_and(|last| END_MARKS.contains(&last.as_str())) {
        words.pop();
    }
    words
}
because!(without_end, ToldLogic, "the marks that close a sentence carry no meaning for a rule, so that a told sentence and a sentence of the library are compared without them");

pub fn fact_words(text: &str) -> Vec<String> {
    without_end(text_words(text).into_iter().map(|word| word.to_lowercase()).collect())
}
because!(fact_words, ToldLogic, "the words of a told sentence or a question as the rules see them: in small letters and without the marks at the end");

pub fn fact_numbers(text: &str) -> Vec<WordId> {
    fact_words(text).iter().map(|word| word_number(word)).collect()
}
because!(fact_numbers, ToldLogic, "the numbers of the words of a told sentence or a question as the rules see them");

fn lone_letter(word: &str) -> Option<char> {
    let mut letters = word.chars();
    let first = letters.next()?;
    letters.next().is_none().then_some(first)
}
because!(lone_letter, ToldLogic, "the letter of a word of one letter, and nothing for a longer word");

fn open_place(word: &str) -> Option<usize> {
    let first = lone_letter(word)?;
    if !(OPEN_FIRST..=OPEN_LAST).contains(&first) || first == NOT_OPEN {
        return None;
    }
    let place = first as usize - OPEN_FIRST as usize;
    Some(if first > NOT_OPEN { place - 1 } else { place })
}
because!(open_place, ToldLogic, "the number of the open place of one or more words that a word of the library names, and nothing for a word that is no such place");

fn one_word_place(word: &str) -> Option<usize> {
    let first = lone_letter(word)?;
    ONE_WORD_PLACES.iter().position(|letter| *letter == first).map(|at| PLAIN_PLACES + at)
}
because!(one_word_place, ToldLogic, "the number of the open place of one word that a word of the library names, after the places of one or more words, and nothing for a word that is no such place");

fn repeated_place(word: &str) -> Option<usize> {
    let letters: Vec<char> = word.chars().collect();
    let first = *letters.first()?;
    if letters.iter().any(|letter| *letter != first) {
        return None;
    }
    let place = open_place(&first.to_string())?;
    match letters.len() {
        MARKED_DOUBLE => Some(MARKED_FIRST + place),
        GROUPED_TIMES => Some(GROUPED_FIRST + place),
        ANY_TIMES if place < ANY_PLACES => Some(ANY_FIRST + place),
        _ => None,
    }
}
because!(repeated_place, ToldLogic, "the number of the open place that a letter of an open place written again names: written twice (AA, BB), a place that may hold marks; written three times (AAA, BBB), a place that may hold whole groups in brackets; one of the first letters written four times (AAAA, BBBB), a place that may hold any signs; nothing for a word that is no such place");

fn one_sign_place(word: &str) -> Option<usize> {
    let letters: Vec<char> = word.chars().collect();
    let [first, second] = letters.as_slice() else { return None };
    if first != second {
        return None;
    }
    ONE_WORD_PLACES.iter().take(ONE_SIGN_PLACES).position(|letter| letter == first).map(|at| ONE_SIGN_FIRST + at)
}
because!(one_sign_place, ToldLogic, "the number of the open place of one sign of any kind that one of the first letters of the places of one word written twice names (RR, SS), and nothing for a word that is no such place");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceTakes {
    Words,
    Marks,
    Groups,
    Signs,
}
because!(PlaceTakes, ToldLogic, "what an open place may hold: words only; also marks (commas, apostrophes, full stops inside a run), so that one rule reads a sentence of people wherever its commas stand; also whole groups in brackets, every bracket that opens in it closed in it, so that a line of the library rewrites a nested sentence one step at a time and a group made by one step is read by the next, while a bracket of a group is never cut apart; or any signs, also brackets that it does not close, as a sequence of brackets is read sign by sign");

pub fn place_takes(place: usize) -> PlaceTakes {
    if place >= ONE_SIGN_FIRST {
        return PlaceTakes::Signs;
    }
    if place >= GROUPED_FIRST {
        return PlaceTakes::Groups;
    }
    if place >= MARKED_FIRST {
        return PlaceTakes::Marks;
    }
    PlaceTakes::Words
}
because!(place_takes, ToldLogic, "what an open place may hold, by the kind of its name: a place of one sign and a place of any signs hold any signs, a letter written three times whole groups, a letter written twice marks, and any other place words only");

pub fn group_turn(token: &str) -> Option<bool> {
    if OPENING_BRACKETS.contains(&token) {
        return Some(true);
    }
    CLOSING_BRACKETS.contains(&token).then_some(false)
}
because!(group_turn, ToldLogic, "whether a piece of a told text opens a group in brackets (true) or closes one (false), and nothing for any other piece");
pub fn not_bracket(token: &str) -> bool {
    !OPENING_BRACKETS.contains(&token) && !CLOSING_BRACKETS.contains(&token)
}
because!(not_bracket, ToldLogic, "whether a piece of a told text is no bracket of a name, the one mark that an open place that may hold marks does not take");

pub fn condition_text(condition: &Condition, piece: &dyn Fn(&LogicPiece) -> String) -> String {
    let run = |pieces: &[LogicPiece]| pieces.iter().map(piece).collect::<Vec<String>>();
    let mut words: Vec<String> = Vec::new();
    match condition {
        Condition::Fact { pattern, when } => {
            if let Some((start, _)) = TIMED_STARTS.iter().find(|(_, moment)| moment == when) {
                words.push(start.trim_end().to_string());
            }
            words.extend(run(pattern));
        }
        Condition::Compare(Comparison::Earlier, first, second) => {
            words.push(piece(first));
            words.extend(ALPHABET_WORDS.iter().map(|word| word.to_string()));
            words.push(piece(second));
            words.extend(ALPHABET_END.iter().map(|word| word.to_string()));
        }
        Condition::Compare(comparison, first, second) => {
            words.push(piece(first));
            words.extend(COMPARISON_WORDS.iter().find(|(_, kind)| kind == comparison).map_or(&[][..], |(said, _)| said).iter().map(|word| word.to_string()));
            words.push(piece(second));
        }
        Condition::Alike { first, second } => {
            words.extend(run(first));
            words.extend(COMPARISON_WORDS.iter().find(|(_, kind)| *kind == Comparison::Same).map_or(&[][..], |(said, _)| said).iter().map(|word| word.to_string()));
            words.extend(run(second));
        }
        Condition::Reckon(reckoning, first, second, result) => {
            match reckoning {
                Reckoning::Whole => {
                    words.extend(WHOLE_PART_WORDS.iter().map(|word| word.to_string()));
                    words.push(piece(first));
                }
                Reckoning::Over => {
                    words.push(piece(first));
                    words.extend(DIVIDED_WORDS.iter().map(|word| word.to_string()));
                    words.push(piece(second));
                }
                _ => {
                    words.push(piece(first));
                    words.extend(RECKONING_WORDS.iter().find(|(_, kind)| kind == reckoning).map(|(said, _)| said.to_string()));
                    words.push(piece(second));
                }
            }
            words.push(RESULT_WORD.to_string());
            words.push(piece(result));
        }
        Condition::Count { result, counted, pattern, tally } => {
            words.push(piece(result));
            let said = match tally {
                Tally::Number => COUNT_WORDS,
                Tally::Sum => SUM_WORDS,
                Tally::Flow { .. } => FLOW_WORDS,
            };
            words.extend(said.iter().map(|word| word.to_string()));
            words.push(piece(&LogicPiece::Open(*counted)));
            words.push(COUNT_WHERE.to_string());
            words.extend(run(pattern));
        }
        Condition::Follows { statement, .. } => {
            words.extend(FOLLOWS_WORDS.iter().map(|word| word.to_string()));
            words.extend(run(statement));
        }
        Condition::Proves { statement, holds } => {
            words.extend(PROVED_WORDS.iter().find(|(_, proved)| proved == holds).map_or(&[][..], |(said, _)| said).iter().map(|word| word.to_string()));
            words.extend(run(statement));
        }
        Condition::Letter { word, letter, place } => {
            match place {
                Some(place) => {
                    words.push(PLACED_LETTER_WORDS[0].to_string());
                    words.push(piece(place));
                    words.extend(PLACED_LETTER_WORDS[1..].iter().map(|word| word.to_string()));
                }
                None => words.extend(LETTER_WORDS.iter().map(|word| word.to_string())),
            }
            words.push(piece(word));
            words.push(LETTER_IS.to_string());
            words.push(piece(letter));
        }
        Condition::Told(sentence) => {
            words.push(piece(sentence));
            words.extend(TOLD_WORDS.iter().map(|word| word.to_string()));
        }
        Condition::Holds { sentence, word } => {
            words.push(piece(sentence));
            words.extend(HOLDS_WORDS.iter().map(|word| word.to_string()));
            words.push(piece(word));
        }
        Condition::HoldsKind { sentence, kind } => {
            words.push(piece(sentence));
            words.push(HOLDS_KIND.to_string());
            words.extend(kind.iter().skip(1).map(|word| word_text(*word).to_string()));
        }
        Condition::Backward { result, run } => {
            words.push(piece(result));
            words.push(BACKWARD_WORDS.0.to_string());
            words.push(piece(run));
            words.push(BACKWARD_WORDS.1.to_string());
        }
        Condition::After { sentence, earlier, later } => {
            words.push(piece(later));
            words.extend(AFTER_WORDS.iter().map(|word| word.to_string()));
            words.push(piece(earlier));
            words.push(AFTER_IN.to_string());
            words.push(piece(sentence));
        }
    }
    words.join(" ")
}
because!(condition_text, ToldLogic, "a condition as the library writes it, each piece as the caller gives it (its letter, or the words that fill it), so that a proof can show the conditions that a rule read");

pub fn place_letter(place: usize) -> String {
    if place >= ANY_FIRST {
        return place_letter(place - ANY_FIRST).repeat(ANY_TIMES);
    }
    if place >= ONE_SIGN_FIRST {
        return place_letter(place - ONE_SIGN_FIRST + PLAIN_PLACES).repeat(MARKED_DOUBLE);
    }
    if place >= GROUPED_FIRST {
        return place_letter(place - GROUPED_FIRST).repeat(GROUPED_TIMES);
    }
    if place >= MARKED_FIRST {
        return place_letter(place - MARKED_FIRST).repeat(MARKED_DOUBLE);
    }
    if let Some(letter) = place.checked_sub(PLAIN_PLACES).and_then(|at| ONE_WORD_PLACES.get(at)) {
        return letter.to_string();
    }
    let first = OPEN_FIRST as usize + place;
    let letter = if first >= NOT_OPEN as usize { first + 1 } else { first };
    char::from_u32(letter as u32).map(String::from).unwrap_or_default()
}
because!(place_letter, ToldLogic, "the letter that names an open place, as a text of the library writes it: the letters from A on, without I, after them the letters of the places of one word, after them the letters written twice of the places that may hold marks, after them the letters written three times of the places that may hold groups in brackets, after them the first letters of the places of one word written twice of the places of one sign, and after them the first letters written four times of the places of any signs");

const MARKED_DOUBLE: usize = 2;
because!(MARKED_DOUBLE, ToldLogic, "a place that may hold marks is named by its letter written twice");

pub fn logic_piece(word: &str) -> LogicPiece {
    if let Some(place) = open_place(word).or_else(|| repeated_place(word)) {
        return LogicPiece::Open(place);
    }
    if let Some(place) = one_word_place(word).or_else(|| one_sign_place(word)) {
        return LogicPiece::OpenWithEnd(place, String::new());
    }
    for ending in WORD_ENDINGS {
        if let Some(stem) = word.strip_suffix(ending)
            && let Some(place) = open_place(stem)
        {
            return LogicPiece::OpenWithEnd(place, ending.to_string());
        }
    }
    LogicPiece::Word(word_number(&word.to_lowercase()))
}
because!(logic_piece, ToldLogic, "the piece that a word of the library stands for: an open place (also one that may hold marks, groups or any signs), an open place of one word or of one sign, which is a stem with no ending, an open place with an ending, or a word in small letters");

pub fn pattern(text: &str) -> Pattern {
    without_end(text_words(text)).iter().map(|word| logic_piece(word)).collect()
}
because!(pattern, ToldLogic, "a sentence of the library as a sentence with open places, without the marks at its end");

fn conditions(text: &str) -> (Vec<Condition>, Vec<Condition>) {
    let mut read = (Vec::new(), Vec::new());
    for part in text.split(BETWEEN_CONDITIONS) {
        add_part(&mut read, part_reading(part));
    }
    read
}

#[derive(Clone, Debug)]
pub enum PartReading {
    Held(Condition),
    Absent(Condition),
}
because!(PartReading, ToldLogic, "one part of the conditions of a rule or an answer as the reader reads it alone: a condition that must hold (also one held at a time) or one after 'not' that must not; the conditions of a text are its parts put together in their order (add_part)");

fn part_reading(part: &str) -> PartReading {
    let part = part.trim_start();
    if let Some(rest) = part.strip_prefix(NOT_START) {
        let pieces = pattern(rest);
        return PartReading::Absent(mark_read(rest).or_else(|| worked_out(&pieces)).unwrap_or(Condition::Fact { pattern: pieces, when: Moment::Any }));
    }
    if let Some((rest, when)) = TIMED_STARTS.iter().find_map(|(start, when)| part.strip_prefix(start).map(|rest| (rest, *when))) {
        return PartReading::Held(Condition::Fact { pattern: pattern(rest), when });
    }
    let pieces = pattern(part);
    PartReading::Held(mark_read(part).or_else(|| worked_out(&pieces)).unwrap_or(Condition::Fact { pattern: pieces, when: Moment::Any }))
}
because!(part_reading, ToldLogic, "one part of the conditions of a rule or an answer read alone: after 'not' a condition that must not hold, with a word of time a fact held at that time, else a mark read, a condition that the engine works out, or a fact; what a part reads does not depend on the parts before it, only how it is put together with them");

fn add_part((held, absent): &mut (Vec<Condition>, Vec<Condition>), reading: PartReading) {
    match reading {
        PartReading::Absent(condition) => absent.push(condition),
        PartReading::Held(condition) => {
            if let Condition::Told(sentence @ LogicPiece::Open(_)) = &condition
                && !held.iter().any(|before: &Condition| matches!(before, Condition::Fact { pattern, .. } if pattern.contains(sentence)))
            {
                held.push(Condition::Fact { pattern: vec![sentence.clone()], when: Moment::Any });
            }
            held.push(condition);
        }
    }
}
because!(add_part, ToldLogic, "one part of the conditions of a text put after the parts before it: a condition after 'not' among those that must not hold, any other among those that must, and a condition that a sentence was told whose place no condition before it fills after a fact of that place alone, so that it reads every told sentence");

fn mark_read(text: &str) -> Option<Condition> {
    let words = text_words(text);
    if !words.last().is_some_and(|last| END_MARKS.contains(&last.as_str())) {
        return None;
    }
    let pieces: Pattern = words.iter().map(|word| logic_piece(word)).collect();
    worked_out(&pieces).filter(|condition| matches!(condition, Condition::Holds { .. } | Condition::Letter { .. }))
}
because!(mark_read, ToldLogic, "a condition that reads one word of a sentence or one letter of a word, when that word or letter is a mark that ends a sentence ('holds the word ?', 'letter K of R is .'): the mark is the thing read, so it is kept, where every other sentence of a rule drops its end marks");
because!(conditions, ToldLogic, "the sentences of the conditions of a rule or of an answer: those that must be facts, some of them only by one fact chosen by its place in the story, those that the engine works out, and those after 'not' that no fact may fit or that the engine works out and must not hold; a condition that a sentence was told whose place no condition before it fills reads any fact first, so that it reads every told sentence");

fn stray_letter(text: &str) -> Option<String> {
    text_words(text).into_iter().find(|word| lone_letter(word).is_some_and(|letter| letter.is_ascii_uppercase() && letter != NOT_OPEN) && open_place(word).is_none() && one_word_place(word).is_none())
}
because!(stray_letter, ToldLogic, "the first word of a text of a rule or an answer that is a capital letter alone and no open place, but the pronoun I, and nothing when it has none");

fn replies(text: &str) -> Vec<Reply> {
    let mut out = Vec::new();
    for part in answer_parts(text).into_iter().map(str::trim).filter(|part| !part.is_empty()) {
        if let Some(words) = part.strip_suffix(ANSWER_OTHERWISE) {
            out.push(Reply { words: pattern(words), conditions: Vec::new(), absent: Vec::new(), text: part.to_string(), every: false });
        } else if let Some((words, held)) = part.split_once(ANSWER_IF) {
            let (conditions, absent) = conditions(held);
            let (every, words) = match words.strip_prefix(EVERY_START) {
                Some(rest) => (true, rest),
                None => (false, words),
            };
            out.push(Reply { words: pattern(words), conditions, absent, text: part.to_string(), every });
        }
    }
    out
}
because!(replies, ToldLogic, "the answers of a question form in their order: each part up to a full stop is an answer with the sentences after 'if', which says every filling when its words start with 'every', or an answer that holds otherwise");

fn answer_parts(text: &str) -> Vec<&str> {
    let signs: Vec<(usize, char)> = text.char_indices().collect();
    let mut parts = Vec::new();
    let mut from = usize::default();
    for (at, (byte, sign)) in signs.iter().enumerate() {
        let in_number = at > usize::default() && signs[at - 1].1.is_ascii_digit() && signs.get(at + 1).is_some_and(|(_, next)| next.is_ascii_digit());
        if *sign == SENTENCE_END && !in_number {
            parts.push(&text[from..*byte]);
            from = byte + sign.len_utf8();
        }
    }
    parts.push(&text[from..]);
    parts
}
because!(answer_parts, ToldLogic, "the answers of a question form, cut at each full stop but not at a point between two digits, so that a number with a point in the conditions of an answer stays one number, as it does in a rule");

fn write_conditions(text: &str) -> (&str, &str) {
    let mut depth = usize::default();
    let mut found: Option<usize> = None;
    for (at, letter) in text.char_indices() {
        match group_turn(&letter.to_string()) {
            Some(true) => depth += 1,
            Some(false) => depth = depth.saturating_sub(1),
            None => {}
        }
        if depth == usize::default() && text[at..].starts_with(ANSWER_IF) && !text[at + ANSWER_IF.len()..].contains(RULE_THEN) {
            found = Some(at);
        }
    }
    found.map_or((text, ""), |at| (&text[..at], &text[at + ANSWER_IF.len()..]))
}
because!(write_conditions, ToldLogic, "the new form of a line that writes a sentence again and its conditions: the last ' if ' outside brackets whose rest holds no ' then ' parts them, so that a new form that is itself a rule ('if (A) then (B)') or holds one in brackets keeps its own 'if'");

const RULE_NAME_END: &str = ":";
because!(RULE_NAME_END, ToldLogic, "the mark after the name of a rule that a story tells ('Rule1: if ... then ...'), so that a told preference can name the rule");

const MOST_NAME_WORDS: usize = 3;
because!(MOST_NAME_WORDS, ToldLogic, "the most words of the name of a told rule before its mark, so that a sentence with a colon after a long clause is not taken for a named rule");

pub fn told_rule(fact: &[WordId], any: &[WordId]) -> Option<ToldRule> {
    let start = fact_numbers(TOLD_RULE_START);
    let name_end = word_number(RULE_NAME_END);
    let (name, fact) = match fact.iter().position(|word| *word == name_end).filter(|at| (1..=MOST_NAME_WORDS).contains(at)) {
        Some(at) if fact[at + 1..].starts_with(&start) => (fact[..at].to_vec(), &fact[at + 1..]),
        _ => (Vec::new(), fact),
    };
    if !fact.starts_with(&start) {
        return None;
    }
    let place = place_letter(0);
    let line = fact[start.len()..].iter().map(|word| if any.contains(word) { place.as_str() } else { word_text(*word) }).collect::<Vec<&str>>().join(" ");
    let (held, outcome) = line.split_once(RULE_THEN)?;
    let (conditions, absent) = conditions(held);
    let reads_a_fact = conditions.iter().any(|condition| matches!(condition, Condition::Fact { .. }));
    (reads_a_fact && !outcome.trim().is_empty()).then(|| ToldRule { conditions, absent, outcome: pattern(outcome), kind: LineKind::Adds, name, learned: false })
}
because!(told_rule, ToldLogic, "the rule that a fact of a story states, when it is 'if <conditions> then <outcome>': each word that stands for any thing is the first open place, every other word stays as told (a word in small letters is never an open place), and the conditions are read as those of a rule of the library ('and' between them, 'not' before one that must not hold); nothing for a fact that is no rule or whose conditions read no fact; a name before a colon ('Rule1: if ...') is kept on the rule, for the preferences that the story tells");

pub(super) fn quoted_answers(line: &str) -> Option<(&str, &str)> {
    let mut lines = line.lines();
    if lines.next() != Some(line) || lines.next().is_some() || line.trim() != line {
        return None;
    }
    line.strip_prefix(QUOTED_START)?.split_once(QUOTED_END)
}
because!(quoted_answers, ToldLogic, "the question and the answers of a text that is one line of a quoted question form and nothing else, cut as read_topic cuts such a line (no other kind of line starts as it does), so that a learner that reads many lines of one question can read the question once; nothing for any other text");

pub(super) fn question_form(question: &str, answers: &str, known: &mut Option<Pattern>) -> Result<AskedForm, String> {
    if let Some(letter) = stray_letter(answers) {
        return Err(letter);
    }
    let question = known.get_or_insert_with(|| pattern(question)).clone();
    Ok(AskedForm { question, replies: replies(answers) })
}
because!(question_form, ToldLogic, "a question form from the text of its question and of its answers, or the capital letter in its answers that is no open place: the answers are checked first and the question read after them, as the reader always did, and a question that the caller read before from the same text is taken as it was read, so that the words of the question are put in the table of words at the same point as before");

const ONE_LINE_BREAK: char = '\n';
because!(ONE_LINE_BREAK, ToldLogic, "the sign that ends a line of a text, which a quoted line read by its parts may not hold");

pub(super) fn plain_filled(question: &str, said: &str, held: &str) -> bool {
    let plain = |text: &str| !text.is_empty() && text.trim() == text && !text.contains(ONE_LINE_BREAK) && !text.contains(SENTENCE_END);
    plain(said) && plain(held) && !question.contains(QUOTED_END) && !question.contains(ONE_LINE_BREAK) && !format!("{said} ").contains(ANSWER_IF) && !format!(" {held}").ends_with(ANSWER_OTHERWISE)
}
because!(plain_filled, ToldLogic, "whether the one quoted line of a question form with one answer and its conditions after 'if' is read by its parts as the reader reads the whole line: one line, the question with no quote mark and colon in it, the answer and its conditions with no full stop, no space at their ends, no 'if' in the answer and no 'otherwise' at the end of the conditions, so that the line has one answer that splits at its first 'if'");

pub(super) fn filled_form(question: &str, (said, held): (&str, &str), known: &mut Option<Pattern>, kept: &mut HashMap<String, PartReading>) -> Result<AskedForm, String> {
    if let Some(letter) = stray_letter(said) {
        return Err(letter);
    }
    let parts: Vec<&str> = held.split(BETWEEN_CONDITIONS).collect();
    if let Some(letter) = parts.iter().filter(|part| !kept.contains_key(**part)).find_map(|part| stray_letter(part)) {
        return Err(letter);
    }
    let question = known.get_or_insert_with(|| pattern(question)).clone();
    let mut read = (Vec::new(), Vec::new());
    for part in parts {
        let reading = match kept.get(part) {
            Some(reading) => reading.clone(),
            None => {
                let reading = part_reading(part);
                kept.insert(part.to_string(), reading.clone());
                reading
            }
        };
        add_part(&mut read, reading);
    }
    let (conditions, absent) = read;
    let (every, words) = match said.strip_prefix(EVERY_START) {
        Some(rest) => (true, rest),
        None => (false, said),
    };
    Ok(AskedForm { question, replies: vec![Reply { words: pattern(words), conditions, absent, text: format!("{said}{ANSWER_IF}{held}"), every }] })
}
because!(filled_form, ToldLogic, "the question form of a quoted line with one answer and its conditions after 'if', read by its parts as question_form reads the whole line when plain_filled holds: the capital letters that are no open place checked first, in the answer and then in each part of the conditions not read before, the question read once as the caller kept it, then the conditions, each part of them read once and kept by its text and put together as the reader puts the parts of a whole text (a joined line of a first condition and a second fact reads its first part once for all its seconds), then the answer, so that the words come into the table of words in the same order as when the line is read whole and a part read again gives the same reading");

pub fn topic_places(topic: &Topic) -> Vec<usize> {
    let seen: std::cell::RefCell<Vec<usize>> = std::cell::RefCell::new(Vec::new());
    let take = |logic_piece: &LogicPiece| {
        if let LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) = logic_piece
            && !seen.borrow().contains(place)
        {
            seen.borrow_mut().push(*place);
        }
        String::new()
    };
    for rule in &topic.rules {
        for condition in rule.conditions.iter().chain(&rule.absent) {
            condition_text(condition, &take);
        }
        rule.outcome.iter().for_each(|logic_piece| {
            take(logic_piece);
        });
    }
    for form in &topic.questions {
        form.question.iter().for_each(|logic_piece| {
            take(logic_piece);
        });
        for answer in &form.replies {
            answer.words.iter().for_each(|logic_piece| {
                take(logic_piece);
            });
            for condition in answer.conditions.iter().chain(&answer.absent) {
                condition_text(condition, &take);
            }
        }
    }
    for line in &topic.reply_lines {
        line.question.iter().chain(&line.answer).chain(&line.said).for_each(|logic_piece| {
            take(logic_piece);
        });
    }
    seen.into_inner()
}
because!(topic_places, ToldLogic, "the open places that the reader takes in the rules, question forms and reply lines of a text, each once, in the order in which it meets them; a told fact holds words only");

pub fn read_topic(name: &str, text: &str) -> Result<Topic, String> {
    let mut topic = Topic { name: name.to_string(), rules: Vec::new(), questions: Vec::new(), reply_lines: Vec::new(), facts: Vec::new(), builds_on: Vec::new(), lines: 0, rules_key: 0 };
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with(LIBRARY_NOTE) {
            continue;
        }
        topic.lines += 1;
        if let Some(names) = line.strip_prefix(BUILDS_ON).filter(|rest| rest.starts_with(BUILDS_ON_COLON) || rest.starts_with(char::is_whitespace)) {
            let names = names.trim_start_matches(BUILDS_ON_COLON).trim();
            let mut list = vec![names.trim_end_matches(SENTENCE_END).to_string()];
            for mark in NAMES_BETWEEN {
                list = list.iter().flat_map(|name| name.split(mark).map(str::to_string).collect::<Vec<String>>()).collect();
            }
            topic.builds_on.extend(list.into_iter().map(|name| name.trim().to_string()).filter(|name| !name.is_empty()));
        } else if let Some(rest) = line.strip_prefix(REPLY_START) {
            let parts = rest.split_once(REPLY_WITH).and_then(|(question, said)| said.split_once(REPLY_SAYS).map(|(answer, sentence)| (question, answer, sentence.trim())));
            let Some((question, answer, sentence)) = parts else { return Err(format!("{name}: {UNREAD_LINE}: {line}")) };
            topic.reply_lines.push(ReplyLine { question: pattern(question), answer: pattern(answer), said: pattern(sentence), sentence: sentence.to_string() });
        } else if let Some(body) = line.strip_prefix(TOLD_RULE_START)
            && let Some((held, outcome)) = body.trim_end_matches(SENTENCE_END).split_once(RULE_THEN)
        {
            if let Some(letter) = stray_letter(body) {
                return Err(format!("{name}: {STRAY_LETTER}: {letter}: {line}"));
            }
            if format!(" {held}").ends_with(BETWEEN_CONDITIONS.trim_end()) {
                return Err(format!("{name}: {CUT_THEN}: {line}"));
            }
            let (conditions, absent) = conditions(held);
            let (outcome, kind) = match outcome.strip_prefix(USUALLY) {
                Some(rest) => (rest, LineKind::AddsByDefault),
                None => (outcome, LineKind::Adds),
            };
            topic.rules.push(ToldRule { conditions, absent, outcome: pattern(outcome), kind, name: Vec::new(), learned: false });
        } else if line == READ_LESSONS {
            topic.rules.push(ToldRule { conditions: Vec::new(), absent: Vec::new(), outcome: Vec::new(), kind: LineKind::ReadsLessons, name: Vec::new(), learned: false });
        } else if let Some(words) = line.strip_prefix(READ_TOLD_RULES).and_then(|rest| rest.trim_end_matches(SENTENCE_END).strip_suffix(ANY_THING_END)) {
            let mut list = vec![words.to_string()];
            for mark in NAMES_BETWEEN {
                list = list.iter().flat_map(|name| name.split(mark).map(str::to_string).collect::<Vec<String>>()).collect();
            }
            let any: Pattern = list.iter().flat_map(|word| fact_numbers(word)).map(LogicPiece::Word).collect();
            topic.rules.push(ToldRule { conditions: Vec::new(), absent: Vec::new(), outcome: any, kind: LineKind::ReadsTold, name: Vec::new(), learned: false });
        } else if let Some(body) = line.strip_prefix(WRITE_START)
            && let Some((old, new)) = body.trim_end_matches(SENTENCE_END).split_once(WRITE_AS)
        {
            if let Some(letter) = stray_letter(body) {
                return Err(format!("{name}: {STRAY_LETTER}: {letter}: {line}"));
            }
            let (new, held) = write_conditions(new);
            let (mut conditions, absent) = if held.is_empty() { (Vec::new(), Vec::new()) } else { conditions(held) };
            let (old, kind) = match old.strip_prefix(WRITE_INSIDE) {
                Some(inner) => (inner, LineKind::ReplacesInside),
                None => (old, LineKind::Replaces),
            };
            conditions.insert(0, Condition::Fact { pattern: pattern(old), when: Moment::Any });
            topic.rules.push(ToldRule { conditions, absent, outcome: pattern(new), kind, name: Vec::new(), learned: false });
        } else if let Some((question, answers)) = line.strip_prefix(QUOTED_START).and_then(|rest| rest.split_once(QUOTED_END)).or_else(|| line.split_once(QUESTION_MARK)) {
            match question_form(question, answers, &mut None) {
                Ok(form) => topic.questions.push(form),
                Err(letter) => return Err(format!("{name}: {STRAY_LETTER}: {letter}: {line}")),
            }
        } else if line.ends_with(SENTENCE_END) {
            topic.facts.push(fact_numbers(line));
        } else {
            return Err(format!("{name}: {UNREAD_LINE}: {line}"));
        }
    }
    topic.rules_key = rules_key(&topic.rules);
    Ok(topic)
}
because!(read_topic, ToldLogic, "the rules, the lines that write a sentence in a new form, the line that reads told rules, question forms, reply lines, told facts and the topics that it builds on of one text of the library, line by line, its notes left out, or the fault of the first line that is none of them or whose rule or answers hold a capital letter that is no open place");
