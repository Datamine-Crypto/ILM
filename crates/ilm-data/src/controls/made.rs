use crate::controls::file::ChangedCase;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use crate::sets::made::family::{Family, MadeTree};
use crate::sets::made::reader::made_answers;
use crate::sets::made::{made_family, made_kind};
use patterns::{because, source};
use std::collections::HashSet;

pub struct MadeChanges;
source!(MadeChanges, "changed texts of a family that a program makes: each lesson of a family comes with its tree, so a change is made on the tree, and the interpreter of the family gives the truth of the changed text. An item becomes the next item of the family, at the first item of the tree and at the last; a function becomes the next function of the family with as many parts, at the outermost function and at the last one; the first and the last part of the first function with more than one part change places; the first function with one part is taken out and its part stays; the first item becomes an item that no lesson holds, in a family that has such items. A changed text is left out when it is the text of its lesson, when a lesson of the model holds it, when its answer has no word, or when the interpreter does not give that answer as the one answer of the text");

const MADE_SORT_ITEM: &str = "item";
because!(MADE_SORT_ITEM, MadeChanges, "the sort of the change where an item becomes the next item of the family");

const MADE_SORT_WORD: &str = "word";
because!(MADE_SORT_WORD, MadeChanges, "the sort of the change where a function becomes another function with as many parts");

const MADE_SORT_SIDES: &str = "sides";
because!(MADE_SORT_SIDES, MadeChanges, "the sort of the change where the first and the last part of a function change places");

const MADE_SORT_OUT: &str = "out";
because!(MADE_SORT_OUT, MadeChanges, "the sort of the change where a function with one part is taken out and its part stays");

const MADE_SORT_NEW: &str = "new";
because!(MADE_SORT_NEW, MadeChanges, "the sort of the change where the first item becomes an item that no lesson holds");

const MADE_TEXT_GAP: &str = " ";
because!(MADE_TEXT_GAP, MadeChanges, "the mark between the words of a changed text, as the texts of the lessons write it");

type TreePath = Vec<usize>;
because!(TreePath, MadeChanges, "the way from the top of a tree to one of its places: the number of the part at each step down");

fn tree_paths(tree: &MadeTree, path: &mut TreePath, items: &mut Vec<TreePath>, functions: &mut Vec<TreePath>) {
    match tree {
        MadeTree::Item(_) => items.push(path.clone()),
        MadeTree::Node(_, _, kids) => {
            functions.push(path.clone());
            for (part, kid) in kids.iter().enumerate() {
                path.push(part);
                tree_paths(kid, path, items, functions);
                path.pop();
            }
        }
    }
}
because!(tree_paths, MadeChanges, "the places of the items and of the functions of a tree, each function before its parts and the parts in their order");

fn tree_at<'a>(tree: &'a MadeTree, path: &[usize]) -> &'a MadeTree {
    path.iter().fold(tree, |down, part| match down {
        MadeTree::Node(_, _, kids) => &kids[*part],
        MadeTree::Item(_) => down,
    })
}
because!(tree_at, MadeChanges, "the item or the function that a way down from the top of a tree reaches, so that a change can look at it before it is made");

fn tree_put(tree: &MadeTree, path: &[usize], new: MadeTree) -> MadeTree {
    match (path.split_first(), tree) {
        (None, _) => new,
        (Some((part, rest)), MadeTree::Node(function, way, kids)) => {
            let mut kids = kids.clone();
            kids[*part] = tree_put(&kids[*part], rest, new);
            MadeTree::Node(*function, *way, kids)
        }
        (Some(_), MadeTree::Item(_)) => tree.clone(),
    }
}
because!(tree_put, MadeChanges, "a tree with a new part at a place and every other part as it was");

fn tree_ends(paths: &[TreePath]) -> Vec<&TreePath> {
    match (paths.first(), paths.last()) {
        (Some(first), Some(last)) if first != last => vec![first, last],
        (Some(first), _) => vec![first],
        _ => Vec::new(),
    }
}
because!(tree_ends, MadeChanges, "the first and the last of some places, once when they are one place");

fn made_changed(family: &Family, tree: &MadeTree) -> Vec<(&'static str, MadeTree)> {
    let (mut items, mut functions) = (Vec::new(), Vec::new());
    tree_paths(tree, &mut Vec::new(), &mut items, &mut functions);
    let mut out: Vec<(&'static str, MadeTree)> = Vec::new();
    for path in tree_ends(&items) {
        if let MadeTree::Item(item) = tree_at(tree, path)
            && let Some(at) = family.items.iter().position(|held| held == item)
        {
            let next = family.items[(at + 1) % family.items.len()].clone();
            out.push((MADE_SORT_ITEM, tree_put(tree, path, MadeTree::Item(next))));
        }
    }
    let ways: Vec<(usize, usize)> = family.functions.iter().enumerate().flat_map(|(function, group)| (0..group.len()).map(move |way| (function, way))).collect();
    for path in tree_ends(&functions) {
        if let MadeTree::Node(function, way, kids) = tree_at(tree, path) {
            let start = ways.iter().position(|held| *held == (*function, *way)).unwrap_or_default();
            let next = ways[start + 1..].iter().chain(&ways[..start]).find(|(other, other_way)| other != function && family.functions[*other][*other_way].parts == kids.len());
            if let Some((other, other_way)) = next {
                out.push((MADE_SORT_WORD, tree_put(tree, path, MadeTree::Node(*other, *other_way, kids.clone()))));
            }
        }
    }
    let turned = functions.iter().find_map(|path| match tree_at(tree, path) {
        MadeTree::Node(function, way, kids) if kids.len() > 1 => {
            let mut kids = kids.clone();
            let last = kids.len() - 1;
            kids.swap(0, last);
            Some(tree_put(tree, path, MadeTree::Node(*function, *way, kids)))
        }
        _ => None,
    });
    out.extend(turned.map(|changed| (MADE_SORT_SIDES, changed)));
    let taken = functions.iter().find_map(|path| match tree_at(tree, path) {
        MadeTree::Node(_, _, kids) if kids.len() == 1 => Some(tree_put(tree, path, kids[0].clone())),
        _ => None,
    });
    out.extend(taken.map(|changed| (MADE_SORT_OUT, changed)));
    if let (Some(new), Some(first)) = (family.new_items.first(), items.first()) {
        out.push((MADE_SORT_NEW, tree_put(tree, first, MadeTree::Item(new.clone()))));
    }
    out
}
because!(made_changed, MadeChanges, "the changed trees of the tree of a lesson, each with the sort of its change, in the order of the sorts");

pub fn made_changes(lessons: &[Case], whole: bool, kind: &str) -> Result<Vec<ChangedCase>, String> {
    let Some((number, _)) = made_kind(kind) else { return Err("no kind of a family that a program makes".to_string()) };
    let family = made_family(number, false)?;
    if family.lessons.len() != lessons.len() || family.lessons.iter().zip(lessons).any(|(made, case)| made.text != case.question || made.answer != case.answer) {
        return Err(format!("the case file does not hold the lessons of the family of seed {number}"));
    }
    let (first, last): (Vec<usize>, Vec<usize>) = if whole { ((0..lessons.len()).collect(), (0..lessons.len()).collect()) } else { inner_parts(lessons) };
    let held: HashSet<&str> = first.iter().map(|place| lessons[*place].question.as_str()).collect();
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let lesson = &family.lessons[place];
        for (sort, tree) in made_changed(&family, &lesson.tree) {
            let text = family.text(&tree).join(MADE_TEXT_GAP);
            let value = family.value(&tree);
            if text == lesson.text || held.contains(text.as_str()) || value.is_empty() {
                continue;
            }
            let text_words: Vec<String> = text.split_whitespace().map(String::from).collect();
            if made_answers(&family, &text_words) != [value.clone()] {
                continue;
            }
            out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences: Vec::new(), question: text, truth: value.join(family.between()) });
        }
    }
    Ok(out)
}
because!(made_changes, MadeChanges, "the changed texts of the lessons of a family that the model does not learn, or of all its lessons for a model of all lessons, each with the place of its lesson, the sort of its change and its true answer; the family is made from the seed in the name of the kind, and its lessons must be the lessons of the case file");
