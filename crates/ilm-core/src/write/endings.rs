use crate::base::keys::NO_NUMBER;
use crate::write::fit::{PlaceFiller, WRITE_UNFIT_CHANCE};
use crate::write::signs::name_ending;
use crate::write::surface::WRITE_ENDING_SIGNS;
use crate::base::first_sight::{all_counts, key_count};
use patterns::source;

pub struct NameEndings;
source!(NameEndings, "the form of a name is counted by the ending of its first word with the unit to its left, as a learner of word forms counts endings: a name does not follow a unit when names of its ending are surely rare there");

impl<'a> PlaceFiller<'a> {
    pub(crate) fn ending_of(&self, value: u32) -> u32 {
        self.lessons.syms.find(&name_ending(self.fact_value_text(value), WRITE_ENDING_SIGNS)).unwrap_or(NO_NUMBER)
    }

    pub fn end_main(&mut self, slot: u32, value: u32) -> u32 {
        let ending = self.ending_of(value);
        if let Some(known) = self.ends_main.get(&(slot, ending)) {
            return *known;
        }
        let found = self.main_of(slot, ending);
        self.ends_main.insert((slot, ending), found);
        found
    }

    fn main_of(&self, slot: u32, ending: u32) -> u32 {
        let Some(table) = self.lessons.surface.left_end.get(slot as usize) else {
            return NO_NUMBER;
        };
        let with_end: Vec<(u32, i64)> = table.rows().iter().map(|(unit, seen)| (*unit, key_count(seen, &ending))).filter(|(_, many)| *many > 0).collect();
        let mut most: Option<(u32, i64)> = None;
        for (unit, many) in &with_end {
            let better = most.is_none_or(|(top, count)| *many > count || (*many == count && self.lessons.rank[*unit as usize] < self.lessons.rank[top as usize]));
            if better {
                most = Some((*unit, *many));
            }
        }
        let Some((most, most_count)) = most else {
            return NO_NUMBER;
        };
        let all: i64 = table.rows().iter().map(|(_, seen)| all_counts(seen)).sum();
        let of_most = table.get(&most).map_or(0, all_counts);
        let ended: i64 = with_end.iter().map(|(_, many)| *many).sum();
        let mean = (((all - of_most) * ended) as f64) / (all as f64);
        let others = ended - most_count;
        let mut endings: Vec<u32> = table.rows().iter().flat_map(|(_, seen)| seen.rows().iter().map(|(ending, _)| *ending)).collect();
        endings.sort();
        endings.dedup();
        if PlaceFiller::rare(others, mean, WRITE_UNFIT_CHANCE / (endings.len() as f64)) { most } else { NO_NUMBER }
    }

    pub fn end_fits(&mut self, slot: u32, unit: u32, value: u32) -> bool {
        let ending = self.ending_of(value);
        if let Some(known) = self.ends_fit.get(&(slot, unit, ending)) {
            return *known;
        }
        let main = self.end_main(slot, value);
        let found = self.fits_after(slot, unit, ending) && (main == NO_NUMBER || main == unit);
        self.ends_fit.insert((slot, unit, ending), found);
        found
    }

    fn fits_after(&self, slot: u32, unit: u32, ending: u32) -> bool {
        let Some(table) = self.lessons.surface.left_end.get(slot as usize) else {
            return true;
        };
        let mut fit = true;
        if let Some(here) = table.get(&unit).filter(|here| !here.rows().is_empty()) {
            let mut top: Option<(u32, i64)> = None;
            for (seen, many) in here.rows() {
                let better = top.is_none_or(|(best, count)| *many > count || (*many == count && self.lessons.rank[*seen as usize] < self.lessons.rank[best as usize]));
                if better {
                    top = Some((*seen, *many));
                }
            }
            let (top, top_count) = top.unwrap_or((NO_NUMBER, 0));
            if ending != top {
                let with_top: i64 = table.rows().iter().map(|(_, seen)| key_count(seen, &top)).sum();
                let all: i64 = table.rows().iter().map(|(_, seen)| all_counts(seen)).sum();
                let mean = (all_counts(here) as f64) * (f64::from(true) - (with_top as f64) / (all as f64));
                let others = all_counts(here) - top_count;
                fit = !PlaceFiller::rare(others, mean, WRITE_UNFIT_CHANCE / (table.rows().len() as f64));
            }
        }
        fit
    }
}
