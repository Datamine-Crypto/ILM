use ilm_core::base::twister::TwisterDraws;
use crate::sets::made::family::{Family, MadeChange, MadeDiscipline, MadeSay, MadeSlot, MadeSort, MadeWay};
use patterns::{because, source};
use std::collections::HashSet;

pub struct FamilyDraw;
source!(FamilyDraw, "the draw of a family from a named seed: its sort, its discipline, its items and what they say, the parts of its functions, how each function is written in one, two or three ways, and what each function does; each choice is one draw of the twister in a fixed order, so the same seed gives the same family in every run");

const DRAW_CONSONANTS: &str = "bdfgjklmnprstvz";
because!(DRAW_CONSONANTS, FamilyDraw, "the letters of the odd places of a made-up word");

const DRAW_VOWELS: &str = "aiou";
because!(DRAW_VOWELS, FamilyDraw, "the letters of the even places of a made-up word");

const DRAW_MARKED: &str = "zvjk";
because!(DRAW_MARKED, FamilyDraw, "a made-up word holds one of these letters, which few words of a language hold, so it is no word of a text by people");

const DRAW_FIRST_LETTER: char = 'a';
because!(DRAW_FIRST_LETTER, FamilyDraw, "the first sign of an answer that is a row of signs: the signs are the small letters in their order, then the digits");

const DRAW_LAST_LETTER: char = 'z';
because!(DRAW_LAST_LETTER, FamilyDraw, "the last small letter among the signs of an answer");

const DRAW_RADIX: u32 = 10;
because!(DRAW_RADIX, FamilyDraw, "the digits that a sign of an answer can be, after the small letters");

const DRAW_BRACKETS: &[(&str, &str)] = &[("(", ")"), ("[", "]"), ("{", "}"), ("<", ">")];
because!(DRAW_BRACKETS, FamilyDraw, "the pairs of brackets that a family can write round the parts of a function or round a part that is a function");

const DRAW_SEPARATORS: &[&str] = &["-", "+", "*", "/", "="];
because!(DRAW_SEPARATORS, FamilyDraw, "the signs that a family can write between the parts of a function");

const DRAW_WORD_LENGTHS: &[usize] = &[4, 5];
because!(DRAW_WORD_LENGTHS, FamilyDraw, "how long a made-up word is: long enough that the words of a family differ, short enough to read");

const DRAW_SORT_WEIGHTS: &[u32] = &[70, 15, 15];
because!(DRAW_SORT_WEIGHTS, FamilyDraw, "how often a family is of each sort: items that say answer words, items that say themselves, the text with a change");

const DRAW_SIGNS_SHARE: f64 = 0.25;
because!(DRAW_SIGNS_SHARE, FamilyDraw, "the share of the families of items that say answer words whose answers are rows of signs");

const DRAW_ITEMS: (usize, usize) = (4, 12);
because!(DRAW_ITEMS, FamilyDraw, "the least and the most items of a family");

const DRAW_NEW_ITEMS: usize = 10;
because!(DRAW_NEW_ITEMS, FamilyDraw, "the items that no lesson holds, for a family whose items say themselves");

const DRAW_POOL: (usize, usize) = (4, 8);
because!(DRAW_POOL, FamilyDraw, "the least and the most answer words of the pool from which runs of answer words are drawn");

const DRAW_RUN_LENGTHS: &[usize] = &[2, 3];
because!(DRAW_RUN_LENGTHS, FamilyDraw, "the lengths of a run of answer words that an item says");

const DRAW_SOME: (usize, usize) = (1, 3);
because!(DRAW_SOME, FamilyDraw, "the least and the most answer words that an item says when it says some");

const DRAW_FUNCTIONS: (usize, usize) = (2, 5);
because!(DRAW_FUNCTIONS, FamilyDraw, "the least and the most functions of a family");

const DRAW_PART_COUNTS: &[usize] = &[1, 2, 3];
because!(DRAW_PART_COUNTS, FamilyDraw, "the parts that a function can have, and the ways that a function can be written");

const DRAW_PART_WEIGHTS: &[u32] = &[40, 45, 15];
because!(DRAW_PART_WEIGHTS, FamilyDraw, "how often a function has one, two or three parts");

const DRAW_WAY_WEIGHTS: &[u32] = &[70, 22, 8];
because!(DRAW_WAY_WEIGHTS, FamilyDraw, "how often a function is written in one, two or three ways");

const DRAW_BETWEEN_WEIGHTS: &[u32] = &[60, 25, 15];
because!(DRAW_BETWEEN_WEIGHTS, FamilyDraw, "how often nothing, the separating sign or a word stands between the parts of a function");

const DRAW_GROUP_WEIGHTS: &[u32] = &[70, 15, 15];
because!(DRAW_GROUP_WEIGHTS, FamilyDraw, "how often the word of a function of more than one part stands between, before or after its parts, in a family of groups");

const DRAW_TURN_TWO: f64 = 0.4;
because!(DRAW_TURN_TWO, FamilyDraw, "the share of the functions of two parts that say the answers of their parts in the other order");

const DRAW_TURN_THREE: f64 = 0.6;
because!(DRAW_TURN_THREE, FamilyDraw, "the share of the functions of three parts that say the answers of their parts in another order");

const DRAW_TIMES: (usize, usize) = (2, 4);
because!(DRAW_TIMES, FamilyDraw, "the least and the most times that a function says the answer of one part");

const DRAW_AGAIN_AT_END: f64 = 0.3;
because!(DRAW_AGAIN_AT_END, FamilyDraw, "the share of the functions that say an answer again where the answer of their first part comes once more at the end");

const DRAW_CHANGED_FUNCTIONS: &[usize] = &[1, 1, 2];
because!(DRAW_CHANGED_FUNCTIONS, FamilyDraw, "how many functions a family of the text with a change changes: one more often than two");

const FEATURE_ONE: &str = "an item says one answer word";
because!(FEATURE_ONE, FamilyDraw, "the feature of a family whose items each say one answer word");

const FEATURE_RUN: &str = "an item says a run of two or three answer words";
because!(FEATURE_RUN, FamilyDraw, "the feature of a family whose items each say a run of answer words");

const FEATURE_SOME: &str = "an item says one to three answer words";
because!(FEATURE_SOME, FamilyDraw, "the feature of a family whose items say some answer words");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawHow {
    One,
    Run,
    Some,
}
because!(DrawHow, FamilyDraw, "how the items of a family say answer words: one word, a run, or some words");

struct DrawState {
    rng: TwisterDraws,
    used: HashSet<String>,
    features: Vec<String>,
}
because!(DrawState, FamilyDraw, "the draws of one family with the words that it used, so that no word is drawn twice, and its features as they come. word draws a made-up word; sign draws a sign; pick draws one of some texts");

impl DrawState {
    fn word(&mut self) -> String {
        loop {
            let length = DRAW_WORD_LENGTHS[self.rng.choice(DRAW_WORD_LENGTHS.len())];
            let mut word = String::with_capacity(length);
            for place in 0..length {
                let letters = if place % DRAW_ITEMS_EVEN == 0 { DRAW_CONSONANTS } else { DRAW_VOWELS };
                word.push(letters.as_bytes()[self.rng.choice(letters.len())] as char);
            }
            if !self.used.contains(&word) && word.chars().any(|letter| DRAW_MARKED.contains(letter)) {
                self.used.insert(word.clone());
                return word;
            }
        }
    }

    fn sign(&mut self) -> String {
        loop {
            let signs: Vec<char> = (DRAW_FIRST_LETTER..=DRAW_LAST_LETTER).chain((0..DRAW_RADIX).filter_map(|digit| char::from_digit(digit, DRAW_RADIX))).collect();
            let sign = signs[self.rng.choice(signs.len())].to_string();
            if !self.used.contains(&sign) {
                self.used.insert(sign.clone());
                return sign;
            }
        }
    }

    fn out(&mut self, signs: bool) -> String {
        if signs { self.sign() } else { self.word() }
    }

    fn pick<T: Copy>(&mut self, options: &[T]) -> T {
        options[self.rng.choice(options.len())]
    }

    fn feature(&mut self, text: &str) {
        self.features.push(text.to_string());
    }
}

const DRAW_ITEMS_EVEN: usize = 2;
because!(DRAW_ITEMS_EVEN, FamilyDraw, "the step between the places of a made-up word that hold letters of one kind: consonants and vowels take turns");

fn draw_joined(parts: usize, gaps: &[Option<String>]) -> Vec<MadeSlot> {
    let mut row: Vec<MadeSlot> = Vec::new();
    for part in 0..parts {
        row.push(MadeSlot::Part(part));
        if let Some(Some(gap)) = gaps.get(part).filter(|_| part + 1 < parts) {
            row.push(MadeSlot::Fixed(gap.clone()));
        }
    }
    row
}
because!(draw_joined, FamilyDraw, "the parts of a way in their order with the fixed words between them");

fn draw_way(layout: Vec<MadeSlot>, level: u32, accept: Option<Vec<Option<u32>>>) -> MadeWay {
    let parts = layout.iter().filter(|slot| matches!(slot, MadeSlot::Part(_))).count();
    MadeWay { layout, level, accept: accept.unwrap_or_else(|| vec![None; parts]), parts, says: Vec::new() }
}
because!(draw_way, FamilyDraw, "a way from its places, its level and the highest level of each part, which is no bound when it is not given");

pub(super) struct DrawnFamily {
    pub(super) family: Family,
}
because!(DrawnFamily, FamilyDraw, "a family as the draw left it, before its texts are made");

impl DrawState {
    fn between(&mut self, parts: usize, separator: &str) -> Vec<Option<String>> {
        if parts <= 1 {
            return vec![None; parts.saturating_sub(1)];
        }
        match self.rng.weighted(DRAW_BETWEEN_WEIGHTS) {
            0 => vec![None; parts - 1],
            1 => {
                self.feature("a separating sign between the parts");
                vec![Some(separator.to_string()); parts - 1]
            }
            _ => {
                self.feature("a word between the parts");
                let word = self.word();
                vec![Some(word); parts - 1]
            }
        }
    }

    fn written(&mut self, discipline: MadeDiscipline, parts: usize, level: u32, brackets: (&str, &str), separator: &str, side_before: bool) -> MadeWay {
        match discipline {
            MadeDiscipline::Before => {
                self.feature("the word of a function before its parts");
                let word = self.word();
                let gaps = self.between(parts, separator);
                let mut layout = vec![MadeSlot::Fixed(word)];
                layout.extend(draw_joined(parts, &gaps));
                draw_way(layout, 0, None)
            }
            MadeDiscipline::After => {
                self.feature("the word of a function after its parts");
                let gaps = self.between(parts, separator);
                let mut layout = draw_joined(parts, &gaps);
                layout.push(MadeSlot::Fixed(self.word()));
                draw_way(layout, 0, None)
            }
            MadeDiscipline::Closed => {
                let how = self.pick(&[DrawClosed::Words, DrawClosed::BracketsAfter, DrawClosed::WordInside]);
                let gaps = self.between(parts, separator);
                let inner = draw_joined(parts, &gaps);
                if how == DrawClosed::Words {
                    self.feature("words of a function on both sides of its parts");
                    let mut layout = vec![MadeSlot::Fixed(self.word())];
                    layout.extend(inner);
                    layout.push(MadeSlot::Fixed(self.word()));
                    return draw_way(layout, 0, None);
                }
                self.feature("brackets round the parts of a function");
                let mut layout = if how == DrawClosed::BracketsAfter { vec![MadeSlot::Fixed(self.word()), MadeSlot::Fixed(brackets.0.to_string())] } else { vec![MadeSlot::Fixed(brackets.0.to_string()), MadeSlot::Fixed(self.word())] };
                layout.extend(inner);
                layout.push(MadeSlot::Fixed(brackets.1.to_string()));
                draw_way(layout, 0, None)
            }
            MadeDiscipline::Group => {
                let how = if parts == 1 { [DrawPlace::Before, DrawPlace::After][self.rng.choice(DRAW_SIDES)] } else { [DrawPlace::Between, DrawPlace::Before, DrawPlace::After][self.rng.weighted(DRAW_GROUP_WEIGHTS)] };
                self.feature(match how {
                    DrawPlace::Before => "the word of a function before its parts",
                    DrawPlace::After => "the word of a function after its parts",
                    DrawPlace::Between => "a word of a function between its parts",
                });
                match how {
                    DrawPlace::Between => {
                        let gaps: Vec<Option<String>> = (0..parts - 1).map(|_| Some(self.word())).collect();
                        draw_way(draw_joined(parts, &gaps), 0, None)
                    }
                    DrawPlace::Before => {
                        let word = self.word();
                        let gaps = self.between(parts, separator);
                        let mut layout = vec![MadeSlot::Fixed(word)];
                        layout.extend(draw_joined(parts, &gaps));
                        draw_way(layout, 0, None)
                    }
                    DrawPlace::After => {
                        let gaps = self.between(parts, separator);
                        let mut layout = draw_joined(parts, &gaps);
                        layout.push(MadeSlot::Fixed(self.word()));
                        draw_way(layout, 0, None)
                    }
                }
            }
            MadeDiscipline::Levels => {
                if parts == 1 {
                    self.feature(if side_before { "the word of a function before its parts" } else { "the word of a function after its parts" });
                    let word = MadeSlot::Fixed(self.word());
                    let layout = if side_before { vec![word, MadeSlot::Part(0)] } else { vec![MadeSlot::Part(0), word] };
                    return draw_way(layout, 1, Some(vec![Some(1)]));
                }
                self.feature("a word of a function between its parts");
                let gaps: Vec<Option<String>> = (0..parts - 1).map(|_| Some(self.word())).collect();
                draw_way(draw_joined(parts, &gaps), level, Some(vec![Some(level - 1); parts]))
            }
        }
    }

    fn does(&mut self, parts: usize, sort: MadeSort, signs: bool) -> Vec<MadeSay> {
        let mut order: Vec<usize> = (0..parts).collect();
        if parts == DRAW_SIDES && self.rng.unit() < DRAW_TURN_TWO {
            order = vec![1, 0];
        } else if parts == DRAW_SIDES + 1 && self.rng.unit() < DRAW_TURN_THREE {
            while order.iter().enumerate().all(|(place, part)| place == *part) {
                self.rng.shuffle(&mut order);
            }
        }
        let kept = order.iter().enumerate().all(|(place, part)| place == *part);
        self.feature(if kept { "the answers of the parts in their order" } else { "the answers of the parts in another order" });
        let mut row: Vec<MadeSay> = order.iter().map(|part| MadeSay::Part(*part, MadeChange::Keep)).collect();
        let mut more = vec![DrawDoes::Twice, DrawDoes::Fixed, DrawDoes::Take, DrawDoes::Leave, DrawDoes::Round];
        if sort == MadeSort::Says {
            more.push(DrawDoes::Look);
        }
        if parts > 1 {
            more.extend([DrawDoes::Nothing, DrawDoes::Nothing]);
        }
        let how = more[self.rng.choice(more.len())];
        let at = self.rng.choice(row.len());
        let part_of = |row: &[MadeSay], at: usize| match &row[at] {
            MadeSay::Part(part, _) => *part,
            MadeSay::Word(_) => 0,
        };
        match how {
            DrawDoes::Twice => {
                let times = self.rng.between(DRAW_TIMES.0, DRAW_TIMES.1);
                if parts > 1 && self.rng.unit() < DRAW_AGAIN_AT_END {
                    row.push(row[0].clone());
                } else {
                    let piece = row[at].clone();
                    row.splice(at..=at, std::iter::repeat_n(piece, times));
                }
                self.feature("the answer of a part two to four times");
            }
            DrawDoes::Fixed => {
                let place = self.rng.choice(row.len() + 1);
                let word = self.out(signs);
                row.insert(place, MadeSay::Word(word));
                self.feature("a fixed answer word");
            }
            DrawDoes::Take => {
                let change = [MadeChange::First, MadeChange::Last][self.rng.choice(DRAW_SIDES)];
                row[at] = MadeSay::Part(part_of(&row, at), change);
                self.feature("the first or the last word of an answer alone");
            }
            DrawDoes::Leave => {
                let change = [MadeChange::Rest, MadeChange::Most][self.rng.choice(DRAW_SIDES)];
                row[at] = MadeSay::Part(part_of(&row, at), change);
                self.feature("an answer without its first or its last word");
            }
            DrawDoes::Round => {
                row[at] = MadeSay::Part(part_of(&row, at), MadeChange::Rev);
                self.feature("an answer turned round");
            }
            DrawDoes::Look => {
                row[at] = MadeSay::Part(part_of(&row, at), MadeChange::Look);
                self.feature("a word looked up in a table");
            }
            DrawDoes::Nothing => {}
        }
        row
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawPlace {
    Before,
    After,
    Between,
}
because!(DrawPlace, FamilyDraw, "where the word of a function stands in a family of groups");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawDoes {
    Twice,
    Fixed,
    Take,
    Leave,
    Round,
    Look,
    Nothing,
}
because!(DrawDoes, FamilyDraw, "the one thing more that a function does: an answer again, a fixed word, the first or the last word alone, an answer without its first or its last word, an answer turned round, a word looked up, or nothing more");

const DRAW_SIDES: usize = 2;
because!(DRAW_SIDES, FamilyDraw, "the parts of a function whose answers a swap turns round, and the options of a choice of a pair: before or after, first or last");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawClosed {
    Words,
    BracketsAfter,
    WordInside,
}
because!(DrawClosed, FamilyDraw, "how a function of a closed family is written: a word on each side of its parts, brackets after its word, or its word inside the brackets");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawText {
    Places,
    Front,
    Word,
}
because!(DrawText, FamilyDraw, "the change of a function of a family of the text with a change: two parts change places, the last part goes to the front, or the word of the function becomes another word");

pub(super) fn draw_family(number: usize, draw: usize) -> DrawnFamily {
    let mut state = DrawState { rng: TwisterDraws::seeded(&format!("made family {number}, draw {draw}")), used: HashSet::new(), features: Vec::new() };
    let sort = [MadeSort::Says, MadeSort::Itself, MadeSort::Text][state.rng.weighted(DRAW_SORT_WEIGHTS)];
    let signs = sort == MadeSort::Says && state.rng.unit() < DRAW_SIGNS_SHARE;
    let disciplines: &[MadeDiscipline] = if sort == MadeSort::Text { &[MadeDiscipline::Before, MadeDiscipline::After, MadeDiscipline::Closed] } else { &[MadeDiscipline::Before, MadeDiscipline::After, MadeDiscipline::Closed, MadeDiscipline::Group, MadeDiscipline::Levels] };
    let discipline = state.pick(disciplines);
    let count = state.rng.between(DRAW_ITEMS.0, DRAW_ITEMS.1);
    let items: Vec<String> = (0..count).map(|_| state.word()).collect();
    let new_items: Vec<String> = if sort != MadeSort::Says { (0..DRAW_NEW_ITEMS).map(|_| state.word()).collect() } else { Vec::new() };
    let mut says: Vec<(String, Vec<String>)> = Vec::new();
    if sort == MadeSort::Says {
        let how = state.pick(&[DrawHow::One, DrawHow::Run, DrawHow::Some]);
        if how == DrawHow::One {
            says = items.iter().map(|item| (item.clone(), vec![state.out(signs)])).collect();
        } else {
            let size = state.rng.between(DRAW_POOL.0, DRAW_POOL.1);
            let pool: Vec<String> = (0..size).map(|_| state.out(signs)).collect();
            loop {
                let mut distinct: Vec<&Vec<String>> = Vec::new();
                for (_, words) in &says {
                    if !distinct.contains(&words) {
                        distinct.push(words);
                    }
                }
                if distinct.len() >= items.len() && !says.is_empty() {
                    break;
                }
                says = items
                    .iter()
                    .map(|item| {
                        let length = if how == DrawHow::Run { DRAW_RUN_LENGTHS[state.rng.choice(DRAW_RUN_LENGTHS.len())] } else { state.rng.between(DRAW_SOME.0, DRAW_SOME.1) };
                        (item.clone(), (0..length).map(|_| pool[state.rng.choice(pool.len())].clone()).collect())
                    })
                    .collect();
            }
        }
        state.feature(match how {
            DrawHow::One => FEATURE_ONE,
            DrawHow::Run => FEATURE_RUN,
            DrawHow::Some => FEATURE_SOME,
        });
    } else {
        state.feature(if sort == MadeSort::Itself { "an item says itself" } else { "the answer is the text with a change" });
        state.feature("the read part holds items that no lesson holds");
    }
    if signs {
        state.feature("the answer is one row of signs with no space");
    }
    let functions_count = state.rng.between(DRAW_FUNCTIONS.0, DRAW_FUNCTIONS.1);
    let mut parts: Vec<usize> = (0..functions_count).map(|_| DRAW_PART_COUNTS[state.rng.weighted(DRAW_PART_WEIGHTS)]).collect();
    if parts.iter().all(|count| *count == 1) {
        let place = state.rng.choice(parts.len());
        parts[place] = DRAW_SIDES;
    }
    if discipline == MadeDiscipline::Levels && parts.iter().all(|count| *count > 1) {
        let place = state.rng.choice(parts.len());
        parts[place] = 1;
        if parts.iter().all(|count| *count == 1) {
            parts.push(DRAW_SIDES);
        }
    }
    let mut kinds: Vec<usize> = parts.clone();
    kinds.sort_unstable();
    kinds.dedup();
    for count in kinds {
        state.feature(&format!("a function of {count} part{}", if count > 1 { "s" } else { "" }));
    }
    let brackets = state.pick(DRAW_BRACKETS);
    let separator = state.pick(DRAW_SEPARATORS);
    let side_before = state.rng.choice(DRAW_SIDES) == 0;
    let mut wrap: Option<(String, String)> = None;
    if discipline == MadeDiscipline::Group {
        wrap = Some((brackets.0.to_string(), brackets.1.to_string()));
        state.feature("a part that is a function stands in brackets");
    }
    if discipline == MadeDiscipline::Levels {
        state.feature("levels with no brackets");
    }
    let mut level: u32 = 1;
    let mut functions: Vec<Vec<MadeWay>> = Vec::new();
    for count in &parts {
        if discipline == MadeDiscipline::Levels && *count > 1 {
            level += 1;
        }
        let ways_count = DRAW_PART_COUNTS[state.rng.weighted(DRAW_WAY_WEIGHTS)];
        let mut ways: Vec<MadeWay> = (0..ways_count).map(|_| state.written(discipline, *count, level, brackets, separator, side_before)).collect();
        if ways.len() > 1 {
            state.feature("two or three ways to write a function");
        }
        let row = (sort != MadeSort::Text).then(|| state.does(*count, sort, signs));
        for way in &mut ways {
            way.says = match &row {
                Some(row) => row.clone(),
                None => way.layout.iter().map(|slot| match slot {
                    MadeSlot::Fixed(word) => MadeSay::Word(word.clone()),
                    MadeSlot::Part(part) => MadeSay::Part(*part, MadeChange::Keep),
                }).collect(),
            };
        }
        functions.push(ways);
    }
    if sort == MadeSort::Text {
        let changed = DRAW_CHANGED_FUNCTIONS[state.rng.choice(DRAW_CHANGED_FUNCTIONS.len())].min(functions.len());
        for function in state.rng.pool_sample(functions.len(), changed) {
            let how = if functions[function][0].parts > 1 { state.pick(&[DrawText::Places, DrawText::Front, DrawText::Word]) } else { state.pick(&[DrawText::Word]) };
            let new = state.word();
            for way in &mut functions[function] {
                let mut row = way.says.clone();
                let at: Vec<usize> = row.iter().enumerate().filter(|(_, piece)| matches!(piece, MadeSay::Part(..))).map(|(place, _)| place).collect();
                match how {
                    DrawText::Places => {
                        if let (Some(first), Some(last)) = (at.first(), at.last()) {
                            row.swap(*first, *last);
                        }
                    }
                    DrawText::Front => {
                        if let Some(last) = at.last() {
                            let piece = row.remove(*last);
                            row.insert(0, piece);
                        }
                    }
                    DrawText::Word => {
                        if let Some(own) = row.iter().position(|piece| matches!(piece, MadeSay::Word(word) if word.chars().count() > 1)) {
                            row[own] = MadeSay::Word(new.clone());
                        }
                    }
                }
                way.says = row;
            }
            state.feature(match how {
                DrawText::Places => "two parts change places",
                DrawText::Front => "the last part goes to the front",
                DrawText::Word => "the word of a function becomes another word",
            });
        }
    }
    let mut table: Vec<(String, String)> = Vec::new();
    let looks = functions.iter().flatten().flat_map(|way| way.says.iter()).any(|piece| matches!(piece, MadeSay::Part(_, MadeChange::Look)));
    if looks {
        let mut known: Vec<String> = says.iter().flat_map(|(_, words)| words.iter().cloned()).collect();
        known.extend(functions.iter().flatten().flat_map(|way| way.says.iter()).filter_map(|piece| match piece {
            MadeSay::Word(word) => Some(word.clone()),
            MadeSay::Part(..) => None,
        }));
        known.sort();
        known.dedup();
        let mut turned = known.clone();
        while known.len() > 1 && known.iter().zip(&turned).any(|(left, right)| left == right) {
            state.rng.shuffle(&mut turned);
        }
        table = known.into_iter().zip(turned).collect();
    }
    let mut fixed: HashSet<String> = functions.iter().flatten().flat_map(|way| way.layout.iter()).filter_map(|slot| match slot {
        MadeSlot::Fixed(word) => Some(word.clone()),
        MadeSlot::Part(_) => None,
    }).collect();
    if let Some((open, close)) = &wrap {
        fixed.insert(open.clone());
        fixed.insert(close.clone());
    }
    let mut features: Vec<String> = Vec::new();
    for feature in state.features {
        if !features.contains(&feature) {
            features.push(feature);
        }
    }
    DrawnFamily {
        family: Family { number, drawn: draw, features, table, wrap, says, fixed, sort, signs, discipline, items, new_items, functions, lessons: Vec::new(), read: Vec::new(), deep: Vec::new() },
    }
}
because!(draw_family, FamilyDraw, "one draw of a family from its seed and the number of the draw, with every choice in the order of the draw");
