use crate::know::titles::names;
use patterns::{because, source};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct TextStore;
source!(TextStore, "the text of an encyclopedia as files of lines, one article on a line: an object of the JSON notation (the data interchange standard of Ecma International) whose members with the names title and text are strings; the store is read as it stands, a model gets no list of titles and no mark that says which article holds an asked subject");

pub struct FactFile;
source!(FactFile, "the facts of a probe as plain lines: the part of the fact (a lesson or an asked fact), the relation, the subject, the object and the sentence form of the relation, with tabs between them, in the order of the set");

const QUOTE: char = '"';
because!(QUOTE, TextStore, "the sign that opens and closes a string of the notation");

const ESCAPE: char = '\\';
because!(ESCAPE, TextStore, "the sign that starts an escaped sign inside a string of the notation");

const CODE_ESCAPE: char = 'u';
because!(CODE_ESCAPE, TextStore, "the letter after the escape sign that starts a code unit written as digits");

const CODE_DIGITS: usize = 4;
because!(CODE_DIGITS, TextStore, "the digits of a code unit in an escape of the notation");

const CODE_BASE: u32 = 16;
because!(CODE_BASE, TextStore, "the base of the digits of a code unit in an escape of the notation");

const HIGH_FIRST: u32 = 0xD800;
because!(HIGH_FIRST, TextStore, "the first code unit that is the first half of a sign outside the first plane");

const LOW_FIRST: u32 = 0xDC00;
because!(LOW_FIRST, TextStore, "the first code unit that is the second half of a sign outside the first plane");

const LOW_LAST: u32 = 0xDFFF;
because!(LOW_LAST, TextStore, "the last code unit that is the second half of a sign outside the first plane");

const SURROGATE_BITS: u32 = 10;
because!(SURROGATE_BITS, TextStore, "the bits that each half of a sign outside the first plane carries");

const PLANE: u32 = 0x10000;
because!(PLANE, TextStore, "the first code point outside the first plane: the two halves count from it");

const ESCAPED: &[(char, char)] = &[('n', '\n'), ('t', '\t'), ('r', '\r'), ('b', '\u{8}'), ('f', '\u{c}')];
because!(ESCAPED, TextStore, "the letters after the escape sign that stand for a control sign, each with its sign; every other sign after the escape sign stands for itself");

const TITLE_NAME: &str = "title";
because!(TITLE_NAME, TextStore, "the name of the member that holds the title of an article");

const TEXT_NAME: &str = "text";
because!(TEXT_NAME, TextStore, "the name of the member that holds the text of an article");

pub const SUBJECTS_FILE: &str = "subjects.jsonl";
because!(SUBJECTS_FILE, TextStore, "the file of the store with the articles that answer to the name of a subject of the probe");

pub const QUESTIONS_FILE: &str = "questions.jsonl";
because!(QUESTIONS_FILE, TextStore, "the file of the store with the articles that answer to the name of a subject of the questions that people wrote, for the one command");

pub const OTHERS_FILE: &str = "others.jsonl";
because!(OTHERS_FILE, TextStore, "the file of the store with other articles, for the run at the full size of the text");

const LESSON_PART: &str = "L";
because!(LESSON_PART, FactFile, "the mark of a lesson fact in a file of facts");

const FACT_BETWEEN: char = '\t';
because!(FACT_BETWEEN, FactFile, "the mark between the parts of a line of a file of facts or of a table of word classes");

fn code_unit(signs: &mut std::str::Chars) -> Option<u32> {
    let digits: String = signs.take(CODE_DIGITS).collect();
    u32::from_str_radix(&digits, CODE_BASE).ok()
}
because!(code_unit, TextStore, "one code unit of an escape, from its digits");

fn string(signs: &mut std::str::Chars) -> Option<String> {
    let mut text = String::new();
    loop {
        let sign = signs.next()?;
        if sign == QUOTE {
            return Some(text);
        }
        if sign != ESCAPE {
            text.push(sign);
            continue;
        }
        let after = signs.next()?;
        if after != CODE_ESCAPE {
            text.push(ESCAPED.iter().find(|(letter, _)| *letter == after).map_or(after, |(_, control)| *control));
            continue;
        }
        let unit = code_unit(signs)?;
        if !(HIGH_FIRST..LOW_FIRST).contains(&unit) {
            text.push(char::from_u32(unit).unwrap_or(char::REPLACEMENT_CHARACTER));
            continue;
        }
        let mut ahead = signs.clone();
        let low = if ahead.next() == Some(ESCAPE) && ahead.next() == Some(CODE_ESCAPE) { code_unit(&mut ahead).filter(|low| (LOW_FIRST..=LOW_LAST).contains(low)) } else { None };
        match low {
            Some(low) => {
                *signs = ahead;
                text.push(char::from_u32(PLANE + ((unit - HIGH_FIRST) << SURROGATE_BITS) + (low - LOW_FIRST)).unwrap_or(char::REPLACEMENT_CHARACTER));
            }
            None => text.push(char::REPLACEMENT_CHARACTER),
        }
    }
}
because!(string, TextStore, "one string of the notation, read from after its opening sign to its closing sign: escaped signs are put back, and two code units that are the halves of one sign are joined");

pub fn string_members(line: &str) -> Vec<(String, String)> {
    let mut members: Vec<(String, String)> = Vec::new();
    let mut signs = line.chars();
    let mut name: Option<String> = None;
    while let Some(sign) = signs.next() {
        if sign != QUOTE {
            continue;
        }
        let Some(text) = string(&mut signs) else { break };
        match name.take() {
            None => name = Some(text),
            Some(held) => members.push((held, text)),
        }
    }
    members
}
because!(string_members, TextStore, "the members of a flat object whose values are strings, as pairs of a name and a value in their order; a line of the store holds no other member");

pub fn article_of(line: &str) -> Option<(String, String)> {
    let mut title: Option<String> = None;
    let mut text: Option<String> = None;
    for (name, value) in string_members(line) {
        if name == TITLE_NAME {
            title = Some(value);
        } else if name == TEXT_NAME {
            text = Some(value);
        }
    }
    Some((title?, text?))
}
because!(article_of, TextStore, "the title and the text of the article on one line of the store");

pub fn articles(path: &Path) -> std::io::Result<impl Iterator<Item = String> + use<>> {
    Ok(BufReader::new(std::fs::File::open(path)?).lines().map_while(Result::ok))
}
because!(articles, TextStore, "the lines of a file of the store, one article on each, read one after the other so that the whole file is never in memory");

#[derive(Default)]
pub struct Texts {
    pub titles: Vec<String>,
    pub bodies: Vec<String>,
    pub place: HashMap<String, usize>,
    pub by_name: HashMap<String, Vec<usize>>,
}
because!(Texts, TextStore, "the articles of some files of the store in the order of the files (an article of a later file with a title that came before takes the place of the earlier one), by their title and by each name of a title. title_of gives the article of a subject: the one with its title, or the only one that answers to its name; a name with several articles and no exact title has none");

impl Texts {
    pub fn read(store: &Path) -> std::io::Result<Texts> {
        Texts::read_files(store, &[SUBJECTS_FILE])
    }

    pub fn read_files(store: &Path, files: &[&str]) -> std::io::Result<Texts> {
        let mut texts = Texts::default();
        for line in files.iter().map(|file| articles(&store.join(file))).collect::<std::io::Result<Vec<_>>>()?.into_iter().flatten() {
            let Some((title, body)) = article_of(&line) else { continue };
            match texts.place.get(&title) {
                Some(place) => texts.bodies[*place] = body,
                None => {
                    texts.place.insert(title.clone(), texts.titles.len());
                    texts.titles.push(title);
                    texts.bodies.push(body);
                }
            }
        }
        for (place, title) in texts.titles.iter().enumerate() {
            for name in names(title) {
                texts.by_name.entry(name).or_default().push(place);
            }
        }
        Ok(texts)
    }

    pub fn get(&self, title: &str) -> Option<&str> {
        self.place.get(title).map(|place| self.bodies[*place].as_str())
    }

    pub fn title_of(&self, subject: &str) -> Option<&str> {
        if let Some(place) = self.place.get(subject) {
            return Some(self.titles[*place].as_str());
        }
        match self.by_name.get(&subject.to_lowercase()).map(Vec::as_slice) {
            Some([only]) => Some(self.titles[*only].as_str()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fact {
    pub lesson: bool,
    pub relation: String,
    pub subject: String,
    pub object: String,
    pub template: String,
}
because!(Fact, FactFile, "one fact of a probe: whether it is a lesson fact, its relation, its subject, its object, and the sentence form of its relation");

pub fn read_facts(path: &Path) -> std::io::Result<Vec<Fact>> {
    let mut facts: Vec<Fact> = Vec::new();
    for line in BufReader::new(std::fs::File::open(path)?).lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(FACT_BETWEEN).collect();
        if let [part, relation, subject, object, template] = parts.as_slice() {
            facts.push(Fact { lesson: *part == LESSON_PART, relation: relation.to_string(), subject: subject.to_string(), object: object.to_string(), template: template.to_string() });
        }
    }
    Ok(facts)
}
because!(read_facts, FactFile, "the facts of a file in their order; a line that is no fact is passed over");
