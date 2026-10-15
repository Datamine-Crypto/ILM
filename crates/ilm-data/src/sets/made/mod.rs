pub mod draw;
pub mod family;
pub mod files;
pub mod reader;
pub mod texts;

use ilm_core::base::words::words;
use crate::sets::made::draw::draw_family;
use crate::sets::made::family::Family;
use crate::sets::made::reader::made_answers;
use patterns::{because, source};

pub struct FamilySeeds;
source!(FamilySeeds, "a family is named by its seed: the kind of a family is its seed after a fixed start, and the kind with the deep read part has a fixed end more; the first draw of a seed that gives texts is the family of the seed");

pub struct HeldFamilies;
source!(HeldFamilies, "a test that no development has seen: the families of a range of seeds are made by the same program as the families of development, but no run of development reads them, so a learner that was tuned on the families of development is measured on new families once");

pub const MADE_FAMILIES: usize = 50;
because!(MADE_FAMILIES, FamilySeeds, "the families of development: every run may make and read them");

pub const MADE_HELD_END: usize = 100;
because!(MADE_HELD_END, HeldFamilies, "the end of the range of held families, which starts after the families of development");

const MADE_DRAWS: usize = 200;
because!(MADE_DRAWS, FamilySeeds, "the draws of a seed that are tried before the seed is said to give no family");

pub const MADE_PREFIX: &str = "made_";
because!(MADE_PREFIX, FamilySeeds, "the start of the name of a kind of a family, before its seed");

pub const MADE_DEEP: &str = "_deep";
because!(MADE_DEEP, FamilySeeds, "the end of the name of the kind of a family whose read part holds the deep texts");

pub fn made_family(number: usize, held_open: bool) -> Result<Family, String> {
    if (MADE_FAMILIES..MADE_HELD_END).contains(&number) && !held_open {
        return Err(format!("the family of seed {number} is held for one test: no run of development makes it"));
    }
    for draw in 0..MADE_DRAWS {
        let mut family = draw_family(number, draw).family;
        if family.make().is_ok() {
            return Ok(family);
        }
    }
    Err(format!("no family for seed {number}"))
}
because!(made_family, FamilySeeds, HeldFamilies, "the family of a seed: the first draw that gives its texts; a held family only when the run is the test of the held families");

pub fn made_kind(kind: &str) -> Option<(usize, bool)> {
    let rest = kind.strip_prefix(MADE_PREFIX)?;
    let (number, deep) = match rest.strip_suffix(MADE_DEEP) {
        Some(number) => (number, true),
        None => (rest, false),
    };
    number.parse::<usize>().ok().map(|number| (number, deep))
}
because!(made_kind, FamilySeeds, "the seed of the family of a kind and whether its read part holds the deep texts, from the name of the kind");

pub fn made_answer(family: &Family, text: &str) -> Option<String> {
    let found = made_answers(family, &words(text));
    match found.as_slice() {
        [one] => Some(one.join(family.between())),
        _ => None,
    }
}
because!(made_answer, FamilySeeds, "the answer of a text by the interpreter of the family, written as the lessons write an answer; nothing for a text that the grammar does not hold or that has two readings with two answers; a text whose answer has no word gives the empty answer");
