use crate::base::exact::Amount;
use crate::base::symbols::Symbols;
use crate::tools::amount::calcs::AmountCalc;
use crate::tools::amount::counts::AmountRight;
use crate::tools::amount::forms::AmountForms;
use crate::tools::amount::steps::AmountItem;
use crate::tools::amount::tokens::{AmountCut, AmountMarks, AmountUnknown};
use patterns::{because, source};
use std::cell::RefCell;

pub struct OneReading;
source!(OneReading, "learning and reading go one way: a case is cut into tokens, given forms and calculated by the same tables and in the same working memory for a lesson and for a case to read, so a rule that was counted on the lessons means the same on a new case");

#[derive(Debug, Default)]
pub(super) struct AmountHand {
    pub(super) cut: AmountCut,
    pub(super) forms: AmountForms,
    pub(super) shown: Vec<(usize, Amount)>,
    pub(super) lower: String,
    pub(super) unknown: AmountUnknown,
    pub(super) found: Vec<(u32, u32)>,
    pub(super) tried: Vec<(u32, u32, u32)>,
    pub(super) items: Vec<AmountItem<Amount>>,
    pub(super) word_items: Vec<AmountItem<bool>>,
    pub(super) right: Vec<AmountRight>,
    pub(super) wheres: Vec<(u32, u32)>,
    pub(super) fixed: Vec<(u32, u32)>,
    pub(super) text: String,
}
because!(AmountHand, OneReading, "the working memory of one thread, kept from case to case so that reading a case takes no new memory once the longest case was read: the tokens, the forms, the examples, the rules to try and the items of a sum");

thread_local! {
    pub(super) static AMOUNT_HANDS: RefCell<AmountHand> = RefCell::new(AmountHand::default());
}

#[derive(Debug)]
pub(super) struct AmountBase {
    pub(super) symbols: Symbols,
    pub(super) marks: AmountMarks,
    pub(super) calcs: Vec<AmountCalc>,
    pub(super) names: Vec<String>,
    pub(super) pairs: usize,
    pub(super) shapes: Vec<Vec<bool>>,
    pub(super) tables: Vec<Vec<bool>>,
}
because!(AmountBase, OneReading, "what learning and reading share: the table of tokens, the marks, the fixed set of calculations with their names, the forms of answers that the lessons have, and the tables of steps over words. count gives the counts of a run of lessons; steps_value and word_value work out a case in steps for one way; find_steps and find_word_steps search the way that is right for the most lessons of the first part, the first of equals in the fixed order of the ways");
