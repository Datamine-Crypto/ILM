use patterns::{because, source};

pub struct CountedTrees;
source!(CountedTrees, "trees of marks grown by counting, as the random forests of Breiman with no chance in them: at each node the mark that leaves the least doubt about the answers of its lessons (the doubt of the lessons with the mark plus the doubt of those without it), with enough lessons on each side; each tree sees a fixed part of the lessons and may ask a fixed part of the marks at a node, chosen by a hash of the place and the tree, so the same lessons give the same trees on every machine");

pub struct PairwiseSums;
source!(PairwiseSums, "a sum of doubles is made in the order of the pairwise sum of numpy, so that a sum has one value on every system: fewer than eight terms one after the other, up to a block of terms in eight running sums joined in pairs, and a longer row cut into two halves at a multiple of eight; the logarithm is the one of the runtime of the system");

pub const TREE_COUNT: u64 = 100;
because!(TREE_COUNT, CountedTrees, "the trees of each forest: enough for the shares of their votes to settle, few enough to count in a second");

const SEEN_SHARE: u64 = 80;
because!(SEEN_SHARE, CountedTrees, "of each hundred lessons a tree sees this many, so that the trees differ and a vote of them is not one tree again");

const ASKED_SHARE: u64 = 20;
because!(ASKED_SHARE, CountedTrees, "of each hundred marks a node may ask this many, so that the trees differ in what they ask");

const HASH_RANGE: u64 = 100;
because!(HASH_RANGE, CountedTrees, "the range of the hash of a place: the shares of the lessons that a tree sees and of the marks that a node may ask are bounds within it");

const LEAST_SIDE: u64 = 2;
because!(LEAST_SIDE, CountedTrees, "the fewest lessons on each side of a mark, so that no mark splits off a single lesson");

const MOST_DEPTH: usize = 16;
because!(MOST_DEPTH, CountedTrees, "the most marks that a tree asks on one way down: deep enough that thousands of lessons with several numbers each can still part their cases, while a tree of a few hundred lessons stops earlier for want of cases");

const DOUBT_GAIN: f64 = 1e-3;
because!(DOUBT_GAIN, CountedTrees, "a mark is asked only when it lowers the doubt by more than this, after both doubts are rounded to thousandths, so that rounding noise is no gain");

const DOUBT_PLACES: usize = 3;
because!(DOUBT_PLACES, CountedTrees, "the places after the point to which a doubt is rounded before two doubts are compared");

const DOUBT_SCALE: f64 = 1000.0;
because!(DOUBT_SCALE, CountedTrees, "the doubts of the marks are rounded to thousandths by a product with this scale, an even rounding and a division, as numpy rounds an array");

const PLACE_PRIME: u64 = 2_654_435_761;
because!(PLACE_PRIME, CountedTrees, "the multiplier of the place in the hash: the prime of Knuth near the word of thirty-two bits divided by the golden ratio");

const TREE_PRIME: u64 = 40_503;
because!(TREE_PRIME, CountedTrees, "a prime that spreads the number of the tree over the bits of the hash, so two trees ask apart");

const NODE_PRIME: u64 = 9_973;
because!(NODE_PRIME, CountedTrees, "a prime that spreads the number of the node over the bits of the hash, so two nodes of a tree ask apart");

const TREE_HASH_START: u64 = 12_345;
because!(TREE_HASH_START, CountedTrees, "the start of the hash, so that tree nought and node nought do not give nought");

const HASH_MIX: u64 = 2_246_822_519;
because!(HASH_MIX, CountedTrees, "the second multiplier of the hash, the prime of the hash of Collet, which spreads the bits of the first");

const HASH_SHIFT_FIRST: u32 = 15;
because!(HASH_SHIFT_FIRST, CountedTrees, "the shift that folds the high bits of the hash into the low bits before the second product");

const HASH_SHIFT_LAST: u32 = 13;
because!(HASH_SHIFT_LAST, CountedTrees, "the shift that folds the high bits of the hash into the low bits at the end");

const HASH_MASK: u64 = 0xFFFF_FFFF;
because!(HASH_MASK, CountedTrees, "the hash keeps thirty-two bits");

const SUM_BLOCK: usize = 8;
because!(SUM_BLOCK, PairwiseSums, "the running sums of a pairwise sum, and the fewest terms that are not added one after the other");

const SUM_LONG: usize = 128;
because!(SUM_LONG, PairwiseSums, "the most terms that the running sums take in one block; a longer row is cut into two");

const MARK_SIDES: u64 = 2;
because!(MARK_SIDES, CountedTrees, "a mark parts the lessons of a node into those that have it and those that do not; the nodes below a node are numbered as in a heap, so every node of a tree has its own number in the hash");

const SUM_HALVES: usize = 2;
because!(SUM_HALVES, PairwiseSums, "a long row, and the running sums of a block, are cut into this many halves that are added apart and then together");

pub fn place_hash(place: u64, tree: u64, node: u64) -> u64 {
    let start = (tree * TREE_PRIME + node * NODE_PRIME + TREE_HASH_START) & HASH_MASK;
    let mut hash = (place * PLACE_PRIME + start) & HASH_MASK;
    hash ^= hash >> HASH_SHIFT_FIRST;
    hash = (hash * HASH_MIX) & HASH_MASK;
    hash ^= hash >> HASH_SHIFT_LAST;
    hash % HASH_RANGE
}
because!(place_hash, CountedTrees, "for a place a fixed number below a hundred, made from the place, the tree and the node: no clock and no chance, so the same lessons give the same trees");

pub fn pairwise_sum(terms: &[f64]) -> f64 {
    if terms.len() < SUM_BLOCK {
        let mut sum = f64::default();
        for term in terms {
            sum += *term;
        }
        return sum;
    }
    if terms.len() <= SUM_LONG {
        let mut running: Vec<f64> = terms[..SUM_BLOCK].to_vec();
        let full = terms.len() - terms.len() % SUM_BLOCK;
        let mut at = SUM_BLOCK;
        while at < full {
            for (lane, sum) in running.iter_mut().enumerate() {
                *sum += terms[at + lane];
            }
            at += SUM_BLOCK;
        }
        let mut sum = pair_up(&running);
        for term in &terms[full..] {
            sum += *term;
        }
        return sum;
    }
    let mut half = terms.len() / SUM_HALVES;
    half -= half % SUM_BLOCK;
    pairwise_sum(&terms[..half]) + pairwise_sum(&terms[half..])
}
because!(pairwise_sum, PairwiseSums, "the sum of a row of doubles in the order of the pairwise sum of numpy");

fn pair_up(sums: &[f64]) -> f64 {
    if sums.len() == 1 {
        return sums[0];
    }
    let half = sums.len() / SUM_HALVES;
    pair_up(&sums[..half]) + pair_up(&sums[half..])
}
because!(pair_up, PairwiseSums, "the running sums of a block joined in pairs, the first half and the second half each joined before the two are added");

pub fn doubt(counts: &[u64]) -> f64 {
    let all: u64 = counts.iter().sum();
    let terms: Vec<f64> = counts.iter().map(|count| (*count as f64) * (*count as f64).max(f64::from(1u8)).ln()).collect();
    (all as f64) * (all as f64).max(f64::from(1u8)).ln() - pairwise_sum(&terms)
}
because!(doubt, CountedTrees, "the doubt of a row of counts: the whole times its logarithm less the sum of each count times its logarithm, which is the information of the row in natural units times its size; a count of nought or one adds nothing");

fn even_round(value: f64) -> f64 {
    (value * DOUBT_SCALE).round_ties_even() / DOUBT_SCALE
}
because!(even_round, CountedTrees, "a doubt rounded to thousandths as numpy rounds an array: times the scale, to the even whole number, over the scale");

fn exact_round(value: f64) -> f64 {
    format!("{value:.*}", DOUBT_PLACES).parse::<f64>().unwrap_or(value)
}
because!(exact_round, CountedTrees, "a doubt rounded to thousandths as the language of the first count rounds one number: the nearest decimal of the exact value, ties to the even one");

#[derive(Clone, Debug)]
pub enum TreeNode {
    Leaf(Vec<(u32, u64)>),
    Split { mark: u32, yes: Box<TreeNode>, no: Box<TreeNode>, leaf: Vec<(u32, u64)> },
}
because!(TreeNode, CountedTrees, "a node of a tree: a leaf with the weight of each answer of its lessons, or a mark with the node for the lessons that have it, the node for the others, and the leaf of all its lessons, which is asked when the end below has none of the answers that are left");

#[derive(Clone, Debug, Default)]
pub struct Forest {
    pub names: Vec<String>,
    pub answers: usize,
    pub roots: Vec<TreeNode>,
}
because!(Forest, CountedTrees, "the trees of one question: the marks by their texts in their order, the number of the answers, and the roots. grow makes them from rows of marks and weights of answers; leaves gives for each tree the weights of the answers that are left at the end where a problem leads, or at the nearest node above it that has one of them; each weight over their sum is the share of its answer");

pub struct ForestRows<'a> {
    pub marks: &'a [Vec<u32>],
    pub weights: &'a [Vec<(u32, u64)>],
    pub mark_count: usize,
    pub answers: usize,
}
because!(ForestRows, CountedTrees, "the rows that a forest is counted from: for each row the numbers of its marks and the weight of each of its answers, the number of the marks and the number of the answers");

impl Forest {
    pub fn grow(names: Vec<String>, rows: &ForestRows, salt: u64) -> Forest {
        let mut roots: Vec<TreeNode> = Vec::with_capacity(TREE_COUNT as usize);
        for tree in 0..TREE_COUNT {
            let seen: Vec<usize> = (0..rows.marks.len()).filter(|row| place_hash(*row as u64, tree, SEEN_SALT + salt) < SEEN_SHARE).collect();
            roots.push(grow_node(rows, &seen, tree + TREE_SALT * salt, 1, 0));
        }
        Forest { names, answers: rows.answers, roots }
    }

    pub fn leaves(&self, has: &[bool], good: Option<&[bool]>) -> Vec<Vec<(u32, u64)>> {
        let mut out: Vec<Vec<(u32, u64)>> = Vec::with_capacity(self.roots.len());
        for root in &self.roots {
            let mut node = root;
            let mut path: Vec<&Vec<(u32, u64)>> = Vec::new();
            let end = loop {
                match node {
                    TreeNode::Leaf(leaf) => break leaf,
                    TreeNode::Split { mark, yes, no, leaf } => {
                        path.push(leaf);
                        node = if has[*mark as usize] { yes } else { no };
                    }
                }
            };
            for leaf in std::iter::once(end).chain(path.into_iter().rev()) {
                let left: Vec<(u32, u64)> = leaf.iter().filter(|(answer, _)| good.is_none_or(|good| good[*answer as usize])).copied().collect();
                if left.is_empty() {
                    continue;
                }
                out.push(left);
                break;
            }
        }
        out
    }
}

const SEEN_SALT: u64 = 7;
because!(SEEN_SALT, CountedTrees, "the node number in the hash that picks the lessons a tree sees, apart from the node numbers of the marks");

const TREE_SALT: u64 = 1000;
because!(TREE_SALT, CountedTrees, "the forests of one model hash their trees apart: the tree number in the hash of the marks is the tree plus this many times the number of the forest");

fn grow_node(rows: &ForestRows, members: &[usize], tree: u64, node: u64, level: usize) -> TreeNode {
    let mut here = vec![0u64; rows.answers];
    for row in members {
        for (answer, weight) in &rows.weights[*row] {
            here[*answer as usize] += *weight;
        }
    }
    let some: Vec<usize> = (0..rows.answers).filter(|answer| here[*answer] != 0).collect();
    let leaf: Vec<(u32, u64)> = some.iter().map(|answer| (*answer as u32, here[*answer])).collect();
    if level >= MOST_DEPTH || (members.len() as u64) < LEAST_SIDE * MARK_SIDES || leaf.len() <= 1 {
        return TreeNode::Leaf(leaf);
    }
    let mut with_mark = vec![0u64; rows.mark_count];
    for row in members {
        for mark in &rows.marks[*row] {
            with_mark[*mark as usize] += 1;
        }
    }
    let size = members.len() as u64;
    let may: Vec<usize> = (0..rows.mark_count).filter(|mark| place_hash(*mark as u64, tree, node) < ASKED_SHARE && with_mark[*mark] >= LEAST_SIDE && size - with_mark[*mark] >= LEAST_SIDE).collect();
    if may.is_empty() {
        return TreeNode::Leaf(leaf);
    }
    let mut slot = vec![usize::MAX; rows.mark_count];
    for (at, mark) in may.iter().enumerate() {
        slot[*mark] = at;
    }
    let mut column = vec![usize::MAX; rows.answers];
    for (at, answer) in some.iter().enumerate() {
        column[*answer] = at;
    }
    let mut yes = vec![0u64; may.len() * some.len()];
    for row in members {
        for mark in &rows.marks[*row] {
            let at = slot[*mark as usize];
            if at == usize::MAX {
                continue;
            }
            for (answer, weight) in &rows.weights[*row] {
                let place = column[*answer as usize];
                if place != usize::MAX {
                    yes[at * some.len() + place] += *weight;
                }
            }
        }
    }
    let whole: Vec<u64> = some.iter().map(|answer| here[*answer]).collect();
    let mut best: Option<(usize, f64)> = None;
    let mut rest = vec![0u64; some.len()];
    for at in 0..may.len() {
        let counted = &yes[at * some.len()..(at + 1) * some.len()];
        for (place, count) in counted.iter().enumerate() {
            rest[place] = whole[place] - *count;
        }
        let total = even_round(doubt(counted) + doubt(&rest));
        if best.is_none_or(|(_, top)| total < top) {
            best = Some((at, total));
        }
    }
    let Some((at, total)) = best else { return TreeNode::Leaf(leaf) };
    if total >= exact_round(doubt(&whole)) - DOUBT_GAIN {
        return TreeNode::Leaf(leaf);
    }
    let mark = may[at];
    let (with, without): (Vec<usize>, Vec<usize>) = members.iter().partition(|row| rows.marks[**row].binary_search(&(mark as u32)).is_ok());
    let yes = grow_node(rows, &with, tree, node * MARK_SIDES, level + 1);
    let no = grow_node(rows, &without, tree, node * MARK_SIDES + 1, level + 1);
    TreeNode::Split { mark: mark as u32, yes: Box::new(yes), no: Box::new(no), leaf }
}
because!(grow_node, CountedTrees, "one node of a tree from the lessons that reach it: a leaf when the tree is deep enough, the lessons are few or they agree, else the mark of the least doubt among the marks that the node may ask, when it lowers the doubt of the node");
