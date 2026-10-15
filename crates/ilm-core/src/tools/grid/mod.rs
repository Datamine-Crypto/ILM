pub mod learn;
pub mod read;
pub mod solve;

use crate::tools::call::Tool;
use crate::tools::grid::learn::learn_grid;
use patterns::{because, source};

pub struct GridTool;
source!(GridTool, "a tool for a question that asks in which of its places a thing stands, where the story lists kinds of things that fill the places one each and tells statements about them: it learns from the lessons what each form of a statement means for the places of its things, and it answers with the place that the arrangements allowed by the told statements give the asked thing");

pub const GRID_TOOL: Tool = Tool { name: "grid", needs_labels: false, probe: false, asks: false, over: None, learn: learn_grid };
because!(GRID_TOOL, GridTool, "the tool in the list of the tools: it needs no labels and gives no control by the form of the question, because it reads the lists and the statements of the story together");
