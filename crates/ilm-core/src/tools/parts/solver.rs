use crate::tools::parts::grammar::{FormState, Grammar, Meter, ROW_KEY, Reading, FormShape, ShapeKind, State, Stop, UNSOLVED, Weight, solved_row, weight_of, Apply};
use crate::tools::parts::language::{GUESSES, PartAnswers, ONLY_PLAIN, Piece, Program, WordRow, AnswerView, Word, cut_programs, gap_programs, shortest_program};
use patterns::{because, source};
use std::collections::HashSet;

pub struct SolvedFromAnswer;
source!(SolvedFromAnswer, "learning a grammar from texts with their meanings by solving for the one thing that is not known, in the way of the inverse semantics of Zettlemoyer and Collins and of the version space of Mitchell: a lesson is read with one state left open, the row that the open state must give is solved from the true answer, and every program of the language that makes that row from the parts is a candidate; the lessons that follow say which candidate stays");

pub struct Light<'a> {
    pub meter: &'a Meter,
    pub rank: &'a [u32],
}
because!(Light, SolvedFromAnswer, "what every step of a search needs beside the grammar: the count of the words read, and the place of each word in the order of the texts of the words, for a choice among equals that must not depend on the order of the lessons");

impl Light<'_> {
    pub fn rank_of(&self, word: Word) -> u32 {
        match self.rank.get(word as usize) {
            Some(place) => *place,
            None => word,
        }
    }

    pub fn weight(&self, state: &State) -> Weight {
        weight_of(state, &|word| self.rank_of(word))
    }
}

pub struct Pending {
    pub word: Word,
    pub shape: FormShape,
    pub place: usize,
    pub said: Option<WordRow>,
    pub pieces: Vec<Option<Piece>>,
}
because!(Pending, SolvedFromAnswer, "a form whose solved row holds words that no part says: the word, the shape, the place in the order of use, what the word says, and the program with a gap at each such word; a search counts what fills the gaps");

const ONE_FIXED: &[ShapeKind] = &[ShapeKind::Between, ShapeKind::After, ShapeKind::Before];
because!(ONE_FIXED, SolvedFromAnswer, "the shapes with one fixed word, in the order in which a search tries them for a word: the word between two parts first, because it takes the most of the text");

const TWO_FIXED: &[ShapeKind] = &[ShapeKind::BeforeTwo, ShapeKind::BetweenTwo, ShapeKind::Around];
because!(TWO_FIXED, SolvedFromAnswer, "the shapes with a second fixed word after the first, in the order in which a search tries them");

pub fn guessed_programs(target: &[Word], given: &PartAnswers, shape: FormShape, light: &Light) -> Vec<Program> {
    let letters: HashSet<Word> = given.rows.iter().flatten().flatten().copied().collect();
    let mut found: Vec<Program> = Vec::new();
    let mut seen: HashSet<Program> = HashSet::new();
    for start in 0..target.len() {
        for end in start + 1..=target.len() {
            if !letters.contains(&target[end - 1]) {
                break;
            }
            for program in cut_programs(&target[start..end], given, ONLY_PLAIN) {
                if seen.insert(program.clone()) {
                    found.push(program);
                }
            }
        }
    }
    found.sort_by_cached_key(|program| light.weight(&State::form(shape, Some(program.clone()), 0, None)));
    found.truncate(GUESSES);
    found
}
because!(guessed_programs, SolvedFromAnswer, "the programs with no turn and no cut whose row from these parts stands in the true answer as one run, the simplest first: a form that a lesson uses several times cannot be solved, so its program is guessed from the parts of its first use");

fn form_state(shape: FormShape, program: Option<Program>, place: usize, said: &Option<WordRow>) -> State {
    State::Form(FormState { shape, program, place, said: said.clone() })
}
because!(form_state, SolvedFromAnswer, "the state of a word with a form, with what the word goes on saying");

impl Grammar {
    fn solving_read(&self, words: &[Word], light: &Light, kept: &mut Option<PartAnswers>) -> Option<WordRow> {
        self.read(words, light.meter, Reading::solving(kept)).unwrap_or_default()
    }

    pub fn reads(&self, words: &[Word], light: &Light) -> Option<WordRow> {
        self.read(words, light.meter, Reading::plain(true)).unwrap_or_default()
    }

    pub fn changes(&self, words: &[Word], target: &[Word], word: Word, views: &[AnswerView], second: Option<Word>, only_kept: bool, mut pending: Option<&mut Vec<Pending>>, light: &Light) -> Vec<State> {
        let mut found: Vec<State> = Vec::new();
        let now = self.state(word);
        let said: Option<WordRow> = match &now {
            State::Table(row) => Some(row.clone()),
            State::Form(form) => form.said.clone(),
            _ => None,
        };
        if now == State::Mark && self.forms.values().any(|(shape, _)| shape.second == Some(word)) {
            return found;
        }
        if let (State::Form(form), false) = (&now, only_kept) {
            found.extend(self.places(word, Some(words)).into_iter().map(|place| form_state(form.shape, form.program.clone(), place, &form.said)));
            if let Some(program) = &form.program {
                for (number, piece) in program.pieces.iter().enumerate() {
                    if piece.looked() {
                        continue;
                    }
                    for other in views.iter().filter(|other| **other != piece.view) {
                        let mut pieces = program.pieces.clone();
                        pieces[number] = Piece::of(piece.slot as usize, *other);
                        found.push(form_state(form.shape, Some(shortest_program(&pieces.repeat(program.times))), form.place, &form.said));
                    }
                }
            }
        }
        if word != ROW_KEY && second.is_none() && !only_kept {
            if self.copies && now == State::NotKnown {
                found.push(State::Table(vec![word]));
            }
            if now != State::Out {
                found.push(State::Out);
            }
            let probe = self.with_state(word, &State::Table(vec![UNSOLVED]));
            if let Some(row) = probe.solving_read(words, light, &mut None).and_then(|last| solved_row(&last, target)) {
                found.push(State::Table(row));
            }
        }
        let mut shapes: Vec<(FormShape, Option<WordRow>)> = Vec::new();
        if word == ROW_KEY {
            shapes.push((FormShape::plain(ShapeKind::InARow), None));
        } else if !only_kept {
            if second.is_none() {
                shapes.extend(ONE_FIXED.iter().map(|kind| (FormShape::plain(*kind), None)));
            }
            let seconds = second.map_or_else(|| self.seconds(words, word), |second| vec![second]);
            shapes.extend(seconds.iter().flat_map(|other| TWO_FIXED.iter().map(|kind| (FormShape::two(*kind, *other), None))));
        }
        if let (true, true, Some(said), Some(at)) = (only_kept, word != ROW_KEY, &said, words.iter().position(|other| *other == word)) {
            let after: Vec<Word> = second.map_or_else(|| self.seconds(words, word), |second| vec![second]).into_iter().filter(|other| words[at + 1..].contains(other)).collect();
            let before: Vec<Word> = second.map_or_else(|| self.firsts(words, word), |second| vec![second]).into_iter().filter(|other| words[..at].contains(other)).collect();
            shapes.extend(after.iter().flat_map(|other| TWO_FIXED.iter().map(|kind| (FormShape::two(*kind, *other), Some(said.clone())))));
            shapes.extend(before.iter().map(|other| (FormShape::two(ShapeKind::SecondTwo, *other), Some(said.clone()))));
        }
        let mut tried: Vec<((FormShape, PartAnswers), Vec<Program>)> = Vec::new();
        for (shape, more) in &shapes {
            for place in self.places(word, Some(words)) {
                let probe = self.with_state(word, &form_state(*shape, None, place, more));
                let mut kept: Option<PartAnswers> = None;
                let last = match probe.read(words, light.meter, Reading::solving(&mut kept)) {
                    Ok(last) => last,
                    Err(_) => {
                        if let Some(first) = kept {
                            let key = (*shape, first);
                            if !tried.iter().any(|(seen, _)| *seen == key) {
                                let programs = guessed_programs(target, &key.1, *shape, light);
                                tried.push((key.clone(), programs));
                            }
                            let programs = tried.iter().find(|(seen, _)| *seen == key).map(|(_, programs)| programs.clone()).unwrap_or_default();
                            found.extend(programs.into_iter().map(|program| form_state(*shape, Some(program), place, more)));
                        }
                        continue;
                    }
                };
                let (Some(last), Some(first)) = (last, kept) else { continue };
                let Some(row) = solved_row(&last, target) else { continue };
                let programs = cut_programs(&row, &first, views);
                if programs.is_empty() {
                    if let Some(pending) = pending.as_mut() {
                        pending.extend(gap_programs(&row, &first).into_iter().map(|pieces| Pending { word, shape: *shape, place, said: more.clone(), pieces }));
                    }
                }
                found.extend(programs.into_iter().map(|program| form_state(*shape, Some(program), place, more)));
            }
        }
        let mut seen: HashSet<State> = HashSet::new();
        let mut kept: Vec<State> = Vec::new();
        for state in found {
            if !seen.insert(state.clone()) {
                continue;
            }
            if state != now && self.with_state(word, &state).reads(words, light).as_deref() == Some(target) {
                kept.push(state);
            }
        }
        let itself = State::Table(vec![word]);
        if self.copies && kept.contains(&itself) && kept.contains(&State::Out) {
            kept.retain(|state| *state != State::Out);
        }
        kept
    }

    pub fn asked(&self, words: &[Word], target: &[Word], lacking: Word, views: &[AnswerView], mut pending: Option<&mut Vec<Pending>>, light: &Light) -> Vec<(Word, State)> {
        let mut found: Vec<(Word, State)> = Vec::new();
        let mut seen: Vec<Word> = Vec::new();
        for word in words {
            if seen.contains(word) {
                continue;
            }
            seen.push(*word);
            if *word != lacking && self.table.get(word).is_some() && !self.forms.holds(word) && !self.marks.contains(word) {
                let states = self.changes(words, target, *word, views, Some(lacking), true, pending.as_deref_mut(), light);
                found.extend(states.into_iter().map(|state| (*word, state)));
            }
        }
        found
    }

    pub fn guesses(&self, words: &[Word], target: &[Word], word: Word, others: &[Word], light: &Light) -> Vec<State> {
        let mut found: Vec<State> = Vec::new();
        if word != ROW_KEY {
            found.push(State::Out);
            found.push(State::Table(Vec::new()));
            let said: HashSet<Word> = words.iter().filter_map(|other| self.table.get(other)).flatten().copied().collect();
            let mut runs: Vec<WordRow> = Vec::new();
            let mut at: Option<usize> = None;
            for place in 0..=target.len() {
                if place < target.len() && !said.contains(&target[place]) {
                    at = at.or(Some(place));
                } else if let Some(start) = at.take() {
                    let run = target[start..place].to_vec();
                    if !runs.contains(&run) {
                        runs.push(run);
                    }
                }
            }
            found.extend(runs.into_iter().map(State::Table));
        }
        let kinds: &[ShapeKind] = if word == ROW_KEY { &[ShapeKind::InARow] } else { ONE_FIXED };
        for kind in kinds {
            let shape = FormShape::plain(*kind);
            let mut seen: Vec<(PartAnswers, Vec<Program>)> = Vec::new();
            for place in self.places(word, Some(words)) {
                let probe = self.with_state(word, &form_state(shape, None, place, &None));
                let how = Reading { strict: true, apply: Apply::FirstUse, used: None, hold: others };
                let Err(Stop::FirstUse(given)) = probe.read(words, light.meter, how) else { continue };
                if !seen.iter().any(|(first, _)| *first == given) {
                    let programs = guessed_programs(target, &given, shape, light);
                    seen.push((given.clone(), programs));
                }
                let programs = seen.iter().find(|(first, _)| *first == given).map(|(_, programs)| programs.clone()).unwrap_or_default();
                found.extend(programs.into_iter().map(|program| form_state(shape, Some(program), place, &None)));
            }
        }
        found
    }

    pub fn explain(&self, words: &[Word], target: &[Word], things: &[Word], views: &[AnswerView], light: &Light) -> Vec<Grammar> {
        if let [thing] = things {
            let mut found: Vec<Grammar> = self.changes(words, target, *thing, views, None, false, None, light).iter().map(|state| self.with_state(*thing, state)).collect();
            if found.is_empty() && *thing != ROW_KEY && self.row.is_some() && self.state(*thing) == State::NotKnown {
                for (word, state) in self.asked(words, target, *thing, views, None, light) {
                    let mut other = self.with_state(word, &state);
                    other.extra = Some(vec![word]);
                    found.push(other);
                }
            }
            return found;
        }
        let mut found: Vec<Grammar> = Vec::new();
        if let ([first, second], false) = (things, things.contains(&ROW_KEY)) {
            for (word, other) in [(*first, *second), (*second, *first)] {
                found.extend(self.changes(words, target, word, views, Some(other), false, None, light).iter().map(|state| self.with_state(word, state)));
            }
        }
        for inner in things {
            let others: Vec<Word> = things.iter().copied().filter(|thing| thing != inner).collect();
            for state in self.guesses(words, target, *inner, &others, light) {
                found.extend(self.with_state(*inner, &state).explain(words, target, &others, views, light));
            }
        }
        found
    }
}
