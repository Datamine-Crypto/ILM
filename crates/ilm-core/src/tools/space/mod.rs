pub mod learn;
pub mod read;
pub mod words;

use crate::tools::call::Tool;
use crate::tools::space::learn::learn_space;
use patterns::{because, source};

pub struct SpaceTool;
source!(SpaceTool, "a tool for a question that asks how one named thing stands to another, where told sentences each link two things: it learns from the lessons the step that each form of a link means on a plane and the step that each answer names, and it answers with the answer of the sum of the steps along the chain of links between the two things");

pub const SPACE_TOOL: Tool = Tool { name: "space", needs_labels: false, probe: false, asks: false, over: None, learn: learn_space };
because!(SPACE_TOOL, SpaceTool, "the tool in the list of the tools: it needs no labels and gives no control by the form of the question, because it reads the links of the story and the question together");
