use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberTable};
use crate::base::symbols::Symbols;
use crate::tools::link::joins::{Joins, LINK_PAIR, LinkLessons, chart_by, joined_entry, link_sure};
use patterns::{because, source};

pub struct LocalTrials;
source!(LocalTrials, "the trial of one change of a label read only where it can act: a change of the label of one leaf moves only the lessons that hold the leaf and the entries of the join table whose keys hold the leaf or its new label, so a lesson that holds no leaf and whose chart read no moved entry comes out as before, and only the other lessons are read again; the gain and the loss are those of a count of all lessons");

pub(super) struct LinkTrials<'a> {
    lessons: &'a LinkLessons,
    items: &'a Symbols,
    joins: &'a Joins,
    right: &'a [bool],
    labels: Vec<Vec<u32>>,
    readings: FirstSightTable<(u32, u32), Vec<usize>>,
    looked: NumberTable<(u32, u32), Vec<usize>>,
    holding: NumberTable<u32, Vec<usize>>,
    keys_with: NumberTable<u32, Vec<(u32, u32)>>,
}
because!(LinkTrials, LocalTrials, "what the trials of one round share: the lessons, the texts of the labels, the join table of step zero and the lessons that come out right under the labels of the round, the labels of each lesson, the lessons of each key of two labels in their order, the lessons whose chart read each key, the lessons of more than one link that hold each label, and the keys of two labels that hold each label. over makes it; trial gives the gain and the loss of one change");

impl<'a> LinkTrials<'a> {
    pub(super) fn over(lessons: &'a LinkLessons, same: &dyn Fn(u32) -> u32, joins: &'a Joins, right: &'a [bool], items: &'a Symbols) -> LinkTrials<'a> {
        let labels: Vec<Vec<u32>> = lessons.lessons.iter().map(|(labels, _)| labels.iter().map(|label| same(*label)).collect()).collect();
        let mut readings: FirstSightTable<(u32, u32), Vec<usize>> = FirstSightTable::new();
        let mut looked: NumberTable<(u32, u32), Vec<usize>> = NumberTable::default();
        let mut holding: NumberTable<u32, Vec<usize>> = NumberTable::default();
        let mut asked: Vec<(u32, u32)> = Vec::new();
        for (number, mine) in labels.iter().enumerate() {
            if mine.len() < LINK_PAIR {
                continue;
            }
            if let [one, other] = mine.as_slice() {
                readings.at((*one, *other)).push(number);
            }
            for label in mine {
                let row = holding.entry(*label).or_default();
                if row.last() != Some(&number) {
                    row.push(number);
                }
            }
            asked.clear();
            chart_by(mine, |left, right| {
                asked.push((left, right));
                joined_entry(joins, left, right)
            });
            asked.sort_unstable();
            asked.dedup();
            for key in &asked {
                looked.entry(*key).or_default().push(number);
            }
        }
        let mut keys_with: NumberTable<u32, Vec<(u32, u32)>> = NumberTable::default();
        for (key, _) in readings.rows() {
            keys_with.entry(key.0).or_default().push(*key);
            if key.1 != key.0 {
                keys_with.entry(key.1).or_default().push(*key);
            }
        }
        LinkTrials { lessons, items, joins, right, labels, readings, looked, holding, keys_with }
    }

    pub(super) fn trial(&self, leaf: u32, result: u32) -> (usize, usize) {
        let swap = |label: u32| if label == leaf { result } else { label };
        let leaf_keys: &[(u32, u32)] = self.keys_with.get(&leaf).map_or(&[], Vec::as_slice);
        let mut moved: Vec<((u32, u32), Option<u32>)> = Vec::new();
        let mut images: Vec<(u32, u32)> = Vec::new();
        for key in leaf_keys {
            if self.joins.table.holds(key) {
                moved.push((*key, None));
            }
            let image = (swap(key.0), swap(key.1));
            if !images.contains(&image) {
                images.push(image);
            }
        }
        for image in &images {
            let mut merged: Vec<usize> = self.readings.get(image).cloned().unwrap_or_default();
            for key in leaf_keys.iter().filter(|key| (swap(key.0), swap(key.1)) == *image) {
                merged.extend(self.readings.get(key).into_iter().flatten());
            }
            merged.sort_unstable();
            let mut counts: FirstSightCount<u32> = FirstSightCount::new();
            for number in merged {
                counts.add(self.lessons.lessons[number].1, 1);
            }
            let entry = link_sure(&counts);
            if entry != self.joins.table.get(image).copied() {
                moved.push((*image, entry));
            }
        }
        let mut touched: Vec<usize> = self.holding.get(&leaf).cloned().unwrap_or_default();
        for (key, _) in &moved {
            touched.extend(self.looked.get(key).into_iter().flatten());
        }
        touched.sort_unstable();
        touched.dedup();
        let find = |left: u32, right: u32| match moved.iter().find(|(key, _)| *key == (left, right)) {
            Some((_, entry)) => entry.map(|got| (got, 0)),
            None => joined_entry(self.joins, left, right),
        };
        let (mut gain, mut lost) = (0, 0);
        for number in touched {
            let labels: Vec<u32> = self.labels[number].iter().map(|label| swap(*label)).collect();
            let span = chart_by(&labels, find);
            let now = self.lessons.best(&span[labels.len()], self.items) == Some(self.lessons.lessons[number].1);
            let before = self.right[number];
            gain += usize::from(now && !before);
            lost += usize::from(!now && before);
        }
        (gain, lost)
    }
}
