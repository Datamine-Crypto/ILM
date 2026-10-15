use ilm_core::base::cursor::ItemReader;
use ilm_core::know::signs::{is_space, is_word_sign};
use patterns::{because, source};

pub struct PythonPatterns;
source!(PythonPatterns, "the regular expressions of the re module of Python for texts, matched by backtracking as Python matches them: the leftmost match, the alternatives tried from the left, a greedy repeat tried from its longest and a lazy one from its shortest, groups that capture, classes of signs, the word signs, white space and decimal digits of Unicode, the start and the end of a text (the end also before a last line break), a look behind of a fixed width, the flag that lets the any sign match a line break, and after an empty match a next match that must not be empty at the same place");

const ESCAPE_SIGN: char = '\\';
because!(ESCAPE_SIGN, PythonPatterns, "the sign that makes the next sign of a pattern stand for itself, for a control sign or for a sort of signs");

const ANY_SIGN: char = '.';
because!(ANY_SIGN, PythonPatterns, "the sign of a pattern that matches any sign, but a line break when the pattern has no flag for it");

const LINE_BREAK: char = '\n';
because!(LINE_BREAK, PythonPatterns, "the line break, which the any sign matches only with the flag, and before which the end of a text also matches when it is the last sign");

const DOT_ALL: &str = "(?s)";
because!(DOT_ALL, PythonPatterns, "the flag at the start of a pattern that lets the any sign match a line break too");

const OPEN_GROUP: char = '(';
because!(OPEN_GROUP, PythonPatterns, "the sign before the pieces that a repeat takes as one and whose matched text is kept unless a sort follows it");

const CLOSE_GROUP: char = ')';
because!(CLOSE_GROUP, PythonPatterns, "the sign after the pieces that a repeat takes as one");

const OPEN_CLASS: char = '[';
because!(OPEN_CLASS, PythonPatterns, "the sign that opens a class of signs");

const CLOSE_CLASS: char = ']';
because!(CLOSE_CLASS, PythonPatterns, "the sign that closes a class of signs, and as the first sign of a class a sign of it");

const NOT_CLASS: char = '^';
because!(NOT_CLASS, PythonPatterns, "the sign at the start of a class that makes it match every other sign, and outside a class the start of the text");

const END_SIGN: char = '$';
because!(END_SIGN, PythonPatterns, "the sign that matches where the text stops, or before a last line break");

const RANGE_SIGN: char = '-';
because!(RANGE_SIGN, PythonPatterns, "the sign between the two ends of a range inside a class, unless it is the last sign of the class");

const ALTERNATIVE: char = '|';
because!(ALTERNATIVE, PythonPatterns, "the sign between two alternatives");

const STAR: char = '*';
because!(STAR, PythonPatterns, "the repeat of nought times or more");

const PLUS: char = '+';
because!(PLUS, PythonPatterns, "the repeat of once or more");

const OPTIONAL_SIGN: char = '?';
because!(OPTIONAL_SIGN, PythonPatterns, "the repeat of nought times or once, after a repeat the mark of a lazy one, and after an opening bracket the start of a group of another sort");

const OPEN_COUNT: char = '{';
because!(OPEN_COUNT, PythonPatterns, "the sign that opens a counted repeat, which this reader refuses so that no pattern means another thing here than in Python");

const NO_CAPTURE: char = ':';
because!(NO_CAPTURE, PythonPatterns, "the sign after the opening of a group that captures nothing");

const BEHIND: char = '<';
because!(BEHIND, PythonPatterns, "the sign that opens a look behind, before the equal sign");

const POSITIVE: char = '=';
because!(POSITIVE, PythonPatterns, "the sign of a look that must match");

const PATTERN_ESCAPES: &[(char, char)] = &[('n', '\n'), ('t', '\t'), ('r', '\r'), ('f', '\u{c}'), ('v', '\u{b}'), ('a', '\u{7}')];
because!(PATTERN_ESCAPES, PythonPatterns, "the letters after a backslash that stand for signs of control; after a backslash every other sign that names no sort stands for itself");

const SORT_ESCAPES: &[(char, SignSort, bool)] = &[('w', SignSort::Word, true), ('W', SignSort::Word, false), ('s', SignSort::Space, true), ('S', SignSort::Space, false), ('d', SignSort::Digit, true), ('D', SignSort::Digit, false)];
because!(SORT_ESCAPES, PythonPatterns, "the letters after a backslash that name a sort of signs: the small letter its signs, the capital letter every other sign");

const DECIMAL_RUNS: &[(u32, u32)] = &[
    (0x30, 0x39), (0x660, 0x669), (0x6F0, 0x6F9), (0x7C0, 0x7C9), (0x966, 0x96F), (0x9E6, 0x9EF), (0xA66, 0xA6F), (0xAE6, 0xAEF),
    (0xB66, 0xB6F), (0xBE6, 0xBEF), (0xC66, 0xC6F), (0xCE6, 0xCEF), (0xD66, 0xD6F), (0xDE6, 0xDEF), (0xE50, 0xE59), (0xED0, 0xED9),
    (0xF20, 0xF29), (0x1040, 0x1049), (0x1090, 0x1099), (0x17E0, 0x17E9), (0x1810, 0x1819), (0x1946, 0x194F), (0x19D0, 0x19D9), (0x1A80, 0x1A89),
    (0x1A90, 0x1A99), (0x1B50, 0x1B59), (0x1BB0, 0x1BB9), (0x1C40, 0x1C49), (0x1C50, 0x1C59), (0xA620, 0xA629), (0xA8D0, 0xA8D9), (0xA900, 0xA909),
    (0xA9D0, 0xA9D9), (0xA9F0, 0xA9F9), (0xAA50, 0xAA59), (0xABF0, 0xABF9), (0xFF10, 0xFF19), (0x104A0, 0x104A9), (0x10D30, 0x10D39), (0x11066, 0x1106F),
    (0x110F0, 0x110F9), (0x11136, 0x1113F), (0x111D0, 0x111D9), (0x112F0, 0x112F9), (0x11450, 0x11459), (0x114D0, 0x114D9), (0x11650, 0x11659), (0x116C0, 0x116C9),
    (0x11730, 0x11739), (0x118E0, 0x118E9), (0x11950, 0x11959), (0x11C50, 0x11C59), (0x11D50, 0x11D59), (0x11DA0, 0x11DA9), (0x11F50, 0x11F59), (0x16A60, 0x16A69),
    (0x16AC0, 0x16AC9), (0x16B50, 0x16B59), (0x1D7CE, 0x1D7FF), (0x1E140, 0x1E149), (0x1E2F0, 0x1E2F9), (0x1E4F0, 0x1E4F9), (0x1E950, 0x1E959), (0x1FBF0, 0x1FBF9),
];
because!(DECIMAL_RUNS, PythonPatterns, "the first and the last code point of each run of decimal digits of Unicode, the signs that the escape of digits matches");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SignSort {
    Word,
    Space,
    Digit,
}
because!(SignSort, PythonPatterns, "the three sorts of signs that an escape names: word signs, white space, decimal digits");

fn of_sort(sort: SignSort, sign: char) -> bool {
    match sort {
        SignSort::Word => is_word_sign(sign),
        SignSort::Space => is_space(sign),
        SignSort::Digit => DECIMAL_RUNS.iter().any(|(first, last)| (*first..=*last).contains(&u32::from(sign))),
    }
}
because!(of_sort, PythonPatterns, "whether a sign is of a sort, as Python counts the word signs, white space and decimal digits");

#[derive(Clone, Debug)]
enum ClassItem {
    One(char),
    Range(char, char),
    Sorted(SignSort, bool),
}
because!(ClassItem, PythonPatterns, "one item of a class: a sign, a range of signs, or a sort of signs with the mark of whether the signs of the sort or the other signs match");

#[derive(Clone, Copy, Debug)]
struct Times {
    least: usize,
    most: usize,
    greedy: bool,
}
because!(Times, PythonPatterns, "the least and the most times of a repeat, and whether it tries the most times first");

#[derive(Clone, Debug)]
enum PatternPiece {
    Sign(char),
    Any(bool),
    Class(Vec<ClassItem>, bool),
    Start,
    End,
    Group(Box<PatternPiece>, Option<usize>),
    Row(Vec<PatternPiece>),
    Either(Vec<PatternPiece>),
    Repeat(Box<PatternPiece>, Times),
    Behind(Box<PatternPiece>, usize),
}
because!(PatternPiece, PythonPatterns, "one piece of a parsed pattern: a sign, any sign (with the mark of whether it matches a line break), a class (with the mark of the other signs), the start, the end, a group with its number when it captures, a row of pieces, alternatives, a repeat, a look behind with its width");

fn class_holds(items: &[ClassItem], negated: bool, sign: char) -> bool {
    let found = items.iter().any(|item| match item {
        ClassItem::One(held) => *held == sign,
        ClassItem::Range(low, high) => (*low..=*high).contains(&sign),
        ClassItem::Sorted(sort, yes) => of_sort(*sort, sign) == *yes,
    });
    found != negated
}
because!(class_holds, PythonPatterns, "whether a class matches a sign: some item holds it, or for a class of the other signs no item holds it");

fn one_sign(piece: &PatternPiece, sign: char) -> Option<bool> {
    match piece {
        PatternPiece::Sign(held) => Some(*held == sign),
        PatternPiece::Any(breaks) => Some(*breaks || sign != LINE_BREAK),
        PatternPiece::Class(items, negated) => Some(class_holds(items, *negated, sign)),
        _ => None,
    }
}
because!(one_sign, PythonPatterns, "whether a piece that matches one sign matches this one, or nothing for a piece of another sort");

fn fixed_width(piece: &PatternPiece) -> Option<usize> {
    match piece {
        PatternPiece::Sign(_) | PatternPiece::Any(_) | PatternPiece::Class(_, _) => Some(1),
        PatternPiece::Start | PatternPiece::End | PatternPiece::Behind(_, _) => Some(0),
        PatternPiece::Group(inner, _) => fixed_width(inner),
        PatternPiece::Row(pieces) => pieces.iter().map(fixed_width).sum(),
        PatternPiece::Either(options) => {
            let widths: Vec<Option<usize>> = options.iter().map(fixed_width).collect();
            let first = *widths.first()?;
            widths.iter().all(|width| *width == first).then_some(first)?
        }
        PatternPiece::Repeat(inner, times) => (times.least == times.most).then(|| fixed_width(inner).map(|width| width * times.least)).flatten(),
    }
}
because!(fixed_width, PythonPatterns, "the number of signs that a piece always matches, or nothing when it can match rows of other lengths, as a look behind needs");

struct Parser {
    signs: Vec<char>,
    at: usize,
    groups: usize,
    breaks: bool,
}
because!(Parser, PythonPatterns, "the reader of a pattern: its signs, the place it has come to, the groups that capture so far, and whether the any sign matches a line break");

impl ItemReader<char> for Parser {
    fn items_at(&self) -> (&[char], usize) {
        (&self.signs, self.at)
    }
}

impl Parser {
    fn take(&mut self) -> Result<char, String> {
        let sign = self.peek().ok_or("a pattern ends too early")?;
        self.at += 1;
        Ok(sign)
    }

    fn escape(&mut self) -> Result<PatternPiece, String> {
        let sign = self.take()?;
        if let Some((_, sort, yes)) = SORT_ESCAPES.iter().find(|(held, _, _)| *held == sign) {
            return Ok(PatternPiece::Class(vec![ClassItem::Sorted(*sort, *yes)], false));
        }
        Ok(PatternPiece::Sign(PATTERN_ESCAPES.iter().find(|(held, _)| *held == sign).map_or(sign, |(_, made)| *made)))
    }

    fn class_sign(&mut self) -> Result<Result<char, ClassItem>, String> {
        let sign = self.take()?;
        if sign != ESCAPE_SIGN {
            return Ok(Ok(sign));
        }
        match self.escape()? {
            PatternPiece::Sign(made) => Ok(Ok(made)),
            PatternPiece::Class(mut items, _) => items.pop().map(Err).ok_or_else(|| "an empty sort".to_string()),
            _ => Err("an escape that a class cannot hold".to_string()),
        }
    }

    fn class(&mut self) -> Result<PatternPiece, String> {
        let negated = self.peek() == Some(NOT_CLASS);
        if negated {
            self.at += 1;
        }
        let mut items: Vec<ClassItem> = Vec::new();
        loop {
            if self.peek() == Some(CLOSE_CLASS) && !items.is_empty() {
                self.at += 1;
                return Ok(PatternPiece::Class(items, negated));
            }
            let low = match self.class_sign()? {
                Ok(sign) => sign,
                Err(item) => {
                    items.push(item);
                    continue;
                }
            };
            let ranged = self.peek() == Some(RANGE_SIGN) && self.signs.get(self.at + 1).is_some_and(|next| *next != CLOSE_CLASS);
            if !ranged {
                items.push(ClassItem::One(low));
                continue;
            }
            self.at += 1;
            match self.class_sign()? {
                Ok(high) => items.push(ClassItem::Range(low, high)),
                Err(_) => return Err("a range that ends in a sort".to_string()),
            }
        }
    }

    fn group(&mut self) -> Result<PatternPiece, String> {
        let mut number = None;
        let mut behind = false;
        if self.peek() == Some(OPTIONAL_SIGN) {
            self.at += 1;
            let sort = self.take()?;
            behind = sort == BEHIND && self.take()? == POSITIVE;
            if !behind && sort != NO_CAPTURE {
                return Err(format!("a group of the sort {sort} is not read"));
            }
        } else {
            self.groups += 1;
            number = Some(self.groups);
        }
        let inner = self.alternatives()?;
        if self.take()? != CLOSE_GROUP {
            return Err("a group with no end".to_string());
        }
        if behind {
            let width = fixed_width(&inner).ok_or("a look behind of no fixed width")?;
            return Ok(PatternPiece::Behind(Box::new(inner), width));
        }
        Ok(PatternPiece::Group(Box::new(inner), number))
    }

    fn atom(&mut self) -> Result<PatternPiece, String> {
        Ok(match self.take()? {
            ESCAPE_SIGN => self.escape()?,
            ANY_SIGN => PatternPiece::Any(self.breaks),
            NOT_CLASS => PatternPiece::Start,
            END_SIGN => PatternPiece::End,
            OPEN_CLASS => self.class()?,
            OPEN_GROUP => self.group()?,
            OPEN_COUNT => return Err("a counted repeat is not read".to_string()),
            sign => PatternPiece::Sign(sign),
        })
    }

    fn repeated(&mut self, piece: PatternPiece) -> PatternPiece {
        let (least, most) = match self.peek() {
            Some(STAR) => (0, usize::MAX),
            Some(PLUS) => (1, usize::MAX),
            Some(OPTIONAL_SIGN) => (0, 1),
            _ => return piece,
        };
        self.at += 1;
        let greedy = self.peek() != Some(OPTIONAL_SIGN);
        if !greedy {
            self.at += 1;
        }
        PatternPiece::Repeat(Box::new(piece), Times { least, most, greedy })
    }

    fn row(&mut self) -> Result<PatternPiece, String> {
        let mut pieces: Vec<PatternPiece> = Vec::new();
        while self.peek().is_some_and(|sign| sign != ALTERNATIVE && sign != CLOSE_GROUP) {
            let atom = self.atom()?;
            pieces.push(self.repeated(atom));
        }
        Ok(PatternPiece::Row(pieces))
    }

    fn alternatives(&mut self) -> Result<PatternPiece, String> {
        let mut options: Vec<PatternPiece> = vec![self.row()?];
        while self.peek() == Some(ALTERNATIVE) {
            self.at += 1;
            options.push(self.row()?);
        }
        Ok(if options.len() == 1 { options.swap_remove(0) } else { PatternPiece::Either(options) })
    }
}

type Spans = Vec<Option<(usize, usize)>>;
because!(Spans, PythonPatterns, "the places where each group of a match starts and ends, nothing for a group that took no part");

type Then<'a> = &'a mut dyn FnMut(usize, &mut Spans) -> bool;
because!(Then, PythonPatterns, "what a piece calls with the place where it ends and the groups so far, to match the rest of the pattern; true ends the search with a match");

struct Run<'a> {
    signs: &'a [char],
}
because!(Run, PythonPatterns, "the signs of a text that a pattern is matched on");

impl Run<'_> {
    fn step(&self, piece: &PatternPiece, at: usize, spans: &mut Spans, next: Then) -> bool {
        match piece {
            PatternPiece::Sign(_) | PatternPiece::Any(_) | PatternPiece::Class(_, _) => self.signs.get(at).is_some_and(|sign| one_sign(piece, *sign) == Some(true)) && next(at + 1, spans),
            PatternPiece::Start => at == 0 && next(at, spans),
            PatternPiece::End => (at == self.signs.len() || (at + 1 == self.signs.len() && self.signs[at] == LINE_BREAK)) && next(at, spans),
            PatternPiece::Group(inner, None) => self.step(inner, at, spans, next),
            PatternPiece::Group(inner, Some(number)) => self.step(inner, at, spans, &mut |end, spans: &mut Spans| {
                let old = spans[*number];
                spans[*number] = Some((at, end));
                next(end, spans) || {
                    spans[*number] = old;
                    false
                }
            }),
            PatternPiece::Row(pieces) => self.row(pieces, at, spans, next),
            PatternPiece::Either(options) => options.iter().any(|option| self.step(option, at, spans, next)),
            PatternPiece::Repeat(inner, times) if one_sign(inner, LINE_BREAK).is_some() => self.run(inner, *times, at, spans, next),
            PatternPiece::Repeat(inner, times) => self.repeat(inner, *times, 0, at, spans, next),
            PatternPiece::Behind(inner, width) => at >= *width && self.step(inner, at - width, &mut spans.clone(), &mut |end, _| end == at) && next(at, spans),
        }
    }

    fn row(&self, pieces: &[PatternPiece], at: usize, spans: &mut Spans, next: Then) -> bool {
        let Some((first, rest)) = pieces.split_first() else { return next(at, spans) };
        self.step(first, at, spans, &mut |end, spans: &mut Spans| self.row(rest, end, spans, next))
    }

    fn run(&self, inner: &PatternPiece, times: Times, at: usize, spans: &mut Spans, next: Then) -> bool {
        let count = self.signs[at..].iter().take(times.most).take_while(|sign| one_sign(inner, **sign) == Some(true)).count();
        if count < times.least {
            return false;
        }
        if times.greedy {
            (times.least..=count).rev().any(|taken| next(at + taken, spans))
        } else {
            (times.least..=count).any(|taken| next(at + taken, spans))
        }
    }

    fn repeat(&self, inner: &PatternPiece, times: Times, done: usize, at: usize, spans: &mut Spans, next: Then) -> bool {
        let more = |spans: &mut Spans, next: Then| done < times.most && self.step(inner, at, spans, &mut |end, spans: &mut Spans| (end != at || done < times.least) && self.repeat(inner, times, done + 1, end, spans, next));
        if times.greedy {
            more(spans, next) || (done >= times.least && next(at, spans))
        } else {
            (done >= times.least && next(at, spans)) || more(spans, next)
        }
    }
}

pub struct TextPattern {
    root: PatternPiece,
    groups: usize,
}
because!(TextPattern, PythonPatterns, "a parsed pattern with the number of its groups that capture. find gives the leftmost match and its groups as search of Python, or the match of the whole text as fullmatch of Python; find_all gives what each match gives, as findall of Python (the whole match with no group, else each group); replace_all puts a text in the place of each match, as sub of Python with a plain text; split gives the pieces between the matches, as split of Python for a pattern with no group");

type Matched = (usize, usize, Spans);
because!(Matched, PythonPatterns, "one match: where it starts, where it ends, and its groups");

impl TextPattern {
    pub fn new(pattern: &str) -> Result<TextPattern, String> {
        let (breaks, body) = pattern.strip_prefix(DOT_ALL).map_or((false, pattern), |rest| (true, rest));
        let mut parser = Parser { signs: body.chars().collect(), at: 0, groups: 0, breaks };
        let root = parser.alternatives()?;
        if parser.at != parser.signs.len() {
            return Err(format!("the pattern {pattern} has a closing bracket with no opening one"));
        }
        Ok(TextPattern { root, groups: parser.groups })
    }

    fn first_match(&self, signs: &[char], from: usize, advance: bool, whole: bool) -> Option<Matched> {
        let run = Run { signs };
        let last = if whole { from } else { signs.len() };
        (from..=last).find_map(|start| {
            let mut found: Option<(usize, Spans)> = None;
            run.step(&self.root, start, &mut vec![None; self.groups + 1], &mut |end, spans: &mut Spans| {
                let fits = !(advance && start == from && end == start) && (!whole || end == signs.len());
                if fits {
                    found = Some((end, spans.clone()));
                }
                fits
            });
            found.map(|(end, spans)| (start, end, spans))
        })
    }

    fn all_matches(&self, signs: &[char]) -> Vec<Matched> {
        let mut out: Vec<Matched> = Vec::new();
        let (mut from, mut advance) = (0usize, false);
        while let Some(found) = self.first_match(signs, from, advance, false) {
            advance = found.1 == found.0;
            from = found.1;
            out.push(found);
        }
        out
    }

    pub fn find(&self, text: &str, whole: bool) -> Option<Vec<Option<String>>> {
        let signs: Vec<char> = text.chars().collect();
        let (start, end, mut spans) = self.first_match(&signs, 0, false, whole)?;
        spans[0] = Some((start, end));
        Some(spans.into_iter().map(|span| span.map(|(low, high)| signs[low..high].iter().collect())).collect())
    }

    pub fn find_all(&self, text: &str) -> Vec<Vec<String>> {
        let signs: Vec<char> = text.chars().collect();
        let piece = |span: Option<(usize, usize)>| span.map_or_else(String::new, |(low, high)| signs[low..high].iter().collect());
        self.all_matches(&signs)
            .into_iter()
            .map(|(start, end, spans)| if self.groups == 0 { vec![piece(Some((start, end)))] } else { spans.into_iter().skip(1).map(piece).collect() })
            .collect()
    }

    pub fn replace_all(&self, text: &str, with: &str) -> String {
        let signs: Vec<char> = text.chars().collect();
        let mut out = String::new();
        let mut kept = 0usize;
        for (start, end, _) in self.all_matches(&signs) {
            out.extend(&signs[kept..start]);
            out.push_str(with);
            kept = end;
        }
        out.extend(&signs[kept..]);
        out
    }

    pub fn split(&self, text: &str) -> Vec<String> {
        let signs: Vec<char> = text.chars().collect();
        let mut out: Vec<String> = Vec::new();
        let mut kept = 0usize;
        for (start, end, _) in self.all_matches(&signs) {
            out.push(signs[kept..start].iter().collect());
            kept = end;
        }
        out.push(signs[kept..].iter().collect());
        out
    }
}
