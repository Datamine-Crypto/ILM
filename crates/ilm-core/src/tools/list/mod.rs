pub mod amounts;
pub mod classes;
pub mod model;
pub mod words;

use crate::tools::call::Tool;
use crate::tools::list::model::learn_list;
use patterns::{because, source};

pub struct ListTool;
source!(ListTool, "a learner of counts over a told list: how many things of a class a list holds. It reads the list and counts, from the lessons, the amount of each word and the class of each thing; each step of the sum is an exact addition; no word of any set stands in it, and no list of words");

pub const LIST_TOOL: Tool = Tool { name: "list", needs_labels: false, probe: false, asks: false, over: None, learn: learn_list };
because!(LIST_TOOL, ListTool, "the list tool in the list of the tools: it needs no label of a lesson sentence, it has no rule by the form of the question alone, and it asks for no answer");
