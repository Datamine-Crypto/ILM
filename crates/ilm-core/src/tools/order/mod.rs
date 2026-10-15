pub mod counted;
pub mod model;
pub mod runs;

use crate::tools::call::Tool;
use crate::tools::order::model::learn_order;
use patterns::{because, source};

pub struct OrderTool;
source!(OrderTool, "a learner of an order that no case tells and that is the same in all cases: it counts over the lessons which thing stood before which, and uses that order in a new case; it holds no order of letters, no list of words and no name of a task");

pub const ORDER_TOOL: Tool = Tool { name: "order", needs_labels: false, probe: false, asks: false, over: None, learn: learn_order };
because!(ORDER_TOOL, OrderTool, "the order tool in the list of the tools: it needs no label of a lesson sentence, it has no rule by the form of the question alone, and it asks for no answer");
