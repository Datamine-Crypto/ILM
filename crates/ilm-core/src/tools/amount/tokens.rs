use crate::base::exact::Amount;
use crate::base::symbols::Symbols;
use patterns::{because, source};

pub struct AmountTokens;
source!(AmountTokens, "a text as tokens for a learner of calculations: every number is one token with its exact amount, a word is in small letters, each sign of arithmetic and each bracket is a token of its own, and marks that no text can hold stand at the end of each told sentence and before the question");

pub(super) const AMOUNT_OPEN: &str = "*";
because!(AMOUNT_OPEN, AmountTokens, "the mark of a place of a form that is left open. The sign of a product is the same token, so a form with that sign at a place is the form with the place left open");

pub(crate) const AMOUNT_NUMBER: &str = "#";
because!(AMOUNT_NUMBER, AmountTokens, "the token of every number of a text and of the number in the form of an answer");

pub(crate) const AMOUNT_END: &str = "|";
because!(AMOUNT_END, AmountTokens, "the token at the end of each told sentence; no token of a text can be this mark");

pub(crate) const AMOUNT_ASK: &str = "||";
because!(AMOUNT_ASK, AmountTokens, "the token before the question; no token of a text can be this mark");

pub(super) const AMOUNT_FIRST: &str = "(start)";
because!(AMOUNT_FIRST, AmountTokens, "the token before an operand that is the first token of a case");

pub(super) const AMOUNT_LAST: &str = "(end)";
because!(AMOUNT_LAST, AmountTokens, "the token after an operand that is the last token of a case");

pub(super) const AMOUNT_NO_TOKEN: &str = "";
because!(AMOUNT_NO_TOKEN, AmountTokens, "the token of a gap that holds no token: two operands that follow one another directly");

pub(super) const AMOUNT_EXAMPLES: &str = "(examples)";
because!(AMOUNT_EXAMPLES, AmountTokens, "the first token of the form of a calculation that the examples inside a case show; no token of a text can be this word");

pub(super) const AMOUNT_IN_STEPS: &str = "(steps)";
because!(AMOUNT_IN_STEPS, AmountTokens, "the form of a calculation in steps; no token of a text can be this word");

pub(super) const AMOUNT_WORD_STEPS: &str = "(steps of words)";
because!(AMOUNT_WORD_STEPS, AmountTokens, "the form of a calculation in steps over the answer words of a kind; no token of a text can be this word");

pub(super) const AMOUNT_SIGN_TOKENS: &str = "-+*/=<>()[]{}^%$";
because!(AMOUNT_SIGN_TOKENS, AmountTokens, "the marks that are tokens of their own: the signs of arithmetic, the brackets, and the signs of a power, a share and money");

pub(super) const AMOUNT_JOINTS: &str = "^/";
because!(AMOUNT_JOINTS, AmountTokens, "the marks that join the parts of a unit into one word, as in a unit of area or of speed");

pub(super) const AMOUNT_BEFORE_SIGN: &str = " \t\n([{=+-*/<>:,";
because!(AMOUNT_BEFORE_SIGN, AmountTokens, "a minus sign after one of these marks, or at the start of a text, can belong to the number that follows it; after a letter, a digit or a closing bracket it is a subtraction");

pub(super) const AMOUNT_MINUS: char = '-';
because!(AMOUNT_MINUS, AmountTokens, "the sign that makes the number after it negative when it stands where no subtraction can stand");

pub(super) const AMOUNT_POINT: char = '.';
because!(AMOUNT_POINT, AmountTokens, "the mark between the whole part of a number of a text and its digits below one");

const AMOUNT_GROUP: char = ',';
because!(AMOUNT_GROUP, AmountTokens, "the mark that a text may write between the groups of digits of a large whole number");

const GROUP_DIGITS: usize = 3;
because!(GROUP_DIGITS, AmountTokens, "the digits of each group after the mark between groups: a comma with another number of digits after it parts two numbers");

fn group_follows(bytes: &[u8], at: usize) -> bool {
    let end = at + 1 + GROUP_DIGITS;
    end <= bytes.len() && char::from(bytes[at]) == AMOUNT_GROUP && bytes[at + 1..end].iter().all(u8::is_ascii_digit) && !bytes.get(end).is_some_and(u8::is_ascii_digit)
}
because!(group_follows, AmountTokens, "whether a group of a large number goes on at a place: the mark between groups and then exactly the digits of one group");

pub(super) const AMOUNT_UNDERLINE: char = '_';
because!(AMOUNT_UNDERLINE, AmountTokens, "the mark that a text of a program writes inside a name; it is a letter of a word");

pub(super) const AMOUNT_BRACKETS: &[(&str, &str)] = &[("(", ")"), ("[", "]"), ("{", "}")];
because!(AMOUNT_BRACKETS, AmountTokens, "each opening bracket with the bracket that closes it");

pub(super) const AMOUNT_BETWEEN: &str = " ";
because!(AMOUNT_BETWEEN, AmountTokens, "the mark between the words of an answer and between the tokens of a form in a note");

#[derive(Clone, Debug, Default)]
pub(super) struct AmountMarks {
    pub(super) open: u32,
    pub(super) number: u32,
    pub(super) end: u32,
    pub(super) ask: u32,
    pub(super) first: u32,
    pub(super) last: u32,
    pub(super) none: u32,
    pub(super) examples: u32,
    pub(super) steps: u32,
    pub(super) word_steps: u32,
    pub(super) brackets: Vec<(u32, u32)>,
    pub(super) names: Vec<u32>,
}
because!(AmountMarks, AmountTokens, "the numbers of the tokens that the tool itself puts into a form or looks for, taken once from the table of tokens: the marks of a form, the brackets, and the name of each calculation, which stands in the form of a calculation shown by examples");

impl AmountMarks {
    pub(super) fn new(symbols: &mut Symbols, names: &[String]) -> AmountMarks {
        AmountMarks {
            open: symbols.number(AMOUNT_OPEN),
            number: symbols.number(AMOUNT_NUMBER),
            end: symbols.number(AMOUNT_END),
            ask: symbols.number(AMOUNT_ASK),
            first: symbols.number(AMOUNT_FIRST),
            last: symbols.number(AMOUNT_LAST),
            none: symbols.number(AMOUNT_NO_TOKEN),
            examples: symbols.number(AMOUNT_EXAMPLES),
            steps: symbols.number(AMOUNT_IN_STEPS),
            word_steps: symbols.number(AMOUNT_WORD_STEPS),
            brackets: AMOUNT_BRACKETS.iter().map(|(open, close)| (symbols.number(open), symbols.number(close))).collect(),
            names: names.iter().map(|name| symbols.number(name)).collect(),
        }
    }

    pub(super) fn is_bracket(&self, token: u32) -> bool {
        self.brackets.iter().any(|(open, close)| *open == token || *close == token)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AmountCut {
    pub(crate) words: Vec<u32>,
    pub(crate) spots: Vec<u32>,
    pub(crate) values: Vec<Option<Amount>>,
}
because!(AmountCut, AmountTokens, "a text as the tool reads it: the numbers of its tokens, the places of the tokens that are numbers, and the exact amount of each such number (nothing for a number that left the width of the amounts). scan adds the tokens of one text: a run of digits with or without a decimal part is a number; a run of letters is a word in small letters, and it may go on after a joint mark; a sign is a token of its own; a minus sign directly before digits makes the number negative when no letter, digit or closing bracket stands before the sign. stands says if an amount stands in the text as a number that is no operand");

impl AmountCut {
    pub(super) fn clear(&mut self) {
        self.words.clear();
        self.spots.clear();
        self.values.clear();
    }

    pub(crate) fn scan<N: FnMut(&str) -> u32>(&mut self, text: &str, number: u32, lower: &mut String, name: &mut N) {
        let bytes = text.as_bytes();
        let mut at = 0;
        let mut minus_end: Option<usize> = None;
        while at < bytes.len() {
            let start = at;
            let sign = char::from(bytes[at]);
            if bytes[at].is_ascii_digit() {
                while at < bytes.len() && bytes[at].is_ascii_digit() {
                    at += 1;
                }
                if at - start <= GROUP_DIGITS {
                    while group_follows(bytes, at) {
                        at += 1 + GROUP_DIGITS;
                    }
                }
                let whole_end = at;
                if at + 1 < bytes.len() && char::from(bytes[at]) == AMOUNT_POINT && bytes[at + 1].is_ascii_digit() {
                    at += 1;
                    while at < bytes.len() && bytes[at].is_ascii_digit() {
                        at += 1;
                    }
                }
                let after = if at > whole_end { &text[whole_end + 1..at] } else { AMOUNT_NO_TOKEN };
                let whole: String = text[start..whole_end].chars().filter(|sign| *sign != AMOUNT_GROUP).collect();
                let mut value = Amount::from_digits(&whole, after);
                if minus_end == Some(start) {
                    self.words.pop();
                    value = value.and_then(Amount::negated);
                }
                self.spots.push(self.words.len() as u32);
                self.words.push(number);
                self.values.push(value);
                minus_end = None;
            } else if bytes[at].is_ascii_alphabetic() || sign == AMOUNT_UNDERLINE {
                while at < bytes.len() && (bytes[at].is_ascii_alphabetic() || char::from(bytes[at]) == AMOUNT_UNDERLINE) {
                    at += 1;
                }
                while at + 1 < bytes.len() && AMOUNT_JOINTS.contains(char::from(bytes[at])) && bytes[at + 1].is_ascii_alphanumeric() {
                    at += 1;
                    while at < bytes.len() && bytes[at].is_ascii_alphanumeric() {
                        at += 1;
                    }
                }
                lower.clear();
                lower.push_str(&text[start..at]);
                lower.make_ascii_lowercase();
                self.words.push(name(lower));
                minus_end = None;
            } else if bytes[at].is_ascii() && AMOUNT_SIGN_TOKENS.contains(sign) {
                at += 1;
                let may_join = sign == AMOUNT_MINUS && (start == 0 || (bytes[start - 1].is_ascii() && AMOUNT_BEFORE_SIGN.contains(char::from(bytes[start - 1]))));
                minus_end = may_join.then_some(at);
                self.words.push(name(&text[start..at]));
            } else {
                at += 1;
            }
        }
    }

    pub(super) fn stands(&self, value: Amount, from: usize, size: usize) -> bool {
        self.values.iter().enumerate().any(|(place, held)| *held == Some(value) && !(from..from + size).contains(&place))
    }
}

pub(super) fn amount_cut_case<N: FnMut(&str) -> u32>(sentences: &[String], question: &str, marks: &AmountMarks, cut: &mut AmountCut, lower: &mut String, name: &mut N) {
    cut.clear();
    for sentence in sentences {
        cut.scan(sentence, marks.number, lower, name);
        cut.words.push(marks.end);
    }
    cut.words.push(marks.ask);
    cut.scan(question, marks.number, lower, name);
}
because!(amount_cut_case, AmountTokens, "the tokens of a case: each told sentence with its end mark, then the question after its mark");

#[derive(Debug, Default)]
pub(super) struct AmountUnknown {
    pub(super) text: String,
    pub(super) ends: Vec<usize>,
}
because!(AmountUnknown, AmountTokens, "the tokens of one case that no lesson held, each with a number above the numbers of the table of tokens, so that two different new tokens give two different forms, and the limit of forms cuts a case with new tokens where it cuts the same case with known tokens; the texts are kept for the one case where a new token is a word of the answer");

impl AmountUnknown {
    pub(super) fn clear(&mut self) {
        self.text.clear();
        self.ends.clear();
    }

    pub(super) fn name(&self, place: usize) -> &str {
        let from = if place == 0 { 0 } else { self.ends[place - 1] };
        &self.text[from..self.ends[place]]
    }

    pub(super) fn number(&mut self, word: &str, known: usize) -> u32 {
        let place = match (0..self.ends.len()).find(|place| self.name(*place) == word) {
            Some(place) => place,
            None => {
                self.text.push_str(word);
                self.ends.push(self.text.len());
                self.ends.len() - 1
            }
        };
        (known + place) as u32
    }
}
