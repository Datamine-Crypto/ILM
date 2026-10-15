use crate::tools::parts::counted::{FORCED_LESSONS, SURE_LESSONS, counted_table, sure_of};
use crate::tools::parts::grammar::{FormState, Grammar, GrammarKey, Lookup, ProgramTables, ROW_KEY, Reading, FormShape, State, Weight, solved_row};
use crate::tools::parts::language::{FIRST_OF, PartAnswers, KEY_VIEWS, LAST_OF, LOOKED, NO_OPERATION, PARTS_OF_FORM, PLAIN, Piece, Program, WordRow, VIEWS, AnswerView, Word, laid_on, whole_pieces};
use crate::tools::parts::ordered::Kept;
use crate::tools::parts::says::WordSays;
use crate::tools::parts::solver::{Light, Pending};
use patterns::{because, source};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

pub struct OneChangeSearch;
source!(OneChangeSearch, "the search for a grammar by one change at a time, a hill climb over the states of the words in the way of the greedy search of grammar induction: each lesson that is not read right proposes the states that would read it, the gain of a change is the lessons that it makes right less the lessons that it makes wrong, and the change with the highest gain is taken; a change that the lessons cannot tell apart from another one waits for lessons that can");

pub const SEARCH_LESSONS_MOST: usize = 2000;
because!(SEARCH_LESSONS_MOST, OneChangeSearch, "the lessons that a search reads, the shortest first: a short text has few parts, so it is explained by one change, and more lessons of the same forms add no evidence");

const PROPOSE_BATCH: usize = 40;
because!(PROPOSE_BATCH, OneChangeSearch, "the lessons that propose changes before the changes are compared: enough that a true change is proposed by several of them, few enough that a round is quick");

const BATCHES: usize = 10;
because!(BATCHES, OneChangeSearch, "the batches of one round at most: after so many with no change that can be taken, more lessons of this sort tell no more");

const NONE_MOST: usize = 80;
because!(NONE_MOST, OneChangeSearch, "the lessons with no unknown thing that a round asks, the shortest first: such a lesson is wrong by a state that is there, and every word of it must be tried");

const TRIED: usize = 12;
because!(TRIED, OneChangeSearch, "the proposed changes with a gain that are compared with each other in one round");

const TRIED_MOST: usize = 60;
because!(TRIED_MOST, OneChangeSearch, "the proposed changes whose gain is counted in one round, those that the most lessons propose first: the count of a gain reads many lessons");

const PLACES: usize = 3;
because!(PLACES, OneChangeSearch, "the places in the order of use at which a proposed change is compared with the others, those that the most lessons propose first");

pub const SEARCH_CHANGES_MOST: usize = 400;
because!(SEARCH_CHANGES_MOST, OneChangeSearch, "the changes of one search at most: a grammar of a kind of text has far fewer states, so a search that goes on beyond this number goes in a circle");

const SEVERAL_LESSONS: usize = 30;
because!(SEVERAL_LESSONS, OneChangeSearch, "the lessons that propose two changes at once: the pairs of states of one lesson are many, so few lessons are asked");

const TABLES_MOST: usize = 3;
because!(TABLES_MOST, OneChangeSearch, "the tables of one looked-up piece at most: the words of a program whose lessons do not agree on one table are put in groups, and with more groups than this the piece is no table but a list of cases");

const LOOK_LESSONS: usize = 200;
because!(LOOK_LESSONS, OneChangeSearch, "the lessons with one word that are read to count a table: a table of single words is sure after far fewer");

pub const SEARCH_WORK_FIRST: u64 = 500000;
because!(SEARCH_WORK_FIRST, OneChangeSearch, "the words of text that a search reads at most while it reads no lesson right: a kind that this learner can read shows a first right lesson long before");

pub const SEARCH_WORK_EACH: u64 = 40000;
because!(SEARCH_WORK_EACH, OneChangeSearch, "the words that a search may read more for each lesson that it reads right, so that a search that makes way goes on and one that does not ends");

pub const SEARCH_WORK_MOST: u64 = 12000000;
because!(SEARCH_WORK_MOST, OneChangeSearch, "the words that a search reads at most in all; it then stops with the grammar that it has");

const SLOT_SAID: usize = 2;
because!(SLOT_SAID, OneChangeSearch, "the place of what the fixed word of a form says, after the places of the parts in the answers that a program is run on");

const SLOT_SECOND: usize = 3;
because!(SLOT_SECOND, OneChangeSearch, "the place of what the second fixed word of a form says, the last of the answers that a program is run on");

pub struct Held {
    pub grammar: Grammar,
    memo: RefCell<HashMap<usize, bool>>,
}
because!(Held, OneChangeSearch, "a grammar of a search with what is known of it: for each lesson that it was asked, whether it reads the lesson right, so that a lesson is read once for each grammar");

impl Held {
    pub fn of(grammar: Grammar) -> Rc<Held> {
        Rc::new(Held { grammar, memo: RefCell::new(HashMap::new()) })
    }
}

#[derive(Clone)]
struct Proposal {
    lessons: usize,
    things: Vec<Word>,
    other: Rc<Held>,
}
because!(Proposal, OneChangeSearch, "one proposed change: the lessons that propose it, the things that it changes, and the grammar with the change");

type Proposed = Kept<GrammarKey, Proposal>;
because!(Proposed, OneChangeSearch, "the proposed changes of a round by the states that they give and the order of use, in the order in which the lessons proposed them");

type Content = Vec<(Word, State)>;
because!(Content, OneChangeSearch, "the states that a change gives to its things, without their places in the order of use");

type Best = (isize, Vec<Word>, Rc<Held>);
because!(Best, OneChangeSearch, "the change that a round takes: its gain, the things that it changes, and the grammar with the change");

type Votes = Kept<(Word, State), (HashSet<usize>, State)>;
because!(Votes, OneChangeSearch, "for the state of one thing: the lessons where it is one of two changes that read the lesson right, and the state with its place");

type LookGroups = Kept<(FormShape, usize, Word), (Kept<Word, Option<WordRow>>, Kept<Vec<Option<Piece>>, ()>)>;
because!(LookGroups, OneChangeSearch, "the forms of a batch whose rows hold words that no part says, by shape, place and first fixed word: the words with what each says, and the programs with gaps");

struct ChangeGroup {
    content: Content,
    entries: Vec<Proposal>,
}
because!(ChangeGroup, OneChangeSearch, "the proposals of one change at its places in the order of use, the place that the most lessons propose first");

struct RoundChoice {
    gain: isize,
    things: Vec<Word>,
    other: Rc<Held>,
    fixed: Vec<usize>,
    group: usize,
}
because!(RoundChoice, OneChangeSearch, "a proposed change with its gain at its best place: the things, the grammar, the lessons that it makes right, and the group of its proposals");

pub struct Search<'a> {
    seen: Vec<(Vec<Word>, WordRow)>,
    by_word: HashMap<Word, Vec<usize>>,
    right: Vec<bool>,
    began: u64,
    done: usize,
    light: &'a Light<'a>,
}
because!(Search, OneChangeSearch, "one search: its lessons (each text with its answer once, the shortest first), the lessons of each word, which lessons the grammar reads right, and the count of words read at its start. grammar_of runs the search from the counted table or from a grammar to go on from, and gives the grammar that reads the most lessons right");

fn key_pieces() -> Vec<Piece> {
    let mut keys: Vec<Piece> = (0..PARTS_OF_FORM).flat_map(|slot| KEY_VIEWS.iter().map(move |view| Piece::of(slot, *view))).collect();
    keys.push(Piece::of(SLOT_SAID, PLAIN));
    keys
}
because!(key_pieces, OneChangeSearch, "the pieces that can be the key of a table, in the order in which they are tried: one word at an end of a part or next to it, for each part; then what the fixed word says");

fn most_of(counts: &Kept<WordRow, usize>) -> usize {
    counts.most().map_or(0, |(_, count)| count)
}
because!(most_of, OneChangeSearch, "the count of the most frequent word of a key");

impl<'a> Search<'a> {
    pub fn grammar_of(lessons: &[(Vec<Word>, WordRow)], start: Option<&Grammar>, light: &'a Light<'a>) -> Grammar {
        let first = match start {
            None => {
                let counted = counted_table(lessons, &|word| light.rank_of(word));
                let mut grammar = Grammar::default();
                grammar.table = WordSays::of(counted.table.into_iter().collect());
                grammar.out = Arc::new(counted.out);
                grammar.copies = counted.copies;
                grammar.settle();
                grammar
            }
            Some(start) => {
                let mut grammar = start.with_state(ROW_KEY, &start.state(ROW_KEY));
                grammar.copy = false;
                grammar
            }
        };
        let mut unique: Kept<(Vec<Word>, WordRow), ()> = Kept::default();
        for lesson in lessons {
            unique.put(lesson.clone(), ());
        }
        let mut seen: Vec<(Vec<Word>, WordRow)> = unique.into_pairs().map(|(lesson, _)| lesson).collect();
        seen.sort_by_key(|(words, _)| words.len());
        seen.truncate(SEARCH_LESSONS_MOST);
        let mut by_word: HashMap<Word, Vec<usize>> = HashMap::new();
        for (number, (words, _)) in seen.iter().enumerate() {
            let told: HashSet<Word> = words.iter().copied().collect();
            for word in told {
                by_word.entry(word).or_default().push(number);
            }
        }
        let mut search = Search { seen, by_word, right: Vec::new(), began: light.meter.words.get(), done: 0, light };
        search.run(first)
    }

    fn over(&self) -> bool {
        self.light.meter.words.get() - self.began >= SEARCH_WORK_MOST.min(SEARCH_WORK_FIRST + SEARCH_WORK_EACH * self.done as u64)
    }

    fn touched(&self, things: &[Word]) -> Vec<usize> {
        if things.contains(&ROW_KEY) {
            return (0..self.seen.len()).collect();
        }
        let mut found: Vec<usize> = things.iter().filter_map(|thing| self.by_word.get(thing)).flatten().copied().collect();
        found.sort_unstable();
        found.dedup();
        found
    }

    fn reads(&self, other: &Held, lesson: usize) -> bool {
        if let Some(known) = other.memo.borrow().get(&lesson) {
            return *known;
        }
        let (words, target) = &self.seen[lesson];
        let right = other.grammar.reads(words, self.light).as_ref() == Some(target);
        other.memo.borrow_mut().insert(lesson, right);
        right
    }

    fn gain_of(&self, things: &[Word], other: &Held) -> (isize, Vec<usize>) {
        let now = self.touched(things);
        let fixed: Vec<usize> = now.iter().copied().filter(|lesson| !self.right[*lesson] && self.reads(other, *lesson)).collect();
        let mut broken = 0;
        for lesson in &now {
            if self.right[*lesson] && !self.reads(other, *lesson) {
                broken += 1;
                if broken >= fixed.len() {
                    break;
                }
            }
        }
        (fixed.len() as isize - broken as isize, fixed)
    }

    fn weights(&self, other: &Held, things: &[Word]) -> Vec<Weight> {
        let mut weights: Vec<Weight> = things.iter().map(|thing| self.light.weight(&other.grammar.state(*thing))).collect();
        weights.sort();
        weights
    }

    fn by_states(&self, proposed: &Proposed) -> Vec<ChangeGroup> {
        let mut groups: Kept<Content, Vec<Proposal>> = Kept::default();
        for (key, entry) in proposed.pairs() {
            groups.or_put(key.states.clone(), Vec::new).push(entry.clone());
        }
        let mut found: Vec<ChangeGroup> = groups.into_pairs().map(|(content, mut entries)| {
            entries.sort_by_key(|entry| Reverse(entry.lessons));
            ChangeGroup { content, entries }
        }).collect();
        found.sort_by_cached_key(|group| {
            let top = &group.entries[0];
            (Reverse(top.lessons), self.weights(&top.other, &top.things), top.things.iter().map(|thing| self.light.rank_of(*thing)).collect::<Vec<u32>>())
        });
        found
    }

    fn placed(&self, choice: &RoundChoice, groups: &[ChangeGroup]) -> Best {
        let (mut gain, mut things, mut other) = (choice.gain, choice.things.clone(), choice.other.clone());
        for entry in &groups[choice.group].entries {
            if !Rc::ptr_eq(&entry.other, &choice.other) {
                let (found, _) = self.gain_of(&entry.things, &entry.other);
                if found > gain {
                    (gain, things, other) = (found, entry.things.clone(), entry.other.clone());
                }
            }
        }
        (gain, things, other)
    }

    fn taken(&self, proposed: &Proposed, forced: bool, beside: Option<&Proposed>, sure: bool) -> Option<Best> {
        let mut groups = self.by_states(proposed);
        let mut gains: Vec<RoundChoice> = Vec::new();
        for (number, group) in groups.iter().enumerate().take(TRIED_MOST) {
            let mut best: Option<RoundChoice> = None;
            for entry in group.entries.iter().take(PLACES) {
                let (gain, fixed) = self.gain_of(&entry.things, &entry.other);
                if gain > 0 && best.as_ref().is_none_or(|top| gain > top.gain) {
                    best = Some(RoundChoice { gain, things: entry.things.clone(), other: entry.other.clone(), fixed, group: number });
                }
            }
            if let Some(best) = best {
                gains.push(best);
                if gains.len() >= TRIED {
                    break;
                }
            }
        }
        gains.sort_by_cached_key(|choice| (Reverse(choice.gain), self.weights(&choice.other, &choice.things)));
        if forced {
            return gains.first().filter(|choice| choice.gain >= FORCED_LESSONS as isize).map(|choice| self.placed(choice, &groups));
        }
        if gains.is_empty() {
            return None;
        }
        let own = groups.len();
        if let Some(beside) = beside {
            groups.extend(self.by_states(beside));
        }
        let mut rivals: HashMap<Word, Vec<(&State, &Proposal)>> = HashMap::new();
        for group in &groups {
            for (thing, state) in &group.content {
                rivals.entry(*thing).or_default().push((state, &group.entries[0]));
            }
        }
        let mut waiting: Vec<(&Content, isize)> = Vec::new();
        for choice in &gains {
            let content = &groups[choice.group].content;
            if choice.gain < FORCED_LESSONS as isize && content.iter().any(|(_, state)| *state == State::Out) {
                continue;
            }
            let differs = |other: &Content| other.iter().any(|(thing, state)| content.iter().any(|(mine, held)| mine == thing && held != state));
            if waiting.iter().any(|(other, gain)| *gain > choice.gain && differs(other)) {
                waiting.push((content, choice.gain));
                continue;
            }
            let mine = choice.other.grammar.turns_and_cuts(&choice.things);
            let mut seen_rivals: Vec<*const Held> = Vec::new();
            let mut alone = true;
            'things: for (thing, state) in content {
                for (other_state, entry) in rivals.get(thing).map(Vec::as_slice).unwrap_or_default() {
                    let rival = &entry.other;
                    if *other_state == state || seen_rivals.contains(&Rc::as_ptr(rival)) {
                        continue;
                    }
                    seen_rivals.push(Rc::as_ptr(rival));
                    if choice.fixed.iter().all(|lesson| self.reads(rival, *lesson)) {
                        let now = self.touched(&entry.things);
                        let reads = now.iter().filter(|lesson| self.reads(rival, **lesson)).count() as isize;
                        let its = reads - now.iter().filter(|lesson| self.right[**lesson]).count() as isize;
                        let theirs = rival.grammar.turns_and_cuts(&entry.things);
                        let simpler = mine.0 == theirs.0 && mine.1 < theirs.1;
                        if its > choice.gain || (its == choice.gain && !simpler) {
                            alone = false;
                            break 'things;
                        }
                    }
                }
            }
            if alone || (sure && choice.gain >= SURE_LESSONS as isize) {
                return Some(self.placed(choice, &groups[..own]));
            }
            waiting.push((content, choice.gain));
        }
        None
    }

    fn rows_of(&self, other: &Grammar, word: Word, form: &FormState) -> Vec<(PartAnswers, WordRow)> {
        let probe = other.with_state(word, &State::form(form.shape, None, form.place, form.said.clone()));
        let mut found: Vec<(PartAnswers, WordRow)> = Vec::new();
        for lesson in self.by_word.get(&word).map(Vec::as_slice).unwrap_or_default().iter().take(LOOK_LESSONS) {
            let (words, target) = &self.seen[*lesson];
            let mut kept: Option<PartAnswers> = None;
            let Ok(Some(last)) = probe.read(words, self.light.meter, Reading::solving(&mut kept)) else { continue };
            if let (Some(given), Some(row)) = (kept, solved_row(&last, target)) {
                found.push((given, row));
            }
        }
        found
    }

    fn fill(&self, other: &Grammar) -> Lookup {
        let mut counted: Kept<(Word, Program), Kept<usize, Kept<WordRow, Kept<WordRow, usize>>>> = Kept::default();
        for (word, (shape, program)) in other.forms.pairs() {
            let Some(program) = program.as_ref().filter(|program| program.looks()) else { continue };
            let State::Form(form) = other.state(*word) else { continue };
            let rows = self.rows_of(other, *word, &form);
            let tables = counted.or_put((shape.owner(*word), program.clone()), Kept::default);
            for (given, row) in rows {
                for (piece, key, value) in laid_on(&whole_pieces(program), &given, &row).unwrap_or_default() {
                    if let Some(key) = key {
                        tables.or_put(piece, Kept::default).or_put(key, Kept::default).count(value, 1);
                    }
                }
            }
        }
        let mut lookup: Lookup = HashMap::new();
        for (owner, tables) in counted.into_pairs() {
            let mut pieces: ProgramTables = HashMap::new();
            for (piece, keys) in tables.into_pairs() {
                let mut table: HashMap<WordRow, WordRow> = HashMap::new();
                for (key, found) in keys.into_pairs() {
                    if let Some((word, count)) = found.most() {
                        if found.total() >= FORCED_LESSONS && sure_of(count, found.total()) {
                            table.insert(key, word.clone());
                        }
                    }
                }
                pieces.insert(piece, table);
            }
            lookup.insert(owner, pieces);
        }
        lookup
    }

    fn offer(&self, proposed: &mut Proposed, things: Vec<Word>, mut other: Grammar, lessons: usize) {
        other.lookup = Arc::new(self.fill(&other));
        let key = other.key(&things);
        if !proposed.holds(&key) {
            proposed.put(key, Proposal { lessons, things, other: Held::of(other) });
        }
    }

    fn top_place(&self, grammar: &Grammar, word: Word, place: usize) -> usize {
        place.min(grammar.places(word, None).into_iter().max().unwrap_or_default())
    }

    fn filled(&self, g: &Grammar, groups: &LookGroups, proposed: &mut Proposed) {
        for ((shape, place, owner), (words, programs)) in groups.pairs() {
            let mut rows: HashMap<Word, Vec<(PartAnswers, WordRow)>> = HashMap::new();
            for (word, more) in words.pairs() {
                if self.over() {
                    return;
                }
                rows.insert(*word, self.rows_of(g, *word, &FormState { shape: *shape, program: None, place: *place, said: more.clone() }));
            }
            let lies = |pieces: &[Option<Piece>], word: &Word| rows[word].iter().filter(|(given, row)| laid_on(pieces, given, row).is_some()).count();
            let same_shape = |word: &Word| g.forms.get(word).is_none_or(|(held, _)| held == shape);
            let with_owner: &[usize] = self.by_word.get(owner).map(Vec::as_slice).unwrap_or_default();
            let mut there: Vec<&Program> = Vec::new();
            for (held, program) in g.forms.values() {
                if let Some(program) = program.as_ref().filter(|program| held == shape && program.looks() && !there.contains(program)) {
                    there.push(program);
                }
            }
            for program in there {
                for (word, more) in words.pairs() {
                    let count = lies(&whole_pieces(program), word);
                    let same_program = g.forms.get(word).is_some_and(|(_, held)| held.as_ref() == Some(program));
                    if count >= FORCED_LESSONS && !same_program && same_shape(word) {
                        let state = State::form(*shape, Some(program.clone()), self.top_place(g, *word, *place), more.clone());
                        self.offer(proposed, vec![*word], g.with_state(*word, &state), count);
                    }
                }
            }
            for pieces in programs.keys() {
                let mut members: Kept<Word, Option<WordRow>> = Kept::default();
                for (word, more) in words.pairs() {
                    if same_shape(word) && lies(pieces, word) >= FORCED_LESSONS {
                        members.put(*word, more.clone());
                    }
                }
                if members.is_empty() {
                    continue;
                }
                let mut seen_rows: Vec<(Word, &PartAnswers, Vec<(usize, Option<WordRow>, WordRow)>)> = Vec::new();
                for word in members.keys() {
                    for (given, row) in &rows[word] {
                        if let Some(got) = laid_on(pieces, given, row) {
                            seen_rows.push((*word, given, got));
                        }
                    }
                }
                let mut program: Vec<Option<Piece>> = pieces.clone();
                let mut fixed_words: Vec<(usize, WordRow)> = Vec::new();
                let mut apart: HashMap<usize, HashMap<Word, u8>> = HashMap::new();
                let mut whole = true;
                for number in (0..pieces.len()).filter(|number| pieces[*number].is_none()) {
                    let words_here: Vec<WordRow> = seen_rows.iter().map(|(_, _, got)| got.iter().find(|(piece, _, _)| *piece == number).map(|(_, _, value)| value.clone()).unwrap_or_default()).collect();
                    let one_word = words_here.iter().all(|value| *value == words_here[0]);
                    let in_answers = with_owner.iter().filter(|lesson| words_here[0].first().is_some_and(|word| self.seen[**lesson].1.contains(word))).count();
                    if one_word && sure_of(in_answers, with_owner.len()) {
                        fixed_words.push((number, words_here[0].clone()));
                        continue;
                    }
                    let mut best: Option<(usize, Piece)> = None;
                    let mut split: Option<(Piece, Vec<Vec<Word>>)> = None;
                    for key_piece in key_pieces() {
                        let mut by_key: Kept<WordRow, Kept<WordRow, usize>> = Kept::default();
                        let mut per: Kept<Word, Kept<WordRow, Kept<WordRow, usize>>> = Kept::default();
                        let mut every = true;
                        for ((word, given, _), value) in seen_rows.iter().zip(&words_here) {
                            let key = given.row(key_piece.slot as usize).filter(|part| !part.is_empty()).map(|part| key_piece.view.on(part)).unwrap_or_default();
                            if key.len() != 1 {
                                every = false;
                                break;
                            }
                            by_key.or_put(key.clone(), Kept::default).count(value.clone(), 1);
                            per.or_put(*word, Kept::default).or_put(key, Kept::default).count(value.clone(), 1);
                        }
                        if !every || by_key.is_empty() {
                            continue;
                        }
                        let agree: usize = by_key.values().map(most_of).sum();
                        let support: usize = by_key.values().filter(|found| found.total() >= FORCED_LESSONS).map(most_of).sum();
                        if sure_of(agree, words_here.len()) {
                            if best.as_ref().is_none_or(|(top, _)| support > *top) {
                                best = Some((support, key_piece.under(LOOKED)));
                            }
                        } else if best.is_none() {
                            let mut order: Vec<&Word> = per.keys().collect();
                            order.sort_by_cached_key(|word| Reverse(per.get(word).map_or(0, |keys| keys.values().map(Kept::total).sum::<usize>())));
                            let mut sets: Vec<(Vec<Word>, Kept<WordRow, WordRow>)> = Vec::new();
                            for word in order {
                                let Some(keys) = per.get(word) else { continue };
                                let mine: Vec<(WordRow, WordRow)> = keys.pairs().filter_map(|(key, found)| found.most().map(|(value, _)| (key.clone(), value.clone()))).collect();
                                let fits = sets.iter().position(|(_, table)| mine.iter().all(|(key, value)| table.get(key).is_none_or(|held| held == value)));
                                match fits {
                                    Some(set) => {
                                        sets[set].0.push(*word);
                                        for (key, value) in mine {
                                            sets[set].1.put(key, value);
                                        }
                                    }
                                    None => {
                                        let mut table: Kept<WordRow, WordRow> = Kept::default();
                                        for (key, value) in mine {
                                            table.put(key, value);
                                        }
                                        sets.push((vec![*word], table));
                                    }
                                }
                            }
                            let inside: usize = per.values().flat_map(Kept::values).map(most_of).sum();
                            if sets.len() <= TABLES_MOST && sure_of(inside, words_here.len()) && split.as_ref().is_none_or(|(_, held)| sets.len() < held.len()) {
                                split = Some((key_piece, sets.into_iter().map(|(members, _)| members).collect()));
                            }
                        }
                    }
                    if let Some((_, piece)) = best {
                        program[number] = Some(piece);
                    } else if let Some((piece, sets)) = split {
                        program[number] = Some(piece.under(LOOKED));
                        let mut tables: HashMap<Word, u8> = HashMap::new();
                        for (set, members) in sets.iter().enumerate() {
                            for word in members {
                                tables.insert(*word, LOOKED + set as u8);
                            }
                        }
                        apart.insert(number, tables);
                    } else {
                        whole = false;
                        break;
                    }
                }
                if !whole || fixed_words.len() > PARTS_OF_FORM {
                    continue;
                }
                let mut other = g.clone();
                let mut things: Vec<Word> = members.keys().copied().collect();
                if !fixed_words.is_empty() {
                    let said: WordRow = fixed_words.iter().flat_map(|(_, value)| value.iter().copied()).collect();
                    let second = shape.second.filter(|second| g.table.get(second).is_none_or(|held| *held == said) && !g.forms.holds(second));
                    let slot = if let Some(second) = second {
                        other = other.with_state(second, &State::Table(said.clone()));
                        things.push(second);
                        SLOT_SECOND
                    } else if members.values().all(Option::is_none) {
                        for (_, more) in members.pairs_mut() {
                            *more = Some(said.clone());
                        }
                        SLOT_SAID
                    } else {
                        continue;
                    };
                    for (order, (number, _)) in fixed_words.iter().enumerate() {
                        let end = if order == 0 { FIRST_OF } else { LAST_OF };
                        let view = if fixed_words.len() == 1 { PLAIN } else { AnswerView { first: end, then: NO_OPERATION } };
                        program[*number] = Some(Piece::of(slot, view));
                    }
                }
                for (word, more) in members.pairs() {
                    let mine: Vec<Piece> = program.iter().enumerate().filter_map(|(number, piece)| piece.map(|piece| match apart.get(&number) {
                        Some(tables) => piece.under(tables.get(word).copied().unwrap_or(LOOKED)),
                        None => piece,
                    })).collect();
                    let place = self.top_place(&other, *word, *place);
                    other = other.with_state(*word, &State::form(*shape, Some(Program { pieces: mine, times: 1 }), place, more.clone()));
                }
                self.offer(proposed, things, other, seen_rows.len());
            }
        }
    }

    fn propose(&self, proposed: &mut Proposed, lesson_keys: &mut HashSet<GrammarKey>, other: Grammar, things: Vec<Word>) {
        let key = other.key(&things);
        if !lesson_keys.insert(key.clone()) {
            return;
        }
        match proposed.get_mut(&key) {
            Some(entry) => entry.lessons += 1,
            None => {
                proposed.put(key, Proposal { lessons: 1, things, other: Held::of(other) });
            }
        }
    }

    fn one_change(&self, held: &Rc<Held>, open: &[usize], lacks: &HashMap<usize, Vec<Word>>, views: &[AnswerView]) -> (Proposed, Option<Best>) {
        let g = &held.grammar;
        let none: Vec<usize> = open.iter().copied().filter(|lesson| lacks[lesson].is_empty()).collect();
        let mut single: Vec<usize> = none.iter().copied().take(PROPOSE_BATCH).collect();
        single.extend(open.iter().copied().filter(|lesson| lacks[lesson].len() == 1));
        single.extend(none.iter().copied().take(NONE_MOST).skip(PROPOSE_BATCH));
        let mut proposed: Proposed = Kept::default();
        let mut before: Option<HashSet<Content>> = None;
        for batch in 0..BATCHES {
            let mut groups: LookGroups = Kept::default();
            for lesson in single.iter().skip(batch * PROPOSE_BATCH).take(PROPOSE_BATCH) {
                let (words, target) = &self.seen[*lesson];
                let lacking = &lacks[lesson];
                let mut mine: HashSet<GrammarKey> = HashSet::new();
                let mut explained = false;
                let mut pending: Vec<Pending> = Vec::new();
                let mut distinct: Vec<Word> = Vec::new();
                for word in words {
                    if !distinct.contains(word) {
                        distinct.push(*word);
                    }
                }
                let mut asked: Vec<Word> = if lacking.is_empty() { distinct.clone() } else { lacking.clone() };
                if lacking.is_empty() || g.row.is_none() {
                    asked.push(ROW_KEY);
                }
                for word in asked {
                    if self.over() {
                        break;
                    }
                    for state in g.changes(words, target, word, views, None, false, None, self.light) {
                        explained = true;
                        self.propose(&mut proposed, &mut mine, g.with_state(word, &state), vec![word]);
                    }
                }
                if !explained && g.row.is_some() && !self.over() {
                    for unknown in lacking {
                        for (word, state) in g.asked(words, target, *unknown, views, Some(&mut pending), self.light) {
                            explained = true;
                            self.propose(&mut proposed, &mut mine, g.with_state(word, &state), vec![word, *unknown]);
                        }
                    }
                    if lacking.is_empty() {
                        for word in &distinct {
                            if g.table.get(word).is_some() && !self.over() {
                                for state in g.changes(words, target, *word, views, None, true, Some(&mut pending), self.light) {
                                    explained = true;
                                    self.propose(&mut proposed, &mut mine, g.with_state(*word, &state), vec![*word]);
                                }
                            }
                        }
                    }
                    if !explained {
                        for found in pending {
                            let group = groups.or_put((found.shape, found.place, found.shape.owner(found.word)), || (Kept::default(), Kept::default()));
                            group.0.put(found.word, found.said);
                            group.1.put(found.pieces, ());
                        }
                    }
                }
            }
            if !groups.is_empty() && !self.over() {
                self.filled(g, &groups, &mut proposed);
            }
            let best = self.taken(&proposed, false, None, true);
            if best.is_some() || (batch + 1) * PROPOSE_BATCH >= single.len() || self.over() {
                return (proposed, best);
            }
            let now: HashSet<Content> = proposed.keys().map(|key| key.states.clone()).collect();
            if !now.is_empty() && before.as_ref() == Some(&now) {
                break;
            }
            before = Some(now);
        }
        (proposed, None)
    }

    fn two_changes(&self, held: &Rc<Held>, open: &[usize], lacks: &HashMap<usize, Vec<Word>>, views: &[AnswerView]) -> (Proposed, Votes) {
        let g = &held.grammar;
        let mut proposed: Proposed = Kept::default();
        let mut places: HashMap<GrammarKey, HashSet<usize>> = HashMap::new();
        for lesson in open.iter().filter(|lesson| (1..=PARTS_OF_FORM).contains(&lacks[*lesson].len())).take(SEVERAL_LESSONS) {
            if self.over() {
                break;
            }
            let (words, target) = &self.seen[*lesson];
            let mut mine: HashSet<GrammarKey> = HashSet::new();
            let mut sorted: Vec<Word> = lacks[lesson].clone();
            sorted.sort_by_key(|word| self.light.rank_of(*word));
            let mut with_row = sorted.clone();
            with_row.push(ROW_KEY);
            for things in [sorted, with_row] {
                if things.len() == PARTS_OF_FORM && (!things.contains(&ROW_KEY) || g.row.is_none()) {
                    for other in g.explain(words, target, &things, views, self.light) {
                        let mut changed = things.clone();
                        changed.extend(other.extra.iter().flatten().copied());
                        self.propose(&mut proposed, &mut mine, other, changed);
                    }
                }
            }
            for key in mine {
                places.entry(key).or_default().insert(*lesson);
            }
        }
        let mut votes: Votes = Kept::default();
        for (key, entry) in proposed.pairs() {
            if entry.lessons >= FORCED_LESSONS {
                for thing in &entry.things {
                    let state = entry.other.grammar.state(*thing);
                    let vote = votes.or_put((*thing, state.content()), || (HashSet::new(), state));
                    vote.0.extend(places.get(key).into_iter().flatten().copied());
                }
            }
        }
        (proposed, votes)
    }

    fn voted(&self, held: &Rc<Held>, votes: &Votes) -> Option<Best> {
        let g = &held.grammar;
        let mut best: Kept<Word, Vec<(usize, Weight, &State)>> = Kept::default();
        for ((thing, _), (lessons, state)) in votes.pairs() {
            best.or_put(*thing, Vec::new).push((lessons.len(), self.light.weight(state), state));
        }
        let mut found: Option<(usize, Word, &State)> = None;
        for (thing, states) in best.pairs_mut() {
            states.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| one.1.cmp(&other.1)));
            let alone = states.len() == 1 || states[0].0 > states[1].0;
            if states[0].0 >= FORCED_LESSONS && alone {
                let rank = (states[0].0, self.light.rank_of(*thing));
                if found.is_none_or(|(count, top, _)| rank > (count, self.light.rank_of(top))) {
                    found = Some((states[0].0, *thing, states[0].2));
                }
            }
        }
        let (_, thing, state) = found?;
        let state = match state {
            State::Form(form) => State::form(form.shape, form.program.clone(), self.top_place(g, thing, form.place), form.said.clone()),
            other => other.clone(),
        };
        Some((0, vec![thing], Held::of(g.with_state(thing, &state))))
    }

    fn run(&mut self, first: Grammar) -> Grammar {
        let mut first = first;
        if first.forms.values().any(|(_, program)| program.as_ref().is_some_and(Program::looks)) {
            first.lookup = Arc::new(self.fill(&first));
        }
        self.right = self.seen.iter().map(|(words, target)| first.reads(words, self.light).as_ref() == Some(target)).collect();
        self.began = self.light.meter.words.get();
        self.done = self.right.iter().filter(|right| **right).count();
        let mut g = Held::of(first);
        for _ in 0..SEARCH_CHANGES_MOST {
            let open: Vec<usize> = (0..self.seen.len()).filter(|lesson| !self.right[*lesson]).collect();
            let lacks: HashMap<usize, Vec<Word>> = open.iter().map(|lesson| (*lesson, g.grammar.unknown(&self.seen[*lesson].0))).collect();
            let (ones, mut best) = self.one_change(&g, &open, &lacks, VIEWS);
            if best.is_none() && !self.over() {
                let (twos, votes) = self.two_changes(&g, &open, &lacks, VIEWS);
                best = self.taken(&twos, false, Some(&ones), false).or_else(|| self.voted(&g, &votes)).or_else(|| self.taken(&ones, true, None, false)).or_else(|| self.taken(&twos, true, None, false));
            }
            let Some((_, things, next)) = best else { break };
            g = next;
            for lesson in self.touched(&things) {
                self.right[lesson] = self.reads(&g, lesson);
            }
            self.done = self.right.iter().filter(|right| **right).count();
            if self.over() {
                break;
            }
        }
        let stopped = self.over();
        let mut used: HashSet<Word> = HashSet::new();
        for (lesson, (words, _)) in self.seen.iter().enumerate() {
            if self.right[lesson] {
                let how = Reading { strict: true, apply: crate::tools::parts::grammar::Apply::Run, used: Some(&mut used), hold: &[] };
                let _ = g.grammar.read(words, self.light.meter, how);
            }
        }
        let idle: Vec<Word> = g.grammar.order.iter().flatten().copied().filter(|key| !used.contains(key)).collect();
        for key in idle {
            let state = g.grammar.table.get(&key).map_or(State::NotKnown, |row| State::Table(row.clone()));
            let other = Held::of(g.grammar.with_state(key, &state));
            let now = self.touched(&[key]);
            if now.iter().filter(|lesson| self.right[**lesson]).all(|lesson| self.reads(&other, *lesson)) {
                g = other;
                for lesson in now {
                    self.right[lesson] = self.reads(&g, lesson);
                }
            }
        }
        let mut grammar = g.grammar.clone();
        let plain: Vec<&Word> = grammar.table.keys().filter(|word| !grammar.forms.holds(word) && !grammar.marks.contains(word)).collect();
        let themselves = plain.iter().filter(|word| grammar.table.get(**word).is_some_and(|row| *row == [***word])).count();
        grammar.copy = plain.len() >= SURE_LESSONS && sure_of(themselves, plain.len());
        grammar.right = self.right.iter().filter(|right| **right).count();
        grammar.seen = self.seen.len();
        grammar.stopped = stopped;
        grammar
    }
}
