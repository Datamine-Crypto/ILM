use crate::tools::explain::chain::{Facts, every_fit};
use crate::tools::explain::text::{Pattern, fact_numbers, fact_words, pattern};
use crate::tools::explain::worlds_solve::{Knowing, MOST_PERSONS, Quantity, World, WorldStatement, Worlds};
use patterns::{because, source};
use std::sync::OnceLock;

pub struct AnnouncedWorlds;
source!(AnnouncedWorlds, "a building block of the engine, the logic of public announcements (Plaza; van Ditmarsch, van der Hoek and Kooi), for any story where persons see some facts of each other and hear what is announced (muddy foreheads, cards shown to some, inner states): a world says for each person whether the fact of that person holds; a person cannot tell apart two worlds that agree on every fact that the person sees, and knows a statement in a world when it holds in every world that the person cannot tell apart from it; an announcement keeps the worlds where it holds, in the order of the story; a statement follows when it holds in every world that is left. The texts of the library bring the words of a story to a few fixed sentences that this block reads, as they bring them to the sentences of a count; a statement is worked out once for all worlds, as the set of the worlds where it holds, the way a model checker does it");

const PERSON: &str = "A is a person";
because!(PERSON, AnnouncedWorlds, "the sentence that names a person of the worlds");

const PERSON_COUNT: &str = "the number of persons is A";
because!(PERSON_COUNT, AnnouncedWorlds, "the sentence that gives the number of persons, some of whom the story may not name");

const SEES: &str = "A sees the fact of B";
because!(SEES, AnnouncedWorlds, "the sentence by which one named person sees the fact of another");

const SEE_OTHERS: &str = "every person sees the facts of the others";
because!(SEE_OTHERS, AnnouncedWorlds, "the sentence by which each person sees the facts of every other person, as of a forehead that others see and the person does not");

const SEE_OWN: &str = "every person sees the person's own fact";
because!(SEE_OWN, AnnouncedWorlds, "the sentence by which each person sees the fact of the person, as of an inner state or a card that the person looks at");

const ANNOUNCED: &str = "it is announced that A";
because!(ANNOUNCED, AnnouncedWorlds, "the sentence of a statement that every person hears, which keeps the worlds where it holds");

const HOLDS: &str = "holds";
because!(HOLDS, AnnouncedWorlds, "the word after a person whose fact holds");

const NOT_HOLDS: &str = "does not hold";
because!(NOT_HOLDS, AnnouncedWorlds, "the words after a person whose fact does not hold");

const KNOWINGS: &[(&str, Knowing)] = &[("knows that", Knowing::That), ("knows whether", Knowing::Whether), ("does not know that", Knowing::NotThat), ("does not know whether", Knowing::NotWhether)];
because!(KNOWINGS, AnnouncedWorlds, "the words after a person and before a statement, for what the person knows of it");

const QUANTITIES: &[(&str, Quantity)] = &[("some person holds", Quantity::Some), ("every person holds", Quantity::Every), ("no person holds", Quantity::No), ("not every person holds", Quantity::NotEvery)];
because!(QUANTITIES, AnnouncedWorlds, "the statements about the facts of all persons at once");

fn sentence(text: &'static str, slot: &'static OnceLock<Pattern>) -> &'static Pattern {
    slot.get_or_init(|| pattern(text))
}
because!(sentence, AnnouncedWorlds, "a fixed sentence of the block as a sentence with open places, read once");

static ANNOUNCED_SENTENCE: OnceLock<Pattern> = OnceLock::new();
because!(ANNOUNCED_SENTENCE, AnnouncedWorlds, "the sentence of an announcement, read once, also for the strata of the rules that wait for every announcement");

pub fn announced_sentence() -> &'static Pattern {
    sentence(ANNOUNCED, &ANNOUNCED_SENTENCE)
}
because!(announced_sentence, AnnouncedWorlds, "the sentence of an announcement with its open place, which a condition that a statement follows reads in every fact");

static PERSON_SENTENCE: OnceLock<Pattern> = OnceLock::new();
because!(PERSON_SENTENCE, AnnouncedWorlds, "the sentence that names a person, read once");

static COUNT_SENTENCE: OnceLock<Pattern> = OnceLock::new();
because!(COUNT_SENTENCE, AnnouncedWorlds, "the sentence of the number of persons, read once");

static SEES_SENTENCE: OnceLock<Pattern> = OnceLock::new();
because!(SEES_SENTENCE, AnnouncedWorlds, "the sentence by which one person sees the fact of another, read once");

fn read_statement(words: &[String], names: &[Vec<String>]) -> Option<WorldStatement> {
    if let Some((_, quantity)) = QUANTITIES.iter().find(|(text, _)| fact_words(text) == words) {
        return Some(WorldStatement::All(*quantity));
    }
    let mut order: Vec<usize> = (0..names.len()).collect();
    order.sort_by_key(|at| std::cmp::Reverse(names[*at].len()));
    for person in order {
        let Some(rest) = words.strip_prefix(names[person].as_slice()) else { continue };
        if rest == fact_words(HOLDS).as_slice() {
            return Some(WorldStatement::Fact(person, true));
        }
        if rest == fact_words(NOT_HOLDS).as_slice() {
            return Some(WorldStatement::Fact(person, false));
        }
        for (text, knowing) in KNOWINGS {
            if let Some(inner) = rest.strip_prefix(fact_words(text).as_slice())
                && let Some(statement) = read_statement(inner, names)
            {
                return Some(WorldStatement::Knows(*knowing, person, Box::new(statement)));
            }
        }
    }
    None
}
because!(read_statement, AnnouncedWorlds, "the statement that words of the fixed sentences of the block say: a statement about all persons, or a named person (the longest name first) whose fact holds or does not hold, or who knows or does not know that or whether a statement holds, read again inside; nothing for words of no such sentence");

pub(super) fn follows(statement: &[String], facts: &Facts) -> bool {
    let first_words = |found: Vec<(Vec<Option<Vec<String>>>, usize)>| -> Vec<(Vec<String>, usize)> { found.into_iter().filter_map(|(filling, time)| filling.into_iter().flatten().next().map(|words| (words, time))).collect() };
    let mut names: Vec<Vec<String>> = Vec::new();
    for (name, _) in first_words(every_fit(sentence(PERSON, &PERSON_SENTENCE), facts)) {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    let told_count = first_words(every_fit(sentence(PERSON_COUNT, &COUNT_SENTENCE), facts)).into_iter().filter_map(|(words, _)| words.join(" ").parse::<usize>().ok()).max().unwrap_or_default();
    let persons = told_count.max(names.len());
    if persons == usize::default() || persons > MOST_PERSONS {
        return false;
    }
    let others = facts.holds(&fact_numbers(SEE_OTHERS));
    let own = facts.holds(&fact_numbers(SEE_OWN));
    let mut sees: Vec<World> = (0..persons).map(|person| {
        let all: World = ((1 as World) << persons) - 1;
        let mine: World = (1 as World) << person;
        (if others { all & !mine } else { World::default() }) | if own { mine } else { World::default() }
    }).collect();
    for (filling, _) in every_fit(sentence(SEES, &SEES_SENTENCE), facts) {
        let mut placed = filling.into_iter().flatten();
        let (Some(seer), Some(seen)) = (placed.next(), placed.next()) else { continue };
        if let (Some(seer), Some(seen)) = (names.iter().position(|name| *name == seer), names.iter().position(|name| *name == seen)) {
            sees[seer] |= (1 as World) << seen;
        }
    }
    let mut worlds = Worlds::new(persons, sees);
    let mut announced = first_words(every_fit(announced_sentence(), facts));
    announced.sort_by_key(|(_, time)| *time);
    for (words, _) in announced {
        let Some(statement) = read_statement(&words, &names) else { return false };
        worlds.announce(&statement);
    }
    read_statement(statement, &names).is_some_and(|statement| worlds.follows(&statement))
}
because!(follows, AnnouncedWorlds, "whether a statement follows from the facts of a story: the persons that the facts name and their number, what each sees, the announcements in the order of the story, each keeping the worlds where it holds, and then the statement holding in every world that is left; it does not follow when the persons are none or too many, or an announcement or the statement is no sentence of the block");
