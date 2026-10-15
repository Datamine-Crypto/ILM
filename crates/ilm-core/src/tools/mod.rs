pub mod amount;
pub mod arrange;
pub mod call;
pub mod frames;
pub mod frequent;
pub mod graph;
pub mod link;
pub mod list;
pub mod order;
pub mod parts;
pub mod pattern;
pub mod plain;
pub mod ranks;
pub mod requests;
pub mod rule;
pub mod state;
pub mod table;
pub mod together;

use crate::tools::amount::AMOUNT_TOOL;
use crate::tools::arrange::ARRANGE_TOOL;
use crate::tools::frames::FRAMES_TOOL;
use crate::tools::graph::GRAPH_TOOL;
use crate::tools::link::LINK_TOOL;
use crate::tools::list::LIST_TOOL;
use crate::tools::order::ORDER_TOOL;
use crate::tools::parts::PARTS_TOOL;
use crate::tools::pattern::PATTERNS_TOOL;
use crate::tools::requests::REQUESTS_TOOL;
use crate::tools::rule::RULE_TOOL;
use crate::tools::state::STATE_TOOL;
use crate::tools::table::TABLE_TOOL;
use crate::tools::together::TOGETHER_TOOL;
use crate::tools::call::Tool;
use crate::tools::plain::{PLAIN_TOOL, QUESTION_TOOL};
use crate::tools::ranks::RANKS_TOOL;
use patterns::{because, source};

pub struct ToolList;
source!(ToolList, "the tools of the choice in one fixed list: with equal shares the tool that comes first in the list is taken, so a tool later in the list takes a kind only when it reads more of the lessons right; the plain controls have names beside them, so that a control is measured through the same call as a tool");

const TOOLS: &[Tool] = &[PATTERNS_TOOL, STATE_TOOL, AMOUNT_TOOL, ORDER_TOOL, LINK_TOOL, RULE_TOOL, PARTS_TOOL, TOGETHER_TOOL, LIST_TOOL, REQUESTS_TOOL, FRAMES_TOOL, TABLE_TOOL, ARRANGE_TOOL, GRAPH_TOOL, RANKS_TOOL];
because!(TOOLS, ToolList, "the tools that the choice is made among, in the order of the list");

const CONTROLS: &[Tool] = &[PLAIN_TOOL, QUESTION_TOOL];
because!(CONTROLS, ToolList, "the plain controls as tools with a name; they are in no choice unless a run names them");

const BETWEEN_NAMES: char = ',';
because!(BETWEEN_NAMES, ToolList, "the mark between the names of tools in a list that a run gives");

pub fn all_tools() -> Vec<Tool> {
    TOOLS.to_vec()
}
because!(all_tools, ToolList, "the tools of the choice in their order");

pub fn named_tools(names: &str) -> Result<Vec<Tool>, String> {
    names
        .split(BETWEEN_NAMES)
        .map(|name| TOOLS.iter().chain(CONTROLS.iter()).find(|tool| tool.name == name.trim()).copied().ok_or_else(|| format!("no tool has the name {name}; the names are: {}", tool_names().join(", "))))
        .collect()
}
because!(named_tools, ToolList, "the tools that a list of names asks for, in the order of the list: a run with a part of the tools shows what the others add");

pub fn tool_names() -> Vec<&'static str> {
    TOOLS.iter().chain(CONTROLS.iter()).map(|tool| tool.name).collect()
}
because!(tool_names, ToolList, "the names of all tools and controls, for the help of the program");
