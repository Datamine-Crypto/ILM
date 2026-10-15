use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use crate::cases::Case;
use crate::tools::pattern::classes::{Classes, KIND_ROUNDS, ABSENT, sorted_texts, sorted_words, word_classes};
use crate::tools::pattern::kinds::{ENDS_NAMES, IN_LESSONS, JoinedForms, MOST_RUN, PAIR_RUN, PatternWords, STARTS_NAMES, joined_text, put};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::hash::{CheapMap, CheapSet};
use crate::tools::pattern::holes::run_contexts;
use patterns::{because, source};

pub struct SubstitutionRuns;
source!(SubstitutionRuns, "the substitution test of Harris for a constituent: a run of words is read as one word when it stands in a text where one word stands in another text that is the same in all other places");

pub struct PluralForms;
source!(PluralForms, "the regular plural of English: a word with the ending of a plural is the same word as the word without it, when the texts hold both");

pub struct StoryAnswers;
source!(StoryAnswers, "an irregular form: an answer word that its story never holds is the one story word of the same first letter that comes with it in nearly every case, as mice comes with mouse");

pub const LEAST_REST: usize = 3;
because!(LEAST_REST, SubstitutionRuns, "a run is looked for only where the text holds this many words beside it, so that the place of the run is a place in a text and not the whole text");

pub const SURE_PART: f64 = 0.98;
because!(SURE_PART, StoryAnswers, "the share of the cases of an answer word in which a story word must stand to be read as that answer word: nearly all of them, so that a few cases that were written wrong do not hide the word");

pub const LEAST_ANSWERS: u32 = 5;
because!(LEAST_ANSWERS, StoryAnswers, "an answer word that its story does not hold is joined with a story word only when this many cases or more have it");

pub const LEAST_RUNS: usize = 3;
because!(LEAST_RUNS, SubstitutionRuns, "a fixed word starts or ends names when this many runs of two words or more begin or end with it and their other words are of one kind");

pub const ENDINGS: &[(&str, &str)] = &[("ves", "f"), ("s", ""), ("es", "")];
because!(ENDINGS, PluralForms, "the endings of an English plural with what stands in their place in the singular; the ending that changes the stem comes first, because the plain ending would cut such a word at the wrong letter");

pub fn ending_base(word: &str, seen: &dyn Fn(&str) -> bool) -> Option<String> {
    for (ending, instead) in ENDINGS {
        if let Some(stem) = word.strip_suffix(ending) {
            let base = format!("{stem}{instead}");
            if seen(&base) {
                return Some(base);
            }
        }
    }
    None
}
because!(ending_base, JoinedForms, "the singular of a plural that the texts hold: the first ending in the order of the table whose other form stands in the texts");

pub fn prefix_len(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}
because!(prefix_len, JoinedForms, "the letters that two forms of a word share at their start; what comes after is the change of ending");

impl PatternWords {
    pub fn set_runs(&mut self, symbols: &mut Symbols, runs: &CheapSet<Vec<u32>>) {
        self.runs.clear();
        self.run_parts.clear();
        for run in sorted_texts(symbols, runs) {
            let text = joined_text(symbols, &run);
            let joined = symbols.number(&text);
            self.runs.insert(run.clone(), joined);
            self.run_parts.insert(joined, run);
        }
    }

    pub fn learn_same(&mut self, symbols: &mut Symbols, cases: &[Case]) {
        let mut seen: Vec<u32> = Vec::new();
        let mut known: CheapSet<u32> = CheapSet::default();
        for case in cases {
            let mut found: Vec<String> = Vec::new();
            for text in case.sentences.iter().chain(std::iter::once(&case.question)) {
                found.extend(words(text));
            }
            found.extend(answer_words(&case.answer));
            for word in found {
                let number = symbols.number(&word);
                if known.insert(number) {
                    seen.push(number);
                }
            }
        }
        for word in &seen {
            let text = symbols.name(*word).to_string();
            let base = ending_base(&text, &|base: &str| symbols.find(base).is_some_and(|number| known.contains(&number)));
            if let Some(base) = base {
                let base = symbols.number(&base);
                put(&mut self.read_as, *word, base);
            }
        }
    }

    pub fn learn_answer_forms(&mut self, symbols: &mut Symbols, cases: &[Case]) {
        for case in cases {
            for raw in answer_words(&case.answer) {
                let raw = symbols.number(&raw);
                let word = self.norm(raw);
                self.answer_forms.entry(word).or_default().add(raw, 1);
            }
        }
        let joined: CheapSet<u32> = self.read_as.iter().copied().filter(|base| *base != ABSENT).collect();
        if joined.is_empty() {
            return;
        }
        let mut changes: FirstSightTable<(String, String), u32> = FirstSightTable::new();
        for case in cases {
            for raw in answer_words(&case.answer) {
                let raw = symbols.number(&raw);
                let word = self.norm(raw);
                if raw == word && !joined.contains(&word) {
                    continue;
                }
                let mut forms: Vec<u32> = Vec::new();
                for text in case.sentences.iter().chain(std::iter::once(&case.question)) {
                    for told in words(text) {
                        let told = symbols.number(&told);
                        if self.norm(told) == word && !forms.contains(&told) {
                            forms.push(told);
                        }
                    }
                }
                if forms.is_empty() || forms.contains(&raw) {
                    continue;
                }
                let written = symbols.name(raw).to_string();
                for form in forms {
                    let form = symbols.name(form);
                    let shared = prefix_len(form, &written);
                    changes.add((form[shared..].to_string(), written[shared..].to_string()), 1);
                }
            }
        }
        self.answer_changes = changes.rows().to_vec();
    }

    pub fn learn_answer_words(&mut self, symbols: &mut Symbols, cases: &[Case]) {
        let mut with_answer: FirstSightTable<u32, (CheapMap<u32, u32>, u32)> = FirstSightTable::new();
        let mut all_answers: CheapSet<u32> = CheapSet::default();
        let mut told: CheapSet<Vec<u32>> = CheapSet::default();
        for case in cases {
            let mut story: CheapSet<u32> = CheapSet::default();
            for sentence in &case.sentences {
                let text = self.norm_row(symbols, &words(sentence));
                story.extend(text.iter().copied());
                told.insert(text);
            }
            let answer: CheapSet<u32> = self.norm_row(symbols, &answer_words(&case.answer)).into_iter().collect();
            for word in sorted_words(symbols, &answer) {
                all_answers.insert(word);
                if !story.contains(&word) {
                    let row = with_answer.entry(word, (CheapMap::default(), 0));
                    for told in &story {
                        *row.0.entry(*told).or_insert(0) += 1;
                    }
                    row.1 += 1;
                }
            }
        }
        if with_answer.rows().is_empty() {
            return;
        }
        let has_kind = word_classes(symbols, &sorted_texts(symbols, &told), &[]);
        for (answer, (count, cases_with)) in with_answer.rows() {
            let first = symbols.name(*answer).bytes().next();
            let close: Vec<u32> = count
                .iter()
                .filter(|(word, seen)| {
                    symbols.name(**word).bytes().next() == first && f64::from(**seen) >= SURE_PART * f64::from(*cases_with) && !all_answers.contains(*word) && has_kind.known(**word)
                })
                .map(|(word, _)| *word)
                .collect();
            if *cases_with >= LEAST_ANSWERS && close.len() == 1 {
                put(&mut self.read_as, close[0], *answer);
            }
        }
    }

    pub fn found_runs(texts: &[Vec<u32>], open_words: &Classes) -> (CheapSet<Vec<u32>>, Vec<(u32, Vec<u32>)>) {
        let slice = |(number, from, to): (u32, u32, u32)| &texts[number as usize][from as usize..to as usize];
        let mut runs: CheapSet<Vec<u32>> = CheapSet::default();
        let mut pairs: Vec<(u32, Vec<u32>)> = Vec::new();
        for found in run_contexts(texts, MOST_RUN, LEAST_REST) {
            for one in found.iter().map(|held| slice(*held)) {
                if one.len() != 1 {
                    continue;
                }
                let kind_one = open_words.class(one[0]);
                for run in found.iter().map(|held| slice(*held)) {
                    if run.len() > 1 && !run.contains(&one[0]) && (kind_one.is_none() || run.iter().all(|word| open_words.class(*word) != kind_one)) {
                        runs.insert(run.to_vec());
                        pairs.push((one[0], run.to_vec()));
                    }
                }
            }
        }
        (runs, pairs)
    }

    pub fn name_words(&mut self, symbols: &Symbols, texts: &[Vec<u32>], open_words: &Classes, runs: &mut CheapSet<Vec<u32>>, pairs: &[(u32, Vec<u32>)]) {
        let mut found: [CheapMap<u32, u32>; PAIR_RUN] = [CheapMap::default(), CheapMap::default()];
        for (at, kept) in found.iter_mut().enumerate() {
            let mut group: FirstSightTable<u32, CheapMap<u32, CheapSet<u32>>> = FirstSightTable::new();
            for run in sorted_texts(symbols, runs) {
                if run.len() == PAIR_RUN && !open_words.known(run[at]) {
                    if let Some(kind) = open_words.class(run[1 - at]) {
                        group.entry(run[at], CheapMap::default()).entry(kind).or_default().insert(run[1 - at]);
                    }
                }
            }
            for (word, by_kind) in group.rows() {
                let mut only = by_kind.iter();
                if let (Some((kind, words_of)), None) = (only.next(), only.next()) {
                    if words_of.len() >= LEAST_RUNS {
                        kept.insert(*word, *kind);
                    }
                }
            }
        }
        for (at, bit) in [STARTS_NAMES, ENDS_NAMES].into_iter().enumerate() {
            for word in found[at].keys() {
                self.flag(*word, bit);
            }
        }
        if found.iter().all(|kept| kept.is_empty()) {
            return;
        }
        let mut run_kind: CheapMap<Vec<u32>, u32> = CheapMap::default();
        for (one, run) in pairs {
            if let Some(kind) = open_words.class(*one) {
                run_kind.entry(run.clone()).or_insert(kind);
            }
        }
        let mut more: CheapSet<Vec<u32>> = CheapSet::default();
        for text in texts {
            for from in 0..text.len() {
                for to in from + PAIR_RUN..=text.len().min(from + MOST_RUN) {
                    let group = &text[from..to];
                    let sides: [(u32, &[u32], &CheapMap<u32, u32>); PAIR_RUN] = [(group[0], &group[1..], &found[0]), (group[group.len() - 1], &group[..group.len() - 1], &found[1])];
                    for (word, rest, kept) in sides {
                        if let Some(kind) = kept.get(&word) {
                            let one = rest.len() == 1 && open_words.class(rest[0]) == Some(*kind);
                            if one || (runs.contains(rest) && run_kind.get(rest) == Some(kind)) {
                                more.insert(group.to_vec());
                            }
                        }
                    }
                }
            }
        }
        runs.extend(more);
    }

    pub fn drop_runs(&mut self, symbols: &mut Symbols, cases: &[Case], open_words: &Classes, mut runs: CheapSet<Vec<u32>>) -> CheapSet<Vec<u32>> {
        let mut read: FirstSightTable<(Vec<String>, String, String), (Vec<Vec<u32>>, Vec<u32>, CheapSet<u32>)> = FirstSightTable::new();
        for case in cases {
            let key = (case.sentences.clone(), case.question.clone(), case.answer.clone());
            if read.holds(&key) {
                continue;
            }
            let mut story: Vec<Vec<u32>> = Vec::new();
            for sentence in &case.sentences {
                let text = self.norm_row(symbols, &words(sentence));
                if !story.contains(&text) {
                    story.push(text);
                }
            }
            let question = self.norm_row(symbols, &words(&case.question));
            let answer: CheapSet<u32> = self.norm_row(symbols, &answer_words(&case.answer)).into_iter().collect();
            read.entry(key, (story, question, answer));
        }
        for _ in 0..KIND_ROUNDS {
            self.set_runs(symbols, &runs);
            let mut joined: CheapMap<&Vec<u32>, (Vec<Vec<u32>>, Vec<u32>)> = CheapMap::default();
            let mut bad: CheapSet<Vec<u32>> = CheapSet::default();
            for (_, (story, question, answer)) in read.rows() {
                let mut here: CheapSet<&Vec<u32>> = CheapSet::default();
                let mut alone: CheapSet<u32> = answer.clone();
                alone.extend(self.join(question).into_iter().filter(|word| !self.run_parts.contains_key(word)));
                for sentence in story {
                    let (as_runs, single) = joined.entry(sentence).or_insert_with(|| {
                        let text = self.join(sentence);
                        let as_runs: Vec<Vec<u32>> = text.iter().filter_map(|word| self.run_parts.get(word).cloned()).collect();
                        let single: Vec<u32> = text.iter().copied().filter(|word| !self.run_parts.contains_key(word)).collect();
                        (as_runs, single)
                    });
                    alone.extend(single.iter().copied());
                    here.extend(as_runs.iter().filter_map(|run| runs.get(run)));
                }
                for run in here {
                    if run.iter().any(|word| alone.contains(word) && open_words.known(*word)) {
                        bad.insert(run.clone());
                    }
                }
            }
            if bad.is_empty() {
                break;
            }
            runs.retain(|run| !bad.contains(run));
        }
        runs
    }

    pub fn name_kinds(&mut self, symbols: &Symbols, texts: &[Vec<u32>], classes: &Classes) {
        let mut often: CheapMap<u32, u32> = CheapMap::default();
        for word in texts.iter().flatten() {
            *often.entry(*word).or_insert(0) += 1;
        }
        let mut name: CheapMap<u32, u32> = CheapMap::default();
        let mut members = classes.members();
        members.sort_by(|a, b| symbols.name(*a).cmp(symbols.name(*b)));
        for word in &members {
            let Some(root) = classes.class(*word) else { continue };
            let count = often.get(word).copied().unwrap_or(0);
            if name.get(&root).is_none_or(|best| count > often.get(best).copied().unwrap_or(0)) {
                name.insert(root, *word);
            }
        }
        let mut names: Vec<String> = name.values().map(|word| symbols.name(*word).to_uppercase()).collect();
        names.sort();
        names.dedup();
        for word in &members {
            if let Some(root) = classes.class(*word) {
                let text = symbols.name(name[&root]).to_uppercase();
                let number = names.binary_search(&text).unwrap_or(0);
                put(&mut self.kinds, *word, number as u32);
            }
        }
        self.members = vec![Vec::new(); names.len()];
        self.kind_names = names;
    }

    pub fn learn_answer_kinds(&mut self, symbols: &mut Symbols, cases: &[Case], stories: &CheapSet<Vec<u32>>) {
        let mut answers: FirstSightTable<u32, u32> = FirstSightTable::new();
        for case in cases {
            for word in self.read(symbols, &answer_words(&case.answer)) {
                answers.add(word, 1);
            }
        }
        let story_words: CheapSet<u32> = stories.iter().flat_map(|story| self.join(story)).collect();
        for (word, _) in answers.by_count() {
            let kind = self.kind_of(*word);
            if kind != ABSENT && story_words.contains(word) && !self.answer_kinds.contains(&kind) {
                self.answer_kinds.push(kind);
            }
        }
    }

    pub fn learn_shapes(&mut self, symbols: &Symbols, texts: &[Vec<u32>], stories: &CheapSet<Vec<u32>>) {
        for text in texts {
            for word in text {
                self.flag(*word, IN_LESSONS);
            }
            let shape = self.shape_of(text, &[]);
            self.shapes.entry(text.len()).or_default().add(shape, 1);
        }
        let mut with_kind: Vec<u32> = (0..self.kinds.len() as u32).filter(|word| self.kind_of(*word) != ABSENT && self.has(*word, IN_LESSONS)).collect();
        with_kind.sort_by(|a, b| symbols.name(*a).cmp(symbols.name(*b)));
        for word in with_kind {
            let kind = self.kind_of(word) as usize;
            self.members[kind].push(word);
        }
        for word in 0..self.flags.len() as u32 {
            if self.has(word, STARTS_NAMES | ENDS_NAMES) {
                self.flag(word, IN_LESSONS);
            }
        }
        for story in stories {
            let text = self.join(story);
            let shape = self.shape_of(&text, &[]);
            self.sent_shapes.entry(text.len()).or_default().insert(shape);
        }
    }

    pub fn learn(cases: &[Case], symbols: &mut Symbols) -> PatternWords {
        let mut found = PatternWords::default();
        found.learn_same(symbols, cases);
        found.learn_answer_forms(symbols, cases);
        found.learn_answer_words(symbols, cases);
        let mut stories: CheapSet<Vec<u32>> = CheapSet::default();
        let mut both: CheapSet<Vec<u32>> = CheapSet::default();
        for case in cases {
            for sentence in &case.sentences {
                stories.insert(found.norm_row(symbols, &words(sentence)));
            }
            both.insert(found.norm_row(symbols, &words(&case.question)));
        }
        both.extend(stories.iter().cloned());
        let texts = sorted_texts(symbols, &both);
        let open_words = word_classes(symbols, &texts, &[]);
        let (mut runs, pairs) = PatternWords::found_runs(&texts, &open_words);
        found.name_words(symbols, &texts, &open_words, &mut runs, &pairs);
        let runs = found.drop_runs(symbols, cases, &open_words, runs);
        found.set_runs(symbols, &runs);
        let joined: CheapSet<Vec<u32>> = texts.iter().map(|text| found.join(text)).collect();
        let joined_texts = sorted_texts(symbols, &joined);
        let unions: Vec<(u32, u32)> = pairs.iter().filter_map(|(one, run)| found.runs.get(run).map(|joined| (*one, *joined))).collect();
        let classes = word_classes(symbols, &joined_texts, &unions);
        found.name_kinds(symbols, &joined_texts, &classes);
        found.learn_answer_kinds(symbols, cases, &stories);
        found.learn_shapes(symbols, &joined_texts, &stories);
        found
    }
}
