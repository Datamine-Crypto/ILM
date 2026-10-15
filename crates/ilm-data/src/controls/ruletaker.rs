use crate::controls::deduction::{FormalFact, FormalItem, FormalUnit, deduction_exact};
use crate::controls::file::ChangedCase;
use ilm_core::know::signs::is_space;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use crate::sets::export::Described;
use crate::sets::readers::formal::{FormItem, LabelFact, form_pairs};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct RuleTakerReading;
source!(RuleTakerReading, "lessons of deduction whose set gives no formal forms, read by two tables counted from the pairs of a sentence and its formal form that a later set of the same generator gives for its train theories. Sentences: each name, thing, property and verb of the formal form is found in the sentence in its order and becomes a place, and the rest of the sentence is its form, which stands for the formal form with the same places; a form with two formal forms is left out. Clauses: a rule of the shape if A and B then C is read clause by clause, each a subject (a name, a word for any thing, or no word: the subject of the clause before) and a predicate, both taken from the rules of the pairs, a subject or predicate with two meanings left out. A sentence is read by the first table, a rule with no form there by clauses; a lesson whose sentences and statement are all read gets its answer from the exact reasoner of the closed form, and its needed told sentences are taken out as in the control of deduction, at even steps over the lessons so that the changes come from all theories");

const RT_FORMS: &str = "data/sets/src_proofwriter.txt";
because!(RT_FORMS, RuleTakerReading, "the description of the set whose train files give the pairs of sentences and formal forms, when no other description is named");

const RT_ANY: &str = "?";
because!(RT_ANY, RuleTakerReading, "how the notation writes any thing in the place of a subject or an object");

const RT_IS: &str = "is";
because!(RT_IS, RuleTakerReading, "the relation of a property, whose object is the property and no thing");

const RT_NAME: &str = "<E>";
because!(RT_NAME, RuleTakerReading, "the mark of a place that holds a name or a thing");

const RT_PROPERTY: &str = "<P>";
because!(RT_PROPERTY, RuleTakerReading, "the mark of a place that holds a property");

const RT_VERB_S: &str = "<Vs>";
because!(RT_VERB_S, RuleTakerReading, "the mark of a place that holds a verb with its last s");

const RT_VERB: &str = "<V>";
because!(RT_VERB, RuleTakerReading, "the mark of a place that holds a verb without its last s");

const RT_MARK_START: char = '<';
because!(RT_MARK_START, RuleTakerReading, "the first sign of the mark of a place, which no word of a sentence has");

const RT_S: &str = "s";
because!(RT_S, RuleTakerReading, "the last letter of a verb after a single subject");

const RT_LOW_LINE: char = '_';
because!(RT_LOW_LINE, RuleTakerReading, "the sign that joins the words of a name into one token");

const RT_GAP: &str = " ";
because!(RT_GAP, RuleTakerReading, "the space between the words of a name");

const RT_IF: &str = "If";
because!(RT_IF, RuleTakerReading, "the first word of a rule of the shape that is read by clauses");

const RT_THEN: &str = "then";
because!(RT_THEN, RuleTakerReading, "the word before the head of a rule");

const RT_AND: &str = "and";
because!(RT_AND, RuleTakerReading, "the word between two conditions of a rule");

const RT_END: &str = ".";
because!(RT_END, RuleTakerReading, "the sign that closes a sentence of a rule, after its head");

const RT_LEAST_RULE: usize = 5;
because!(RT_LEAST_RULE, RuleTakerReading, "the fewest tokens of a rule that is read by clauses: if, a condition, then, a head and the end mark");

const RT_MOST: usize = 200;
because!(RT_MOST, RuleTakerReading, "the lessons that the even steps aim at, so that the changed lessons come from all theories");

const RT_TRUE: &str = "True";
because!(RT_TRUE, RuleTakerReading, "what the exact reasoner says for a statement that holds");

const RT_ENTAILED: &str = "entailment";
because!(RT_ENTAILED, RuleTakerReading, "the label of the set for a statement that holds");

const RT_NOT_ENTAILED: &str = "not entailment";
because!(RT_NOT_ENTAILED, RuleTakerReading, "the label of the set for a statement that does not hold");

const SORT_RT_FACT: &str = "a needed told fact taken out";
because!(SORT_RT_FACT, RuleTakerReading, "the sort of the change where a told fact that the answer needs is taken out");

const SORT_RT_RULE: &str = "a needed told rule taken out";
because!(SORT_RT_RULE, RuleTakerReading, "the sort of the change where a told rule that the answer needs is taken out");

const SORT_RT_FREE: &str = "a sentence that is not needed taken out";
because!(SORT_RT_FREE, RuleTakerReading, "the sort of the change where the first told sentence that the answer does not need is taken out");

type RtToken = (String, Option<String>);
because!(RtToken, RuleTakerReading, "a token of a sentence: a word, or the mark of a place with the word that fills it");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum FormSlot {
    Place(usize),
    Word(String),
}
because!(FormSlot, RuleTakerReading, "a part of a fact of a form: the number of a place of the sentence, or a word that the notation writes as it is");

type ShapeFact = (FormSlot, FormSlot, FormSlot, bool);
because!(ShapeFact, RuleTakerReading, "a fact of a form: its subject, relation and object as places or words, and whether it holds");

type SentenceShape = (bool, Vec<ShapeFact>);
because!(SentenceShape, RuleTakerReading, "the formal form of a form: whether it is a fact, and its facts with places, the head of a rule last");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Subject {
    Named,
    Any,
    Before,
}
because!(Subject, RuleTakerReading, "what the subject of a clause is: the name that it holds, any thing, or the subject of the clause before");

type Predicate = (bool, usize, bool);
because!(Predicate, RuleTakerReading, "the meaning of a predicate: whether its relation is the relation of a property, the place of its object among its places, and whether its fact holds");

#[derive(Default)]
struct Words {
    names: HashSet<String>,
    properties: HashSet<String>,
    verbs: HashSet<String>,
}
because!(Words, RuleTakerReading, "the names and things, the properties and the verbs of the formal forms of the pairs");

#[derive(Default)]
struct Tables {
    words: Words,
    forms: HashMap<Vec<String>, SentenceShape>,
    subjects: HashMap<Vec<String>, Subject>,
    predicates: HashMap<Vec<String>, Predicate>,
}
because!(Tables, RuleTakerReading, "the two tables: the forms of whole sentences with their formal forms, and the subjects and predicates of clauses with their meanings");

fn plain_fact(fact: &LabelFact) -> FormalFact {
    (fact.subject.clone(), fact.relation.clone(), fact.thing.clone(), fact.yes)
}
because!(plain_fact, RuleTakerReading, "a fact of the reader of the forms as the reasoner takes it");

fn item_facts(item: &FormalItem) -> Vec<&FormalFact> {
    match item {
        FormalItem::Told(fact) => vec![fact],
        FormalItem::Rule(conditions, head) => conditions.iter().chain([head]).collect(),
    }
}
because!(item_facts, RuleTakerReading, "the facts of a form: the fact itself, or the conditions of a rule and its head");

fn pair_words(pairs: &[(String, FormalItem)]) -> Words {
    let mut words = Words::default();
    for (_, item) in pairs {
        for (subject, relation, thing, _) in item_facts(item) {
            let named: Vec<&String> = if relation == RT_IS { vec![subject] } else { vec![subject, thing] };
            words.names.extend(named.into_iter().filter(|name| *name != RT_ANY).cloned());
            if relation == RT_IS {
                words.properties.insert(thing.clone());
            } else {
                words.verbs.insert(relation.clone());
            }
        }
    }
    words
}
because!(pair_words, RuleTakerReading, "the words of the formal forms of the pairs: the subjects and the things, the properties, and the verbs");

fn sentence_tokens(text: &str, words: &Words) -> Vec<RtToken> {
    let mut text = text.to_string();
    for name in words.names.iter().filter(|name| name.contains(RT_GAP)) {
        text = text.replace(name.as_str(), &name.replace(RT_GAP, &RT_LOW_LINE.to_string()));
    }
    let mut out: Vec<RtToken> = Vec::new();
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut Vec<RtToken>| {
        if run.is_empty() {
            return;
        }
        let word = std::mem::take(run);
        let plain = word.replace(RT_LOW_LINE, RT_GAP);
        let low = word.to_lowercase();
        let with_s = format!("{word}{RT_S}");
        let token = if words.names.contains(&plain) {
            (RT_NAME.to_string(), Some(plain))
        } else if words.properties.contains(&low) {
            (RT_PROPERTY.to_string(), Some(low))
        } else if words.verbs.contains(&word) {
            (RT_VERB_S.to_string(), Some(word))
        } else if words.verbs.contains(&with_s) {
            (RT_VERB.to_string(), Some(with_s))
        } else {
            (word, None)
        };
        out.push(token);
    };
    for sign in text.chars() {
        if sign.is_ascii_alphabetic() || sign == RT_LOW_LINE {
            run.push(sign);
            continue;
        }
        flush(&mut run, &mut out);
        if !is_space(sign) {
            run.push(sign);
            flush(&mut run, &mut out);
        }
    }
    flush(&mut run, &mut out);
    out
}
because!(sentence_tokens, RuleTakerReading, "the tokens of a sentence: runs of letters and low lines, and each other sign that is no space; a word of the formal forms is a place with its value, a name with spaces being one token");

fn token_parts(tokens: &[RtToken]) -> (Vec<String>, Vec<String>) {
    let mut marks: Vec<String> = Vec::with_capacity(tokens.len());
    let mut values: Vec<String> = Vec::new();
    for (mark, value) in tokens {
        marks.push(mark.clone());
        values.extend(value.iter().cloned());
    }
    (marks, values)
}
because!(token_parts, RuleTakerReading, "the marks and words of the tokens, which are the form of a sentence, and the words that fill its places in their order");

fn clauses_of(marks: &[String]) -> Option<Vec<(usize, usize)>> {
    let thens = marks.iter().filter(|mark| *mark == RT_THEN).count();
    if marks.len() < RT_LEAST_RULE || marks[0] != RT_IF || marks.last().map(String::as_str) != Some(RT_END) || thens != 1 {
        return None;
    }
    let then = marks.iter().position(|mark| mark == RT_THEN)?;
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut first = 1;
    for at in 1..=then {
        if marks[at] == RT_AND || marks[at] == RT_THEN {
            out.push((first, at));
            first = at + 1;
        }
    }
    out.push((then + 1, marks.len() - 1));
    Some(out)
}
because!(clauses_of, RuleTakerReading, "the places of the clauses of a rule of the shape if, conditions joined by and, then, a head and the end mark: the conditions first and the head last; nothing for a sentence of another shape");

#[derive(Default)]
struct ClauseCounts {
    subjects: HashMap<Vec<String>, HashSet<Subject>>,
    predicates: HashMap<Vec<String>, HashSet<Predicate>>,
}
because!(ClauseCounts, RuleTakerReading, "the meanings that each subject and each predicate of the clauses of the pairs was seen with. note adds the subject and the predicate of one clause");

impl ClauseCounts {
    fn note(&mut self, subject: Vec<String>, kind: Subject, predicate: Vec<String>, fact: &FormalFact) {
        self.subjects.entry(subject).or_default().insert(kind);
        let is_property = fact.1 == RT_IS;
        self.predicates.entry(predicate).or_default().insert((is_property, usize::from(!is_property), fact.3));
    }
}

fn learn_clauses(pairs: &[(String, FormalItem)], words: &Words) -> (HashMap<Vec<String>, Subject>, HashMap<Vec<String>, Predicate>) {
    let mut rows: Vec<(Vec<RtToken>, FormalFact, bool, Option<String>)> = Vec::new();
    for (text, item) in pairs {
        let FormalItem::Rule(conditions, head) = item else { continue };
        let read = sentence_tokens(text, words);
        let facts: Vec<&FormalFact> = conditions.iter().chain([head]).collect();
        let Some(parts) = clauses_of(&token_parts(&read).0).filter(|parts| parts.len() == facts.len()) else { continue };
        let mut made: Vec<(Vec<RtToken>, FormalFact, bool, Option<String>)> = Vec::new();
        let mut before: Option<String> = None;
        let mut whole = true;
        for ((first, end), fact) in parts.iter().zip(&facts) {
            let clause: Vec<RtToken> = read[*first..*end].to_vec();
            let values = token_parts(&clause).1;
            let slots: Vec<String> = [&fact.0, &fact.1, &fact.2].into_iter().filter(|word| *word != RT_ANY && *word != RT_IS).cloned().collect();
            if slots == values {
                made.push((clause, (*fact).clone(), fact.0 != RT_ANY, before.clone()));
            } else if fact.0 != RT_ANY && Some(&fact.0) == before.as_ref() && slots.get(1..) == Some(&values[..]) {
                made.push((clause, (*fact).clone(), false, before.clone()));
            } else {
                whole = false;
                break;
            }
            before = Some(fact.0.clone());
        }
        if whole {
            rows.extend(made);
        }
    }
    let mut found = ClauseCounts::default();
    for (clause, fact, named, _) in &rows {
        if *named {
            let marks = token_parts(clause).0;
            let Some(at) = clause.iter().position(|(_, value)| value.is_some()) else { continue };
            found.note(marks[..=at].to_vec(), Subject::Named, marks[at + 1..].to_vec(), fact);
        }
    }
    let inside: HashSet<String> = found.predicates.keys().flatten().filter(|mark| !mark.starts_with(RT_MARK_START)).cloned().collect();
    for (clause, fact, named, before) in &rows {
        if *named {
            continue;
        }
        let marks = token_parts(clause).0;
        let Some(first) = marks.first() else { continue };
        if first.starts_with(RT_MARK_START) || inside.contains(first) {
            if Some(&fact.0) == before.as_ref() {
                found.note(Vec::new(), Subject::Before, marks.clone(), fact);
            }
        } else if fact.0 == RT_ANY {
            found.note(marks[..1].to_vec(), Subject::Any, marks[1..].to_vec(), fact);
        }
    }
    let ClauseCounts { subjects, predicates } = found;
    let single = |found: &HashSet<_>| found.len() == 1;
    let subjects = subjects.into_iter().filter(|(_, found)| single(found)).filter_map(|(form, found)| found.into_iter().next().map(|kind| (form, kind))).collect();
    let predicates = predicates.into_iter().filter(|(_, found)| found.len() == 1).filter_map(|(form, found)| found.into_iter().next().map(|shape| (form, shape))).collect();
    (subjects, predicates)
}
because!(learn_clauses, RuleTakerReading, "the table of clauses from the rules of the pairs whose clauses name the words of their facts in order: first the subjects and predicates of the clauses that name their subject, then those of clauses with no subject word or with a word for any thing; a form with two meanings is left out");

fn rule_tables(pairs: &[(String, FormalItem)]) -> Tables {
    let words = pair_words(pairs);
    let mut seen: HashMap<Vec<String>, HashSet<SentenceShape>> = HashMap::new();
    for (text, item) in pairs {
        let read = sentence_tokens(text, &words);
        let places = token_parts(&read).1;
        let mut slots: Vec<String> = Vec::new();
        let mut shape: Vec<ShapeFact> = Vec::new();
        for (subject, relation, thing, yes) in item_facts(item) {
            let mut slot = |word: &String| -> FormSlot {
                if word == RT_ANY || word == RT_IS {
                    FormSlot::Word(word.clone())
                } else {
                    slots.push(word.clone());
                    FormSlot::Place(slots.len() - 1)
                }
            };
            let (first, second, third) = (slot(subject), slot(relation), slot(thing));
            shape.push((first, second, third, *yes));
        }
        if slots != places {
            continue;
        }
        seen.entry(token_parts(&read).0).or_default().insert((matches!(item, FormalItem::Told(_)), shape));
    }
    let forms = seen.into_iter().filter(|(_, shapes)| shapes.len() == 1).filter_map(|(form, shapes)| shapes.into_iter().next().map(|shape| (form, shape))).collect();
    let (subjects, predicates) = learn_clauses(pairs, &words);
    Tables { words, forms, subjects, predicates }
}
because!(rule_tables, RuleTakerReading, "the two tables from the pairs: the forms of the sentences that name the words of their formal form in its order, each with its one formal form, and the table of clauses");

fn by_clauses(found: &[RtToken], tables: &Tables) -> Option<FormalItem> {
    let parts = clauses_of(&token_parts(found).0)?;
    let mut facts: Vec<FormalFact> = Vec::new();
    let mut before: Option<String> = None;
    for (first, end) in parts {
        let clause = &found[first..end];
        let marks = token_parts(clause).0;
        let mut out: HashSet<FormalFact> = HashSet::new();
        for at in 0..=marks.len() {
            let (Some(kind), Some(&(is_property, object, yes))) = (tables.subjects.get(&marks[..at]), tables.predicates.get(&marks[at..])) else { continue };
            let (head, rest) = (token_parts(&clause[..at]).1, token_parts(&clause[at..]).1);
            let subject = match kind {
                Subject::Any => Some(RT_ANY.to_string()),
                Subject::Before => before.clone(),
                Subject::Named => (head.len() == 1).then(|| head[0].clone()),
            };
            let wanted = if is_property { 1 } else { RT_ROLE_PLACES };
            if let Some(subject) = subject
                && rest.len() == wanted
            {
                let relation = if is_property { RT_IS.to_string() } else { rest[0].clone() };
                out.insert((subject, relation, rest[object].clone(), yes));
            }
        }
        if out.len() != 1 {
            return None;
        }
        let fact = out.into_iter().next()?;
        before = Some(fact.0.clone());
        facts.push(fact);
    }
    let head = facts.pop()?;
    Some(FormalItem::Rule(facts, head))
}
because!(by_clauses, RuleTakerReading, "the formal form of a rule read clause by clause, when each clause has one reading by the table of clauses; nothing otherwise");

const RT_ROLE_PLACES: usize = 2;
because!(RT_ROLE_PLACES, RuleTakerReading, "the places of a predicate of a verb: the verb and its object");

fn read_sentence(text: &str, tables: &Tables) -> Option<FormalItem> {
    let found = sentence_tokens(text, &tables.words);
    let Some((is_fact, shape)) = tables.forms.get(&token_parts(&found).0) else { return by_clauses(&found, tables) };
    let places = token_parts(&found).1;
    let fill = |slot: &FormSlot| -> Option<String> {
        match slot {
            FormSlot::Place(at) => places.get(*at).cloned(),
            FormSlot::Word(word) => Some(word.clone()),
        }
    };
    let mut facts: Vec<FormalFact> = Vec::new();
    for (subject, relation, thing, yes) in shape {
        facts.push((fill(subject)?, fill(relation)?, fill(thing)?, *yes));
    }
    if *is_fact {
        return facts.into_iter().next().map(FormalItem::Told);
    }
    let head = facts.pop()?;
    Some(FormalItem::Rule(facts, head))
}
because!(read_sentence, RuleTakerReading, "the formal form of a sentence: by the table of whole sentences, or for a rule with no form there by clauses; nothing when no table reads it");

fn rt_answer(units: &[&FormalUnit], statement: &FormalFact) -> &'static str {
    if deduction_exact(units, statement, true) == RT_TRUE { RT_ENTAILED } else { RT_NOT_ENTAILED }
}
because!(rt_answer, RuleTakerReading, "the label of the set that the exact reasoner of the closed form gives");

fn rt_pairs(forms: &Path) -> Result<Vec<(String, FormalItem)>, String> {
    let described = Described::read(forms)?;
    let kind = described.kinds().into_iter().next().unwrap_or_default();
    let found = form_pairs(&described, &kind)?;
    Ok(found
        .into_iter()
        .map(|(text, item)| {
            let item = match item {
                FormItem::Fact(fact) => FormalItem::Told(plain_fact(&fact)),
                FormItem::Rule(conditions, head) => FormalItem::Rule(conditions.iter().map(plain_fact).collect(), plain_fact(&head)),
            };
            (text, item)
        })
        .collect())
}
because!(rt_pairs, RuleTakerReading, "the pairs of a text and its formal form from the train files that a description names for its forms");

pub fn ruletaker_changes(lessons: &[Case], whole: bool, forms: Option<&Path>) -> Result<Vec<ChangedCase>, String> {
    let tables = rule_tables(&rt_pairs(forms.unwrap_or(Path::new(RT_FORMS)))?);
    let mut known: HashMap<&str, Option<FormalItem>> = HashMap::new();
    for case in lessons {
        for text in case.sentences.iter().chain([&case.question]) {
            known.entry(text.as_str()).or_insert_with(|| read_sentence(text, &tables));
        }
    }
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let step = (last.len() / RT_MOST).max(1);
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last.into_iter().step_by(step) {
        let case = &lessons[place];
        let items: Option<Vec<FormalItem>> = case.sentences.iter().map(|text| known[text.as_str()].clone()).collect();
        let (Some(items), Some(FormalItem::Told(statement))) = (items, known[case.question.as_str()].clone()) else { continue };
        let units: Vec<FormalUnit> = items.into_iter().enumerate().map(|(at, item)| FormalUnit { first: at, end: at + 1, items: vec![item] }).collect();
        let all: Vec<&FormalUnit> = units.iter().collect();
        if rt_answer(&all, &statement) != case.answer {
            continue;
        }
        let without = |at: usize| -> Vec<String> { case.sentences.iter().enumerate().filter(|(other, _)| *other != at).map(|(_, sentence)| sentence.clone()).collect() };
        let mut free: Option<usize> = None;
        for (at, unit) in units.iter().enumerate() {
            let rest: Vec<&FormalUnit> = units.iter().enumerate().filter(|(other, _)| *other != at).map(|(_, other)| other).collect();
            let truth = rt_answer(&rest, &statement);
            if truth == case.answer {
                free = free.or(Some(at));
                continue;
            }
            let sort = if matches!(unit.items[0], FormalItem::Told(_)) { SORT_RT_FACT } else { SORT_RT_RULE };
            out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences: without(unit.first), question: case.question.clone(), truth: truth.to_string() });
        }
        if let Some(free) = free {
            out.push(ChangedCase { place: Some(place), sort: SORT_RT_FREE.to_string(), sentences: without(units[free].first), question: case.question.clone(), truth: case.answer.clone() });
        }
    }
    Ok(out)
}
because!(ruletaker_changes, RuleTakerReading, "the changed theories of the lessons that the model does not learn, or of all lessons for a model of all lessons, at even steps, whose sentences and statement the tables read and whose reasoner gives the label of the set");
