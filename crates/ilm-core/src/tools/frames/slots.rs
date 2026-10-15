use patterns::{because, source};

pub struct SlotFrames;
source!(SlotFrames, "a relation word of a sentence is a slot, and a slot gives a fact of an owner and a person: the person is the relation of the owner. The things next to a slot are its anchors: the last thing before it, the thing before that one with another name, the first thing after it, the thing after that one with another name, and the first thing of the sentence when it is a third name; and out, a name of the story that is not in the sentence, or a person with no name. The frame of a slot is what stands around it with every thing written as one sign: the token before it, the token after it, how many things stand before and after it, how far the nearest ones are, and whether the second thing before it stands next to the first. Its keys go from the fullest to the shortest: both tokens with the token after the second thing before the slot; the tokens between the last thing and the slot when they are few; both tokens with the whole layout, with the counts of things, alone; one token with the layout; one token with the distance on its side; the layout; the distances; and all slots");

pub struct PluralSlots;
source!(PluralSlots, "a plural of a relation word is a slot of the same word that can have more than one person: a regular English plural ends in s or es, and a word that ends in f or fe has its plural in ves");

const PLURAL_TAILS: &[(&str, &[&str])] = &[("ves", &["f", "fe"]), ("es", &[""]), ("s", &[""])];
because!(PLURAL_TAILS, PluralSlots, "the endings of a plural, each with what can stand in its place in the singular, in the order in which they are tried");

const THING_SIGN: &str = "X";
because!(THING_SIGN, SlotFrames, "the sign of a thing in the frame of a slot; a word of a text is in small letters and never this sign");

const NO_BEFORE: &str = "<";
because!(NO_BEFORE, SlotFrames, "the token before a slot at the start of its sentence");

const NO_AFTER: &str = ">";
because!(NO_AFTER, SlotFrames, "the token after a slot at the end of its sentence");

const MOST_COUNT: usize = 2;
because!(MOST_COUNT, SlotFrames, "the most things on each side of a slot that its frame counts: more names say nothing more of the place of its owner and its person");

const MOST_DISTANCE: usize = 3;
because!(MOST_DISTANCE, SlotFrames, "the most tokens from a slot to its nearest thing on each side that its frame tells apart");

const GAP_SHORTEST: usize = 3;
because!(GAP_SHORTEST, SlotFrames, "the least distance from the last thing before a slot at which the tokens between them are a key of their own");

const GAP_LONGEST: usize = 4;
because!(GAP_LONGEST, SlotFrames, "the most distance from the last thing before a slot at which the tokens between them are a key of their own");

pub(super) const OUT_TEXT: &str = "out";
because!(OUT_TEXT, SlotFrames, "the name of the anchor outside the sentence, which a person with no name has in its text");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Who {
    Name(String),
    NoName(usize),
}
because!(Who, SlotFrames, "the owner or the person of a fact: a name of the story, or a person with no name, known by the place of its slot. text gives it as it is written when facts are put in order");

impl Who {
    pub(super) fn text(&self) -> String {
        match self {
            Who::Name(name) => name.clone(),
            Who::NoName(place) => format!("('{OUT_TEXT}', {place})"),
        }
    }

    pub(super) fn named(&self) -> bool {
        matches!(self, Who::Name(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Anchor {
    First,
    LeftOne,
    LeftTwo,
    RightOne,
    RightTwo,
    Far,
    Out,
}
because!(Anchor, SlotFrames, "where the owner or the person of a slot stands: the first thing of the sentence, the last thing before the slot, the one before that, the first thing after the slot, the one after that, another thing of the sentence, or outside the sentence; the order is the order of their short names, so that of two places with equal counts the same one is taken in every run");

pub(super) const ANCHOR_ORDER: &[Anchor] = &[Anchor::LeftOne, Anchor::RightOne, Anchor::LeftTwo, Anchor::RightTwo, Anchor::First];
because!(ANCHOR_ORDER, SlotFrames, "the anchors in the order in which a name of a step is looked for among them: the nearest first");

pub(super) type At = (Anchor, Anchor);
because!(At, SlotFrames, "the anchors of the owner and of the person of a slot");

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Where {
    pub(super) left_one: Option<usize>,
    pub(super) left_two: Option<usize>,
    pub(super) right_one: Option<usize>,
    pub(super) right_two: Option<usize>,
    pub(super) first: Option<usize>,
}
because!(Where, SlotFrames, "the places of the anchors of a slot in its sentence; get gives the place of one anchor, and nothing for an anchor that the sentence does not have");

impl Where {
    pub(super) fn get(&self, anchor: Anchor) -> Option<usize> {
        match anchor {
            Anchor::LeftOne => self.left_one,
            Anchor::LeftTwo => self.left_two,
            Anchor::RightOne => self.right_one,
            Anchor::RightTwo => self.right_two,
            Anchor::First => self.first,
            Anchor::Far | Anchor::Out => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Layout {
    before: usize,
    after: usize,
    left: usize,
    right: usize,
    next_to: bool,
}
because!(Layout, SlotFrames, "the layout of a slot: the things before and after it, the distances to the nearest ones, and whether the second thing before it stands next to the first");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum SlotKey {
    Second(String, String, usize, String),
    Gap(Vec<String>, String, usize, usize),
    Full(String, String, Layout),
    Counts(String, String, usize, usize),
    Tokens(String, String),
    Before(String, Layout),
    After(String, Layout),
    Left(String, usize, bool),
    Right(String, usize),
    Plan(Layout),
    Near(usize, usize, bool),
    All,
}
because!(SlotKey, SlotFrames, "one key of the frame of a slot. has_token says whether the key holds a token next to the slot, so that a sure place can be read from it");

impl SlotKey {
    pub(super) fn has_token(&self) -> bool {
        matches!(self, SlotKey::Second(..) | SlotKey::Gap(..) | SlotKey::Full(..) | SlotKey::Counts(..) | SlotKey::Tokens(..) | SlotKey::Before(..) | SlotKey::After(..))
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Sentence<'a> {
    pub(super) words: &'a [String],
    pub(super) places: &'a [usize],
    pub(super) lone: &'a [usize],
}
because!(Sentence, SlotFrames, "a sentence as the slots read it: its words, the places of its things in their order, and the places of slots whose person has no name and which stand as things for the other slots. who gives the owner or person at a place of a thing");

impl Sentence<'_> {
    pub(super) fn who(&self, place: usize) -> Who {
        if self.lone.contains(&place) { Who::NoName(place) } else { Who::Name(self.words[place].clone()) }
    }

    pub(super) fn holds(&self, place: usize) -> bool {
        self.places.contains(&place)
    }

    pub(super) fn anchors(&self, slot: usize) -> Where {
        let mut out = Where::default();
        let before: Vec<usize> = self.places.iter().copied().filter(|place| *place < slot).collect();
        let after: Vec<usize> = self.places.iter().copied().filter(|place| *place > slot).collect();
        if let Some(last) = before.last() {
            out.left_one = Some(*last);
            let mine = self.who(*last);
            if let Some(other) = before.iter().rev().find(|place| self.who(**place) != mine) {
                out.left_two = Some(*other);
                let start = self.who(before[0]);
                if start != mine && start != self.who(*other) {
                    out.first = Some(before[0]);
                }
            }
        }
        if let Some(next) = after.first() {
            out.right_one = Some(*next);
            let mine = self.who(*next);
            out.right_two = after.iter().find(|place| self.who(**place) != mine).copied();
        }
        out
    }

    fn token(&self, place: usize) -> String {
        if self.holds(place) { THING_SIGN.to_string() } else { self.words[place].clone() }
    }

    fn named_count(&self, keep: impl Fn(usize) -> bool) -> usize {
        let mut seen: Vec<Who> = Vec::new();
        for place in self.places.iter().copied().filter(|place| keep(*place)) {
            let who = self.who(place);
            if !seen.contains(&who) {
                seen.push(who);
            }
        }
        seen.len().min(MOST_COUNT)
    }

    pub(super) fn keys(&self, slot: usize, at: &Where) -> Vec<SlotKey> {
        let before = match slot.checked_sub(1) {
            Some(place) => self.token(place),
            None => NO_BEFORE.to_string(),
        };
        let after = if slot + 1 < self.words.len() { self.token(slot + 1) } else { NO_AFTER.to_string() };
        let counted_before = self.named_count(|place| place < slot);
        let counted_after = self.named_count(|place| place > slot);
        let left = at.left_one.map(|place| (slot - place).min(MOST_DISTANCE)).unwrap_or_default();
        let right = at.right_one.map(|place| (place - slot).min(MOST_DISTANCE)).unwrap_or_default();
        let next_to = matches!((at.left_two, at.left_one), (Some(two), Some(one)) if two + 1 == one);
        let layout = Layout { before: counted_before, after: counted_after, left, right, next_to };
        let mut keys: Vec<SlotKey> = Vec::new();
        if let Some(two) = at.left_two {
            keys.push(SlotKey::Second(before.clone(), after.clone(), counted_after, self.token(two + 1)));
        }
        if let Some(one) = at.left_one
            && (GAP_SHORTEST..=GAP_LONGEST).contains(&(slot - one))
        {
            keys.push(SlotKey::Gap(self.words[one + 1..slot].to_vec(), after.clone(), counted_before, counted_after));
        }
        keys.push(SlotKey::Full(before.clone(), after.clone(), layout));
        keys.push(SlotKey::Counts(before.clone(), after.clone(), counted_before, counted_after));
        keys.push(SlotKey::Tokens(before.clone(), after.clone()));
        keys.push(SlotKey::Before(before.clone(), layout));
        keys.push(SlotKey::After(after.clone(), layout));
        keys.push(SlotKey::Left(before, left, next_to));
        keys.push(SlotKey::Right(after, right));
        keys.push(SlotKey::Plan(layout));
        keys.push(SlotKey::Near(left, right, next_to));
        keys.push(SlotKey::All);
        keys
    }
}

pub(super) fn plural_forms(word: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (end, tails) in PLURAL_TAILS {
        if let Some(stem) = word.strip_suffix(end) {
            out.extend(tails.iter().map(|tail| format!("{stem}{tail}")));
        }
    }
    out
}
because!(plural_forms, PluralSlots, "the singulars that a word can be the plural of, in the order in which they are tried");
