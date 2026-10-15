pub mod choices;
pub mod climb;
pub mod fresh;
pub mod kinds;
pub mod model;
pub mod read;
pub mod reader;
pub mod search;
pub mod split;
pub mod table;

use crate::tools::call::Tool;
use crate::tools::state::model::learn_state;
use patterns::{because, source};

pub struct StateTool;
source!(StateTool, "one tool among the tools of the choice: the tool is named for its way of reading, a state that the sentences change, and the choice gives it the kinds where its programs read more lessons right than the other tools do");

pub const STATE_TOOL: Tool = Tool { name: "state", needs_labels: false, probe: false, asks: false, over: None, learn: learn_state };
because!(STATE_TOOL, StateTool, "the state tool as a tool of the choice: it needs no labels of the lesson sentences, gives no control by the form of the question, and asks for no answer of a text that it builds");
