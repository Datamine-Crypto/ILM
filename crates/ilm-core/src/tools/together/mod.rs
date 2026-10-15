pub mod asked;
pub mod marks;
pub mod model;
pub mod ratio;
pub mod roles;
pub mod tokens;
pub mod trees;
pub mod vote;
pub mod walk;

use crate::tools::call::Tool;
use crate::tools::together::model::learn_together;
use patterns::{because, source};

pub struct TogetherTool;
source!(TogetherTool, "tools that work together inside one problem: the roles of the numbers of a problem that tells numbers and asks for a number, and the walk of a place that the sentences of a story move and turn; no tool is changed, no word of any set stands here, and no list of words");

pub const TOGETHER_TOOL: Tool = Tool { name: "together", needs_labels: false, probe: false, asks: false, over: None, learn: learn_together };
because!(TOGETHER_TOOL, TogetherTool, "the tools together in the list of the tools: they need no label of a lesson sentence, have no rule by the form of the question alone, and ask for no answer");
