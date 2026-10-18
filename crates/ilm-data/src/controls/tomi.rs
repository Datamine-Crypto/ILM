use ilm_core::base::first_sight::{FirstSightCount, FirstSightTable};
use crate::controls::file::ChangedCase;
use crate::sets::readers::common::stripped;
use ilm_core::base::signs::is_word_sign;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::{BTreeSet, HashMap};

pub struct StoryState;
source!(StoryState, "a story of a thing that is placed and moved in a room, read as events and run as a state by the plain rules of such stories: a person sees a placing or a move when the person is in the room of the thing at that time, and a person who enters the room of the thing later sees where it is; the first belief of a person is the last place that the person saw. The room of the thing at its placing is the room where the most persons are, and at a move the room of the mover. The state gives the true label of a statement of where the thing really is, where it was at the beginning, and where a person will look for it, so it gives the label of a changed story too. A story is changed so that the thing is first in a place that the story does not hold, is moved to such a place, is not moved, is moved once more, or is seen moved by the person of the statement, who no longer leaves before the move");

const TOMI_ENTERED: &str = " entered the ";
because!(TOMI_ENTERED, StoryState, "the words between a person and the room that the person enters");

const TOMI_EXITED: &str = " exited the ";
because!(TOMI_EXITED, StoryState, "the words between a person and the room that the person leaves");

const TOMI_MOVED: &str = " moved the ";
because!(TOMI_MOVED, StoryState, "the words between a person and the thing that the person moves");

const TOMI_TO: &str = " to the ";
because!(TOMI_TO, StoryState, "the words before the place where a thing is moved or where a person will look");

const TOMI_PLACED: &str = "The ";
because!(TOMI_PLACED, StoryState, "the words before the thing of a placing sentence");

const TOMI_IS_IN: &str = " is in the ";
because!(TOMI_IS_IN, StoryState, "the words between the thing of a placing sentence and its place");

const TOMI_END: char = '.';
because!(TOMI_END, StoryState, "the mark at the end of a sentence of an event and of a statement");

const TOMI_LINE: &str = "\n";
because!(TOMI_LINE, StoryState, "the end of a line that may follow the mark at the end of a sentence");

const TOMI_GAP: char = ' ';
because!(TOMI_GAP, StoryState, "the space between the words of a name of a room, a thing or a place");

const TOMI_JOIN: char = '_';
because!(TOMI_JOIN, StoryState, "the sign that joins the words of a name of a room, a thing or a place into one word in the state");

const TOMI_ARTICLES: &[&str] = &["The ", "the "];
because!(TOMI_ARTICLES, StoryState, "the first word of a statement about the thing, with or without its capital letter");

const TOMI_REALLY: &str = " really is in the ";
because!(TOMI_REALLY, StoryState, "the words of a statement of where the thing really is");

const TOMI_BEGINNING: &str = " at the beginning was in the ";
because!(TOMI_BEGINNING, StoryState, "the words of a statement of where the thing was at the beginning");

const TOMI_LOOK: &str = " look for the ";
because!(TOMI_LOOK, StoryState, "the words of a statement of where a person will look for the thing");

const TOMI_REAL_KIND: &str = "tomi_real";
because!(TOMI_REAL_KIND, StoryState, "the kind of statements of where the thing really is");

const TOMI_BEGIN_KIND: &str = "tomi_begin";
because!(TOMI_BEGIN_KIND, StoryState, "the kind of statements of where the thing was at the beginning");

const TOMI_LOOK_KIND: &str = "tomi_look";
because!(TOMI_LOOK_KIND, StoryState, "the kind of statements of where a person will look for the thing");

const TOMI_TRUE: &str = "entailment";
because!(TOMI_TRUE, StoryState, "the label of a statement that the story makes true");

const TOMI_FALSE: &str = "not_entailment";
because!(TOMI_FALSE, StoryState, "the label of a statement that the story does not make true");

const TOMI_WORD_GAP: &str = " ";
because!(TOMI_WORD_GAP, StoryState, "the mark between the told sentences of a story when they are searched as one text");

const SORT_FIRST_ELSEWHERE: &str = "the thing is first in another place";
because!(SORT_FIRST_ELSEWHERE, StoryState, "the sort of the change where the placing names a place that the story does not hold");

const SORT_MOVED_ELSEWHERE: &str = "the thing is moved to another place";
because!(SORT_MOVED_ELSEWHERE, StoryState, "the sort of the change where the last move names a place that the story does not hold");

const SORT_MOVE_OUT: &str = "the move taken out";
because!(SORT_MOVE_OUT, StoryState, "the sort of the change where the last move is taken out");

const SORT_MOVED_AGAIN: &str = "moved again";
because!(SORT_MOVED_AGAIN, StoryState, "the sort of the change where the mover moves the thing once more after the last move");

const SORT_LEAVING_OUT: &str = "the leaving taken out";
because!(SORT_LEAVING_OUT, StoryState, "the sort of the change where the person of the statement does not leave before the move");

fn run_end(text: &str, start: usize, spaced: bool) -> usize {
    let length: usize = text[start..].chars().take_while(|sign| is_word_sign(*sign) || (spaced && *sign == TOMI_GAP)).map(char::len_utf8).sum();
    start + length
}
because!(run_end, StoryState, "the end of the run of word signs from a place on, with spaces in it when asked");

fn sentence_end(text: &str, at: usize) -> bool {
    let rest = &text[at..];
    rest.strip_prefix(TOMI_END).is_some_and(|after| after.is_empty() || after == TOMI_LINE)
}
because!(sentence_end, StoryState, "whether the end mark of a sentence stands at a place and nothing but the end of a line follows it");

fn spaced_tail(text: &str, start: usize) -> Option<&str> {
    let end = run_end(text, start, true);
    (end > start && sentence_end(text, end)).then(|| &text[start..end])
}
because!(spaced_tail, StoryState, "the name with spaces that runs from a place to the end mark of a sentence, or nothing");

fn person_room(text: &str, verb: &str) -> Option<(String, String)> {
    text.char_indices().find_map(|(start, _)| {
        let end = run_end(text, start, false);
        if end == start || !text[end..].starts_with(verb) {
            return None;
        }
        let room = spaced_tail(text, end + verb.len())?;
        Some((text[start..end].to_string(), room.to_string()))
    })
}
because!(person_room, StoryState, "the person and the room of a sentence where a person enters or leaves a room, searched from the left");

fn lazy_pair<'a>(text: &'a str, start: usize, between: &str) -> Option<(&'a str, &'a str)> {
    let run = run_end(text, start, true);
    text[start..run].char_indices().skip(1).map(|(length, _)| start + length).chain([run]).find_map(|end| {
        if !text[end..].starts_with(between) {
            return None;
        }
        let tail = spaced_tail(text, end + between.len())?;
        Some((&text[start..end], tail))
    })
}
because!(lazy_pair, StoryState, "the shortest name with spaces from a place on that is followed by the given words and a name with spaces up to the end of the sentence, with that last name");

fn moved(text: &str) -> Option<(String, String, String)> {
    text.char_indices().find_map(|(start, _)| {
        let end = run_end(text, start, false);
        if end == start || !text[end..].starts_with(TOMI_MOVED) {
            return None;
        }
        let (thing, place) = lazy_pair(text, end + TOMI_MOVED.len(), TOMI_TO)?;
        Some((text[start..end].to_string(), thing.to_string(), place.to_string()))
    })
}
because!(moved, StoryState, "the mover, the thing and the new place of a sentence of a move, searched from the left");

fn placed(text: &str) -> Option<(String, String)> {
    text.match_indices(TOMI_PLACED).find_map(|(start, _)| {
        let (thing, place) = lazy_pair(text, start + TOMI_PLACED.len(), TOMI_IS_IN)?;
        Some((thing.to_string(), place.to_string()))
    })
}
because!(placed, StoryState, "the thing and the place of a sentence of a placing, searched from the left");

#[derive(Clone, Debug)]
enum StoryEvent {
    Enter(String, String),
    Exit(String),
    Move(String, String),
    Place(String),
}
because!(StoryEvent, StoryState, "an event of a story: a person enters a room, a person leaves, a person moves the thing to a place, or the thing is placed in a place");

fn name_as_word(name: &str) -> String {
    name.replace(TOMI_GAP, &TOMI_JOIN.to_string())
}
because!(name_as_word, StoryState, "a name with its spaces made joining signs, so that it is one word in the state");

fn story_events(sentences: &[String], thing: Option<&str>) -> Vec<StoryEvent> {
    let ours = |name: &str| thing.is_none_or(|asked| name_as_word(name) == asked);
    let mut out: Vec<StoryEvent> = Vec::new();
    for sentence in sentences {
        if let Some((person, room)) = person_room(sentence, TOMI_ENTERED) {
            out.push(StoryEvent::Enter(name_as_word(&person), name_as_word(&room)));
        } else if let Some((person, _)) = person_room(sentence, TOMI_EXITED) {
            out.push(StoryEvent::Exit(name_as_word(&person)));
        } else if let Some((person, moved_thing, place)) = moved(sentence) {
            if ours(&moved_thing) {
                out.push(StoryEvent::Move(name_as_word(&person), name_as_word(&place)));
            }
        } else if let Some((placed_thing, place)) = placed(sentence) {
            if ours(&placed_thing) {
                out.push(StoryEvent::Place(name_as_word(&place)));
            }
        } else if thing.is_some()
            && let Some((person, room)) = person_room(sentence, TOMQA_PERSON_IS_IN)
        {
            out.push(StoryEvent::Enter(name_as_word(&person), name_as_word(&room)));
        }
    }
    out
}
because!(story_events, StoryState, "the events of a story in their order; a sentence that is no event gives none; with a thing named, only the placings and moves of that thing, since a story of the belief set places and moves other things too, and a person who is in a room at the start counts as entering it");

struct ThingState {
    real: Option<String>,
    begin: Option<String>,
    first: HashMap<String, String>,
    second: HashMap<(String, String), String>,
}
because!(ThingState, StoryState, "what the state of a story gives for its statements: where the thing really is, where it was at the beginning, the first belief of each person, and for two persons the place where the first thinks that the second believes the thing to be");

fn thing_state(events: &[StoryEvent]) -> ThingState {
    let mut where_is: FirstSightTable<String, Option<String>> = FirstSightTable::new();
    let mut first: HashMap<String, String> = HashMap::new();
    let mut second: HashMap<(String, String), String> = HashMap::new();
    let (mut room, mut real, mut begin): (Option<String>, Option<String>, Option<String>) = (None, None, None);
    for event in events {
        let place = match event {
            StoryEvent::Enter(person, entered) => {
                where_is.put(person.clone(), Some(entered.clone()));
                if Some(entered) == room.as_ref()
                    && let Some(seen) = &real
                {
                    first.insert(person.clone(), seen.clone());
                    for (other, held) in where_is.rows() {
                        if held.as_ref() == Some(entered) {
                            second.insert((person.clone(), other.clone()), seen.clone());
                            second.insert((other.clone(), person.clone()), seen.clone());
                        }
                    }
                }
                continue;
            }
            StoryEvent::Exit(person) => {
                where_is.put(person.clone(), None);
                continue;
            }
            StoryEvent::Place(place) => {
                let mut rooms: FirstSightCount<String> = FirstSightCount::new();
                for (_, held) in where_is.rows() {
                    if let Some(held) = held {
                        rooms.add(held.clone(), 1);
                    }
                }
                room = rooms.first_most().map(|(found, _)| found.clone());
                begin = begin.or_else(|| Some(place.clone()));
                place.clone()
            }
            StoryEvent::Move(person, place) => {
                if let Some(Some(held)) = where_is.get(person) {
                    room = Some(held.clone());
                }
                place.clone()
            }
        };
        real = Some(place.clone());
        let here: Vec<String> = where_is.rows().iter().filter(|(_, held)| held.is_some() && *held == room).map(|(person, _)| person.clone()).collect();
        for person in &here {
            first.insert(person.clone(), place.clone());
            for other in &here {
                second.insert((person.clone(), other.clone()), place.clone());
            }
        }
    }
    ThingState { real, begin, first, second }
}
because!(thing_state, StoryState, "the state of a story after its events");

fn statement_parts(kind: &str, statement: &str) -> Option<Vec<String>> {
    let text = stripped(statement).trim_end_matches(TOMI_END);
    let steps: Vec<Option<&str>> = match kind {
        TOMI_REAL_KIND | TOMI_BEGIN_KIND => {
            let the = TOMI_ARTICLES.iter().find(|the| text.starts_with(**the))?;
            vec![Some(*the), None, Some(if kind == TOMI_REAL_KIND { TOMI_REALLY } else { TOMI_BEGINNING }), None]
        }
        TOMI_LOOK_KIND => vec![None, Some(TOMI_LOOK), None, Some(TOMI_TO), None],
        _ => return None,
    };
    named_steps(text, &steps)
}
because!(statement_parts, StoryState, "the names of a statement of a kind: the thing and the place, or the person, the thing and the place; nothing for a statement of another form");

fn named_steps(text: &str, steps: &[Option<&str>]) -> Option<Vec<String>> {
    let mut parts: Vec<String> = Vec::new();
    let mut at = 0;
    for step in steps {
        match step {
            Some(words) => {
                if !text[at..].starts_with(words) {
                    return None;
                }
                at += words.len();
            }
            None => {
                let end = run_end(text, at, false);
                if end == at {
                    return None;
                }
                parts.push(text[at..end].to_string());
                at = end;
            }
        }
    }
    let rest = &text[at..];
    (rest.is_empty() || rest == TOMI_LINE).then_some(parts)
}
because!(named_steps, StoryState, "the names of a text that follows fixed words with a place for one name between them, or nothing when the text has another form");

fn tomi_label(kind: &str, sentences: &[String], statement: &str) -> Option<&'static str> {
    let parts = statement_parts(kind, statement)?;
    let state = thing_state(&story_events(sentences, None));
    let place = match kind {
        TOMI_REAL_KIND => state.real,
        TOMI_BEGIN_KIND => state.begin,
        _ => state.first.get(&parts[0]).cloned(),
    };
    Some(if place.as_ref() == parts.last() { TOMI_TRUE } else { TOMI_FALSE })
}
because!(tomi_label, StoryState, "the true label of a statement by the state of the story");

fn free_place<'a>(sentences: &[String], free: &'a [String]) -> Option<&'a String> {
    let told = sentences.join(TOMI_WORD_GAP);
    free.iter().find(|place| !told.contains(place.as_str()))
}
because!(free_place, StoryState, "the first place of the lessons that the story does not hold");

fn without(sentences: &[String], at: usize) -> Vec<String> {
    sentences.iter().enumerate().filter(|(other, _)| *other != at).map(|(_, sentence)| sentence.clone()).collect()
}
because!(without, StoryState, "the told sentences with one of them taken out");

fn moves_at(sentences: &[String]) -> Vec<usize> {
    sentences.iter().enumerate().filter(|(_, sentence)| moved(sentence).is_some()).map(|(at, _)| at).collect()
}
because!(moves_at, StoryState, "the places of the sentences of a move");

fn elsewhere(sentences: &[String], free: &[String], placing: bool) -> Option<Vec<String>> {
    let last_place = |sentence: &String| if placing { placed(sentence).map(|(_, place)| place) } else { moved(sentence).map(|(_, _, place)| place) };
    let at = sentences.iter().rposition(|sentence| last_place(sentence).is_some())?;
    let old = last_place(&sentences[at])?;
    let place = free_place(sentences, free)?;
    let mut out = sentences.to_vec();
    let cut = out[at].rfind(old.as_str())?;
    out[at] = format!("{}{place}{}", &sentences[at][..cut], &sentences[at][cut + old.len()..]);
    Some(out)
}
because!(elsewhere, StoryState, "the story with the place of its last placing, or of its last move, made a place that the story does not hold");

fn moved_again(sentences: &[String], free: &[String]) -> Option<Vec<String>> {
    let at = *moves_at(sentences).last()?;
    let (person, thing, _) = moved(&sentences[at])?;
    let place = free_place(sentences, free)?;
    let mut out = sentences.to_vec();
    out.insert(at + 1, format!("{person}{TOMI_MOVED}{thing}{TOMI_TO}{place}{TOMI_END}"));
    Some(out)
}
because!(moved_again, StoryState, "the story with one more move of the thing by the last mover, right after the last move, to a place that the story does not hold");

fn leaving_out(kind: &str, sentences: &[String], statement: &str) -> Option<Vec<String>> {
    if kind != TOMI_LOOK_KIND {
        return None;
    }
    let who = statement_parts(kind, statement)?.first()?.clone();
    let at = *moves_at(sentences).last()?;
    let exit = sentences[..at].iter().rposition(|sentence| person_room(sentence, TOMI_EXITED).is_some_and(|(person, _)| person == who))?;
    Some(without(sentences, exit))
}
because!(leaving_out, StoryState, "for a statement of where a person will look: the story without the last leaving of that person before the last move");

pub fn tomi_changes(lessons: &[Case], whole: bool, kind: &str) -> Vec<ChangedCase> {
    let part: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let free: Vec<String> = lessons.iter().flat_map(|case| case.sentences.iter().filter_map(|sentence| placed(sentence).map(|(_, place)| place))).collect::<BTreeSet<String>>().into_iter().collect();
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in part {
        let case = &lessons[place];
        if tomi_label(kind, &case.sentences, &case.question) != Some(case.answer.as_str()) {
            continue;
        }
        let changes = [
            (SORT_FIRST_ELSEWHERE, elsewhere(&case.sentences, &free, true)),
            (SORT_MOVED_ELSEWHERE, elsewhere(&case.sentences, &free, false)),
            (SORT_MOVE_OUT, moves_at(&case.sentences).last().map(|at| without(&case.sentences, *at))),
            (SORT_MOVED_AGAIN, moved_again(&case.sentences, &free)),
            (SORT_LEAVING_OUT, leaving_out(kind, &case.sentences, &case.question)),
        ];
        for (sort, changed) in changes {
            let Some(sentences) = changed else { continue };
            let Some(truth) = tomi_label(kind, &sentences, &case.question) else { continue };
            out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences, question: case.question.clone(), truth: truth.to_string() });
        }
    }
    out
}
because!(tomi_changes, StoryState, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons, whose state gives the label of the set, each with the label that the state gives");

const TOMQA_WHERE_IS: &str = "Where is the ";
because!(TOMQA_WHERE_IS, StoryState, "the words before the thing of a question of where it really is");

const TOMQA_REALLY: &str = " really?";
because!(TOMQA_REALLY, StoryState, "the words after the thing of a question of where it really is");

const TOMQA_WHERE_WAS: &str = "Where was the ";
because!(TOMQA_WHERE_WAS, StoryState, "the words before the thing of a question of where it was at the beginning");

const TOMQA_BEGINNING: &str = " at the beginning?";
because!(TOMQA_BEGINNING, StoryState, "the words after the thing of a question of where it was at the beginning");

const TOMQA_WHERE_WILL: &str = "Where will ";
because!(TOMQA_WHERE_WILL, StoryState, "the words before the person of a question of where the person will look");

const TOMQA_LOOK_FOR: &str = " look for the ";
because!(TOMQA_LOOK_FOR, StoryState, "the words between the person and the thing of a question of where the person will look");

const TOMQA_WHERE_DOES: &str = "Where does ";
because!(TOMQA_WHERE_DOES, StoryState, "the words before the first person of a question of where one person thinks that another person searches");

const TOMQA_THINK_THAT: &str = " think that ";
because!(TOMQA_THINK_THAT, StoryState, "the words between the two persons of a question of where one person thinks that another person searches");

const TOMQA_SEARCHES_FOR: &str = " searches for the ";
because!(TOMQA_SEARCHES_FOR, StoryState, "the words between the second person and the thing of a question of where one person thinks that another person searches");

const TOMQA_ASKED_END: &str = "?";
because!(TOMQA_ASKED_END, StoryState, "the mark at the end of a question of where a person will look or searches");

const TOMQA_PERSON_IS_IN: &str = " is in the ";
because!(TOMQA_PERSON_IS_IN, StoryState, "the words between a person and the room where the person is at the start of a story");

const SORT_ANOTHER_LEAVES: &str = "another person leaves in place of the person who left before the move";
because!(SORT_ANOTHER_LEAVES, StoryState, "the sort of the change where a person that the story does not hold enters before the placing and is the one who leaves, so that the person who left sees the move");

const SORT_COMES_BACK: &str = "the person who left comes back before the move";
because!(SORT_COMES_BACK, StoryState, "the sort of the change where the person who left enters the room again right after leaving, so that the person sees the move");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Asked {
    Real,
    Begin,
    Look,
    Think,
}
because!(Asked, StoryState, "what a question of the belief set asks: where the thing really is, where it was at the beginning, where a person will look for it, or where one person thinks that another person searches for it");

const TOMQA_FORMS: &[(Asked, &[Option<&str>])] = &[
    (Asked::Real, &[Some(TOMQA_WHERE_IS), None, Some(TOMQA_REALLY)]),
    (Asked::Begin, &[Some(TOMQA_WHERE_WAS), None, Some(TOMQA_BEGINNING)]),
    (Asked::Look, &[Some(TOMQA_WHERE_WILL), None, Some(TOMQA_LOOK_FOR), None, Some(TOMQA_ASKED_END)]),
    (Asked::Think, &[Some(TOMQA_WHERE_DOES), None, Some(TOMQA_THINK_THAT), None, Some(TOMQA_SEARCHES_FOR), None, Some(TOMQA_ASKED_END)]),
];
because!(TOMQA_FORMS, StoryState, "the forms of the questions of the belief set: fixed words, and a place for one name between them (a person or the thing)");

fn asked_parts(question: &str) -> Option<(Asked, Vec<String>)> {
    TOMQA_FORMS.iter().find_map(|(asked, steps)| named_steps(stripped(question), steps).map(|names| (*asked, names)))
}
because!(asked_parts, StoryState, "what a question of the belief set asks and its names, the thing last; nothing for a question of another form");

fn tomqa_answer(sentences: &[String], question: &str) -> Option<String> {
    let (asked, names) = asked_parts(question)?;
    let state = thing_state(&story_events(sentences, Some(names.last()?)));
    match asked {
        Asked::Real => state.real,
        Asked::Begin => state.begin,
        Asked::Look => state.first.get(&names[0]).cloned(),
        Asked::Think => state.second.get(&(names[0].clone(), names[1].clone())).cloned(),
    }
}
because!(tomqa_answer, StoryState, "the true answer of a question of the belief set by the state of the story: the place of the thing, its first place, the first belief of the person, or the place where the first person thinks that the second believes it to be");

fn one_leaving(sentences: &[String], thing: &str) -> Option<(usize, usize, usize)> {
    let placing = sentences.iter().rposition(|sentence| placed(sentence).is_some_and(|(placed_thing, _)| name_as_word(&placed_thing) == thing))?;
    let moves: Vec<usize> = moves_at(sentences).into_iter().filter(|at| *at > placing && moved(&sentences[*at]).is_some_and(|(_, moved_thing, _)| name_as_word(&moved_thing) == thing)).collect();
    let exits: Vec<usize> = sentences.iter().enumerate().filter(|(at, sentence)| *at > placing && person_room(sentence, TOMI_EXITED).is_some()).map(|(at, _)| at).collect();
    match (moves.as_slice(), exits.as_slice()) {
        ([moving], [leaving]) if leaving < moving => Some((placing, *leaving, *moving)),
        _ => None,
    }
}
because!(one_leaving, StoryState, "the places of the placing of the thing, of the one leaving after it and of the one move of the thing after it, when the leaving comes before the move: the stories where a person does not see the move");

fn another_leaves(sentences: &[String], thing: &str, free: &[String]) -> Option<Vec<String>> {
    let (placing, leaving, _) = one_leaving(sentences, thing)?;
    let (_, room) = person_room(&sentences[leaving], TOMI_EXITED)?;
    let told = sentences.join(TOMI_WORD_GAP);
    let third = free.iter().find(|name| !told.split(TOMI_GAP).any(|word| word.trim_end_matches(TOMI_END) == name.as_str()))?;
    let mut out = sentences.to_vec();
    out[leaving] = format!("{third}{TOMI_EXITED}{room}{TOMI_END}");
    out.insert(placing, format!("{third}{TOMI_ENTERED}{room}{TOMI_END}"));
    Some(out)
}
because!(another_leaves, StoryState, "the story where a person that it does not hold enters before the placing and leaves in place of the person who left, so that this person stays and sees the move");

fn comes_back(sentences: &[String], thing: &str) -> Option<Vec<String>> {
    let (_, leaving, _) = one_leaving(sentences, thing)?;
    let (person, room) = person_room(&sentences[leaving], TOMI_EXITED)?;
    let mut out = sentences.to_vec();
    out.insert(leaving + 1, format!("{person}{TOMI_ENTERED}{room}{TOMI_END}"));
    Some(out)
}
because!(comes_back, StoryState, "the story where the person who left enters the room again right after leaving, so that the person sees the move");

pub fn tomqa_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let part: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let free: Vec<String> = lessons.iter().flat_map(|case| case.sentences.iter().filter_map(|sentence| person_room(sentence, TOMI_ENTERED).map(|(person, _)| person))).collect::<BTreeSet<String>>().into_iter().collect();
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in part {
        let case = &lessons[place];
        let Some((asked, names)) = asked_parts(&case.question) else { continue };
        if !matches!(asked, Asked::Look | Asked::Think) || tomqa_answer(&case.sentences, &case.question).as_deref() != Some(case.answer.as_str()) {
            continue;
        }
        let Some(thing) = names.last() else { continue };
        for (sort, changed) in [(SORT_ANOTHER_LEAVES, another_leaves(&case.sentences, thing, &free)), (SORT_COMES_BACK, comes_back(&case.sentences, thing))] {
            let Some(sentences) = changed else { continue };
            let Some(truth) = tomqa_answer(&sentences, &case.question) else { continue };
            out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences, question: case.question.clone(), truth });
        }
    }
    out
}
because!(tomqa_changes, StoryState, "the changed stories of the belief questions of the lessons that the model does not learn, or of all lessons for a model of all lessons, whose state gives the answer of the set: a person who did not see the move now sees it, so a learner that tracks the persons follows the new answer and a learner that reads the order of the words does not");
