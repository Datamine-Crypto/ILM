use ilm_core::cases::Case;
use crate::sets::export::{Described, LabelLines};
use crate::sets::json::parse_json;
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{stripped, table_columns, text_value};
use crate::sets::textmatch::TextPattern;
use patterns::{because, source};
use std::collections::HashMap;
use std::path::Path;

pub struct FormalNotation;
source!(FormalNotation, "the formal forms that ProofWriter of Tafjord, Dalvi and Clark gives for its train theories, after the notation of RuleTaker: each fact and each rule of a theory with the text of its written unit, in the source copy of rlhf-and-friends (a JSON text of facts and one of rules, each with its text and its notation, so that one told sentence is one fact or one rule) and in the copy of hitachi-nlp of ParaRules, where people put the facts of a person and each rule in their own words (groups of facts, of rules, the unit of each and the texts of the units, so that a unit can be two told sentences); a theory is found by its letters and digits in small letters, the first file that holds it giving its units; a run of up to four told sentences that ends as soon as its letters are those of a unit is that unit; the statements of the theories that hold it give the fact that a statement says. These forms are labels of the lessons alone");

const FORM_FILES: &str = "forms";
because!(FORM_FILES, FormalNotation, "the role in the description of the train files whose facts and rules are JSON texts, one told sentence each, in the order in which they are searched");

const JOINED_FILES: &str = "joined";
because!(JOINED_FILES, FormalNotation, "the role in the description of the train files of the words of people, where a unit can hold several facts, searched after the other files");

const NOTATION_FACT: &str = r#"\("([^"]+)" "([^"]+)" "([^"]+)" "([^"]+)"\)"#;
because!(NOTATION_FACT, FormalNotation, "a fact in the notation of the sets: the subject, the relation, the object and the sign of whether it holds, each in quotes, all in brackets");

const RULE_ARROW: &str = " -> ";
because!(RULE_ARROW, FormalNotation, "the mark between the conditions and the head of a rule in the notation");

const ANY_WORDS: &[&str] = &["something", "someone"];
because!(ANY_WORDS, FormalNotation, "how the notation writes any thing in the place of a subject or an object");

const ANY_THING: &str = "?";
because!(ANY_THING, FormalNotation, "how a label line writes any thing");

const HOLDS_SIGN: &str = "+";
because!(HOLDS_SIGN, FormalNotation, "the sign of the notation for a fact that holds; the other sign says that it does not");

const MOST_JOINED: usize = 4;
because!(MOST_JOINED, FormalNotation, "the most told sentences that one written unit is searched over");

pub(crate) const LABEL_FIELD: &str = "\t";
because!(LABEL_FIELD, FormalNotation, "the mark between the fields of a label line");

pub(crate) const LABEL_UNIT: &str = "U";
because!(LABEL_UNIT, FormalNotation, "the mark of the line that opens a written unit with its first told sentence and the sentence after its last");

pub(crate) const LABEL_FACT: &str = "F";
because!(LABEL_FACT, FormalNotation, "the mark of the line of a fact of a unit");

const LABEL_CONDITION: &str = "C";
because!(LABEL_CONDITION, FormalNotation, "the mark of the line of a condition of a rule of a unit");

const LABEL_HEAD: &str = "H";
because!(LABEL_HEAD, FormalNotation, "the mark of the line of the head of a rule, which closes the rule");

const LABEL_STATEMENT: &str = "T";
because!(LABEL_STATEMENT, FormalNotation, "the mark of the line of the fact that the statement of a lesson says");

pub(crate) const LABEL_YES: &str = "yes";
because!(LABEL_YES, FormalNotation, "how a label line writes a fact that holds");

const LABEL_NO: &str = "no";
because!(LABEL_NO, FormalNotation, "how a label line writes a fact that does not hold");

const THEORY_COLUMN: &str = "theory";
because!(THEORY_COLUMN, FormalNotation, "the column whose letters find a lesson theory in the train files");

const UNIT_TEXT: &str = "text";
because!(UNIT_TEXT, FormalNotation, "the field of a fact or rule with the text of its written unit");

const UNIT_NOTATION: &str = "representation";
because!(UNIT_NOTATION, FormalNotation, "the field of a fact, rule or statement with its notation");

const ASKED_TEXT: &str = "question";
because!(ASKED_TEXT, FormalNotation, "the field whose letters find the statement of a lesson among the statements of a theory");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LabelFact {
    pub(crate) subject: String,
    pub(crate) relation: String,
    pub(crate) thing: String,
    pub(crate) yes: bool,
}
because!(LabelFact, FormalNotation, "a fact of the notation: subject, relation, object, and whether it holds");

#[derive(Clone, Debug)]
pub(crate) enum FormItem {
    Fact(LabelFact),
    Rule(Vec<LabelFact>, LabelFact),
}
because!(FormItem, FormalNotation, "one formal form of a unit: a fact, or a rule with its conditions and its head");

type WrittenUnit = (usize, usize, Vec<FormItem>);
because!(WrittenUnit, FormalNotation, "the told sentences from the first to the one before the end as one written unit, with its formal forms in the order of the set");

fn plain_letters(text: &str) -> String {
    text.to_lowercase().chars().filter(|sign| sign.is_ascii_lowercase() || sign.is_ascii_digit()).collect()
}
because!(plain_letters, FormalNotation, "the plain small letters and digits of a text in small letters, by which a theory, a unit and a statement are found");

fn label_fact(found: &[String]) -> Result<LabelFact, String> {
    let [subject, relation, thing, sign] = found else { return Err("a fact of no known form".to_string()) };
    let any = |word: &String| if ANY_WORDS.contains(&word.as_str()) { ANY_THING.to_string() } else { word.clone() };
    Ok(LabelFact { subject: any(subject), relation: relation.clone(), thing: any(thing), yes: sign == HOLDS_SIGN })
}
because!(label_fact, FormalNotation, "a fact from the four parts of its notation");

fn notation_fact(pattern: &TextPattern, text: &str) -> Result<LabelFact, String> {
    let groups = pattern.find(stripped(text), true).ok_or_else(|| format!("no fact in {text}"))?;
    label_fact(&groups.into_iter().skip(1).map(Option::unwrap_or_default).collect::<Vec<String>>())
}
because!(notation_fact, FormalNotation, "a fact from a notation that is one fact and nothing more");

fn form_item(pattern: &TextPattern, notation: &str) -> Result<FormItem, String> {
    let Some((left, right)) = notation.split_once(RULE_ARROW) else { return Ok(FormItem::Fact(notation_fact(pattern, notation)?)) };
    let conditions = pattern.find_all(left).iter().map(|found| label_fact(found)).collect::<Result<Vec<LabelFact>, String>>()?;
    let head = pattern.find_all(right).first().ok_or_else(|| format!("a rule with no head: {notation}")).and_then(|found| label_fact(found))?;
    Ok(FormItem::Rule(conditions, head))
}
because!(form_item, FormalNotation, "a fact, or a rule whose conditions are the facts before the arrow and whose head is the first fact after it");

fn key_number(key: &str) -> Result<usize, String> {
    key.trim_start_matches(|sign: char| !sign.is_ascii_digit()).parse().map_err(|_| format!("no number in the name {key}"))
}
because!(key_number, FormalNotation, "the number at the end of the name of a fact, a rule or a unit of the words of people, by which their order is given");

fn filled_text<'a>(value: Option<&'a Value>) -> Option<&'a str> {
    value.and_then(Value::text).filter(|text| !text.is_empty())
}
because!(filled_text, FormalNotation, "the text of a field when it holds a text that is not empty");

fn json_units(pattern: &TextPattern, triples: &Value, rules: &Value) -> Result<Vec<(String, Vec<FormItem>)>, String> {
    let mut out: Vec<(String, Vec<FormItem>)> = Vec::new();
    for group in [triples, rules] {
        let Value::Struct(fields) = parse_json(group.text().ok_or("forms that are no text")?)? else { return Err("forms that are no group".to_string()) };
        for (_, field) in fields {
            let text = field.get(UNIT_TEXT).and_then(Value::text).ok_or("a form with no text")?;
            let notation = field.get(UNIT_NOTATION).and_then(Value::text).ok_or("a form with no notation")?;
            out.push((text.to_string(), vec![form_item(pattern, notation)?]));
        }
    }
    Ok(out)
}
because!(json_units, FormalNotation, "the units of a theory of the source copy: each fact and then each rule with its text and its form");

fn joined_units(pattern: &TextPattern, groups: [&Value; JOINED_WIDTH]) -> Result<Vec<(String, Vec<FormItem>)>, String> {
    let [triples, rules, mappings, sentences] = groups;
    let mut units: Vec<(String, Vec<FormItem>)> = Vec::new();
    for group in [triples, rules] {
        let Value::Struct(fields) = group else { return Err("forms that are no group".to_string()) };
        let mut keyed: Vec<(usize, &str, &str)> = Vec::new();
        for (key, field) in fields {
            if let Some(notation) = filled_text(field.get(UNIT_NOTATION)) {
                keyed.push((key_number(key)?, key, notation));
            }
        }
        keyed.sort_by_key(|(number, _, _)| *number);
        for (_, key, notation) in keyed {
            let Some(unit) = filled_text(mappings.get(key)) else { continue };
            let item = form_item(pattern, notation)?;
            match units.iter_mut().find(|(held, _)| held == unit) {
                Some((_, items)) => items.push(item),
                None => units.push((unit.to_string(), vec![item])),
            }
        }
    }
    let mut numbered: Vec<(usize, String, Vec<FormItem>)> = Vec::new();
    for (unit, items) in units {
        numbered.push((key_number(&unit)?, unit, items));
    }
    numbered.sort_by_key(|(number, _, _)| *number);
    Ok(numbered.into_iter().filter_map(|(_, unit, items)| filled_text(sentences.get(&unit)).map(|text| (text.to_string(), items))).collect())
}
because!(joined_units, FormalNotation, "the units of a theory of the words of people: the facts and then the rules in the order of their numbers, each put into the unit that it maps to, the units in the order of their numbers with their texts");

const JOINED_WIDTH: usize = 4;
because!(JOINED_WIDTH, FormalNotation, "the groups of a theory of the words of people that its units are made of: facts, rules, the unit of each, the texts of the units");

struct FormBook {
    files: Vec<(String, bool)>,
    index: HashMap<String, Vec<(usize, usize)>>,
    columns: HashMap<usize, Vec<Vec<Value>>>,
    questions: HashMap<usize, Vec<Value>>,
    asked: HashMap<(usize, usize), HashMap<String, LabelFact>>,
    pattern: TextPattern,
}
because!(FormBook, FormalNotation, "the train files of the forms in the order in which they are searched, each with the mark of the words of people; the places of each theory by its letters; and the columns of the files as far as they were read");

impl FormBook {
    fn open(described: &Described, kind: &str) -> Result<FormBook, String> {
        let plain = described.files(kind, FORM_FILES).into_iter().map(|place| (place, false));
        let files: Vec<(String, bool)> = plain.chain(described.files(kind, JOINED_FILES).into_iter().map(|place| (place, true))).collect();
        let mut index: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        for (number, (place, _)) in files.iter().enumerate() {
            if !Path::new(place).exists() {
                continue;
            }
            let [theories] = table_columns(place, [THEORY_COLUMN])?;
            for (row, theory) in theories.iter().enumerate() {
                index.entry(plain_letters(&text_value(theory, place)?)).or_default().push((number, row));
            }
        }
        Ok(FormBook { files, index, columns: HashMap::new(), questions: HashMap::new(), asked: HashMap::new(), pattern: TextPattern::new(NOTATION_FACT)? })
    }

    fn places(&self, sentences: &[String]) -> Vec<(usize, usize)> {
        self.index.get(&plain_letters(&sentences.concat())).cloned().unwrap_or_default()
    }

    fn units_of(&mut self, number: usize, row: usize) -> Result<Vec<(String, Vec<FormItem>)>, String> {
        let (place, joined) = self.files[number].clone();
        if !self.columns.contains_key(&number) {
            let read: Vec<Vec<Value>> = if joined { table_columns(&place, ["triples", "rules", "mappings", "sentences"])?.into() } else { table_columns(&place, ["triples", "rules"])?.into() };
            self.columns.insert(number, read);
        }
        let values: Vec<&Value> = self.columns[&number].iter().map(|values| values.get(row)).collect::<Option<Vec<&Value>>>().ok_or_else(|| format!("{place}: no row {row}"))?;
        match values.as_slice() {
            [triples, rules, mappings, sentences] => joined_units(&self.pattern, [triples, rules, mappings, sentences]),
            [triples, rules] => json_units(&self.pattern, triples, rules),
            _ => Err(format!("{place}: other columns than asked")),
        }
    }

    fn forms(&mut self, sentences: &[String]) -> Result<Option<Vec<WrittenUnit>>, String> {
        let Some(&(number, row)) = self.places(sentences).first() else { return Ok(None) };
        let mut units: HashMap<String, Vec<FormItem>> = HashMap::new();
        for (text, items) in self.units_of(number, row)? {
            units.entry(plain_letters(&text)).or_insert(items);
        }
        let told: Vec<String> = sentences.iter().map(|sentence| plain_letters(sentence)).collect();
        let mut out: Vec<WrittenUnit> = Vec::new();
        let mut first = 0usize;
        while first < told.len() {
            let mut text = String::new();
            let mut end = first;
            while end < told.len() && end - first < MOST_JOINED {
                text.push_str(&told[end]);
                end += 1;
                if let Some(items) = units.get(&text) {
                    out.push((first, end, items.clone()));
                    first = end - 1;
                    break;
                }
            }
            first += 1;
        }
        Ok(Some(out))
    }

    fn statement(&mut self, sentences: &[String], question: &str) -> Result<Option<LabelFact>, String> {
        let wanted = plain_letters(question);
        for (number, row) in self.places(sentences) {
            if !self.asked.contains_key(&(number, row)) {
                let found = self.asked_of(number, row)?;
                self.asked.insert((number, row), found);
            }
            if let Some(fact) = self.asked[&(number, row)].get(&wanted) {
                return Ok(Some(fact.clone()));
            }
        }
        Ok(None)
    }

    fn asked_of(&mut self, number: usize, row: usize) -> Result<HashMap<String, LabelFact>, String> {
        let place = self.files[number].0.clone();
        if !self.questions.contains_key(&number) {
            let [questions] = table_columns(&place, ["questions"])?;
            self.questions.insert(number, questions);
        }
        let held = self.questions[&number].get(row).ok_or_else(|| format!("{place}: no row {row}"))?;
        let asked = match held {
            Value::Text(text) => parse_json(text)?,
            other => other.clone(),
        };
        let Value::Struct(statements) = asked else { return Err(format!("{place}: statements that are no group")) };
        let mut out: HashMap<String, LabelFact> = HashMap::new();
        for (_, statement) in &statements {
            let (Some(text), Some(notation)) = (filled_text(statement.get(ASKED_TEXT)), filled_text(statement.get(UNIT_NOTATION))) else { continue };
            out.insert(plain_letters(text), notation_fact(&self.pattern, notation)?);
        }
        Ok(out)
    }
}

pub(crate) fn form_pairs(described: &Described, kind: &str) -> Result<Vec<(String, FormItem)>, String> {
    let pattern = TextPattern::new(NOTATION_FACT)?;
    let mut out: Vec<(String, FormItem)> = Vec::new();
    for place in described.files(kind, FORM_FILES) {
        let [triples, rules, questions] = table_columns(&place, ["triples", "rules", "questions"])?;
        for (triple, rule) in triples.iter().zip(&rules) {
            for (text, mut items) in json_units(&pattern, triple, rule)? {
                if items.len() == 1 {
                    out.push((text, items.remove(0)));
                }
            }
        }
        for held in &questions {
            let asked = match held {
                Value::Text(text) => parse_json(text)?,
                other => other.clone(),
            };
            let Value::Struct(statements) = asked else { return Err(format!("{place}: statements that are no group")) };
            for (_, statement) in &statements {
                let (Some(text), Some(notation)) = (filled_text(statement.get(ASKED_TEXT)), filled_text(statement.get(UNIT_NOTATION))) else { continue };
                out.push((text.to_string(), FormItem::Fact(notation_fact(&pattern, notation)?)));
            }
        }
    }
    Ok(out)
}
because!(form_pairs, FormalNotation, "every told sentence of the train files of the forms with its one fact or rule, and every statement of those files with its fact, file by file: the pairs of a text and its form from which a reader of sets without forms can be counted");

fn label_line(mark: &str, fact: &LabelFact) -> String {
    [mark, &fact.subject, &fact.relation, &fact.thing, if fact.yes { LABEL_YES } else { LABEL_NO }].join(LABEL_FIELD)
}
because!(label_line, FormalNotation, "a label line of a fact: its mark, the subject, the relation, the object, and yes or no");

fn unit_lines(units: &[WrittenUnit]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for (first, end, items) in units {
        lines.push([LABEL_UNIT.to_string(), first.to_string(), end.to_string()].join(LABEL_FIELD));
        for item in items {
            match item {
                FormItem::Fact(fact) => lines.push(label_line(LABEL_FACT, fact)),
                FormItem::Rule(conditions, head) => {
                    lines.extend(conditions.iter().map(|fact| label_line(LABEL_CONDITION, fact)));
                    lines.push(label_line(LABEL_HEAD, head));
                }
            }
        }
    }
    lines
}
because!(unit_lines, FormalNotation, "the label lines of the units of a story: for each unit its line, then the line of each fact, or the lines of the conditions and of the head of each rule");

pub fn formal_labels(described: &Described, kind: &str, lessons: &[&Case]) -> Result<Vec<LabelLines>, String> {
    let mut book = FormBook::open(described, kind)?;
    let mut told: HashMap<Vec<String>, Option<Vec<WrittenUnit>>> = HashMap::new();
    let mut out: Vec<LabelLines> = Vec::new();
    for lesson in lessons {
        if !told.contains_key(&lesson.sentences) {
            let units = book.forms(&lesson.sentences)?;
            told.insert(lesson.sentences.clone(), units);
        }
        match told[&lesson.sentences].as_deref() {
            Some(units) if !units.is_empty() => {
                let fact = book.statement(&lesson.sentences, &lesson.question)?;
                out.push((unit_lines(units), fact.map(|fact| label_line(LABEL_STATEMENT, &fact))));
            }
            _ => out.push((Vec::new(), None)),
        }
    }
    Ok(out)
}
because!(formal_labels, FormalNotation, "the label lines of the lessons of a kind: for a lesson story that the train files give units for, the lines of its units, and the line of the fact of its statement when a theory that holds the story gives it");
