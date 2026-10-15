pub mod calcs;
pub mod counts;
pub mod forms;
pub mod learn;
pub mod model;
pub mod reading;
pub mod shared;
pub mod steps;
pub mod tokens;

use crate::tools::amount::learn::learn_amount;
use crate::tools::call::Tool;
use patterns::{because, source};

pub struct AmountTool;
source!(AmountTool, "a learner of calculations with the numbers of a text. It holds a fixed small set of exact calculations and no word of any set; which calculation a form of text means is found by counting over the lessons and kept by the sure test, so what it learned is a list of forms with the name of a calculation beside each");

pub const AMOUNT_TOOL: Tool = Tool { name: "amount", needs_labels: false, probe: false, asks: false, over: None, learn: learn_amount };
because!(AMOUNT_TOOL, AmountTool, "the amount tool in the list of the tools: it needs no label of a lesson sentence, it has no rule by the form of the question alone, and it asks for no answer");
