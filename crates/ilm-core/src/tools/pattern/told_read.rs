use crate::base::words::words as cut_words;
use crate::tools::pattern::classes::{ABSENT, KIND_MARK};
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::{IN_LESSONS, JOINT, PatternWords};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::WordTexts;
use crate::tools::pattern::story::Desk;
use crate::tools::pattern::told::{Implication, Known, PENDING, Shape, Statement, TOLD_ROUNDS, TableRow, ToldRules, View, as_written, is_word};
use patterns::{because, source};
use std::collections::HashSet;

pub struct ReadParts;
source!(ReadParts, "a told sentence that is no statement is read from left to right into statements: a run of words with the form of a statement is one part, and a single named word fills the open place of its kind in the part before it, or repeats that part with this word, or makes the most asked statement form with one place of its kind; a word for a named word stands for the thing of its sentence, or for the last named word of the sentences before");

pub struct ForwardChaining;
source!(ForwardChaining, "forward chaining to a fixed point, as in the production systems of Forgy and in Datalog: every rule is used on the facts and on what rules gave, round by round, until nothing new comes; a statement that a rule needs to be absent holds when it can not be made, which is checked against all that the rules can give, again until it is stable, as in the well founded reading of Van Gelder, Ross and Schlipf");

pub type Things = CheapMap<u32, Vec<u32>>;
because!(Things, ReadParts, "the named words of each kind in a story, each one time, in the order in which the story names them");

pub struct SortedStory {
    pub facts: Vec<Known>,
    pub sentences: Vec<(Shape, Vec<Statement>)>,
    pub things: Things,
}
because!(SortedStory, ReadParts, "the told sentences of a story as they are sorted: the told statements, every other sentence with its form and its parts, and the named words of each kind");

pub struct ReadStory {
    pub facts: Vec<Known>,
    pub rules: Vec<Implication>,
    pub more: Vec<Known>,
    pub things: Things,
}
because!(ReadStory, ReadParts, "a story as it is read: its facts (the told statements and the facts that were read from lists), its rules, the facts that were read from lists alone, and the named words of each kind");

pub type Binding = Vec<(u32, u32)>;
because!(Binding, ForwardChaining, "the words that fill the open places of a rule, each open place with its word");

fn bound(binding: &Binding, place: u32) -> Option<u32> {
    binding.iter().find(|(held, _)| *held == place).map(|(_, word)| *word)
}
because!(bound, ForwardChaining, "the word that fills an open place, when one fills it");

fn filled(binding: &Binding, values: &[u32]) -> Vec<u32> {
    values.iter().map(|value| if is_word(*value) { *value } else { bound(binding, *value).unwrap_or(*value) }).collect()
}
because!(filled, ForwardChaining, "the values of a statement with each open place replaced by the word that fills it");

fn holds_pending(found: &[Statement], class: u32) -> bool {
    found.iter().any(|part| part.values.contains(&(PENDING | class)))
}
because!(holds_pending, ReadParts, "a part of the sentence so far has a place of the kind that nothing has filled");

fn stands_for(found: &[Statement], here: &[(u32, u32)], refers: &mut Vec<u32>, class: u32) -> u32 {
    if holds_pending(found, class) {
        return PENDING | class;
    }
    if let Some((_, word)) = here.iter().rev().find(|(held, _)| *held == class) {
        return *word;
    }
    if !refers.contains(&class) {
        refers.push(class);
    }
    PENDING | class
}
because!(stands_for, ReadParts, "what a word for a named word of a kind stands for: the thing of the sentence so far, which is a place that nothing has filled or the last named word of the sentence; else a place that a later named word of the sentence or the last named word of the sentences before fills");

fn fill_named(found: &mut [Statement], here: &mut Vec<(u32, u32)>, class: u32, word: u32) {
    for part in found.iter_mut() {
        for value in part.values.iter_mut() {
            if *value == PENDING | class {
                *value = word;
            }
        }
    }
    here.push((class, word));
}
because!(fill_named, ReadParts, "a named word fills the places of its kind that nothing has filled, and it is the last named word of its kind in the sentence");

impl ToldRules {
    fn run_form(&self, words: &PatternWords, view: &View, shape: &[u32], tokens: &[u32], from: usize) -> Option<(Vec<usize>, usize)> {
        let mut at: Vec<usize> = Vec::with_capacity(shape.len());
        let mut place = from;
        let count = tokens.len();
        for entry in shape {
            if entry & KIND_MARK != 0 {
                let class = entry & !KIND_MARK;
                while place + 1 < count
                    && self.open.contains(&tokens[place])
                    && !self.refer.contains_key(&tokens[place])
                    && self.class_of(words, view, tokens[place]) == class
                    && self.class_of(words, view, tokens[place + 1]) == class
                {
                    place += 1;
                }
                if place >= count || self.class_of(words, view, tokens[place]) != class {
                    return None;
                }
            } else if place >= count || tokens[place] != *entry {
                return None;
            }
            at.push(place);
            place += 1;
        }
        Some((at, place))
    }

    pub fn parts_of(&self, words: &PatternWords, view: &View, tokens: &[u32]) -> (Vec<Statement>, Vec<u32>) {
        let mut found: Vec<Statement> = Vec::new();
        let mut refers: Vec<u32> = Vec::new();
        let mut here: Vec<(u32, u32)> = Vec::new();
        let mut pending: Vec<u32> = Vec::new();
        let mut at = 0;
        'tokens: while at < tokens.len() {
            for shape in &self.by_length {
                let Some((places, next)) = self.run_form(words, view, shape, tokens, at) else { continue };
                let (base, keep): (Shape, Vec<usize>) = match self.neg.get(shape) {
                    Some((base, keep)) => (base.clone(), keep.clone()),
                    None => (shape.clone(), (0..shape.len()).collect()),
                };
                let mut values: Vec<u32> = Vec::with_capacity(base.len());
                for (entry, place) in base.iter().zip(&keep) {
                    let word = tokens[places[*place]];
                    if entry & KIND_MARK == 0 {
                        values.push(*entry);
                        continue;
                    }
                    let class = entry & !KIND_MARK;
                    if let Some(fixed) = self.fixed.get(&class) {
                        values.push(*fixed);
                    } else if self.refer.get(&word) == Some(&class) {
                        values.push(stands_for(&found, &here, &mut refers, class));
                    } else if self.open.contains(&word) {
                        values.push(*entry);
                    } else {
                        values.push(word);
                        fill_named(&mut found, &mut here, class, word);
                    }
                }
                found.push(Statement { absent: self.neg.contains_key(shape), base, values });
                at = next;
                pending.clear();
                continue 'tokens;
            }
            let mut word = tokens[at];
            let mut class = self.class_of(words, view, word);
            at += 1;
            if self.neg_words.contains(&word) {
                pending.push(word);
                continue;
            }
            if let Some(refer) = self.refer.get(&word) {
                class = *refer;
                word = stands_for(&found, &here, &mut refers, class);
            } else if !(class != ABSENT && self.named.holds(&class) && !self.open.contains(&word) && !self.fixed.contains_key(&class)) {
                pending.clear();
                continue;
            }
            if pending.is_empty() && holds_pending(&found, class) {
                if is_word(word) {
                    fill_named(&mut found, &mut here, class, word);
                }
                continue;
            }
            let slot = KIND_MARK | class;
            let (base, mut values, place) = match found.last() {
                Some(last) if !self.start.contains(&class) && last.base.contains(&slot) => {
                    let place = last.base.iter().rposition(|entry| *entry == slot).unwrap_or(0);
                    (last.base.clone(), last.values.clone(), place)
                }
                _ => match self.single.get(&class) {
                    Some(base) => {
                        let values: Vec<u32> = base.iter().map(|entry| if entry & KIND_MARK == 0 { *entry } else { self.fixed.get(&(entry & !KIND_MARK)).copied().unwrap_or(PENDING | (entry & !KIND_MARK)) }).collect();
                        let place = base.iter().position(|entry| *entry == slot).unwrap_or(0);
                        (base.clone(), values, place)
                    }
                    None => {
                        pending.clear();
                        continue;
                    }
                },
            };
            values[place] = word;
            if is_word(word) {
                here.push((class, word));
            }
            let opposite = if pending.is_empty() { None } else { self.before.get(&(base.clone(), place, pending.clone())) };
            match opposite {
                None => found.push(Statement { absent: false, base, values }),
                Some(opposite) if self.neg.contains_key(opposite) => found.push(Statement { absent: true, base, values }),
                Some(opposite) => {
                    let values = self.up(opposite, &values);
                    found.push(Statement { absent: false, base: opposite.clone(), values });
                }
            }
            pending.clear();
        }
        (found, refers)
    }

    pub fn parts(&self, words: &PatternWords, view: &View, tokens: &[u32], last: &[(u32, u32)]) -> Vec<Statement> {
        let (mut found, refers) = self.parts_of(words, view, tokens);
        for part in found.iter_mut() {
            for value in part.values.iter_mut() {
                if *value & PENDING != 0 {
                    let class = *value & !PENDING;
                    let told = last.iter().rev().find(|(held, _)| *held == class).map(|(_, word)| *word);
                    *value = match told {
                        Some(word) if refers.contains(&class) => word,
                        _ => KIND_MARK | class,
                    };
                }
            }
        }
        found
    }

    pub fn rule(&self, atoms: &[Statement]) -> Option<Implication> {
        if atoms.len() < PAIR_PARTS {
            return None;
        }
        let (head, needs) = if self.last { (atoms[atoms.len() - 1].clone(), atoms[..atoms.len() - 1].to_vec()) } else { (atoms[0].clone(), atoms[1..].to_vec()) };
        if head.absent {
            return None;
        }
        let open: HashSet<u32> = needs.iter().flat_map(|part| part.values.iter().copied()).filter(|value| !is_word(*value)).collect();
        if head.values.iter().any(|value| !is_word(*value) && !open.contains(value)) {
            return None;
        }
        Some(Implication { needs, head })
    }

    pub fn sort(&self, words: &PatternWords, view: &View, sents: &[Vec<u32>]) -> SortedStory {
        let mut facts: Vec<Known> = Vec::new();
        let mut sentences: Vec<(Shape, Vec<Statement>)> = Vec::new();
        let mut things: Things = CheapMap::default();
        let mut last: Vec<(u32, u32)> = Vec::new();
        for tokens in sents {
            let shape = self.shape_of(words, view, tokens);
            let mut found: Option<Vec<Statement>> = None;
            if self.pos.contains(&shape) && !tokens.iter().any(|word| self.refer.contains_key(word)) {
                let values = self.values(&shape, tokens);
                if values.iter().all(|value| is_word(*value)) {
                    facts.push((shape.clone(), values.clone()));
                    found = Some(vec![Statement { absent: false, base: shape.clone(), values }]);
                }
            }
            let found = match found {
                Some(found) => found,
                None => {
                    let mut parts = self.parts(words, view, tokens, &last);
                    if !self.must.is_empty() {
                        parts.retain(|part| !part.values.iter().any(|value| !is_word(*value) && self.must.contains(&(value & !(KIND_MARK | PENDING)))));
                    }
                    sentences.push((shape, parts.clone()));
                    parts
                }
            };
            for part in &found {
                for (entry, value) in part.base.iter().zip(&part.values) {
                    if entry & KIND_MARK != 0 && is_word(*value) {
                        let class = entry & !KIND_MARK;
                        let held = things.entry(class).or_default();
                        if !held.contains(value) {
                            held.push(*value);
                        }
                        if !self.fixed.contains_key(&class) {
                            last.push((class, *value));
                        }
                    }
                }
            }
        }
        SortedStory { facts, sentences, things }
    }

    pub fn read(&self, words: &PatternWords, view: &View, sents: &[Vec<u32>], plain: bool) -> ReadStory {
        let SortedStory { mut facts, sentences, things } = self.sort(words, view, sents);
        let mut rules: Vec<Implication> = Vec::new();
        let mut more: Vec<Known> = Vec::new();
        let mut before: Option<Vec<Statement>> = None;
        if !plain {
            for (form, atoms) in sentences {
                if atoms.is_empty() || self.bad.contains(&form) {
                    before = None;
                    continue;
                }
                if self.facts_too && atoms.iter().all(|part| part.values.iter().all(|value| is_word(*value))) {
                    for part in &atoms {
                        let known = (part.base.clone(), part.values.clone());
                        if !part.absent && !facts.contains(&known) {
                            more.push(known);
                        }
                    }
                    before = None;
                    continue;
                }
                if self.carry && atoms.len() == 1 && !atoms[0].absent {
                    if let Some(earlier) = &before {
                        let open: HashSet<u32> = earlier.iter().flat_map(|part| part.values.iter().copied()).filter(|value| !is_word(*value)).collect();
                        if atoms[0].values.iter().all(|value| is_word(*value) || open.contains(value)) {
                            rules.push(Implication { needs: earlier.clone(), head: atoms[0].clone() });
                        }
                        continue;
                    }
                }
                if let Some(found) = self.rule(&atoms) {
                    rules.push(found);
                }
                before = Some(atoms);
            }
        }
        facts.extend(more.iter().cloned());
        ReadStory { facts, rules, more, things }
    }

    pub fn row(&self, words: &PatternWords, view: &View, q: &[u32], have: &HashSet<Known>) -> Option<TableRow> {
        let shape = self.shape_of(words, view, q);
        if let Some((base, keep)) = self.neg.get(&shape) {
            let taken: Vec<u32> = keep.iter().map(|at| q[*at]).collect();
            return Some((true, false, have.contains(&(base.clone(), self.values(base, &taken)))));
        }
        if !self.pos.contains(&shape) {
            return None;
        }
        let values = self.values(&shape, q);
        let other = match self.pair_of(&shape) {
            Some((_, base, keep)) => {
                let taken: Vec<u32> = keep.iter().map(|at| q[*at]).collect();
                have.contains(&(base.clone(), self.values(base, &taken)))
            }
            None => self.pairs_of.get(&shape).is_some_and(|opposites| opposites.iter().any(|opposite| self.pos.contains(opposite) && have.contains(&(opposite.clone(), self.up(opposite, &values))))),
        };
        Some((false, have.contains(&(shape, values)), other))
    }

    fn bindings<N: WordTexts>(&self, names: &N, needs: &[Statement], by: &CheapMap<Shape, FirstSightTable<Vec<u32>, ()>>, absent: Option<&dyn Fn(&Known) -> bool>, things: &Things, out: &mut Vec<Binding>) {
        let pos: Vec<&Statement> = needs.iter().filter(|part| !part.absent).collect();
        let neg: Vec<&Statement> = needs.iter().filter(|part| part.absent).collect();
        let mut stack: Vec<(usize, Binding)> = vec![(0, Vec::new())];
        let mut walked: Vec<Binding> = Vec::new();
        while let Some((at, binding)) = stack.pop() {
            if at == pos.len() {
                walked.push(binding);
                continue;
            }
            let part = pos[at];
            let Some(group) = by.get(&part.base) else { continue };
            let mut next: Vec<(usize, Binding)> = Vec::new();
            'facts: for (fact, _) in group.rows() {
                let mut more = binding.clone();
                for (value, word) in part.values.iter().zip(fact) {
                    if is_word(*value) {
                        if value != word {
                            continue 'facts;
                        }
                    } else {
                        match bound(&more, *value) {
                            Some(held) if held != *word => continue 'facts,
                            Some(_) => {}
                            None => more.push((*value, *word)),
                        }
                    }
                }
                next.push((at + 1, more));
            }
            stack.extend(next.into_iter().rev());
        }
        for binding in walked {
            self.close(names, &neg, 0, binding, absent, things, out);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn close<N: WordTexts>(&self, names: &N, neg: &[&Statement], at: usize, binding: Binding, absent: Option<&dyn Fn(&Known) -> bool>, things: &Things, out: &mut Vec<Binding>) {
        if at == neg.len() {
            out.push(binding);
            return;
        }
        let part = neg[at];
        let free = part.values.iter().find(|value| !is_word(**value) && bound(&binding, **value).is_none());
        if let Some(free) = free {
            let mut named: Vec<u32> = things.get(&(free & !(KIND_MARK | PENDING))).cloned().unwrap_or_default();
            named.sort_by(|a, b| names.name(*a).cmp(names.name(*b)));
            for thing in named {
                let mut more = binding.clone();
                more.push((*free, thing));
                self.close(names, neg, at, more, absent, things, out);
            }
            return;
        }
        let known = (part.base.clone(), filled(&binding, &part.values));
        if absent.is_none_or(|test| test(&known)) {
            self.close(names, neg, at + 1, binding, absent, things, out);
        }
    }

    fn run<N: WordTexts>(&self, names: &N, facts: &[Known], rules: &[Implication], things: &Things, absent: Option<&dyn Fn(&Known) -> bool>) -> (HashSet<Known>, Vec<Known>) {
        let mut by: CheapMap<Shape, FirstSightTable<Vec<u32>, ()>> = CheapMap::default();
        for (shape, values) in facts {
            by.entry(shape.clone()).or_default().entry(values.clone(), ());
        }
        let mut order: Vec<Known> = Vec::new();
        let mut found: Vec<Binding> = Vec::new();
        for _ in 0..TOLD_ROUNDS {
            let mut new: FirstSightTable<Known, ()> = FirstSightTable::new();
            for rule in rules {
                found.clear();
                self.bindings(names, &rule.needs, &by, absent, things, &mut found);
                for binding in &found {
                    let values = filled(binding, &rule.head.values);
                    if !by.get(&rule.head.base).is_some_and(|group| group.holds(&values)) {
                        new.entry((rule.head.base.clone(), values), ());
                    }
                }
            }
            if new.rows().is_empty() {
                break;
            }
            for (known, _) in new.rows() {
                by.entry(known.0.clone()).or_default().entry(known.1.clone(), ());
                order.push(known.clone());
            }
        }
        let all: HashSet<Known> = by.iter().flat_map(|(shape, group)| group.rows().iter().map(move |(values, _)| (shape.clone(), values.clone()))).collect();
        (all, order)
    }

    pub fn closure<N: WordTexts>(&self, names: &N, facts: &[Known], rules: &[Implication], things: &Things) -> Vec<Known> {
        if rules.is_empty() {
            return Vec::new();
        }
        if !rules.iter().any(|rule| rule.needs.iter().any(|part| part.absent)) {
            return self.run(names, facts, rules, things, None).1;
        }
        let mut upper = self.run(names, facts, rules, things, Some(&|_: &Known| true)).0;
        let mut order: Vec<Known> = Vec::new();
        for _ in 0..TOLD_ROUNDS {
            let (lower, found) = self.run(names, facts, rules, things, Some(&|known: &Known| !upper.contains(known)));
            order = found;
            let again = self.run(names, facts, rules, things, Some(&|known: &Known| !lower.contains(known))).0;
            if again == upper {
                break;
            }
            upper = again;
        }
        order
    }

    pub fn with_new<N: WordTexts>(&self, words: &PatternWords, names: &mut N, view: View, texts: &[String], question: Option<&str>) -> View {
        if self.capital == ABSENT {
            return view;
        }
        let mut seen: FirstSightTable<String, (bool, bool)> = FirstSightTable::new();
        let all: Vec<&str> = texts.iter().map(String::as_str).chain(question).collect();
        for text in all {
            let raws = as_written(text);
            for (at, raw) in raws.iter().enumerate() {
                let low = raw.to_lowercase();
                let number = names.find(&low);
                if number.is_some_and(|word| words.has(word, IN_LESSONS) || self.class_of(words, &view, word) != ABSENT) {
                    continue;
                }
                let after = raws.get(1).and_then(|next| names.find(&next.to_lowercase())).is_some_and(|next| self.after_capital.contains(&words.norm(next)));
                let here = at > 0 || (raws.len() > 1 && after);
                let upper = raw.chars().next().is_some_and(char::is_uppercase);
                let held = seen.get(&low).copied().unwrap_or((true, false));
                *seen.entry(low, held) = (held.0 && upper, held.1 || here);
            }
        }
        let mut new: Vec<u32> = seen.rows().iter().filter(|(_, (always, inside))| *always && *inside).filter_map(|(word, _)| names.find(word)).collect();
        if let Some(question) = question {
            let q = self.tok(&words.read(names, &cut_words(question)));
            let class_new = |word: u32| {
                let class = self.class_of(words, &view, word);
                if class != ABSENT {
                    class
                } else if new.contains(&word) {
                    self.capital
                } else {
                    ABSENT
                }
            };
            let mut fits: Vec<Vec<u32>> = Vec::new();
            for shape in self.by_length.iter().filter(|shape| shape.len() == q.len()) {
                let places: Vec<u32> = shape.iter().zip(&q).filter(|(entry, word)| *entry & KIND_MARK != 0 && class_new(**word) != *entry & !KIND_MARK).map(|(_, word)| *word).collect();
                let fixed_fit = shape.iter().zip(&q).all(|(entry, word)| entry & KIND_MARK != 0 || entry == word);
                let written = places.iter().all(|word| seen.get(&names.name(*word).to_string()).is_some_and(|(always, _)| *always));
                let capital = shape.iter().zip(&q).all(|(entry, word)| entry & KIND_MARK == 0 || !places.contains(word) || *entry & !KIND_MARK == self.capital);
                if fixed_fit && !places.is_empty() && written && capital {
                    fits.push(places);
                }
            }
            if fits.len() == 1 {
                for word in &fits[0] {
                    if !new.contains(word) {
                        new.push(*word);
                    }
                }
            }
        }
        View { new, ..view }
    }

    pub fn story<N: WordTexts>(&self, words: &PatternWords, names: &mut N, sentences: &[String], question: Option<&str>) -> (Vec<String>, HashSet<Known>, View) {
        let asked = question.filter(|text| self.capital != ABSENT && words_of_question(words, names, text));
        let base = Desk::over(words, names).case(sentences, "", &|_| 0);
        let view = self.with_new(words, names, View { extra: base.story.extra.clone(), new: Vec::new() }, sentences, asked);
        let sents: Vec<Vec<u32>> = base.story.sents.iter().map(|told| self.tok(&told.words)).collect();
        let ReadStory { facts, rules, more, things } = self.read(words, &view, &sents, !self.rules_on);
        let mut told: Vec<String> = sentences.to_vec();
        let mut given: Vec<Known> = more;
        given.extend(self.closure(names, &facts, &rules, &things));
        if !given.is_empty() {
            let mut shapes: Vec<Shape> = sents.iter().map(|tokens| self.shape_of(words, &view, tokens)).collect();
            for (shape, values) in &given {
                let at = shapes.iter().rposition(|held| held == shape).map_or(told.len(), |place| place + 1);
                let text: Vec<String> = values.iter().map(|word| names.name(*word).replace(JOINT, " ")).collect();
                told.insert(at, format!("{}.", text.join(" ")));
                shapes.insert(at, shape.clone());
            }
        }
        let mut have: HashSet<Known> = facts.into_iter().collect();
        have.extend(given);
        (told, have, view)
    }

    pub fn written<N: WordTexts>(&self, words: &PatternWords, names: &mut N, sentences: &[String]) -> Vec<String> {
        self.story(words, names, sentences, None).0
    }

    pub fn answer<N: WordTexts>(&self, words: &PatternWords, names: &mut N, sentences: &[String], question: &str, readings: &dyn Fn(&[u32]) -> u32) -> Option<(Vec<u32>, TableRow)> {
        let (_, have, view) = self.story(words, names, sentences, Some(question));
        let case = Desk::over(words, names).case(sentences, question, readings);
        let q: Vec<u32> = case.q.iter().map(|word| case.back.iter().find(|(stand, _)| stand == word).map_or(*word, |(_, new)| *new)).collect();
        let row = self.row(words, &view, &self.tok(&q), &have)?;
        self.table.get(&row).map(|sure| (sure.answer.clone(), row))
    }
}

const PAIR_PARTS: usize = 2;
because!(PAIR_PARTS, ReadParts, "a rule holds what it gives and at least one statement that it needs; a sentence with fewer parts is no rule");

fn words_of_question<N: WordTexts>(words: &PatternWords, names: &N, question: &str) -> bool {
    cut_words(question).iter().any(|word| !names.find(word).is_some_and(|number| words.has(number, IN_LESSONS)))
}
because!(words_of_question, ReadParts, "a question has a word that the lessons never held; only then can a new word get its kind from its place in the asked statement");
