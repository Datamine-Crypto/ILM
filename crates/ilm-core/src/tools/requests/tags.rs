use patterns::{because, source};

pub struct RequestTags;
source!(RequestTags, "the tags of the words of a request: the slot that a word fills, the same slot for a later word of a run of that slot, or no slot. The answers of one request are placed together, so that no word stands in two slots: every way to find the words of an answer as runs of the request is tried, the answer with the fewest ways first");

pub(super) const TAG_OUTSIDE: u32 = 0;
because!(TAG_OUTSIDE, RequestTags, "the tag of a word that fills no slot");

const TAG_KINDS: u32 = 2;
because!(TAG_KINDS, RequestTags, "the tags of each slot: the first word of a run of the slot, and a later word of such a run");

pub(super) fn tag_first(slot: usize) -> u32 {
    slot as u32 * TAG_KINDS + 1
}
because!(tag_first, RequestTags, "the tag of the first word of a run of a slot");

pub(super) fn tag_later(slot: usize) -> u32 {
    slot as u32 * TAG_KINDS + TAG_KINDS
}
because!(tag_later, RequestTags, "the tag of a later word of a run of a slot");

pub(super) fn tag_slot(tag: u32) -> Option<usize> {
    tag.checked_sub(1).map(|rest| (rest / TAG_KINDS) as usize)
}
because!(tag_slot, RequestTags, "the slot that a tag belongs to, for the first word and for a later word of its runs alike; nothing for the tag of a word outside every slot");

pub(super) fn is_later(tag: u32) -> bool {
    tag != TAG_OUTSIDE && tag % TAG_KINDS == 0
}
because!(is_later, RequestTags, "is a tag the tag of a later word of a run");

pub(super) fn tag_count(slots: usize) -> usize {
    slots * TAG_KINDS as usize + 1
}
because!(tag_count, RequestTags, "the tags of a kind with so many slots: two for each slot and the tag of no slot");

pub(super) type RequestRuns = Vec<(usize, usize)>;
because!(RequestRuns, RequestTags, "the runs of a request that hold the words of one answer in their order, each from its first place to the place after it");

pub(super) fn request_located(words: &[u32], answer: &[u32], taken: &[bool], at: usize) -> Vec<RequestRuns> {
    if answer.is_empty() {
        return vec![Vec::new()];
    }
    let mut out: Vec<RequestRuns> = Vec::new();
    for first in at..words.len() {
        if words[first] != answer[0] || taken[first] {
            continue;
        }
        let mut end = first;
        while end < words.len() && end - first < answer.len() && words[end] == answer[end - first] && !taken[end] {
            end += 1;
        }
        for rest in request_located(words, &answer[end - first..], taken, end) {
            let mut way: RequestRuns = vec![(first, end)];
            way.extend(rest);
            out.push(way);
        }
    }
    out
}
because!(request_located, RequestTags, "every way to find the words of an answer as runs of the request, in their order, with no run on a taken place: from each place where the first word stands the longest run that matches, and the rest of the answer after it");

fn request_search(words: &[u32], answers: &[&[u32]], order: &[usize], step: usize, taken: &mut [bool], found: &mut [RequestRuns]) -> bool {
    let Some(answer) = order.get(step) else { return true };
    for runs in request_located(words, answers[*answer], taken, 0) {
        for (first, end) in &runs {
            taken[*first..*end].fill(true);
        }
        if request_search(words, answers, order, step + 1, taken, found) {
            found[*answer] = runs;
            return true;
        }
        for (first, end) in &runs {
            taken[*first..*end].fill(false);
        }
    }
    false
}
because!(request_search, RequestTags, "the first way to place the answers in the given order with no word in two answers: each way of the next answer is tried on the places that are still free");

pub(super) fn request_placed(words: &[u32], answers: &[&[u32]]) -> Option<Vec<RequestRuns>> {
    let free = vec![false; words.len()];
    let ways: Vec<usize> = answers.iter().map(|answer| request_located(words, answer, &free, 0).len()).collect();
    let mut order: Vec<usize> = (0..answers.len()).collect();
    order.sort_by_key(|answer| (ways[*answer], *answer));
    let mut taken = vec![false; words.len()];
    let mut found: Vec<RequestRuns> = vec![Vec::new(); answers.len()];
    request_search(words, answers, &order, 0, &mut taken, &mut found).then_some(found)
}
because!(request_placed, RequestTags, "the runs of each answer of one request, with no word in two answers, or nothing when there is no such way; the answers come in the order of their texts, and the one with the fewest ways is placed first, of equals the earlier one");
