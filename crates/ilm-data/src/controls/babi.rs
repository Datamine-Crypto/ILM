use ilm_core::base::words::answer_words;
use crate::controls::text::paired;
use patterns::{because, source};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct TaskLogic;
source!(TaskLogic, "an exact reading of each of the twenty tasks of Weston and others, for a control alone and seen by no learner: the answer that the told sentences give by the plain logic of the task, or nothing when the story does not give it. A story is read as a world of persons, places and things: who is where, who holds what, where each thing was and who gave what to whom; a thing has one holder, so a story that lost a sentence is read too. The other tasks read their own sentence forms: relations of places, yes or no and maybe, times of the day, kinds and fears, kinds and colors, positions on a grid, sizes, ways between places, and the place that a feeling leads to");

const BABI_PICK: &[&str] = &["got", "took", "grabbed", "picked"];
because!(BABI_PICK, TaskLogic, "the verbs with which a person takes a thing");

const BABI_DROP: &[&str] = &["dropped", "discarded", "put", "left"];
because!(BABI_DROP, TaskLogic, "the verbs with which a person lets a thing go where the person is");

const BABI_GIVE: &[&str] = &["gave", "handed", "passed"];
because!(BABI_GIVE, TaskLogic, "the verbs with which a person gives a thing to another person");

const BABI_LEADS: &[&str] = &["then", "afterwards", "after", "following", "that"];
because!(BABI_LEADS, TaskLogic, "the words that may start a sentence before its person and say nothing of the world");

const BABI_PRONOUNS: &[&str] = &["he", "she", "they"];
because!(BABI_PRONOUNS, TaskLogic, "the words that stand for the persons of the sentence before");

const BABI_TIMES: &[&str] = &["yesterday", "this morning", "this afternoon", "this evening"];
because!(BABI_TIMES, TaskLogic, "the times of a story in their order in the day");

const BABI_TIME_WORDS: &[&str] = &["yesterday", "this", "morning", "afternoon", "evening"];
because!(BABI_TIME_WORDS, TaskLogic, "the words of the times, which are taken out of a sentence before its person and its place are read");

const BABI_OPPOSITE: &[(&str, &str)] = &[("north", "south"), ("south", "north"), ("east", "west"), ("west", "east"), ("left", "right"), ("right", "left"), ("above", "below"), ("below", "above")];
because!(BABI_OPPOSITE, TaskLogic, "each direction with the direction that goes back");

const BABI_COUNTS: &[&str] = &["none", "one", "two", "three", "four", "five", "six"];
because!(BABI_COUNTS, TaskLogic, "the words that answer how many things a person holds, from nothing up");

const BABI_ONE_OF: &[(&str, &str)] = &[("mice", "mouse"), ("wolves", "wolf"), ("cats", "cat"), ("sheep", "sheep")];
because!(BABI_ONE_OF, TaskLogic, "the word for one animal of each kind that a sentence of fears names for many");

const BABI_STEPS: &[(&str, i64, i64)] = &[("left", -1, 0), ("right", 1, 0), ("above", 0, 1), ("below", 0, -1)];
because!(BABI_STEPS, TaskLogic, "the step on a grid that each relation of positions makes");

const BABI_POSITION_FILLERS: &[&str] = &["the", "is", "to", "of"];
because!(BABI_POSITION_FILLERS, TaskLogic, "the small words of a sentence of positions that belong to no thing");

const BABI_FEELINGS: &[(&str, &str)] = &[("tired", "bedroom"), ("hungry", "kitchen"), ("thirsty", "kitchen"), ("bored", "garden")];
because!(BABI_FEELINGS, TaskLogic, "the place that each feeling leads a person to");

const BABI_ARTICLE: &str = "the";
because!(BABI_ARTICLE, TaskLogic, "the word before a thing or a place");

const BABI_TO: &str = "to";
because!(BABI_TO, TaskLogic, "the word before the person who gets a thing or the place a person goes to");

const BABI_AND: &str = "and";
because!(BABI_AND, TaskLogic, "the word that joins the two persons of a sentence");

const BABI_IS: &str = "is";
because!(BABI_IS, TaskLogic, "the verb of a sentence that says how a person feels");

const BABI_OF: &str = "of";
because!(BABI_OF, TaskLogic, "the last word of a question that asks what a place is in a direction of");

const BABI_WHAT: &str = "what";
because!(BABI_WHAT, TaskLogic, "the first word of a question that asks for a thing or a place");

const BABI_WHERE: &str = "where";
because!(BABI_WHERE, TaskLogic, "the first word of a question that asks for a place");

const BABI_RECEIVED: &str = "received";
because!(BABI_RECEIVED, TaskLogic, "the verb of a question that asks who got a thing");

const BABI_DID: &str = "did";
because!(BABI_DID, TaskLogic, "the helping verb of a question that asks to whom a person gave a thing");

const BABI_GO: &str = "go";
because!(BABI_GO, TaskLogic, "the verb of a question that asks why a person went to a place");

const BABI_EITHER: &str = "either";
because!(BABI_EITHER, TaskLogic, "the word of a sentence that names a pair of places where a person may be");

const BABI_NOT: &[&str] = &["not", "longer"];
because!(BABI_NOT, TaskLogic, "the words of a sentence that says where a person is not, or no longer is");

const BABI_AFRAID: &str = "afraid";
because!(BABI_AFRAID, TaskLogic, "the word of a sentence of fears");

const BABI_KIND_MARK: &str = "a";
because!(BABI_KIND_MARK, TaskLogic, "the word that a sentence of a kind has after its verb, where a sentence of a color has the color");

const BABI_YES: &str = "yes";
because!(BABI_YES, TaskLogic, "the answer when the story makes a question true");

const BABI_NO: &str = "no";
because!(BABI_NO, TaskLogic, "the answer when the story makes a question false");

const BABI_MAYBE: &str = "maybe";
because!(BABI_MAYBE, TaskLogic, "the answer when the story gives a pair of places and one of them is asked");

const BABI_NOTHING: &str = "nothing";
because!(BABI_NOTHING, TaskLogic, "the answer for a person who holds no thing");

const BABI_LIST: &str = ",";
because!(BABI_LIST, TaskLogic, "the mark between the things of a list that a person holds");

const BABI_GAP: &str = " ";
because!(BABI_GAP, TaskLogic, "the mark between the words of a joined answer or a joined name");

const BABI_FITS: &str = " fits inside the ";
because!(BABI_FITS, TaskLogic, "the words of a sentence that says that a thing is smaller than another");

const BABI_BIGGER: &str = " is bigger than the ";
because!(BABI_BIGGER, TaskLogic, "the words of a sentence that says that a thing is larger than another");

const BABI_FIT_ASK: &str = " fit in the ";
because!(BABI_FIT_ASK, TaskLogic, "the words of a question that asks whether a thing is smaller than another");

const BABI_BIGGER_ASK: &str = " bigger than the ";
because!(BABI_BIGGER_ASK, TaskLogic, "the words of a question that asks whether a thing is larger than another");

const BABI_DOES: &str = "Does the ";
because!(BABI_DOES, TaskLogic, "the start of a question of sizes that asks whether a thing fits in another");

const BABI_SENTENCE_HEAD: usize = 4;
because!(BABI_SENTENCE_HEAD, TaskLogic, "the signs of the first article and its space at the start of a sentence of sizes");

const BABI_IS_HEAD: usize = 7;
because!(BABI_IS_HEAD, TaskLogic, "the signs of the start of a question of sizes that asks whether a thing is bigger");

const BABI_END: char = '.';
because!(BABI_END, TaskLogic, "the mark at the end of a told sentence");

const BABI_ASK: char = '?';
because!(BABI_ASK, TaskLogic, "the mark at the end of a question");

const BABI_LOW_LINE: char = '_';
because!(BABI_LOW_LINE, TaskLogic, "the sign that may join the parts of a word of a story");

fn babi_words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut run = String::new();
    for sign in text.chars() {
        if sign.is_ascii_alphabetic() || sign == BABI_LOW_LINE {
            run.push(sign.to_ascii_lowercase());
        } else if !run.is_empty() {
            out.push(std::mem::take(&mut run));
        }
    }
    if !run.is_empty() {
        out.push(run);
    }
    out
}
because!(babi_words, TaskLogic, "the words of a sentence in small letters: the runs of letters and low lines");

fn at_from_end(words: &[String], back: usize) -> Option<&str> {
    words.len().checked_sub(back).map(|at| words[at].as_str())
}
because!(at_from_end, TaskLogic, "a word counted from the end of a row of words, the last one first");

fn thing_after_article(rest: &[String]) -> Option<String> {
    let at = rest.iter().position(|word| word == BABI_ARTICLE)?;
    rest.get(at + 1).cloned()
}
because!(thing_after_article, TaskLogic, "the word after the first article: the thing of a sentence");

#[derive(Default)]
struct World {
    place_of: HashMap<String, String>,
    holds: Vec<(String, Vec<String>)>,
    thing_at: HashMap<String, String>,
    was: HashMap<String, Vec<String>>,
    gave: Vec<(String, String, String)>,
    first: HashMap<(String, String), usize>,
}
because!(World, TaskLogic, "the world of a story: the place of each person, the things that each person holds with the persons in the order in which they first held a thing, the place of each thing, the places where each thing was in their order, who gave what to whom, and when each person first held each thing. read runs the sentences; take gives a thing its one holder; put moves a thing to a place");

impl World {
    fn holding(&mut self, person: &str) -> &mut Vec<String> {
        let at = match self.holds.iter().position(|(held, _)| held == person) {
            Some(at) => at,
            None => {
                self.holds.push((person.to_string(), Vec::new()));
                self.holds.len() - 1
            }
        };
        &mut self.holds[at].1
    }

    fn held_by(&self, person: &str) -> Vec<String> {
        self.holds.iter().find(|(held, _)| held == person).map(|(_, things)| things.clone()).unwrap_or_default()
    }

    fn put(&mut self, thing: &str, place: Option<String>) {
        let Some(place) = place else { return };
        self.thing_at.insert(thing.to_string(), place.clone());
        let was = self.was.entry(thing.to_string()).or_default();
        if was.last() != Some(&place) {
            was.push(place);
        }
    }

    fn take(&mut self, thing: &str, person: &str) {
        for (_, held) in &mut self.holds {
            if let Some(at) = held.iter().position(|other| other == thing) {
                held.remove(at);
            }
        }
        self.holding(person).push(thing.to_string());
        let count = self.first.len();
        self.first.entry((person.to_string(), thing.to_string())).or_insert(count);
        let place = self.place_of.get(person).cloned();
        self.put(thing, place);
    }

    fn read(sentences: &[String]) -> World {
        let mut world = World::default();
        let mut last: Vec<String> = Vec::new();
        for sentence in sentences {
            let words = babi_words(sentence);
            let start = words.iter().position(|word| !BABI_LEADS.contains(&word.as_str())).unwrap_or(words.len());
            let words = &words[start..];
            let Some(first) = words.first() else { continue };
            let (who, rest): (Vec<String>, &[String]) = if BABI_PRONOUNS.contains(&first.as_str()) {
                (last.clone(), &words[1..])
            } else if words.len() > BABI_THIRD && words[1] == BABI_AND {
                (vec![first.clone(), words[BABI_THIRD].clone()], &words[BABI_PAIR_WORDS..])
            } else {
                (vec![first.clone()], &words[1..])
            };
            last = who.clone();
            let Some(verb) = rest.first() else { continue };
            let has = |word: &str| rest.iter().any(|held| held == word);
            if BABI_GIVE.contains(&verb.as_str()) && has(BABI_TO) {
                let (Some(thing), Some(other), Some(giver)) = (thing_after_article(rest), rest.last().cloned(), who.first().cloned()) else { continue };
                world.take(&thing, &other);
                world.gave.push((giver, thing, other));
            } else if BABI_PICK.contains(&verb.as_str()) {
                let (Some(thing), Some(taker)) = (thing_after_article(rest), who.first().cloned()) else { continue };
                world.take(&thing, &taker);
            } else if BABI_DROP.contains(&verb.as_str()) && !has(BABI_TO) {
                let (Some(thing), Some(dropper)) = (thing_after_article(rest), who.first().cloned()) else { continue };
                if let Some((_, held)) = world.holds.iter_mut().find(|(held, _)| *held == dropper)
                    && let Some(at) = held.iter().position(|other| *other == thing)
                {
                    held.remove(at);
                }
                let place = world.place_of.get(&dropper).cloned();
                world.put(&thing, place);
            } else if has(BABI_TO) && has(BABI_ARTICLE) {
                let Some(place) = rest.last().cloned() else { continue };
                for person in &who {
                    world.place_of.insert(person.clone(), place.clone());
                    for thing in world.held_by(person) {
                        world.put(&thing, Some(place.clone()));
                    }
                }
            }
        }
        world
    }
}

const BABI_PAIR_WORDS: usize = 3;
because!(BABI_PAIR_WORDS, TaskLogic, "the words of a pair of persons joined by and, after which the verb of the sentence stands");

fn asked_place(sentences: &[String], question: &str, of_thing: bool) -> Option<String> {
    let asked = babi_words(question).pop()?;
    let world = World::read(sentences);
    let places = if of_thing { &world.thing_at } else { &world.place_of };
    places.get(&asked).cloned()
}
because!(asked_place, TaskLogic, "the place of the person or of the thing that the last word of a question names, as the world of the story has it at its end");

fn place_before(sentences: &[String], question: &str) -> Option<String> {
    let asked = babi_words(question);
    let thing = asked.get(BABI_FOURTH)?;
    let place = asked.last()?;
    let was = World::read(sentences).was.get(thing).cloned().unwrap_or_default();
    let at = (1..was.len()).rev().find(|at| was[*at] == *place)?;
    Some(was[at - 1].clone())
}
because!(place_before, TaskLogic, "where a thing was before it came to a place the last time");

const BABI_FOURTH: usize = 3;
because!(BABI_FOURTH, TaskLogic, "the place of the fourth word, where the forms of the tasks put the thing that a question asks about, the direction of a told relation of places, or the verb of a question of why a person went");

fn relation(sentences: &[String], question: &str) -> Option<String> {
    let mut told: HashSet<(String, String, String)> = HashSet::new();
    for sentence in sentences {
        let words = babi_words(sentence);
        let (one, direction, other) = (words.get(1)?, words.get(BABI_FOURTH)?, words.last()?);
        told.insert((one.clone(), direction.clone(), other.clone()));
        told.insert((other.clone(), paired(BABI_OPPOSITE, direction)?.to_string(), one.clone()));
    }
    let asked = babi_words(question);
    let found: Vec<&String> = if asked.last().map(String::as_str) == Some(BABI_OF) {
        let (thing, direction) = (asked.get(BABI_FOURTH)?, asked.get(BABI_FOURTH + 1)?);
        told.iter().filter(|(one, way, _)| one == thing && way == direction).map(|(_, _, other)| other).collect()
    } else {
        let (direction, other) = (asked.get(BABI_THIRD)?, asked.last()?);
        told.iter().filter(|(_, way, held)| way == direction && held == other).map(|(one, _, _)| one).collect()
    };
    let distinct: HashSet<&&String> = found.iter().collect();
    (distinct.len() == 1).then(|| found[0].clone())
}
because!(relation, TaskLogic, "what lies in a direction of a place, or what a place lies in a direction of, when the told relations and their opposites give one answer");

const BABI_THIRD: usize = 2;
because!(BABI_THIRD, TaskLogic, "the place of the third word, where the forms of the tasks put the person that a question asks about, the direction of a question of places, or the article of a sentence of a kind");

fn giving(sentences: &[String], question: &str) -> Option<String> {
    let gave = World::read(sentences).gave;
    let asked = babi_words(question);
    let word = |at: usize| asked.get(at).map(String::as_str);
    let last = asked.last().map(String::as_str);
    let found: Vec<&String> = if word(0) == Some(BABI_WHAT) {
        gave.iter().filter(|(giver, _, taker)| Some(giver.as_str()) == word(BABI_THIRD) && Some(taker.as_str()) == last).map(|(_, thing, _)| thing).collect()
    } else if word(1) == Some(BABI_RECEIVED) {
        gave.iter().filter(|(_, thing, _)| Some(thing.as_str()) == last).map(|(_, _, taker)| taker).collect()
    } else if word(1) == Some(BABI_DID) {
        gave.iter().filter(|(giver, thing, _)| Some(giver.as_str()) == word(BABI_THIRD) && Some(thing.as_str()) == word(BABI_GIVEN_PLACE)).map(|(_, _, taker)| taker).collect()
    } else if at_from_end(&asked, BABI_THIRD) == Some(BABI_ARTICLE) {
        gave.iter().filter(|(_, thing, _)| Some(thing.as_str()) == last).map(|(giver, _, _)| giver).collect()
    } else {
        gave.iter().filter(|(_, thing, taker)| Some(thing.as_str()) == word(BABI_FOURTH) && Some(taker.as_str()) == last).map(|(giver, _, _)| giver).collect()
    };
    found.last().map(|found| found.to_string())
}
because!(giving, TaskLogic, "the thing, the giver or the taker of the last giving that a question asks about");

const BABI_GIVEN_PLACE: usize = 5;
because!(BABI_GIVEN_PLACE, TaskLogic, "the place of the thing in a question of to whom a person gave a thing: after who, did, the person, give and the article");

fn is_in(sentences: &[String], question: &str) -> Option<String> {
    let asked = babi_words(question);
    let place = World::read(sentences).place_of.get(asked.get(1)?).cloned()?;
    Some(if Some(&place) == asked.last() { BABI_YES } else { BABI_NO }.to_string())
}
because!(is_in, TaskLogic, "whether a person is in a place");

fn how_many(sentences: &[String], question: &str) -> Option<String> {
    let asked = babi_words(question);
    let held = World::read(sentences).held_by(asked.get(BABI_FIFTH)?);
    BABI_COUNTS.get(held.len()).map(|count| count.to_string())
}
because!(how_many, TaskLogic, "how many things a person holds");

const BABI_FIFTH: usize = 4;
because!(BABI_FIFTH, TaskLogic, "the place of the person in a question of how many things a person holds, and, counted from the end, of the first of a pair of places of a sentence of maybe and of the start of a question of a way");

fn carrying(sentences: &[String], question: &str) -> Option<String> {
    let asked = babi_words(question);
    let who = asked.get(BABI_THIRD)?;
    let world = World::read(sentences);
    let mut held = world.held_by(who);
    held.sort_by_key(|thing| world.first.get(&(who.clone(), thing.clone())).copied().unwrap_or_default());
    Some(if held.is_empty() { BABI_NOTHING.to_string() } else { held.join(BABI_LIST) })
}
because!(carrying, TaskLogic, "the things that a person holds, in the order in which the person first took each of them, or nothing");

fn maybe_in(sentences: &[String], question: &str) -> Option<String> {
    let mut can: HashMap<String, (Vec<String>, bool)> = HashMap::new();
    for sentence in sentences {
        let words = babi_words(sentence);
        let Some(who) = words.first() else { continue };
        let found = if words.iter().any(|word| word == BABI_EITHER) {
            (vec![at_from_end(&words, BABI_FIFTH)?.to_string(), words.last()?.clone()], false)
        } else if words.iter().any(|word| BABI_NOT.contains(&word.as_str())) {
            (Vec::new(), true)
        } else {
            (vec![words.last()?.clone()], true)
        };
        can.insert(who.clone(), found);
    }
    let asked = babi_words(question);
    let (places, sure) = can.get(asked.get(1)?)?;
    let place = asked.last()?;
    Some(if !places.contains(place) { BABI_NO } else if *sure { BABI_YES } else { BABI_MAYBE }.to_string())
}
because!(maybe_in, TaskLogic, "whether a person is in a place, when sentences say where a person is, is not, or may be");

fn before_in_time(sentences: &[String], question: &str) -> Option<String> {
    let asked = babi_words(question);
    let person = asked.get(BABI_THIRD)?;
    let mut seen: HashMap<usize, String> = HashMap::new();
    for sentence in sentences {
        let words = babi_words(sentence);
        let low = words.join(BABI_GAP);
        let when = BABI_TIMES.iter().position(|time| low.contains(time));
        let rest: Vec<&String> = words.iter().filter(|word| !BABI_TIME_WORDS.contains(&word.as_str())).collect();
        if let Some(when) = when
            && rest.first() == Some(&person)
        {
            seen.insert(when, rest.last()?.to_string());
        }
    }
    let mut times: Vec<usize> = seen.keys().copied().collect();
    times.sort_unstable();
    let order: Vec<&String> = times.iter().map(|time| &seen[time]).collect();
    let place = asked.last()?;
    let at = (1..order.len()).rev().find(|at| order[*at] == place)?;
    Some(order[at - 1].clone())
}
because!(before_in_time, TaskLogic, "where a person was before a place, by the times of the day of the sentences");

fn afraid_of(sentences: &[String], question: &str) -> Option<String> {
    let mut fears: HashMap<&str, &str> = HashMap::new();
    let mut kind: HashMap<String, String> = HashMap::new();
    for sentence in sentences {
        let words = babi_words(sentence);
        if words.iter().any(|word| word == BABI_AFRAID) {
            fears.insert(paired(BABI_ONE_OF, words.first()?)?, paired(BABI_ONE_OF, words.last()?)?);
        } else {
            kind.insert(words.first()?.clone(), words.last()?.clone());
        }
    }
    let asked = babi_words(question);
    let animal = kind.get(asked.get(BABI_THIRD)?)?;
    fears.get(animal.as_str()).map(|fear| fear.to_string())
}
because!(afraid_of, TaskLogic, "what the kind of an animal is afraid of");

fn color_of_kind(sentences: &[String], question: &str) -> Option<String> {
    let mut kind: Vec<(String, String)> = Vec::new();
    let mut color: HashMap<String, String> = HashMap::new();
    for sentence in sentences {
        let words = babi_words(sentence);
        let (who, what) = (words.first()?.clone(), words.last()?.clone());
        if words.get(BABI_THIRD).map(String::as_str) == Some(BABI_KIND_MARK) {
            kind.retain(|(held, _)| *held != who);
            kind.push((who, what));
        } else {
            color.insert(who, what);
        }
    }
    let who = babi_words(question).pop()?;
    if let Some(found) = color.get(&who) {
        return Some(found.clone());
    }
    let own = kind.iter().find(|(held, _)| *held == who).map(|(_, what)| what.clone());
    let like: Vec<&String> = kind.iter().filter(|(other, what)| *other != who && Some(what) == own.as_ref() && color.contains_key(other)).map(|(other, _)| &color[other]).collect();
    like.last().map(|found| found.to_string())
}
because!(color_of_kind, TaskLogic, "the color of an animal: its own, or that of the animal of its kind named last that has a color");

fn position_split(words: &[String]) -> Option<(String, String, String)> {
    let at = words.iter().position(|word| paired(BABI_OPPOSITE, word).is_some())?;
    let keep = |part: &[String]| part.iter().filter(|word| !BABI_POSITION_FILLERS.contains(&word.as_str())).cloned().collect::<Vec<String>>().join(BABI_GAP);
    Some((keep(&words[..at]), words[at].clone(), keep(&words[at + 1..])))
}
because!(position_split, TaskLogic, "the first thing, the relation and the second thing of a sentence of positions");

fn position(sentences: &[String], question: &str) -> Option<String> {
    let step = |direction: &str| BABI_STEPS.iter().find(|(name, _, _)| *name == direction).map(|(_, across, up)| (*across, *up));
    let told: Vec<(String, String, String)> = sentences.iter().map(|sentence| position_split(&babi_words(sentence))).collect::<Option<Vec<_>>>()?;
    let mut at: HashMap<String, (i64, i64)> = HashMap::new();
    for _ in &told {
        for (one, direction, other) in &told {
            if at.is_empty() {
                at.insert(other.clone(), (0, 0));
            }
            if let (Some(base), false) = (at.get(other).copied(), at.contains_key(one)) {
                let (across, up) = step(direction)?;
                at.insert(one.clone(), (base.0 + across, base.1 + up));
            } else if let (Some(base), false) = (at.get(one).copied(), at.contains_key(other)) {
                let (across, up) = step(direction)?;
                at.insert(other.clone(), (base.0 - across, base.1 - up));
            }
        }
    }
    let (one, direction, other) = position_split(&babi_words(question))?;
    let (first, second) = (at.get(&one)?, at.get(&other)?);
    let (across, up) = (first.0 - second.0, first.1 - second.1);
    let (step_across, step_up) = step(&direction)?;
    let holds = (step_across != 0 && across.signum() == step_across) || (step_up != 0 && up.signum() == step_up);
    Some(if holds { BABI_YES } else { BABI_NO }.to_string())
}
because!(position, TaskLogic, "whether a thing lies in a direction of another, by the places of the things on a grid where each told relation is one step");

fn chars_after(text: &str, count: usize) -> &str {
    text.char_indices().nth(count).map_or("", |(at, _)| &text[at..])
}
because!(chars_after, TaskLogic, "a text without its first signs");

fn split_turned(text: &str, mark: &str) -> Vec<String> {
    let mut parts: Vec<String> = text.split(mark).map(String::from).collect();
    parts.reverse();
    parts
}
because!(split_turned, TaskLogic, "the parts of a text between the words of a relation, the last part first, so that a larger thing comes after the smaller one");

fn sizes(sentences: &[String], question: &str) -> Option<String> {
    let mut less: HashSet<Vec<String>> = HashSet::new();
    for sentence in sentences {
        let low = sentence.trim_end_matches(BABI_END);
        if low.contains(BABI_FITS) {
            less.insert(chars_after(low, BABI_SENTENCE_HEAD).split(BABI_FITS).map(String::from).collect());
        } else if low.contains(BABI_BIGGER) {
            less.insert(split_turned(chars_after(low, BABI_SENTENCE_HEAD), BABI_BIGGER));
        }
    }
    loop {
        let mut more: HashSet<Vec<String>> = HashSet::new();
        for one in &less {
            for other in &less {
                if let ([small, middle], [middle_again, large]) = (one.as_slice(), other.as_slice())
                    && middle == middle_again
                {
                    let pair = vec![small.clone(), large.clone()];
                    if !less.contains(&pair) {
                        more.insert(pair);
                    }
                }
            }
        }
        if more.is_empty() {
            break;
        }
        less.extend(more);
    }
    let low = question.trim_end_matches(BABI_ASK);
    let pair: Vec<String> = if let Some(rest) = low.strip_prefix(BABI_DOES) {
        rest.split(BABI_FIT_ASK).map(String::from).collect()
    } else {
        split_turned(chars_after(low, BABI_IS_HEAD), BABI_BIGGER_ASK)
    };
    let turned: Vec<String> = pair.iter().rev().cloned().collect();
    if less.contains(&pair) {
        return Some(BABI_YES.to_string());
    }
    less.contains(&turned).then(|| BABI_NO.to_string())
}
because!(sizes, TaskLogic, "whether a thing is smaller or larger than another, by the told sizes and the steps from a smaller thing to a larger one");

fn way_between(sentences: &[String], question: &str) -> Option<String> {
    let mut go: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for sentence in sentences {
        let words = babi_words(sentence);
        let (one, direction, other) = (words.get(1)?, words.get(BABI_FOURTH)?, words.last()?);
        go.entry(other.clone()).or_default().push((direction.clone(), one.clone()));
        go.entry(one.clone()).or_default().push((paired(BABI_OPPOSITE, direction)?.to_string(), other.clone()));
    }
    let asked = babi_words(question);
    let (start, goal) = (at_from_end(&asked, BABI_FIFTH)?.to_string(), asked.last()?.clone());
    let mut back: HashMap<String, Option<(String, String)>> = HashMap::new();
    back.insert(start.clone(), None);
    let mut queue: VecDeque<String> = VecDeque::from([start]);
    while let Some(place) = queue.pop_front() {
        if place == goal {
            let mut out: Vec<String> = Vec::new();
            let mut at = place;
            while let Some(Some((before, direction))) = back.get(&at).cloned() {
                out.push(direction);
                at = before;
            }
            out.reverse();
            return (!out.is_empty()).then(|| out.join(BABI_GAP));
        }
        for (direction, next) in go.get(&place).cloned().unwrap_or_default() {
            if !back.contains_key(&next) {
                back.insert(next.clone(), Some((place.clone(), direction)));
                queue.push_back(next);
            }
        }
    }
    None
}
because!(way_between, TaskLogic, "the directions of the shortest way from a place to another, by the told relations and their opposites, searched in breadth");

fn why(sentences: &[String], question: &str) -> Option<String> {
    let mut state: HashMap<String, String> = HashMap::new();
    for sentence in sentences {
        let words = babi_words(sentence);
        if let [who, verb, feeling] = words.as_slice()
            && verb == BABI_IS
        {
            state.insert(who.clone(), feeling.clone());
        }
    }
    let asked = babi_words(question);
    let feeling_place = |feeling: &str| BABI_FEELINGS.iter().find(|(name, _)| *name == feeling).map(|(_, place)| place.to_string());
    let who = asked.get(BABI_THIRD)?;
    if asked.first().map(String::as_str) == Some(BABI_WHERE) {
        return feeling_place(state.get(who)?);
    }
    if !state.contains_key(who) && asked.get(BABI_FOURTH).map(String::as_str) == Some(BABI_GO) {
        let place = asked.last()?;
        let fits: Vec<&str> = BABI_FEELINGS.iter().filter(|(_, held)| held == place).map(|(feeling, _)| *feeling).collect();
        return match fits[..] {
            [one] => Some(one.to_string()),
            _ => None,
        };
    }
    state.get(who).cloned()
}
because!(why, TaskLogic, "where a feeling leads a person, or why a person went to a place or took a thing");

#[derive(Clone, Copy, Debug)]
enum TaskReading {
    PersonPlace,
    ThingPlace,
    Story(fn(&[String], &str) -> Option<String>),
}
because!(TaskReading, TaskLogic, "the reading of one task: where a person is, where a thing is, or a reading of its own that gives the answer of the told sentences to a question, or nothing");

const BABI_READINGS: &[(usize, TaskReading)] = &[
    (1, TaskReading::PersonPlace),
    (2, TaskReading::ThingPlace),
    (3, TaskReading::Story(place_before)),
    (4, TaskReading::Story(relation)),
    (5, TaskReading::Story(giving)),
    (6, TaskReading::Story(is_in)),
    (7, TaskReading::Story(how_many)),
    (8, TaskReading::Story(carrying)),
    (9, TaskReading::Story(maybe_in)),
    (10, TaskReading::Story(maybe_in)),
    (11, TaskReading::PersonPlace),
    (12, TaskReading::PersonPlace),
    (13, TaskReading::PersonPlace),
    (14, TaskReading::Story(before_in_time)),
    (15, TaskReading::Story(afraid_of)),
    (16, TaskReading::Story(color_of_kind)),
    (17, TaskReading::Story(position)),
    (18, TaskReading::Story(sizes)),
    (19, TaskReading::Story(way_between)),
    (20, TaskReading::Story(why)),
];
because!(BABI_READINGS, TaskLogic, "the reading of each task by its number: several tasks ask where a person is, and the tasks of maybe share one reading");

pub fn babi_exact(task: usize, sentences: &[String], question: &str) -> Option<Vec<String>> {
    let (_, reading) = BABI_READINGS.iter().find(|(number, _)| *number == task)?;
    let said = match reading {
        TaskReading::PersonPlace => asked_place(sentences, question, false),
        TaskReading::ThingPlace => asked_place(sentences, question, true),
        TaskReading::Story(read) => read(sentences, question),
    };
    said.map(|said| answer_words(&said))
}
because!(babi_exact, TaskLogic, "the words of the answer that the story of a task gives, or nothing");
