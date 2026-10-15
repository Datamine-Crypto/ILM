pub mod learn;
pub mod read;
pub mod solve;

use crate::tools::arrange::learn::learn_arrange;
use crate::tools::call::Tool;
use patterns::{because, source};

pub struct ArrangeTool;
source!(ArrangeTool, "a tool for a question that asks which of its choices is true of things that stand in one order: it learns from the lessons what each form of a statement means for the places of its things, and it answers with the choice that holds in the arrangements that the told statements allow");

pub const ARRANGE_TOOL: Tool = Tool { name: "arrange", needs_labels: false, probe: false, asks: false, over: None, learn: learn_arrange };
because!(ARRANGE_TOOL, ArrangeTool, "the tool in the list of the tools: it needs no labels and gives no control by the form of the question, because it reads the told statements and the choices together");
