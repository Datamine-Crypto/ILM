use patterns::{because, source};
use std::collections::HashSet;

pub struct MadeFamilies;
source!(MadeFamilies, "families of texts that a program makes: a family is drawn from a number, its seed, and has made-up words, a grammar of texts with parts inside parts, and a meaning by parts, so the answer of a text is made from the answers of its parts. Nobody chose a family: the draw decides its words, how a function is written and what it does; and an interpreter of the family gives the answer of every text of its grammar, so the truth of a changed text is known");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MadeSort {
    Says,
    Itself,
    Text,
}
because!(MadeSort, MadeFamilies, "what an item of a family means: answer words that it says, the item itself, or a family whose answer is the text with a change");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MadeDiscipline {
    Before,
    After,
    Closed,
    Group,
    Levels,
}
because!(MadeDiscipline, MadeFamilies, "the one way of writing of a family that gives every text one reading: the word of a function before its parts, after them, on both sides or with brackets, in groups with brackets round a part that is a function, or in levels with no brackets");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MadeSlot {
    Fixed(String),
    Part(usize),
}
because!(MadeSlot, MadeFamilies, "one place of the way a function is written: a fixed word, or the place of one of its parts");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MadeChange {
    Keep,
    First,
    Last,
    Rest,
    Most,
    Rev,
    Look,
}
because!(MadeChange, MadeFamilies, "what a function does with the answer of one part: keeps it, takes its first or its last word, leaves out its first or its last word, turns it round, or looks its first word up in the table of the family");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MadeSay {
    Word(String),
    Part(usize, MadeChange),
}
because!(MadeSay, MadeFamilies, "one piece of the answer of a function: a fixed answer word, or the answer of one of its parts with a change");

#[derive(Clone, Debug, Default)]
pub struct MadeWay {
    pub layout: Vec<MadeSlot>,
    pub level: u32,
    pub accept: Vec<Option<u32>>,
    pub parts: usize,
    pub says: Vec<MadeSay>,
}
because!(MadeWay, MadeFamilies, "one way to write a function: its fixed words and the places of its parts in their order, its level and the highest level that each part may have (in a family of levels), how many parts it has, and what it says");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MadeTree {
    Item(String),
    Node(usize, usize, Vec<MadeTree>),
}
because!(MadeTree, MadeFamilies, "a text of a family as the program made it: an item, or a function in one of its ways with the trees of its parts");

#[derive(Clone, Debug)]
pub struct MadeCase {
    pub text: String,
    pub answer: String,
    pub tree: MadeTree,
}
because!(MadeCase, MadeFamilies, "one case of a family: its text, its answer as the lessons write it, and its tree");

#[derive(Clone, Debug)]
pub struct Family {
    pub number: usize,
    pub drawn: usize,
    pub features: Vec<String>,
    pub table: Vec<(String, String)>,
    pub wrap: Option<(String, String)>,
    pub says: Vec<(String, Vec<String>)>,
    pub fixed: HashSet<String>,
    pub sort: MadeSort,
    pub signs: bool,
    pub discipline: MadeDiscipline,
    pub items: Vec<String>,
    pub new_items: Vec<String>,
    pub functions: Vec<Vec<MadeWay>>,
    pub lessons: Vec<MadeCase>,
    pub read: Vec<MadeCase>,
    pub deep: Vec<MadeCase>,
}
because!(Family, MadeFamilies, "one family: its seed and the draw that gave it, its features in plain words, the table of its looked-up words, the brackets round a part that is a function, what each item says, its fixed words, its sort, whether its answers are rows of signs, its discipline, its items and the items that no lesson holds, its functions with their ways, and its lessons, its read part and its deep read part");

impl Family {
    pub fn said(&self, item: &str) -> Vec<String> {
        if self.sort != MadeSort::Says {
            return vec![item.to_string()];
        }
        self.says.iter().find(|(held, _)| held == item).map(|(_, words)| words.clone()).unwrap_or_default()
    }

    pub fn is_item(&self, word: &str) -> bool {
        if self.sort == MadeSort::Says {
            return self.says.iter().any(|(held, _)| held == word);
        }
        !self.fixed.contains(word) && !word.is_empty() && word.chars().all(char::is_alphanumeric)
    }

    pub fn joined(&self, says: &[MadeSay], parts: &[Vec<String>]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for piece in says {
            match piece {
                MadeSay::Word(word) => out.push(word.clone()),
                MadeSay::Part(part, change) => {
                    let answer = &parts[*part];
                    match change {
                        MadeChange::Keep => out.extend(answer.iter().cloned()),
                        MadeChange::First => out.extend(answer.iter().take(1).cloned()),
                        MadeChange::Last => out.extend(answer.iter().skip(answer.len().saturating_sub(1)).cloned()),
                        MadeChange::Rest => out.extend(answer.iter().skip(1).cloned()),
                        MadeChange::Most => out.extend(answer.iter().take(answer.len().saturating_sub(1)).cloned()),
                        MadeChange::Rev => out.extend(answer.iter().rev().cloned()),
                        MadeChange::Look => {
                            if let Some(first) = answer.first() {
                                out.push(self.table.iter().find(|(held, _)| held == first).map_or_else(|| first.clone(), |(_, turned)| turned.clone()));
                                out.extend(answer.iter().skip(1).cloned());
                            }
                        }
                    }
                }
            }
        }
        out
    }

    pub fn value(&self, tree: &MadeTree) -> Vec<String> {
        match tree {
            MadeTree::Item(item) => self.said(item),
            MadeTree::Node(function, way, kids) => {
                let parts: Vec<Vec<String>> = kids.iter().map(|kid| self.value(kid)).collect();
                self.joined(&self.functions[*function][*way].says, &parts)
            }
        }
    }

    pub fn text(&self, tree: &MadeTree) -> Vec<String> {
        let (function, way, kids) = match tree {
            MadeTree::Item(item) => return vec![item.clone()],
            MadeTree::Node(function, way, kids) => (function, way, kids),
        };
        let mut row: Vec<String> = Vec::new();
        for slot in &self.functions[*function][*way].layout {
            match slot {
                MadeSlot::Fixed(word) => row.push(word.clone()),
                MadeSlot::Part(part) => {
                    let inner = self.text(&kids[*part]);
                    match (&self.wrap, &kids[*part]) {
                        (Some((open, close)), MadeTree::Node(..)) => {
                            row.push(open.clone());
                            row.extend(inner);
                            row.push(close.clone());
                        }
                        _ => row.extend(inner),
                    }
                }
            }
        }
        row
    }

    pub fn depth(tree: &MadeTree) -> usize {
        match tree {
            MadeTree::Item(_) => 0,
            MadeTree::Node(_, _, kids) => 1 + kids.iter().map(Family::depth).max().unwrap_or(0),
        }
    }

    pub fn between(&self) -> &'static str {
        if self.signs { "" } else { " " }
    }
}
