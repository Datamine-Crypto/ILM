use crate::tools::explain::chain::Facts;
use crate::tools::explain::text::fact_numbers;
use crate::tools::explain::words::{WordId, word_number, word_text};
use patterns::{because, source};
use crate::tools::explain::text::{LogicPiece, Pattern};
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct ToldClauses;
source!(ToldClauses, "a building block of the engine, classical logic over told sentences: a sentence told false is a fact of its own (beside 'not' in a condition, which reads what is absent), and a told formula is read in every direction, as clauses of its parts (Davis and Putnam): when all parts of a clause but one are false, the last one holds, which gives modus ponens, modus tollens, one side of an 'or' when the other is false, and the other side of a 'not both' when one is true (unit propagation); a statement is proved when its opposite, added to what is told, leads to a clash (refutation, Robinson). A thing said of any thing is read for each thing that the facts name. The texts of the library bring the words of a story to a few fixed forms that this block reads, as they bring them to the sentences of a count");

const FORMULA_START: &str = "it holds that";
because!(FORMULA_START, ToldClauses, "the words before a told formula, so that a fact with brackets of a name is never read as a formula");

const FALSE_START: &str = "it is false that";
because!(FALSE_START, ToldClauses, "the words before a sentence or a formula in brackets that is false");

const AND_WORD: &str = "and";
because!(AND_WORD, ToldClauses, "the word between two parts in brackets that both hold");

const OR_WORD: &str = "or";
because!(OR_WORD, ToldClauses, "the word between two parts in brackets of which at least one holds");

const IF_WORD: &str = "if";
because!(IF_WORD, ToldClauses, "the word before the part in brackets that a told rule needs");

const THEN_WORD: &str = "then";
because!(THEN_WORD, ToldClauses, "the word between the part that a told rule needs and the part that it gives");

const SOME_START: &str = "for some thing";
because!(SOME_START, ToldClauses, "the words before the word that stands for some thing in a formula, which a comma ends");

const FRESH_THING: &str = "something";
because!(FRESH_THING, ToldClauses, "the word that names a thing that a told formula says there is, with its number after it, as logic names a witness of 'there is' (Skolem)");

const MOST_TOLD_CLAUSES: usize = 65_536;
because!(MOST_TOLD_CLAUSES, ToldClauses, "the most clauses of all that a story tells: past them the block proves nothing, and the text answers as when nothing is proved, in place of a search that runs away on many things times many formulas");

const ANY_START: &str = "for any thing";
because!(ANY_START, ToldClauses, "the words before the word that stands for any thing in a formula, which a comma ends");

const COMMA: &str = ",";
because!(COMMA, ToldClauses, "the mark after the word that stands for any thing");

const FORMULA_OPEN: &str = "(";
because!(FORMULA_OPEN, ToldClauses, "the mark that opens a part of a formula");

const FORMULA_CLOSE: &str = ")";
because!(FORMULA_CLOSE, ToldClauses, "the mark that closes a part of a formula");

const THING: &str = "A is a thing";
because!(THING, ToldClauses, "the sentence that names a thing over which a formula said of any thing is read");

const MOST_CLAUSES: usize = 4096;
because!(MOST_CLAUSES, ToldClauses, "the most clauses that one formula may make: a formula whose clauses grow past this (many 'or' of many 'and') is left unread, as its unfolding would cost more than the block is worth on it");

const STEMS_FACT: &str = "The clause block reads the stems of words.";
because!(STEMS_FACT, ToldClauses, "the fact of a text that asks the block to compare sentences by the stems of their words, so that a text whose sources write one statement in several word forms ('blitzes', 'is a blitz') can ask for it, and every other text keeps its exact reading");

const FIRST_CUTS: &[(&str, &str)] = &[("nesses", ""), ("ness", ""), ("ings", ""), ("ing", "")];
because!(FIRST_CUTS, ToldClauses, "the endings that make a noun or a verb form of a stem, cut first (activeness, kissing), after which the endings of number and time may still be cut");

const LATER_CUTS: &[(&str, &str)] = &[("ies", "y"), ("sses", "ss"), ("shes", "sh"), ("ches", "ch"), ("xes", "x"), ("zes", "z"), ("ss", "ss"), ("s", ""), ("ed", "")];
because!(LATER_CUTS, ToldClauses, "the endings of number and time with what English spelling puts back (flies, kisses, blitzes): the first that fits is cut; a word that ends in a double s keeps it, so that 'kiss' is not cut to 'kis'");

const SILENT_END: &str = "e";
because!(SILENT_END, ToldClauses, "the letter at the end of a stem that English drops before an ending that starts with a vowel, cut from every stem so that 'active' and 'activ' (of activeness) and 'occur' meet");

const LEAST_STEM: usize = 3;
because!(LEAST_STEM, ToldClauses, "the fewest letters that a stem keeps after a cut: shorter words are left whole, so that 'bus' is not cut to 'bu' and 'is' is not cut to 'i'");

const OPPOSITE_START: &str = "non-";
because!(OPPOSITE_START, ToldClauses, "the start of a word that says the opposite of the rest (non-blitz, one word, as a hyphen between letters joins it), which the block reads as the sentence with the rest of the word, false");

fn cut_by(word: &str, cuts: &[(&str, &str)]) -> Option<String> {
    cuts.iter().find_map(|(ending, back)| word.strip_suffix(ending).map(|before| format!("{before}{back}")).filter(|stem| stem.chars().count() >= LEAST_STEM))
}
because!(cut_by, ToldClauses, "a word with the first of the given endings that it has cut and the letters put back, when the stem keeps enough letters");

fn stem(word: &str) -> String {
    let word = cut_by(word, FIRST_CUTS).unwrap_or_else(|| word.to_string());
    let word = cut_by(&word, LATER_CUTS).unwrap_or(word);
    match word.strip_suffix(SILENT_END) {
        Some(before) if before.chars().count() >= LEAST_STEM => before.to_string(),
        _ => word,
    }
}
because!(stem, ToldClauses, "the stem of a word, the same for its forms: the noun or verb ending cut, then the ending of number or time, then a silent e");

fn stemmed(words: &[WordId]) -> (bool, Vec<WordId>) {
    let mut holds = true;
    let mut out = Vec::new();
    for word in words.iter().map(|word| word_text(*word)) {
        let rest = match word.strip_prefix(OPPOSITE_START).filter(|rest| !rest.is_empty()) {
            Some(rest) => {
                holds = !holds;
                rest
            }
            None => word,
        };
        out.push(word_number(&stem(rest)));
    }
    (holds, out)
}
because!(stemmed, ToldClauses, "a sentence by the stems of its words, and whether it says what the stems say or the opposite: each word that starts with 'non-' turns it and is read without that start");

#[derive(Clone, Debug, PartialEq, Eq)]
enum Formula {
    Atom(Vec<WordId>),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Any(Vec<WordId>, Box<Formula>),
    Some(Vec<WordId>, Box<Formula>),
}
because!(Formula, ToldClauses, "a told formula: a sentence, its negation, two parts that both hold, at least one of two parts, a rule from one part to another, a formula said of any thing, or of some thing");

type Literal = (bool, usize);
because!(Literal, ToldClauses, "a sentence by its number among the sentences of the formulas, with whether it holds or is false");

type Clause = Vec<Literal>;
because!(Clause, ToldClauses, "a set of literals of which at least one holds");

struct Words {
    false_start: Vec<WordId>,
    any_start: Vec<WordId>,
    some_start: Vec<WordId>,
    and: WordId,
    or: WordId,
    when: WordId,
    then: WordId,
    comma: WordId,
    open: WordId,
    close: WordId,
}
because!(Words, ToldClauses, "the numbers of the fixed words of the formulas, found once for a reading; closing finds the bracket that closes the one at a place; group reads a part in brackets; read reads a formula: a part in brackets, 'it is false that' a part or a sentence with no bracket, 'if' a part 'then' a part, 'for any thing' a word and a comma and a formula, a part and 'and' or 'or' and the rest, or a sentence with no bracket");

impl Words {
    fn new() -> Words {
        let one = |text: &str| word_number(text);
        Words { false_start: fact_numbers(FALSE_START), any_start: fact_numbers(ANY_START), some_start: fact_numbers(SOME_START), and: one(AND_WORD), or: one(OR_WORD), when: one(IF_WORD), then: one(THEN_WORD), comma: one(COMMA), open: one(FORMULA_OPEN), close: one(FORMULA_CLOSE) }
    }

    fn closing(&self, words: &[WordId], from: usize) -> Option<usize> {
        let mut depth = usize::default();
        for (at, word) in words.iter().enumerate().skip(from) {
            if *word == self.open {
                depth += 1;
            } else if *word == self.close {
                depth = depth.checked_sub(1)?;
                if depth == usize::default() {
                    return Some(at);
                }
            }
        }
        None
    }

    fn group(&self, words: &[WordId]) -> Option<(Formula, usize)> {
        if words.first() != Some(&self.open) {
            return None;
        }
        let end = self.closing(words, usize::default())?;
        Some((self.read(&words[1..end])?, end + 1))
    }

    fn read(&self, words: &[WordId]) -> Option<Formula> {
        if let Some((inner, used)) = self.group(words)
            && used == words.len()
        {
            return Some(inner);
        }
        if let Some(rest) = words.strip_prefix(self.false_start.as_slice()) {
            if let Some((inner, used)) = self.group(rest)
                && used == rest.len()
            {
                return Some(Formula::Not(Box::new(inner)));
            }
            if !rest.is_empty() && !rest.contains(&self.open) {
                return Some(Formula::Not(Box::new(Formula::Atom(rest.to_vec()))));
            }
        }
        if let Some(rest) = words.strip_prefix(&[self.when][..])
            && let Some((first, used)) = self.group(rest)
            && rest.get(used) == Some(&self.then)
            && let Some((second, more)) = self.group(&rest[used + 1..])
            && used + 1 + more == rest.len()
        {
            return Some(Formula::Implies(Box::new(first), Box::new(second)));
        }
        for (start, every) in [(&self.any_start, true), (&self.some_start, false)] {
            if let Some(rest) = words.strip_prefix(start.as_slice())
                && let Some(comma) = rest.iter().position(|word| *word == self.comma)
                && comma > usize::default()
            {
                let (word, inner) = (rest[..comma].to_vec(), Box::new(self.read(&rest[comma + 1..])?));
                return Some(if every { Formula::Any(word, inner) } else { Formula::Some(word, inner) });
            }
        }
        if let Some((first, used)) = self.group(words)
            && let Some(joint) = words.get(used).copied().filter(|word| *word == self.and || *word == self.or)
        {
            let second = self.read(&words[used + 1..])?;
            return Some(if joint == self.and { Formula::And(Box::new(first), Box::new(second)) } else { Formula::Or(Box::new(first), Box::new(second)) });
        }
        (!words.is_empty() && !words.contains(&self.open)).then(|| Formula::Atom(words.to_vec()))
    }
}

fn with_thing(formula: &Formula, word: &[WordId], thing: &[WordId]) -> Formula {
    let swapped = |words: &[WordId]| -> Vec<WordId> {
        let mut out = Vec::new();
        let mut at = usize::default();
        while at < words.len() {
            if words[at..].starts_with(word) {
                out.extend_from_slice(thing);
                at += word.len();
            } else {
                out.push(words[at]);
                at += 1;
            }
        }
        out
    };
    match formula {
        Formula::Atom(words) => Formula::Atom(swapped(words)),
        Formula::Not(inner) => Formula::Not(Box::new(with_thing(inner, word, thing))),
        Formula::And(one, other) => Formula::And(Box::new(with_thing(one, word, thing)), Box::new(with_thing(other, word, thing))),
        Formula::Or(one, other) => Formula::Or(Box::new(with_thing(one, word, thing)), Box::new(with_thing(other, word, thing))),
        Formula::Implies(one, other) => Formula::Implies(Box::new(with_thing(one, word, thing)), Box::new(with_thing(other, word, thing))),
        Formula::Any(inner_word, inner) if inner_word.as_slice() == word => Formula::Any(inner_word.clone(), inner.clone()),
        Formula::Any(inner_word, inner) => Formula::Any(inner_word.clone(), Box::new(with_thing(inner, word, thing))),
        Formula::Some(inner_word, inner) if inner_word.as_slice() == word => Formula::Some(inner_word.clone(), inner.clone()),
        Formula::Some(inner_word, inner) => Formula::Some(inner_word.clone(), Box::new(with_thing(inner, word, thing))),
    }
}
because!(with_thing, ToldClauses, "a formula with a thing in place of the word that stands for any thing, inside every sentence, but not inside an inner formula said of any thing by the same word");

struct Sentences {
    numbers: HashMap<Vec<WordId>, usize>,
    fresh: usize,
    by_stems: bool,
}
because!(Sentences, ToldClauses, "the number of each sentence of the formulas, so that a clause holds numbers, by the exact words of the sentence or, when the text asks for it, by their stems; literal gives a sentence its number and whether it holds, turned by each word that starts with 'non-' when stems are read; clauses gives the clauses of a formula that holds or is false (negation pushed to the sentences, as in negation normal form, and 'or' spread over 'and'), a formula said of any thing that holds, or of some thing that is false, read for each thing, a formula said of some thing that holds, or of any thing that is false, read for a new witness (Skolem), so that a false 'for any thing' needs a thing that fails it and no named thing is taken to be it, and nothing when the clauses grow too many");

impl Sentences {
    fn literal(&mut self, words: &[WordId], holds: bool) -> Literal {
        let (says, key) = if self.by_stems { stemmed(words) } else { (true, words.to_vec()) };
        let next = self.numbers.len();
        (holds == says, *self.numbers.entry(key).or_insert(next))
    }

    fn clauses(&mut self, formula: &Formula, holds: bool, things: &[Vec<WordId>]) -> Option<Vec<Clause>> {
        let clauses = match (formula, holds) {
            (Formula::Atom(words), _) => vec![vec![self.literal(words, holds)]],
            (Formula::Not(inner), _) => self.clauses(inner, !holds, things)?,
            (Formula::And(one, other), true) | (Formula::Or(one, other), false) => {
                let mut both = self.clauses(one, holds, things)?;
                both.extend(self.clauses(other, holds, things)?);
                both
            }
            (Formula::Or(one, other), true) | (Formula::And(one, other), false) => {
                let first = self.clauses(one, holds, things)?;
                either(first, self.clauses(other, holds, things)?)?
            }
            (Formula::Implies(one, other), true) => {
                let first = self.clauses(one, false, things)?;
                either(first, self.clauses(other, true, things)?)?
            }
            (Formula::Implies(one, other), false) => {
                let mut both = self.clauses(one, true, things)?;
                both.extend(self.clauses(other, false, things)?);
                both
            }
            (Formula::Some(word, inner), true) | (Formula::Any(word, inner), false) => {
                self.fresh += 1;
                let witness = vec![word_number(FRESH_THING), word_number(&self.fresh.to_string())];
                self.clauses(&with_thing(inner, word, &witness), holds, things)?
            }
            (Formula::Some(word, inner), false) => {
                let mut every = Vec::new();
                for thing in things {
                    every.extend(self.clauses(&with_thing(inner, word, thing), false, things)?);
                }
                every
            }
            (Formula::Any(word, inner), true) => {
                let mut every = Vec::new();
                for thing in things {
                    every.extend(self.clauses(&with_thing(inner, word, thing), true, things)?);
                }
                every
            }
        };
        (clauses.len() <= MOST_CLAUSES).then_some(clauses)
    }
}

fn either(one: Vec<Clause>, other: Vec<Clause>) -> Option<Vec<Clause>> {
    if one.len() * other.len() > MOST_CLAUSES {
        return None;
    }
    Some(one.iter().flat_map(|first| other.iter().map(move |second| first.iter().chain(second.iter()).copied().collect())).collect())
}
because!(either, ToldClauses, "the clauses of 'at least one of two', each clause of one side with each of the other, and nothing when they would grow too many");

fn clash(clauses: &[Clause], count: usize) -> Option<Vec<usize>> {
    let mut value: Vec<Option<bool>> = vec![None; count];
    let mut reason: Vec<Option<usize>> = vec![None; count];
    loop {
        let mut moved = false;
        for (at, clause) in clauses.iter().enumerate() {
            if clause.iter().any(|(holds, sentence)| value[*sentence] == Some(*holds)) {
                continue;
            }
            let open: Vec<&Literal> = clause.iter().filter(|(_, sentence)| value[*sentence].is_none()).collect();
            match open.as_slice() {
                [] => return Some(cone(clauses, &reason, at)),
                [(holds, sentence)] => {
                    value[*sentence] = Some(*holds);
                    reason[*sentence] = Some(at);
                    moved = true;
                }
                _ => {}
            }
        }
        if !moved {
            return None;
        }
    }
}
because!(clash, ToldClauses, "whether unit propagation reaches a clash, and the clauses that led to it: a clause whose parts are all false; a clause with one open part and no true part makes that part hold, round after round, until nothing moves, each part kept with the clause that made it hold");

fn cone(clauses: &[Clause], reason: &[Option<usize>], last: usize) -> Vec<usize> {
    let mut seen = vec![last];
    let mut at = usize::default();
    while let Some(clause) = seen.get(at).copied() {
        for (_, sentence) in &clauses[clause] {
            if let Some(made) = reason[*sentence]
                && !seen.contains(&made)
            {
                seen.push(made);
            }
        }
        at += 1;
    }
    seen.sort_unstable();
    seen
}
because!(cone, ToldClauses, "the clauses that a clash rests on: the clause that clashed, and for each of its parts the clause that made it hold, back to the clauses that held at once (a proof by refutation, as the implication graph of a solver keeps it)");

const THINGS_FACT: &str = "The clause block takes things from the sentences.";
because!(THINGS_FACT, ToldClauses, "the fact of a text that asks the block to read a formula said of any thing also for the words that fill its open word in the other sentences, so that a text whose stories name their things only inside formulas can ask for it, and every other text keeps its own things");

const MOST_MET_THINGS: usize = 256;
because!(MOST_MET_THINGS, ToldClauses, "the most things that the meeting sentences of one story may give: room for every name of a long story, and a bound on the things that each formula said of any thing is read for, so that a large text cannot grow the clauses without limit before the bound of clauses stops them");

fn sentence_parts<'a>(formula: &'a Formula, open: &mut Vec<&'a [WordId]>, plain: &mut Vec<&'a [WordId]>, with_open: &mut Vec<(&'a [WordId], &'a [WordId])>) {
    match formula {
        Formula::Atom(words) => {
            let inside: Vec<&[WordId]> = open.iter().copied().filter(|word| words.windows(word.len()).any(|part| part == *word)).collect();
            match inside.as_slice() {
                [] => plain.push(words),
                [word] => with_open.push((word, words)),
                _ => {}
            }
        }
        Formula::Not(inner) => sentence_parts(inner, open, plain, with_open),
        Formula::And(one, other) | Formula::Or(one, other) | Formula::Implies(one, other) => {
            sentence_parts(one, open, plain, with_open);
            sentence_parts(other, open, plain, with_open);
        }
        Formula::Any(word, inner) | Formula::Some(word, inner) => {
            open.push(word);
            sentence_parts(inner, open, plain, with_open);
            open.pop();
        }
    }
}
because!(sentence_parts, ToldClauses, "the sentences of a formula: those with no open word of a formula around them, and those with one open word, with that word; a sentence with two open words is left out, since one sentence alone does not fill both");

fn filled_thing(word: &[WordId], pattern: &[WordId], sentence: &[WordId]) -> Option<(usize, usize)> {
    let starts: Vec<usize> = (0..pattern.len()).filter(|at| pattern[*at..].starts_with(word)).collect();
    let first = *starts.first()?;
    (1..=sentence.len().saturating_sub(pattern.len() - word.len() * starts.len())).find_map(|length| {
        let mut made = Vec::new();
        let mut at = usize::default();
        while at < pattern.len() {
            if starts.contains(&at) {
                made.extend_from_slice(&sentence[first..first + length]);
                at += word.len();
            } else {
                made.push(pattern[at]);
                at += 1;
            }
        }
        (made == sentence).then_some((first, first + length))
    })
}
because!(filled_thing, ToldClauses, "the place in a sentence of the words that fill the open word of a sentence of a formula, when the two sentences are the same with those words in the place of each open word, and nothing when they do not meet");

fn said_runs<'a>(word: &[WordId], pattern: &'a [WordId]) -> Vec<&'a [WordId]> {
    let mut runs = Vec::new();
    let mut from = usize::default();
    let mut at = usize::default();
    while at < pattern.len() {
        if pattern[at..].starts_with(word) {
            runs.push(&pattern[from..at]);
            at += word.len();
            from = at;
        } else {
            at += 1;
        }
    }
    runs.push(&pattern[from..]);
    runs.into_iter().filter(|run| !run.is_empty()).collect()
}
because!(said_runs, ToldClauses, "the runs of words that a sentence with an open word says around that word: what the sentence says of a thing, which a name never holds");

fn met_things(formulas: &[&Formula], by_stems: bool) -> Vec<Vec<WordId>> {
    let (mut plain, mut with_open) = (Vec::new(), Vec::new());
    for formula in formulas {
        sentence_parts(formula, &mut Vec::new(), &mut plain, &mut with_open);
    }
    let key = |words: &[WordId]| if by_stems { stemmed(words).1 } else { words.to_vec() };
    let plain: Vec<(&[WordId], Vec<WordId>)> = plain.into_iter().map(|words| (words, key(words))).collect();
    let with_open: Vec<(Vec<WordId>, Vec<WordId>)> = with_open.into_iter().map(|(word, pattern)| (key(word), key(pattern))).collect();
    let said: Vec<&[WordId]> = with_open.iter().flat_map(|(word, pattern)| said_runs(word, pattern)).collect();
    let mut things: Vec<Vec<WordId>> = Vec::new();
    for (word, pattern) in &with_open {
        for (words, sentence) in &plain {
            if let Some((start, end)) = filled_thing(word, pattern, sentence)
                && !said.iter().any(|run| sentence[start..end].windows(run.len()).any(|part| part == *run))
                && !things.iter().any(|thing| thing.as_slice() == &words[start..end])
            {
                if things.len() >= MOST_MET_THINGS {
                    return things;
                }
                things.push(words[start..end].to_vec());
            }
        }
    }
    things
}
because!(met_things, ToldClauses, "the things that the sentences of the formulas name where a sentence with an open word meets a sentence with none (Herbrand instantiation kept to the terms that can meet): the words of the sentence with none in the place of the open word, compared by stems when the text asks for them; never words that hold a run that a sentence with an open word says of a thing: a sentence is a string of words, so 'the cat red a' with 'dog' and 'the cat' with 'red a dog' read the same, and a filling that holds what is said of a thing ('red' of 'x red') would make two readings of one sentence; no more than the bound of such things, the first found kept");

fn told_clauses(facts: &Facts, words: &Words, sentences: &mut Sentences, asked: &Formula, holds: bool) -> Option<(Vec<Clause>, Vec<Option<usize>>)> {
    let formula_start = fact_numbers(FORMULA_START);
    let thing_start = fact_numbers(THING);
    let every: Vec<&[WordId]> = (0..facts.added_to(&Facts::default())).map(|place| facts.fact(place)).collect();
    let mut things: Vec<Vec<WordId>> = every.iter().filter_map(|fact| fact.strip_suffix(&thing_start[1..]).filter(|thing| !thing.is_empty()).map(<[WordId]>::to_vec)).collect();
    let (formulas, said): (Vec<Formula>, Vec<usize>) = every
        .iter()
        .enumerate()
        .filter(|(_, fact)| !fact.contains(&words.open) || fact.starts_with(&formula_start) || fact.starts_with(&words.false_start))
        .filter_map(|(place, fact)| match fact.strip_prefix(formula_start.as_slice()) {
            Some(told) => words.read(told),
            None => words.read(fact),
        }.map(|formula| (formula, place)))
        .unzip();
    if facts.holds(&fact_numbers(THINGS_FACT)) {
        let read: Vec<&Formula> = formulas.iter().chain([asked]).collect();
        for thing in met_things(&read, sentences.by_stems) {
            if !things.contains(&thing) {
                things.push(thing);
            }
        }
    }
    let grounded = |sentences: &mut Sentences, things: &[Vec<WordId>]| -> Option<(Vec<Clause>, Vec<Option<usize>>)> {
        let (mut clauses, from) = ground_all(&formulas, sentences, things)?;
        let mut sources: Vec<Option<usize>> = from.into_iter().map(|at| Some(said[at])).collect();
        let added = sentences.clauses(asked, holds, things)?;
        sources.extend(added.iter().map(|_| None));
        clauses.extend(added);
        Some((clauses, sources))
    };
    let mut clauses = grounded(sentences, &things)?;
    if sentences.fresh > usize::default() {
        things.extend((1..=sentences.fresh).map(|number| vec![word_number(FRESH_THING), word_number(&number.to_string())]));
        sentences.fresh = usize::default();
        clauses = grounded(sentences, &things)?;
    }
    Some(clauses)
}
because!(told_clauses, ToldClauses, "the clauses of everything that the facts tell, each with the place of the fact that it came from (none for the asked statement): a plain sentence holds, 'it is false that' a sentence in brackets is false, and a fact 'it holds that' a formula gives the clauses of the formula, said of any thing for each thing that the facts name (and, when the text asks for it, each thing where the sentences meet, the asked statement among them) and for each witness that a told 'for some thing' or a false 'for any thing' names, in the told formulas or in the asked statement taken to hold or to be false (all read again once the witnesses are known), with the clauses of the asked statement; nothing when all of them grow past the bound of a story or the asked statement makes too many; a fact with brackets of a name that is no formula is passed over");

fn ground_all(formulas: &[Formula], sentences: &mut Sentences, things: &[Vec<WordId>]) -> Option<(Vec<Clause>, Vec<usize>)> {
    let mut clauses = Vec::new();
    let mut from = Vec::new();
    for (at, formula) in formulas.iter().enumerate() {
        if let Some(made) = sentences.clauses(formula, true, things) {
            from.extend(made.iter().map(|_| at));
            clauses.extend(made);
        }
        if clauses.len() > MOST_TOLD_CLAUSES {
            return None;
        }
    }
    Some((clauses, from))
}
because!(ground_all, ToldClauses, "the clauses of the told formulas over the things, each with the place of its formula, a formula whose clauses grow too many passed over, and nothing when all of them grow past the bound of a story");

pub(super) fn refuted(statement: &[WordId], facts: &Facts, holds: bool) -> bool {
    refutation(statement, facts, holds).is_some()
}

pub(super) fn refutation(statement: &[WordId], facts: &Facts, holds: bool) -> Option<Vec<Option<Vec<WordId>>>> {
    let words = Words::new();
    let asked = words.read(statement)?;
    let mut sentences = Sentences { numbers: HashMap::new(), fresh: usize::default(), by_stems: facts.holds(&fact_numbers(STEMS_FACT)) };
    let (clauses, sources) = told_clauses(facts, &words, &mut sentences, &asked, holds)?;
    let used = clash(&clauses, sentences.numbers.len())?;
    let mut out: Vec<Option<Vec<WordId>>> = Vec::new();
    for at in used {
        let said = sources[at].map(|place| facts.fact(place).to_vec());
        if !out.contains(&said) {
            out.push(said);
        }
    }
    Some(out)
}
because!(refuted, ToldClauses, "whether a statement, a sentence or a formula, taken to hold or to be false and added to the clauses of what the facts tell, leads unit propagation to a clash; refutation gives the told facts whose clauses the clash rests on (nothing for the asked statement), in the order of their clauses; with sentences compared by their stems when the facts hold the fact that asks for it: the statement is proved when its opposite clashes, and proved false when it clashes itself");

pub fn opposite(fact: &[WordId]) -> Vec<WordId> {
    let words = Words::new();
    if let Some(inner) = fact.strip_prefix(words.false_start.as_slice()).and_then(|rest| rest.strip_prefix(&[words.open][..])).and_then(|rest| rest.strip_suffix(&[words.close][..])) {
        return inner.to_vec();
    }
    words.false_start.iter().copied().chain([words.open]).chain(fact.iter().copied()).chain([words.close]).collect()
}
because!(opposite, ToldClauses, "the opposite of a sentence: 'it is false that (S)' for S, and S for 'it is false that (S)'");

static ANY_SENTENCE: OnceLock<Pattern> = OnceLock::new();
because!(ANY_SENTENCE, ToldClauses, "a sentence with one open place, which every fact may fit, for the strata of a rule that waits for every fact before it proves");

pub fn any_sentence() -> &'static Pattern {
    ANY_SENTENCE.get_or_init(|| vec![LogicPiece::Open(usize::default())])
}
because!(any_sentence, ToldClauses, "the sentence that a proof waits for: any fact, since a proof reads all that is told");
