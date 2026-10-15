use crate::base::first_sight::{FirstSightCount, FirstSightTable, all_counts, key_count};
use crate::tools::frames::FramesModel;
use crate::tools::frames::counts::{FRAME_CLEAR, LessonStory, MANY, both_in, names_of, plain};
use crate::tools::frames::sides::top_label;
use crate::tools::frames::texts::TEXT_LEAST;
use patterns::{because, source};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct OtherWords;
source!(OtherWords, "a word that is no label can stand in the place of one, as a short word for a mother. It is counted on the lesson sentences that hold two names of a step and no slot: the one word that stands most surely where a slot stands, by a key of its frame with enough readings and nearly all for one pair of anchors, is counted with the label of the step when these anchors are the two names of the step. A word counts with enough readings, in a quarter or more of the lesson sentences with a step that hold it, since a word that stands at many places, as a word that joins two parts, is no relation word. It means the label that fits most of its readings as person and by its other sides, or the two labels of one relation for the two groups; or it has other sides of its own and no label for the person");

const WORD_OFTEN: f64 = 0.25;
because!(WORD_OFTEN, OtherWords, "the least share of the lesson sentences with a step that hold a word in which it must stand where a slot stands before it is read as a slot");

const ONE_RELATION: usize = 2;
because!(ONE_RELATION, OtherWords, "the labels of one relation, one for each group of names");

fn clear_label(counts: Option<&FirstSightCount<String>>) -> Option<String> {
    let counts = counts?;
    let all = all_counts(counts);
    if all < TEXT_LEAST {
        return None;
    }
    let (label, hit) = top_label(counts)?;
    (hit as f64 >= FRAME_CLEAR * all as f64).then(|| label.clone())
}
because!(clear_label, OtherWords, "the label of most of the readings of a word for a group when it has enough readings, and nothing else");

impl FramesModel {
    pub(super) fn learn_words(&mut self, stories: &[LessonStory]) -> Vec<String> {
        let mut said_same: HashMap<String, FirstSightCount<String>> = HashMap::new();
        let mut said_turn: HashMap<String, BTreeMap<String, FirstSightCount<String>>> = HashMap::new();
        let mut readings: BTreeMap<String, i64> = BTreeMap::new();
        let mut stands: HashMap<String, i64> = HashMap::new();
        for (read, steps) in stories {
            for sentence in read {
                let names = names_of(sentence);
                let pairs = both_in(steps, &names);
                if pairs.is_empty() {
                    continue;
                }
                let seen = plain(sentence);
                let held: HashSet<&String> = (0..sentence.words.len()).filter(|place| !seen.holds(*place)).map(|place| &sentence.words[place]).collect();
                for word in held {
                    *stands.entry(word.clone()).or_default() += 1;
                }
                if !self.slots(&seen).is_empty() {
                    continue;
                }
                let mut best: Option<((i64, usize), usize, usize, usize)> = None;
                for place in 0..sentence.words.len() {
                    if seen.holds(place) {
                        continue;
                    }
                    let Some((owner, person, count)) = self.sure(&seen, place) else { continue };
                    if best.is_none_or(|(rank, _, _, _)| count > rank.0 || (count == rank.0 && place < rank.1)) {
                        best = Some(((count, place), place, owner, person));
                    }
                }
                let Some((_, place, owner, person)) = best else { continue };
                let word = sentence.words[place].clone();
                let (owner, person) = (&sentence.words[owner], &sentence.words[person]);
                for (one, label, other) in &pairs {
                    if owner == one && person == other {
                        *readings.entry(word.clone()).or_default() += 1;
                        said_same.entry(word.clone()).or_default().add(label.clone(), 1);
                    } else if owner == other && person == one {
                        *readings.entry(word.clone()).or_default() += 1;
                        let group = self.texts.group.get(label).cloned().unwrap_or_default();
                        said_turn.entry(word.clone()).or_default().entry(group).or_default().add(label.clone(), 1);
                    }
                    if (owner == one && person == other) || (owner == other && person == one) {
                        break;
                    }
                }
            }
        }
        let mut learned: Vec<String> = Vec::new();
        let empty: FirstSightCount<String> = FirstSightCount::new();
        let no_turns: BTreeMap<String, FirstSightCount<String>> = BTreeMap::new();
        for (word, count) in &readings {
            if *count < MANY || (*count as f64) < WORD_OFTEN * stands.get(word).copied().unwrap_or_default() as f64 || self.slot_word(word).is_some() {
                continue;
            }
            let same = said_same.get(word).unwrap_or(&empty);
            let turn = said_turn.get(word).unwrap_or(&no_turns);
            let turns: Vec<(String, String)> = self.texts.all_groups.iter().filter_map(|group| clear_label(turn.get(group)).map(|label| (group.clone(), label))).collect();
            let turned_of = |label: &str, group: &str| turn.get(group).map_or(0, |counts| key_count(counts, &label.to_string()));
            let fits: Vec<(String, i64)> = self
                .texts
                .labels
                .rows()
                .iter()
                .map(|(label, _)| {
                    let by_sides: i64 = self.texts.all_groups.iter().filter_map(|group| self.side.get(&(label.clone(), group.clone())).map(|side| turned_of(side, group))).sum();
                    (label.clone(), key_count(same, label) + by_sides)
                })
                .collect();
            let mut top = fits.iter().map(|(_, fit)| *fit).max().unwrap_or_default();
            let mut alike: Vec<String> = fits.iter().filter(|(_, fit)| *fit == top).map(|(label, _)| label.clone()).collect();
            alike.sort();
            if let [one] = alike.as_slice() {
                let other: Vec<String> = self.texts.all_groups.iter().filter_map(|group| self.pair.get(&(one.clone(), group.clone())).cloned()).collect();
                if let Some(other) = other.first() {
                    let also = key_count(same, other);
                    let fit = fits.iter().find(|(label, _)| label == one).map(|(_, fit)| *fit).unwrap_or_default();
                    if also > 0 && (fit + also) as f64 >= FRAME_CLEAR * *count as f64 {
                        let mut both = vec![one.clone(), other.clone()];
                        both.sort();
                        alike = both;
                        top = fit + also;
                    }
                }
            }
            let enough = top as f64 >= FRAME_CLEAR * *count as f64;
            let means: Vec<String> = if enough && all_counts(same) > 0 && alike.len() == 1 {
                alike.clone()
            } else if enough && alike.len() == ONE_RELATION && self.texts.all_groups.iter().any(|group| self.pair.get(&(alike[0].clone(), group.clone())) == Some(&alike[1])) {
                alike.clone()
            } else if !turns.is_empty() && turns.iter().map(|(group, label)| turned_of(label, group)).sum::<i64>() as f64 >= FRAME_CLEAR * *count as f64 {
                Vec::new()
            } else {
                continue;
            };
            self.means.put(word.clone(), means.clone());
            for group in &self.texts.all_groups {
                if let Some((_, label)) = turns.iter().find(|(held, _)| held == group) {
                    self.side.put((word.clone(), group.clone()), label.clone());
                    continue;
                }
                if let Some(found) = means.iter().find_map(|label| self.side.get(&(label.clone(), group.clone())).cloned()) {
                    self.side.put((word.clone(), group.clone()), found);
                }
            }
            learned.push(word.clone());
        }
        let side = std::mem::take(&mut self.side);
        self.side = self.both_groups(side);
        learned
    }
}

pub(super) fn first_meanings(labels: &FirstSightCount<String>) -> FirstSightTable<String, Vec<String>> {
    let mut means: FirstSightTable<String, Vec<String>> = FirstSightTable::new();
    for (label, _) in labels.rows() {
        means.put(label.clone(), vec![label.clone()]);
    }
    means
}
because!(first_meanings, OtherWords, "each label as a word that means itself, before other words are learned");
