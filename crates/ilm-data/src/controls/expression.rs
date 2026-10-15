use ilm_core::base::cursor::ItemReader;
use patterns::{because, source};

pub struct WrittenExpressions;
source!(WrittenExpressions, "the value of an expression as it is written in a program: whole numbers with plus, minus, times, power and brackets, where power binds before a sign in front of a number, a sign before times and times before plus and minus, and power groups from the right; or truth words with not, and, or and brackets, where not binds before and, and before or, and empty brackets are false. A number with a nought in front of other digits, two numbers or two operations side by side, and brackets right after a value are no expression");

const SUM_PLUS: char = '+';
because!(SUM_PLUS, WrittenExpressions, "the sign of a sum, and a sign in front of a number");

const SUM_MINUS: char = '-';
because!(SUM_MINUS, WrittenExpressions, "the sign of a difference, and the sign in front of a number below nought");

const SUM_TIMES: char = '*';
because!(SUM_TIMES, WrittenExpressions, "the sign of a product; the sign twice side by side is the sign of a power");

const SUM_OPEN: char = '(';
because!(SUM_OPEN, WrittenExpressions, "the bracket that opens a part of an expression");

const SUM_CLOSE: char = ')';
because!(SUM_CLOSE, WrittenExpressions, "the bracket that closes a part of an expression");

const SUM_ZERO: char = '0';
because!(SUM_ZERO, WrittenExpressions, "the digit that may not stand in front of other digits of a number that is not nought");

const SUM_SPACES: &str = " \t\u{c}";
because!(SUM_SPACES, WrittenExpressions, "the signs of space between the parts of an expression on one line");

const SUM_LINES: &str = "\n\r";
because!(SUM_LINES, WrittenExpressions, "the signs that end a line, which are space only inside brackets");

pub const TRUTH_TRUE: &str = "True";
because!(TRUTH_TRUE, WrittenExpressions, "the word whose value holds, and the value that is written when an expression holds");

pub const TRUTH_FALSE: &str = "False";
because!(TRUTH_FALSE, WrittenExpressions, "the word whose value does not hold, and the value that is written when an expression does not hold");

const TRUTH_NOT: &str = "not";
because!(TRUTH_NOT, WrittenExpressions, "the word in front of a part whose value it turns round");

const TRUTH_AND: &str = "and";
because!(TRUTH_AND, WrittenExpressions, "the word that is true when both sides are");

const TRUTH_OR: &str = "or";
because!(TRUTH_OR, WrittenExpressions, "the word that is true when a side is");

pub const TRUTH_OPEN: &str = "(";
because!(TRUTH_OPEN, WrittenExpressions, "the bracket that opens a part of an expression of truths, a word of its own");

pub const TRUTH_CLOSE: &str = ")";
because!(TRUTH_CLOSE, WrittenExpressions, "the bracket that closes a part of an expression of truths, a word of its own");

pub const TRUTH_WORDS: &[&str] = &[TRUTH_TRUE, TRUTH_FALSE, TRUTH_AND, TRUTH_OR, TRUTH_NOT, TRUTH_OPEN, TRUTH_CLOSE];
because!(TRUTH_WORDS, WrittenExpressions, "the words of an expression of truths: the truth words, the joining words, the turning word and the brackets");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SumToken {
    Number(i128),
    Plus,
    Minus,
    Times,
    Power,
    Open,
    Close,
}
because!(SumToken, WrittenExpressions, "one part of an expression of numbers: a number, a sign of an operation, or a bracket");

fn sum_tokens(text: &str) -> Option<Vec<SumToken>> {
    let signs: Vec<char> = text.chars().collect();
    let mut out: Vec<SumToken> = Vec::new();
    let (mut at, mut depth) = (0usize, 0usize);
    while at < signs.len() {
        let sign = signs[at];
        at += 1;
        match sign {
            SUM_PLUS => out.push(SumToken::Plus),
            SUM_MINUS => out.push(SumToken::Minus),
            SUM_TIMES if signs.get(at) == Some(&SUM_TIMES) => {
                at += 1;
                out.push(SumToken::Power);
            }
            SUM_TIMES => out.push(SumToken::Times),
            SUM_OPEN => {
                depth += 1;
                out.push(SumToken::Open);
            }
            SUM_CLOSE => {
                depth = depth.checked_sub(1)?;
                out.push(SumToken::Close);
            }
            _ if SUM_SPACES.contains(sign) => {}
            _ if SUM_LINES.contains(sign) && depth > 0 => {}
            _ if sign.is_ascii_digit() => {
                let start = at - 1;
                while at < signs.len() && signs[at].is_ascii_digit() {
                    at += 1;
                }
                let digits: String = signs[start..at].iter().collect();
                if digits.starts_with(SUM_ZERO) && digits.chars().any(|digit| digit != SUM_ZERO) {
                    return None;
                }
                out.push(SumToken::Number(digits.parse::<i128>().ok()?));
            }
            _ => return None,
        }
    }
    Some(out)
}
because!(sum_tokens, WrittenExpressions, "the parts of an expression of numbers, or nothing for a sign that no expression has");

struct SumReader<'a> {
    tokens: &'a [SumToken],
    at: usize,
}
because!(SumReader, WrittenExpressions, "the reader of an expression of numbers and its place. sum reads plus and minus between terms; term reads times between factors; factor reads a sign in front; power reads a power; atom reads a number or a part in brackets");

impl ItemReader<SumToken> for SumReader<'_> {
    fn items_at(&self) -> (&[SumToken], usize) {
        (self.tokens, self.at)
    }
}

impl SumReader<'_> {
    fn sum(&mut self) -> Option<i128> {
        let mut value = self.term()?;
        loop {
            match self.peek() {
                Some(SumToken::Plus) => {
                    self.at += 1;
                    value = value.checked_add(self.term()?)?;
                }
                Some(SumToken::Minus) => {
                    self.at += 1;
                    value = value.checked_sub(self.term()?)?;
                }
                _ => return Some(value),
            }
        }
    }

    fn term(&mut self) -> Option<i128> {
        let mut value = self.factor()?;
        while self.peek() == Some(SumToken::Times) {
            self.at += 1;
            value = value.checked_mul(self.factor()?)?;
        }
        Some(value)
    }

    fn factor(&mut self) -> Option<i128> {
        match self.peek() {
            Some(SumToken::Plus) => {
                self.at += 1;
                self.factor()
            }
            Some(SumToken::Minus) => {
                self.at += 1;
                self.factor()?.checked_neg()
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Option<i128> {
        let base = self.atom()?;
        if self.peek() != Some(SumToken::Power) {
            return Some(base);
        }
        self.at += 1;
        let exponent = u32::try_from(self.factor()?).ok()?;
        base.checked_pow(exponent)
    }

    fn atom(&mut self) -> Option<i128> {
        let token = self.peek()?;
        self.at += 1;
        match token {
            SumToken::Number(value) => Some(value),
            SumToken::Open => {
                let value = self.sum()?;
                (self.peek() == Some(SumToken::Close)).then(|| self.at += 1)?;
                Some(value)
            }
            _ => None,
        }
    }
}

pub fn number_value(text: &str) -> Option<String> {
    let tokens = sum_tokens(text)?;
    let mut reader = SumReader { tokens: &tokens, at: 0 };
    let value = reader.sum()?;
    (reader.at == tokens.len()).then(|| value.to_string())
}
because!(number_value, WrittenExpressions, "the value of a whole expression of numbers, or nothing when the text is no such expression, when a power has a negative exponent, or when a value is too large to hold");

struct TruthReader<'a> {
    words: &'a [&'a str],
    at: usize,
}
because!(TruthReader, WrittenExpressions, "the reader of an expression of truths and its place. either reads or between parts; both reads and between parts; negated reads not in front; truth_atom reads a truth word or a part in brackets");

impl<'a> ItemReader<&'a str> for TruthReader<'a> {
    fn items_at(&self) -> (&[&'a str], usize) {
        (self.words, self.at)
    }
}

impl<'a> TruthReader<'a> {
    fn either(&mut self) -> Option<bool> {
        let mut value = self.both()?;
        while self.peek() == Some(TRUTH_OR) {
            self.at += 1;
            let right = self.both()?;
            value = value || right;
        }
        Some(value)
    }

    fn both(&mut self) -> Option<bool> {
        let mut value = self.negated()?;
        while self.peek() == Some(TRUTH_AND) {
            self.at += 1;
            let right = self.negated()?;
            value = value && right;
        }
        Some(value)
    }

    fn negated(&mut self) -> Option<bool> {
        if self.peek() == Some(TRUTH_NOT) {
            self.at += 1;
            return self.negated().map(|value| !value);
        }
        self.truth_atom()
    }

    fn truth_atom(&mut self) -> Option<bool> {
        let word = self.peek()?;
        self.at += 1;
        if word == TRUTH_TRUE {
            return Some(true);
        }
        if word == TRUTH_FALSE {
            return Some(false);
        }
        if word != TRUTH_OPEN {
            return None;
        }
        if self.peek() == Some(TRUTH_CLOSE) {
            self.at += 1;
            return Some(false);
        }
        let value = self.either()?;
        (self.peek() == Some(TRUTH_CLOSE)).then(|| self.at += 1)?;
        Some(value)
    }
}

pub fn truth_value(words: &[&str]) -> Option<String> {
    let mut reader = TruthReader { words, at: 0 };
    let value = reader.either()?;
    let word = if value { TRUTH_TRUE } else { TRUTH_FALSE };
    (reader.at == words.len()).then(|| word.to_string())
}
because!(truth_value, WrittenExpressions, "the truth word of a whole expression of truths, or nothing when the words are no such expression");
