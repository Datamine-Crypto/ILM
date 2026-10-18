use crate::cases::Case;
use crate::tools::explain::chain::{Facts, Time};
use crate::tools::explain::text::{Condition, LogicPiece, Moment, Pattern, ToldRule, Topic, fact_numbers, fact_words, plain_piece, text_words, word_not_mark};
use crate::tools::explain::wording::{WHOLE_SHARE, bounded_facts, fact_values, frame_words, places_of};
use crate::tools::explain::words::{WordId, word_number, word_text};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

pub struct StatesOverTime;
source!(StatesOverTime, "a state that holds from one event of a story until another, as the event calculus of Kowalski and Sergot lets an event start a fluent and a later event end it: when no fact of a text names what holds between two told events (who holds a thing from the time that it was got until it was dropped), a learner invents that fact, named from the words of the event that starts it, so that a rule may read it at the time of a later event; the engine reads such a state as it reads any fact that only one fact may hold, by its latest value up to a time, and the event that ends it writes the state with no value");

pub type StateRule = (Pattern, Pattern, Vec<bool>);
because!(StateRule, StatesOverTime, "a rule of an invented state: the event, the state that it makes or ends, and for each place whether every lesson gave it one word, so that the lines of the state name such a place as a place of one word, while the bridges of the state keep the places as the sentences of the text name them");

pub struct StateCandidate {
    pub starts: Vec<StateRule>,
    pub ends: Vec<StateRule>,
}
because!(StateCandidate, StatesOverTime, "an invented state as rules of the library: each event that starts it with the state that it makes, the thing whose state it is first and its value last, and each event that ends it with the same state with no value");

#[derive(Default)]
pub struct StateCounts {
    pub lessons: usize,
    pub events: usize,
    pub pairs: usize,
    pub kept: usize,
    pub things: usize,
    pub told: usize,
    pub explained: usize,
    pub steps: usize,
    pub below: Vec<(Pattern, usize, usize)>,
}
because!(StateCounts, StatesOverTime, "what the search for invented states read, for a reader of its trace: the lessons whose stories it read, the events of two places in them, the pairs of events that name the same things and how many of those enough lessons tell, the asked things whose told events it followed and those told events, and the steps between them that the roles of their sentences explain, of all steps, and the told sentences of asked things that too few lessons hold for a state, each with the lessons that hold it, of how many; a search that reads no event or no pair tells at once why it invented nothing, and a sentence under the bar is one that no state reads");

const STATE_PLACES: usize = 2;
because!(STATE_PLACES, StatesOverTime, "the places of an event that starts a state: the thing whose state it is and the value of the state (a person gets a thing: the thing is held, and by that person)");

const LEAST_STATE_PERCENT: usize = 10;
because!(LEAST_STATE_PERCENT, StatesOverTime, "the least share of the lessons whose stories tell an event and a later one that names the same things, for a state between the two to be tried: a state that the questions of a kind rest on comes in many of its stories");

const ROLE_PERCENT: usize = 90;
because!(ROLE_PERCENT, StatesOverTime, "the least share of the steps between the told events of one thing that the roles of their sentences must explain (a start after an end, an end after a start, two sentences of one role at one time), for those sentences to start and end one state together: the events of a thing that one holds and lets go alternate, while sentences that a story mixes freely explain about half");

const QUESTION_FRAME_PERCENT: usize = 50;
because!(QUESTION_FRAME_PERCENT, StatesOverTime, "the least share of the questions of a kind that hold a word for that word to be of the frame of the questions (where, is, the), and not a thing that they ask about");

const MOST_STATES: usize = 4;
because!(MOST_STATES, StatesOverTime, "the most invented states that a learner tries, the pairs of events that the most stories tell first: each try learns the rules of the concept again");

const ARTICLES: &[&str] = &["the", "a", "an"];
because!(ARTICLES, StatesOverTime, "the words before a thing that name no event, so that the name of a state is the first other word of the event that starts it (got, dropped)");

const STATE_ARTICLE: &str = "the";
because!(STATE_ARTICLE, StatesOverTime, "the first word of the name of a state, as English names a state of a thing (the holder of the ball)");

const STATE_OF: &str = "of";
because!(STATE_OF, StatesOverTime, "the word between the name of a state and the thing whose state it is");

const STATE_IS: &str = "is";
because!(STATE_IS, StatesOverTime, "the word before the value of a state, so that the value stands last, where the engine reads the value of a fact that only one fact may hold (end focus)");

const STATE_ENDED: &[&str] = &["no", "more"];
because!(STATE_ENDED, StatesOverTime, "the words that stand for the value of a state that an event ended: no person or place is named so, so the state then holds no value that a rule reads, as the library writes 'the place of A is B no more'");

type Event = (Time, usize, Vec<Vec<WordId>>);
because!(Event, StatesOverTime, "a fact of a story at a place in the story, with the sentence of the text that reads it and the words that fill each of its places");

fn story_events(facts: &Facts, base: &Facts, patterns: &[Pattern]) -> Vec<Event> {
    let first = base.added_to(&Facts::default());
    let mut out: Vec<Event> = Vec::new();
    for place in first..first + facts.added_to(base) {
        let Some((pattern, values)) = fact_values(facts.fact(place), patterns) else { continue };
        let Some(form) = patterns.iter().position(|known| *known == pattern) else { continue };
        out.extend(facts.times_of(place).iter().map(|time| (*time, form, values.clone())));
    }
    out.sort_by_key(|(time, _, _)| *time);
    out
}
because!(story_events, StatesOverTime, "the facts of one story that a sentence of the text reads, each at every place in the story at which it came, in the order of the story, so that an event is known before the events after it");

fn key_article(pattern: &Pattern, place: usize) -> Option<WordId> {
    let at = pattern.iter().position(|logic_piece| matches!(logic_piece, LogicPiece::Open(known) | LogicPiece::OpenWithEnd(known, _) if *known == place))?;
    match &pattern[at.checked_sub(1)?] {
        LogicPiece::Word(word) if ARTICLES.contains(&word_text(*word)) => Some(*word),
        _ => None,
    }
}
because!(key_article, StatesOverTime, "the article that stands before the thing whose state it is in the event that starts the state (the B of A got the B), which the state keeps before that thing");

pub(super) fn event_word(event: &Pattern) -> Option<WordId> {
    event.iter().find_map(|logic_piece| match logic_piece {
        LogicPiece::Word(word) if word_not_mark(word_text(*word)) && !ARTICLES.contains(&word_text(*word)) => Some(*word),
        _ => None,
    })
}
because!(event_word, StatesOverTime, "the first word of a sentence of an event that is no article and no mark, which names the event (got, dropped); a sentence with none (V B the C, whose verb is a place) names no event of its own, as it reads every sentence of its shape");

fn state_frame(name: WordId, article: Option<WordId>, key: usize) -> Pattern {
    [LogicPiece::Word(word_number(STATE_ARTICLE)), LogicPiece::Word(name), LogicPiece::Word(word_number(STATE_OF))].into_iter().chain(article.map(LogicPiece::Word)).chain([LogicPiece::Open(key), LogicPiece::Word(word_number(STATE_IS))]).collect()
}
because!(state_frame, StatesOverTime, "the words of a state before its value: its name, then the thing whose state it is at the given place, with the article of the starting event, the same words in every rule of the state, so that the engine reads all its values as values of one state");

fn state_made(name: WordId, article: Option<WordId>, (event, key, value): (&Pattern, usize, Pattern)) -> Option<(Pattern, Pattern)> {
    let made: Pattern = state_frame(name, article, key).into_iter().chain(value).collect();
    (made != *event).then(|| (event.clone(), made))
}
because!(state_made, StatesOverTime, "a rule that starts a state: the event, and the state of the thing at the given place with the value at the other places, with the words between them (either in the B or the C gives the value B or the C); nothing when the state is the event itself, as a fact that is a state already (the place of A is B) names no new one");

fn state_ended(name: WordId, article: Option<WordId>, (event, key): (&Pattern, usize)) -> (Pattern, Pattern) {
    (event.clone(), state_frame(name, article, key).into_iter().chain(STATE_ENDED.iter().map(|word| LogicPiece::Word(word_number(word)))).collect())
}
because!(state_ended, StatesOverTime, "a rule that ends a state: the event, and the state of the thing at the given place with no value");

fn state_of((start, value): (&Pattern, usize), (end, end_key): (&Pattern, usize), (start_narrow, end_narrow): (&[bool], &[bool])) -> Option<StateCandidate> {
    let places = places_of(start);
    let (value_place, key_place) = (places[value], places[STATE_PLACES - 1 - value]);
    let name = event_word(start)?;
    event_word(end)?;
    let article = key_article(start, key_place);
    let started = state_made(name, article, (start, key_place, vec![LogicPiece::Open(value_place)]))?;
    let end_place = *places_of(end).get(end_key)?;
    Some(StateCandidate { starts: vec![state_rule(started, start_narrow)], ends: vec![state_rule(state_ended(name, article, (end, end_place)), end_narrow)] })
}
because!(state_of, StatesOverTime, "the invented state between an event that starts it and one that ends it: named by the first word of the starting event that is no article (the got of the B is A, for A got the B), the thing whose state it is first, as the starting event writes it, and the value last; the ending event writes the same state of the same thing with no value; each rule with whether every story gave each of its places one word (state_rule); nothing when either event holds no word of its own, or when the state is the starting event itself");

fn state_rule((event, made): (Pattern, Pattern), narrow: &[bool]) -> StateRule {
    (event, made, narrow.to_vec())
}
because!(state_rule, StatesOverTime, "a rule of a state with whether every lesson gave each of its places one word: a line of the state names such a place as one of one word, as a learned rule does, since a place of more words reads the words after a thing as part of it (the kettle there, of Lucy took the kettle there), and a state of such a run names nothing that a question asks");

fn value_pieces(form: &Pattern, key: usize) -> Pattern {
    let at: Vec<usize> = form.iter().enumerate().filter(|(_, piece)| matches!(piece, LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) if *place != key)).map(|(at, _)| at).collect();
    match (at.first(), at.last()) {
        (Some(first), Some(last)) => form[*first..=*last].to_vec(),
        _ => Vec::new(),
    }
}
because!(value_pieces, StatesOverTime, "the value that a sentence gives the thing at a place: its other places with the words between them, as one value (the B or the C of A is either in the B or the C), so that a sentence that names two places at once gives one value of the state, which a later move replaces");

fn optional_end((short, short_key): &Role, (long, long_key): &Role) -> bool {
    short_key == long_key && long.len() == short.len() + 1 && long.starts_with(short) && matches!(long.last(), Some(LogicPiece::Word(_)))
}
because!(optional_end, StatesOverTime, "whether a told form is another with one more word at its end (A put down the B there, A put down the B): one event told with a word that adds no place, so that the lessons of the two count together for the support of a state and the two stand on one side of it, as a longer story tells the rare form of an event that the first lessons tell in the common one");

const ENDING_PERCENT: usize = 50;
because!(ENDING_PERCENT, StatesOverTime, "the least share of the uses of a person's sentence, after another event of that person, that repeat the value of the event before it, for a step into it to join it as an end: an end names the place that the person leaves (is no longer in the kitchen after moved to the kitchen) in most of its uses, while a start repeats the place before only when the person goes again where the person is, as one place of several");

const ASKED_VALUE_PERCENT: usize = 50;
because!(ASKED_VALUE_PERCENT, StatesOverTime, "the least share of the value words of a person's told event that the questions or answers of the lessons hold, for the event to be a step of the state that the questions ask about: an either names two places that the questions ask about, while a person who takes a thing (Mary took the apple) names a thing that no question about where the person is holds");

fn kept_one_word<K: Eq + Hash>(widths: &mut HashMap<K, Vec<bool>>, key: K, one_word: Vec<bool>) {
    let held = widths.entry(key).or_insert_with(|| one_word.clone());
    for (held, now) in held.iter_mut().zip(&one_word) {
        *held &= *now;
    }
}
because!(kept_one_word, StatesOverTime, "whether each place of a sentence held one word in every use of it so far: a place stays one of one word only while each use gives it one word");

type Role = (Pattern, usize);
because!(Role, StatesOverTime, "a sentence of a told event as a form, with the place in it of the thing whose state it is");

type ThingSteps = Vec<(Vec<Role>, Option<Vec<WordId>>)>;
because!(ThingSteps, StatesOverTime, "the told events of one thing or person in the order of the story, the sentences told at one time together, each with the value that it gives a person (the place where the person is), if it is the event of a person");

fn told_steps(lessons: &[&Case], (with_things, with_people, story_people): (bool, bool, bool)) -> (Vec<Vec<ThingSteps>>, Vec<Vec<ThingSteps>>, HashMap<Role, Vec<bool>>) {
    let mut uses: HashMap<String, usize> = HashMap::new();
    let mut capital: HashSet<String> = HashSet::new();
    let mut small: HashSet<String> = HashSet::new();
    for case in lessons {
        for word in fact_words(&case.question).into_iter().collect::<HashSet<String>>() {
            *uses.entry(word).or_default() += 1;
        }
        for sentence in &case.sentences {
            for written in text_words(sentence) {
                let word = written.to_lowercase();
                if written.chars().next().is_some_and(char::is_uppercase) { capital.insert(word) } else { small.insert(word) };
            }
        }
    }
    let frame: HashSet<String> = uses.into_iter().filter(|(_, count)| count * WHOLE_SHARE >= lessons.len() * QUESTION_FRAME_PERCENT).map(|(word, _)| word).collect();
    let name = |word: &String| capital.contains(word) && !small.contains(word);
    let asked_names: Vec<Option<String>> = lessons.iter().map(|case| match fact_words(&case.question).into_iter().filter(|word| name(word)).collect::<HashSet<String>>().into_iter().collect::<Vec<String>>().as_slice() {
        [asked_name] => Some(asked_name.clone()),
        _ => None,
    }).collect();
    let asking = asked_names.iter().filter(|asked_name| asked_name.is_some()).count() * WHOLE_SHARE >= lessons.len() * LEAST_STATE_PERCENT;
    let named = with_people && asking;
    let told = story_people && !asking;
    let told_frame = if named || told { frame_words(lessons) } else { HashSet::new() };
    let asked_words: HashSet<WordId> = lessons.iter().filter(|_| named || told).flat_map(|case| fact_words(&case.answer).into_iter().chain(if told { Vec::new() } else { fact_words(&case.question) })).map(|word| word_number(&word)).collect();
    let mut widths: HashMap<Role, Vec<bool>> = HashMap::new();
    let mut people: Vec<Vec<ThingSteps>> = Vec::new();
    let steps = lessons
        .iter()
        .enumerate()
        .map(|(lesson, case)| {
            let asked: HashSet<String> = fact_words(&case.question).into_iter().filter(|word| word_not_mark(word) && !frame.contains(word) && !name(word)).collect();
            let mut things: Vec<(Vec<WordId>, ThingSteps)> = Vec::new();
            for sentence in case.sentences.iter().filter(|_| with_things) {
                let mut form: Pattern = Vec::new();
                let mut values: Vec<(Vec<WordId>, bool)> = Vec::new();
                for word in fact_words(sentence) {
                    let thing = asked.contains(&word);
                    match values.last_mut() {
                        Some((run, true)) if thing && matches!(form.last(), Some(LogicPiece::Open(_))) => run.push(word_number(&word)),
                        _ if thing || name(&word) => {
                            form.push(LogicPiece::Open(values.len()));
                            values.push((vec![word_number(&word)], thing));
                        }
                        _ => form.push(LogicPiece::Word(word_number(&word))),
                    }
                }
                let ([(first, first_thing), (second, second_thing)], true) = (values.as_slice(), values.len() == STATE_PLACES) else { continue };
                let (thing, key) = match (first_thing, second_thing) {
                    (true, false) => (first, usize::default()),
                    (false, true) => (second, STATE_PLACES - 1),
                    _ => continue,
                };
                let at = things.iter().position(|(known, _)| known == thing).unwrap_or_else(|| {
                    things.push((thing.clone(), Vec::new()));
                    things.len() - 1
                });
                kept_one_word(&mut widths, (form.clone(), key), values.iter().map(|(run, _)| run.len() == 1).collect());
                things[at].1.push((vec![(form, key)], None));
            }
            let mut keys: Vec<String> = asked_names[lesson].iter().filter(|_| named).cloned().collect();
            if told {
                for sentence in &case.sentences {
                    for word in fact_words(sentence).into_iter().filter(|word| name(word)) {
                        if !keys.contains(&word) {
                            keys.push(word);
                        }
                    }
                }
            }
            let mut persons: Vec<ThingSteps> = Vec::new();
            for asked_name in &keys {
                let mut person: ThingSteps = Vec::new();
                for sentence in &case.sentences {
                    let mut form: Pattern = Vec::new();
                    let mut runs: Vec<(Vec<WordId>, bool)> = Vec::new();
                    for word in fact_words(sentence) {
                        let named_word = name(&word);
                        if !named_word && (!word_not_mark(&word) || told_frame.contains(&word)) {
                            form.push(LogicPiece::Word(word_number(&word)));
                            continue;
                        }
                        match runs.last_mut() {
                            Some((run, false)) if !named_word && matches!(form.last(), Some(LogicPiece::Open(_))) => run.push(word_number(&word)),
                            _ => {
                                form.push(LogicPiece::Open(runs.len()));
                                runs.push((vec![word_number(&word)], named_word));
                            }
                        }
                    }
                    let asked_word = vec![word_number(asked_name)];
                    if !person.is_empty() && !runs.iter().any(|(_, named_word)| *named_word) {
                        person.push((Vec::new(), None));
                        continue;
                    }
                    let Some(key) = runs.iter().position(|(run, named_word)| *named_word && *run == asked_word) else { continue };
                    if runs.len() < STATE_PLACES || runs.iter().enumerate().any(|(at, (_, named_word))| at != key && *named_word) {
                        continue;
                    }
                    let value: Vec<WordId> = runs.iter().enumerate().filter(|(at, _)| *at != key).flat_map(|(_, (run, _))| run.iter().copied().chain([WordId::MAX])).collect();
                    let words: Vec<&WordId> = value.iter().filter(|word| **word != WordId::MAX).collect();
                    if words.iter().filter(|word| asked_words.contains(**word)).count() * WHOLE_SHARE < words.len() * ASKED_VALUE_PERCENT {
                        continue;
                    }
                    kept_one_word(&mut widths, (form.clone(), key), runs.iter().map(|(run, _)| run.len() == 1).collect());
                    person.push((vec![(form, key)], Some(value)));
                }
                if !person.is_empty() {
                    persons.push(person);
                }
            }
            people.push(persons);
            things.into_iter().map(|(_, steps)| steps).collect()
        })
        .collect();
    (steps, people, widths)
}
because!(told_steps, StatesOverTime, "the told events of the things that the questions name, lesson by lesson: each told sentence as a form whose places are the names (words that the told sentences never write in small letters) and the run of a thing that the question of its lesson names (a word of the question that most questions do not hold), kept when it names one name and one thing, as a person does something with a thing; in the order of the story, each sentence one step of its thing; with whether each place of each form held one word in every use (a name always does, the run of a thing may hold more); and, lesson by lesson, the told events of the one person that its question names (or, when asked for, of each person that its story names), a told sentence that names no one (then she went to the kitchen) a gap in the events of each person, as it may be an event of that person that no name tells, so that no step joins the events on its two sides, and each sentence with that name and one or more other runs, runs of words outside the frame of the told sentences (the place, or the two places of an either), as the value of the event (only when enough questions for a state name one person each, as the frame of the told sentences costs a reading of every told sentence); the events of things and of persons only when asked for");

#[derive(Default)]
struct RoleParts {
    parent: Vec<usize>,
    other: Vec<bool>,
}
because!(RoleParts, StatesOverTime, "the sentences of told events joined into parts whose roles are known against each other, as a union of sets that keeps for each its parity against its parent (the bipartite union-find): grow makes room for more sentences, each its own part; root gives the sentence that stands for the part of a sentence and whether its role is the other one; join puts two sentences in one part, with the same role or the other, and says no when their part already says the opposite");

impl RoleParts {
    fn grow(&mut self, count: usize) {
        while self.parent.len() < count {
            self.parent.push(self.parent.len());
            self.other.push(false);
        }
    }

    fn root(&mut self, at: usize) -> (usize, bool) {
        let mut path: Vec<usize> = Vec::new();
        let mut top = at;
        while self.parent[top] != top {
            path.push(top);
            top = self.parent[top];
        }
        let mut flip = false;
        for node in path.iter().rev() {
            flip ^= self.other[*node];
            self.other[*node] = flip;
            self.parent[*node] = top;
        }
        (top, at != top && self.other[at])
    }

    fn join(&mut self, one: usize, two: usize, differ: bool) -> bool {
        let ((first, first_other), (second, second_other)) = (self.root(one), self.root(two));
        if first == second {
            return (first_other != second_other) == differ;
        }
        self.parent[second] = first;
        self.other[second] = first_other ^ second_other ^ differ;
        true
    }
}

fn role_state(lessons: &[Vec<ThingSteps>], (widths, every_part): (&HashMap<Role, Vec<bool>>, bool), counts: &mut StateCounts) -> Option<StateCandidate> {
    let mut nodes: Vec<Role> = Vec::new();
    let node_of = |role: &Role, nodes: &mut Vec<Role>| -> usize {
        nodes.iter().position(|known| known == role).unwrap_or_else(|| {
            nodes.push(role.clone());
            nodes.len() - 1
        })
    };
    let mut endings: HashMap<&Role, (usize, usize)> = HashMap::new();
    for acts in lessons.iter().flatten() {
        for pair in acts.windows(STATE_PLACES) {
            let ((before, Some(one)), (roles, Some(two))) = (&pair[0], &pair[1]) else { continue };
            if before.is_empty() {
                continue;
            }
            for role in roles {
                let entry = endings.entry(role).or_default();
                entry.0 += 1;
                entry.1 += usize::from(one == two);
            }
        }
    }
    let ending = |role: &Role| endings.get(role).is_some_and(|(follows, same)| same * WHOLE_SHARE >= follows * ENDING_PERCENT);
    let mut steps: HashMap<(usize, usize, bool), usize> = HashMap::new();
    let mut step_order: Vec<(usize, usize, bool)> = Vec::new();
    let mut firsts: HashMap<usize, usize> = HashMap::new();
    let mut lessons_of: HashMap<usize, Vec<usize>> = HashMap::new();
    for (lesson, things) in lessons.iter().enumerate() {
        for acts in things {
            let values: Vec<&Option<Vec<WordId>>> = acts.iter().map(|(_, value)| value).collect();
            let acts: Vec<Vec<usize>> = acts.iter().map(|(act, _)| act.iter().map(|role| node_of(role, &mut nodes)).collect()).collect();
            for node in acts.iter().flatten() {
                let held = lessons_of.entry(*node).or_default();
                if held.last() != Some(&lesson) {
                    held.push(lesson);
                }
            }
            if let Some(first) = acts.first() {
                for node in first {
                    *firsts.entry(*node).or_default() += 1;
                }
            }
            let mut count = |step: (usize, usize, bool)| {
                let entry = steps.entry(step).or_default();
                if *entry == usize::default() {
                    step_order.push(step);
                }
                *entry += 1;
            };
            for act in &acts {
                for (at, one) in act.iter().enumerate() {
                    for two in &act[at + 1..] {
                        count((*one, *two, false));
                    }
                }
            }
            for (at, pair) in acts.windows(STATE_PLACES).enumerate() {
                if let (Some(one), Some(two)) = (values[at], values[at + 1])
                    && one != two
                {
                    continue;
                }
                for one in &pair[0] {
                    for two in pair[1].iter().filter(|two| values[at + 1].is_none() || ending(&nodes[**two])) {
                        count((*one, *two, true));
                    }
                }
            }
        }
    }
    let variants: Vec<(usize, usize)> = (0..nodes.len()).flat_map(|short| (0..nodes.len()).map(move |long| (short, long))).filter(|(short, long)| optional_end(&nodes[*short], &nodes[*long])).collect();
    let support = |node: usize| -> usize {
        let mut held: Vec<usize> = variants.iter().filter(|(short, long)| *short == node || *long == node).flat_map(|(short, long)| [*short, *long]).chain([node]).flat_map(|at| lessons_of.get(&at).cloned().unwrap_or_default()).collect();
        held.sort_unstable();
        held.dedup();
        held.len()
    };
    let kept: Vec<bool> = (0..nodes.len()).map(|node| support(node) * WHOLE_SHARE >= lessons.len() * LEAST_STATE_PERCENT).collect();
    counts.below = (0..nodes.len()).filter(|node| !kept[*node]).map(|node| (nodes[node].0.clone(), support(node), lessons.len())).collect();
    counts.below.sort_by(|one, other| other.1.cmp(&one.1));
    let mut ranked: Vec<((usize, usize, bool), usize)> = step_order.into_iter().filter(|(one, two, _)| kept[*one] && kept[*two]).map(|step| (step, steps[&step])).collect();
    ranked.sort_by(|one, other| other.1.cmp(&one.1));
    let mut parts = RoleParts::default();
    parts.grow(nodes.len());
    for (short, long) in variants.iter().filter(|(short, long)| kept[*short] && kept[*long]) {
        parts.join(*short, *long, false);
    }
    let (mut explained, mut all) = (usize::default(), usize::default());
    for ((one, two, differ), count) in &ranked {
        all += count;
        if parts.join(*one, *two, *differ) {
            explained += count;
        }
    }
    counts.explained = explained;
    counts.steps = all;
    if (all == usize::default() && !every_part) || explained * WHOLE_SHARE < all * ROLE_PERCENT {
        return None;
    }
    let root = (0..nodes.len()).filter(|node| kept[*node]).max_by(|one, two| firsts.get(one).cmp(&firsts.get(two)).then_with(|| two.cmp(one)))?;
    let (top, root_other) = parts.root(root);
    let (form, key) = nodes[root].clone();
    let name = event_word(&form)?;
    let article = key_article(&form, places_of(&form)[key]);
    let mut starts: Vec<StateRule> = Vec::new();
    let mut ends: Vec<StateRule> = Vec::new();
    let mut part_starts: HashMap<usize, (usize, bool)> = HashMap::new();
    for node in (0..nodes.len()).filter(|node| kept[*node]) {
        let (part, other) = parts.root(node);
        let first = firsts.get(&node).copied().unwrap_or_default();
        let entry = part_starts.entry(part).or_insert((first, other));
        if first > entry.0 {
            *entry = (first, other);
        }
    }
    for node in (0..nodes.len()).filter(|node| kept[*node]) {
        let (part, other) = parts.root(node);
        let (form, key) = &nodes[node];
        let (_, start_side) = part_starts[&part];
        let joined = every_part || part == top;
        if !joined || event_word(form).is_none() {
            continue;
        }
        let places = places_of(form);
        let narrow = widths.get(&nodes[node]).map(Vec::as_slice).unwrap_or_default();
        if other == if every_part { start_side } else { root_other } {
            starts.extend(state_made(name, article, (form, places[*key], value_pieces(form, places[*key]))).map(|rule| state_rule(rule, narrow)));
        } else {
            ends.push(state_rule(state_ended(name, article, (form, places[*key])), narrow));
        }
    }
    (!starts.is_empty() && (every_part || !ends.is_empty()) && starts.len() + ends.len() > STATE_PLACES).then_some(StateCandidate { starts, ends })
}
because!(role_state, StatesOverTime, "one state that several sentences start and several end (got, took, picked up; dropped, put down, left): the told events of each thing that a question names, in the order of the story; a step from one event to the next joins their two sentences with the other role (for a person, only a step into a sentence that nearly always repeats the value of the event before it, as an end does (is no longer in the B after moved to the B), while a person who goes again to the place where the person is repeats a start; only a step between two events of the same value, as a move to another place replaces the value while a sentence that ends the state names the place that the person leaves), and two sentences told at one time with the same role, the steps that come most often first, a step that its part already says the opposite of left out; kept when the roles explain nearly all the steps, the sentences of the part of the sentence that most often comes first for a thing starting the state, named by it, and the others ending it (for a person, every part, the side of the sentence of the part that most often comes first starting the state, as the sentences that start a person's place and those that end it may never stand next to each other with one value: moved to and is no longer in, is in and is not in; a sentence that no step of one value joins starts the state, as each sentence of a person's place gives a value that the next replaces, and a state of a person needs no sentence that ends it), each with whether every use of its sentence gave each place one word; nothing when the steps do not alternate, or when one sentence starts it and one ends it, as the pairs of events give that state");

pub fn person_state(lessons: &[&Case], (asked, told): (bool, bool)) -> Option<StateCandidate> {
    let (_, people, widths) = told_steps(lessons, (false, asked, told));
    role_state(&people, (&widths, true), &mut StateCounts::default())
}
because!(person_state, StatesOverTime, "the state of the persons that the questions of some lessons name, when asked for, or, when asked for and the questions name no person (where is the football), of each person that the stories name, each event of such a person kept when its value is mostly words that the answers hold (a place, the answer to a question about a thing), as a thing goes where its holder goes (role_state over their told events, the steps between events of one value), read from the told sentences alone with no chain, so that it may read more lessons than the search for states that chains each story: a person's events of one value come in fewer stories than the events of a thing, and a few dozen lessons can leave the sentences that start and end the state apart");

pub fn told_forms_below_bar(lessons: &[&Case]) -> Vec<(Pattern, usize, usize)> {
    let (steps, people, widths) = told_steps(lessons, (true, true, false));
    let mut counts = StateCounts::default();
    role_state(&steps, (&widths, false), &mut counts);
    let mut people_counts = StateCounts::default();
    role_state(&people, (&widths, true), &mut people_counts);
    counts.below.extend(people_counts.below);
    counts.below
}
because!(told_forms_below_bar, StatesOverTime, "the told sentences of asked things that too few of the lessons hold for an invented state to read them (role_state), each with the lessons that hold it, of how many, the most held first, read from the told sentences alone with no chain: a teacher who sees a sentence there writes more lessons with it");

pub(super) fn state_candidates(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], patterns: &[Pattern]) -> (Vec<StateCandidate>, StateCounts) {
    let mut counted: HashMap<(usize, usize, usize, usize), (usize, usize, usize)> = HashMap::new();
    let mut order: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut counts = StateCounts::default();
    let mut widths: HashMap<usize, Vec<bool>> = HashMap::new();
    for (lesson, case) in lessons.iter().enumerate() {
        let Some(facts) = bounded_facts(case, base, topic, false) else { continue };
        let events = story_events(&facts, base, patterns);
        for (_, form, values) in &events {
            kept_one_word(&mut widths, *form, values.iter().map(|value| value.len() == 1).collect());
        }
        counts.lessons += 1;
        counts.events += events.iter().filter(|(_, _, values)| values.len() == STATE_PLACES).count();
        let asked: HashSet<WordId> = fact_numbers(&case.question).into_iter().collect();
        for (at, (time, start, values)) in events.iter().enumerate() {
            if values.len() != STATE_PLACES {
                continue;
            }
            for (later, end, end_values) in &events[at + 1..] {
                if later <= time || end == start {
                    continue;
                }
                for value in 0..STATE_PLACES {
                    let key = &values[STATE_PLACES - 1 - value];
                    let Some(end_key) = end_values.iter().position(|known| known == key) else { continue };
                    if !end_values.contains(&values[value]) || !key.iter().all(|word| asked.contains(word)) {
                        continue;
                    }
                    let found = (*start, value, *end, end_key);
                    let entry = counted.entry(found).or_insert((usize::default(), usize::default(), usize::MAX));
                    if entry.0 == usize::default() {
                        order.push(found);
                    }
                    if entry.2 != lesson {
                        entry.0 += 1;
                        entry.2 = lesson;
                    }
                    entry.1 += 1;
                }
            }
        }
    }
    counts.pairs = order.len();
    let mut kept: Vec<((usize, usize, usize, usize), (usize, usize))> = order.into_iter().map(|found| (found, (counted[&found].0, counted[&found].1))).filter(|(_, (count, _))| count * WHOLE_SHARE >= lessons.len() * LEAST_STATE_PERCENT).collect();
    kept.sort_by(|one, other| other.1.cmp(&one.1));
    counts.kept = kept.len();
    let (steps, _, step_widths) = told_steps(lessons, (true, false, false));
    counts.things = steps.iter().map(Vec::len).sum();
    counts.told = steps.iter().flatten().map(Vec::len).sum();
    let roles = role_state(&steps, (&step_widths, false), &mut counts);
    let narrow_of = |form: usize| -> Vec<bool> {
        let places = places_of(&patterns[form]);
        let mut out = vec![false; places.iter().max().map_or(usize::default(), |last| last + 1)];
        for (place, one_word) in places.iter().zip(widths.get(&form).cloned().unwrap_or_default()) {
            out[*place] = one_word;
        }
        out
    };
    let mut pairs = kept.into_iter().filter_map(|((start, value, end, end_key), _)| state_of((&patterns[start], value), (&patterns[end], end_key), (&narrow_of(start), &narrow_of(end))));
    (pairs.next().into_iter().chain(roles).chain(pairs).take(MOST_STATES).collect(), counts)
}
because!(state_candidates, StatesOverTime, "the states that the stories of the lessons suggest: an event of two places, and a later event of another sentence that names the same two things (a person gets a thing, and later that person drops it), each of the two places in turn the value of the state and the other the thing whose state it is, when the question of the lesson names that thing, as an invented fact serves the answers only when it is about what is asked (where the football is rests on who holds the football); counted once for each lesson whose story tells such a pair, kept when enough lessons tell it, the most told first, and of those told in as many lessons, the one told most often, as an event that more sentences make (a thing got, taken or picked up) starts the state in more stories; with the counts of what it read, for the trace; after the first pair, the one state that the roles of the told sentences of the asked things give, when several sentences start or end it, as a text whose rules make one event of several sentences (took, grabbed: got) needs only that pair, and a text with none needs every sentence; the state of the persons that the questions name is read apart (person_state); each rule of a state with whether every story gave each of its places one word");

pub(super) fn aligned(one: &Pattern, other: &Pattern) -> Option<(HashMap<usize, usize>, Vec<LogicPiece>)> {
    let tail_open = matches!(one.last(), Some(LogicPiece::Open(_)));
    let rest = if other.len() > one.len() && tail_open { &other[one.len()..] } else { &other[other.len()..] };
    if other.len() != one.len() + rest.len() || rest.iter().any(|logic_piece| !matches!(logic_piece, LogicPiece::Word(_))) {
        return None;
    }
    let mut forth: HashMap<usize, usize> = HashMap::new();
    let mut back: HashMap<usize, usize> = HashMap::new();
    for (mine, theirs) in one.iter().zip(other) {
        let (place, given) = match (plain_piece(mine), plain_piece(theirs)) {
            (LogicPiece::Word(word), LogicPiece::Word(given)) if word == given => continue,
            (LogicPiece::Open(place), LogicPiece::Open(given)) => (place, given),
            (LogicPiece::OpenWithEnd(place, ending), LogicPiece::OpenWithEnd(given, other_ending)) if ending == other_ending => (place, given),
            _ => return None,
        };
        if *forth.entry(place).or_insert(given) != given || *back.entry(given).or_insert(place) != place {
            return None;
        }
    }
    Some((forth, rest.to_vec()))
}
because!(aligned, StatesOverTime, "whether a sentence is another sentence with other letters for its places: the same words at the same places and each place of one standing for one place of the other, with that map of the places; a last place of the first may also stand for the last place of the other and the words after it (the B of A took the B for the B there of A took the B there), as a place takes the words that follow it, and those words come with the map");

fn places_at(pattern: &Pattern, place: usize) -> Vec<usize> {
    pattern.iter().enumerate().filter(|(_, logic_piece)| matches!(logic_piece, LogicPiece::Open(known) | LogicPiece::OpenWithEnd(known, _) if *known == place)).map(|(at, _)| at).collect()
}
because!(places_at, StatesOverTime, "where a place stands in a sentence, by the order of its pieces, so that two rules whose facts are one sentence with other letters are compared by where their places go");

pub fn state_covers(given: &ToldRule, discovered: &[ToldRule]) -> Option<(usize, usize)> {
    let one_sentence = |rule: &ToldRule| match rule.conditions.as_slice() {
        [Condition::Fact { pattern, when: Moment::Any }] if rule.absent.is_empty() => Some(pattern.clone()),
        _ => None,
    };
    let told = one_sentence(given)?;
    let mut places = places_of(&told);
    let mut made_places = places_of(&given.outcome);
    places.sort_unstable();
    made_places.sort_unstable();
    if places != made_places {
        return None;
    }
    for (at, first) in discovered.iter().enumerate() {
        let Some((first_map, first_rest)) = one_sentence(first).and_then(|sentence| aligned(&told, &sentence)) else { continue };
        for (other_at, second) in discovered.iter().enumerate().filter(|(other_at, _)| *other_at != at) {
            let Some((second_map, second_rest)) = one_sentence(second).and_then(|sentence| aligned(&given.outcome, &sentence)) else { continue };
            if first_rest != second_rest || !aligned(&first.outcome, &second.outcome).is_some_and(|(_, rest)| rest.is_empty()) {
                continue;
            }
            let agree = places.iter().all(|place| first_map.get(place).map(|mine| places_at(&first.outcome, *mine)) == second_map.get(place).map(|theirs| places_at(&second.outcome, *theirs)));
            if agree {
                return Some((at, other_at));
            }
        }
    }
    None
}
because!(state_covers, StatesOverTime, "the two discovered rules that make a given rule of one sentence true, when its sentence and its outcome each make one fact of the same shape with their places at the same spots (If A took the B then A got the B, when A took the B and A got the B each start the state The got of the B is A, the thing first and the taker last): the given line then reads only as two ways to start or to end one invented state, so a discovery that found the state found what the line says; a last place of the given line may stand for words after the place in both discovered sentences alike (the B for the B there); nothing when no two discovered rules do so");
