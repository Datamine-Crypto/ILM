use crate::base::first_sight::{FirstSightCount, FirstSightTable, all_counts};
use crate::tools::frames::FramesModel;
use crate::tools::frames::counts::{LessonStory, both_in, names_of, plain};
use crate::tools::frames::texts::TEXT_LEAST;
use patterns::{because, source};

pub struct OtherSides;
source!(OtherSides, "if a woman is the mother of a man, he is her son: the other side of a relation word for each group of names. It is counted from the lesson slots whose frame surely puts the owner at the second name of a step and the person at the first: the label of the step is then the other side of the word, for the group of the label; before the frames are counted, a sentence with one slot gives its steps with another label as a first count. An entry needs two readings and most of the readings of its word and group, and it gives the turned entry when that one has fewer readings. With no entry for a group, the word takes the entry that goes with its entry for the other group, when the lessons show the two labels as the two other sides of one word, since such two labels are one relation for the two groups; a word with no entry at all takes the entries of the word that is one relation with it");

const SIDE_SHARE: f64 = 0.6;
because!(SIDE_SHARE, OtherSides, "the share of the readings of a word and a group that gives the word its other side for that group");

pub(super) type Sides = FirstSightTable<(String, String), String>;
because!(Sides, OtherSides, "the other side of each word for each group, in the order in which the entries were found");

pub(super) type SideCounts = FirstSightTable<(String, String), FirstSightCount<String>>;
because!(SideCounts, OtherSides, "the labels counted for each word and group as its other side");

pub(super) fn top_label(counts: &FirstSightCount<String>) -> Option<(&String, i64)> {
    counts.rows().iter().max_by(|left, right| (left.1, &left.0).cmp(&(right.1, &right.0))).map(|(label, count)| (label, *count))
}
because!(top_label, OtherSides, "the label with the most readings, and of equal ones the last in the order of their texts, with its readings");

impl FramesModel {
    pub(super) fn first_sides(&mut self, stories: &[LessonStory]) {
        let mut count: SideCounts = FirstSightTable::new();
        for (read, steps) in stories {
            for sentence in read {
                let spots = self.slots(&plain(sentence));
                let names = names_of(sentence);
                let [spot] = spots.as_slice() else { continue };
                let Some((word, _)) = self.slot_word(&sentence.words[*spot]) else { continue };
                for (one, label, other) in steps {
                    if names.contains(one) && names.contains(other) && *label != word {
                        let group = self.texts.group.get(label).cloned().unwrap_or_default();
                        count.at((word.clone(), group)).add(label.clone(), 1);
                    }
                }
            }
        }
        self.side = self.sides_from(&count);
    }

    pub(super) fn count_sides(&mut self, stories: &[LessonStory]) {
        let mut count: SideCounts = FirstSightTable::new();
        for (read, steps) in stories {
            for sentence in read {
                let pairs = both_in(steps, &names_of(sentence));
                if pairs.is_empty() {
                    continue;
                }
                let seen = plain(sentence);
                for slot in self.slots(&seen) {
                    let Some((word, plural)) = self.slot_word(&sentence.words[slot]) else { continue };
                    let found = if plural { None } else { self.sure(&seen, slot) };
                    let Some((owner, person, _)) = found else { continue };
                    for (one, label, other) in &pairs {
                        if sentence.words[owner] == *other && sentence.words[person] == *one {
                            let group = self.texts.group.get(label).cloned().unwrap_or_default();
                            count.at((word.clone(), group)).add(label.clone(), 1);
                        }
                    }
                }
            }
        }
        self.side = self.sides_from(&count);
    }

    pub(super) fn sides_from(&mut self, count: &SideCounts) -> Sides {
        let mut side: Sides = FirstSightTable::new();
        let mut readings: FirstSightTable<(String, String), i64> = FirstSightTable::new();
        for (key, counts) in count.rows() {
            let Some((label, hit)) = top_label(counts) else { continue };
            if hit >= TEXT_LEAST && hit as f64 >= SIDE_SHARE * all_counts(counts) as f64 {
                side.put(key.clone(), label.clone());
                readings.put(key.clone(), hit);
            }
        }
        let mut entries: Vec<((String, String), String)> = side.rows().to_vec();
        entries.sort();
        for ((word, group), label) in entries {
            let Some(own) = self.texts.group.get(&word) else { continue };
            let turned = (label.clone(), own.clone());
            if label != word && readings.get(&turned).copied().unwrap_or_default() < readings.get(&(word.clone(), group)).copied().unwrap_or_default() {
                side.put(turned, word);
            }
        }
        self.both_groups(side)
    }

    pub(super) fn both_groups(&mut self, mut side: Sides) -> Sides {
        let mut pair: FirstSightTable<(String, String), String> = FirstSightTable::new();
        for ((word, group), label) in side.rows() {
            for ((other_word, other_group), other_label) in side.rows() {
                if other_word == word && other_group != group && other_label != label {
                    pair.put((label.clone(), other_group.clone()), other_label.clone());
                }
            }
        }
        let mut words: Vec<String> = side.rows().iter().map(|((word, _), _)| word.clone()).collect();
        words.sort();
        words.dedup();
        for word in &words {
            for group in &self.texts.all_groups {
                if side.holds(&(word.clone(), group.clone())) {
                    continue;
                }
                let mut entries: Vec<((String, String), String)> = side.rows().to_vec();
                entries.sort();
                for ((other_word, other_group), label) in entries {
                    if other_word == *word && other_group != *group && let Some(found) = pair.get(&(label, group.clone())) {
                        side.put((word.clone(), group.clone()), found.clone());
                        break;
                    }
                }
            }
        }
        let mut meant: Vec<String> = self.means.rows().iter().map(|(word, _)| word.clone()).collect();
        meant.sort();
        for word in &meant {
            for group in &self.texts.all_groups {
                let other = self.texts.all_groups.iter().find_map(|held| pair.get(&(word.clone(), held.clone())));
                if side.holds(&(word.clone(), group.clone())) {
                    continue;
                }
                if let Some(found) = other.and_then(|other| side.get(&(other.clone(), group.clone()))).cloned() {
                    side.put((word.clone(), group.clone()), found);
                }
            }
        }
        self.pair = pair;
        side
    }
}
