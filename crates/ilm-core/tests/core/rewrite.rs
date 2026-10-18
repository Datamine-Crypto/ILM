use ilm_core::base::words::answer_words;
use ilm_core::cases::Case;
use ilm_core::tools::call::{AnswerMark, Learner, Reading};
use ilm_core::tools::explain::wording_rewrite::RewriteModel;

const NOUNS: &[&str] = &["blick", "dax", "fep", "wug", "zorp"];

const DETERMINERS: &[&str] = &["the", "my", "some"];

const VERBS: &[&str] = &["admired", "amused", "greeted", "pushed"];

const JOINING: &[&str] = &["near", "behind", "under"];

fn phrase(seed: usize, with_phrase: bool) -> (String, bool) {
    let plural = seed / 7 % 2 == 0;
    let noun = NOUNS[seed % NOUNS.len()];
    let head = format!("{} {noun}{}", DETERMINERS[seed / 3 % DETERMINERS.len()], if plural { "s" } else { "" });
    if !with_phrase {
        return (head, plural);
    }
    (format!("{head} {} {} {}", JOINING[seed / 5 % JOINING.len()], DETERMINERS[seed / 11 % DETERMINERS.len()], NOUNS[(seed / 13 + 1) % NOUNS.len()]), plural)
}

fn rewrite_case(at: usize) -> Case {
    let mixed = ((at as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) as usize;
    let verb = VERBS[mixed % VERBS.len()];
    if mixed / 17 % 2 == 0 {
        let (subject, _) = phrase(mixed / 19, false);
        let (object, plural) = phrase(mixed / 23, mixed / 29 % 2 == 0);
        Case { sentences: Vec::new(), question: format!("passiv: {subject} {verb} {object} ."), answer: format!("{object} {} {verb} by {subject} .", if plural { "were" } else { "was" }) }
    } else {
        let (subject, _) = phrase(mixed / 19, mixed / 31 % 2 == 0);
        let (object, _) = phrase(mixed / 23, mixed / 29 % 2 == 0);
        let sentence = format!("{subject} {verb} {object} .");
        Case { sentences: Vec::new(), question: format!("decl: {sentence}"), answer: sentence }
    }
}

#[test]
fn a_unit_learned_from_moving_runs_carries_a_phrase_after_the_noun_of_a_subject_that_no_passive_lesson_shows() {
    let cases: Vec<Case> = (0..160).map(rewrite_case).collect();
    assert!(cases.iter().any(|case| case.question.starts_with("passiv:") && case.answer.contains("near")), "the lessons hold passives with a phrase after the object's noun");
    let model = RewriteModel::learn(&cases.iter().collect::<Vec<&Case>>()).expect("templates are learned");
    assert_eq!(model.reading(), Some(Reading::Learned));
    let new_build = model.answer(&[], "passiv: the blick near my dax admired some feps .");
    assert_eq!(new_build.words, answer_words("some feps were admired by the blick near my dax ."), "{}", new_build.note);
    let singular = model.answer(&[], "passiv: my wugs behind the zorp greeted the dax under some blick .");
    assert_eq!(singular.words, answer_words("the dax under some blick was greeted by my wugs behind the zorp ."), "{}", singular.note);
    assert!(model.lines_from_lessons().iter().any(|line| line.starts_with("A unit is D N")), "{:?}", model.lines_from_lessons());
    assert_eq!(new_build.mark, AnswerMark::Proof);
    let new_verb = model.answer(&[], "passiv: the blick zapped some fep .");
    assert_eq!(new_verb.words, answer_words("some fep was zapped by the blick ."), "{}", new_verb.note);
    assert_eq!(new_verb.mark, AnswerMark::Guess, "a word that no lesson and no class holds makes the answer a guess");
}

const MADE_NOUNS: &[&str] = &["zib", "plon", "trask", "veem"];

fn question_case(at: usize) -> Case {
    let mixed = ((at as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) as usize;
    let noun = |seed: usize| format!("{} {}", DETERMINERS[seed % DETERMINERS.len()], MADE_NOUNS[seed / 3 % MADE_NOUNS.len()]);
    let subject = if mixed / 5 % 2 == 0 { noun(mixed) } else { format!("{} {} {}", noun(mixed), JOINING[mixed / 7 % JOINING.len()], noun(mixed / 11)) };
    let helper = ["has", "hasn't"][mixed / 13 % 2];
    let verb = VERBS[mixed / 17 % VERBS.len()];
    if mixed / 19 % 2 == 0 {
        Case { sentences: Vec::new(), question: format!("quest: {subject} {helper} {verb} ."), answer: format!("{helper} {subject} {verb} ?") }
    } else {
        let sentence = format!("{subject} {helper} {verb} .");
        Case { sentences: Vec::new(), question: format!("decl: {sentence}"), answer: sentence }
    }
}

#[test]
fn a_noun_that_no_fact_names_is_learned_as_the_word_after_a_determiner() {
    let cases: Vec<Case> = (0..120).map(question_case).collect();
    let model = RewriteModel::learn(&cases.iter().collect::<Vec<&Case>>()).expect("templates are learned");
    let said = model.answer(&[], "quest: the plon behind some veem hasn't amused .");
    assert_eq!(said.words, answer_words("hasn't the plon behind some veem amused ?"), "{}", said.note);
    assert!(model.lines_from_lessons().iter().any(|line| line.starts_with("A unit is D N")), "the head of a unit is a determiner and a learned noun: {:?}", model.lines_from_lessons());
    let facts = model.facts_used(&[], "quest: the plon behind some veem hasn't amused .");
    assert!(facts.contains(&("plon is a noun".to_string(), false)), "a learned noun is a fact that the lessons gave, not the library: {facts:?}");
    assert!(facts.contains(&("the is a determiner".to_string(), false)), "a determiner is learned from the lessons too: {facts:?}");
}

const GERMAN_DETERMINERS: &[(&str, &str, &str, bool)] = &[("der", "den", "dem", false), ("die", "die", "den", true), ("mein", "meinen", "meinem", false), ("meine", "meine", "meinen", true), ("ein", "einen", "einem", false), ("einige", "einige", "einigen", true)];

const GERMAN_NOUNS: &[(&str, &str, &str, &str)] = &[("Wug", "Wuge", "Wug", "Wugen"), ("Fep", "Fepe", "Fep", "Fepen"), ("Zorp", "Zorpe", "Zorp", "Zorpen"), ("Plon", "Plone", "Plon", "Plonen"), ("Kesser", "Kesser", "Kesser", "Kessern"), ("Trabe", "Traben", "Traben", "Traben")];

const GERMAN_VERBS: &[(&str, &str, &str)] = &[("zappte", "zappten", "gezappt"), ("lobte", "lobten", "gelobt"), ("rief", "riefen", "gerufen"), ("mied", "mieden", "gemieden")];

const EITHER_NOUN: usize = 4;

fn german_phrase(det: usize, noun: usize, case_place: usize) -> String {
    let (nominative, accusative, dative, plural) = GERMAN_DETERMINERS[det];
    let (singular, many, oblique, many_dative) = GERMAN_NOUNS[noun];
    let noun_form = match (plural, case_place) {
        (true, 2) => many_dative,
        (true, _) => many,
        (false, 0) => singular,
        (false, _) => oblique,
    };
    format!("{} {noun_form}", [nominative, accusative, dative][case_place])
}

fn german_case(at: usize) -> Case {
    let mixed = ((at as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) as usize;
    let pick = |seed: usize| {
        let noun = seed / 7 % GERMAN_NOUNS.len();
        let det = seed % GERMAN_DETERMINERS.len();
        (if noun == EITHER_NOUN && GERMAN_DETERMINERS[det].3 { 1 } else { det }, noun)
    };
    let ((subject_det, subject_noun), (object_det, object_noun)) = (pick(mixed / 3), pick(mixed / 41));
    let (past, many_past, participle) = GERMAN_VERBS[mixed / 13 % GERMAN_VERBS.len()];
    let verb = if GERMAN_DETERMINERS[subject_det].3 { many_past } else { past };
    let beside = if mixed / 17 % 2 == 0 { format!(" neben {}", german_phrase(mixed / 19 % GERMAN_DETERMINERS.len(), mixed / 23 % EITHER_NOUN, 2)) } else { String::new() };
    let object = format!("{}{beside}", german_phrase(object_det, object_noun, 1));
    if mixed / 29 % 2 == 0 {
        let written = format!("{} {}{beside}", GERMAN_DETERMINERS[object_det].0, german_phrase(object_det, object_noun, 1).split_whitespace().nth(1).unwrap_or_default());
        let helper = if GERMAN_DETERMINERS[object_det].3 { "wurden" } else { "wurde" };
        Case { sentences: Vec::new(), question: format!("passiv: {} {verb} {object} .", german_phrase(subject_det, subject_noun, 0)), answer: format!("{written} {helper} von {} {participle} .", german_phrase(subject_det, subject_noun, 2)) }
    } else {
        let sentence = format!("{}{} {verb} {object} .", german_phrase(subject_det, subject_noun, 0), if mixed / 31 % 2 == 0 { format!(" neben {}", german_phrase(mixed / 37 % GERMAN_DETERMINERS.len(), mixed / 43 % EITHER_NOUN, 2)) } else { String::new() });
        Case { sentences: Vec::new(), question: format!("decl: {sentence}"), answer: sentence }
    }
}

#[test]
fn the_forms_of_a_german_passive_are_learned_from_the_lessons() {
    let cases: Vec<Case> = (0..400).map(german_case).collect();
    assert!(!cases.iter().any(|case| case.question.contains("meine Kesser")), "no lesson has the plural Kesser after meine");
    let model = RewriteModel::learn(&cases.iter().collect::<Vec<&Case>>()).expect("templates are learned");
    let new_build = model.answer(&[], "passiv: meine Kesser neben einem Fep zappten den Wug .");
    assert_eq!(new_build.words, answer_words("der Wug wurde von meinen Kessern neben einem Fep gezappt ."), "{}", new_build.note);
    assert_eq!(new_build.mark, AnswerMark::Proof);
    let singular = model.answer(&[], "passiv: der Kesser lobte einige Wuge .");
    assert_eq!(singular.words, answer_words("einige Wuge wurden von dem Kesser gelobt ."), "{}", singular.note);
    let weak = model.answer(&[], "passiv: der Trabe mied die Plone .");
    assert_eq!(weak.words, answer_words("die Plone wurden von dem Traben gemieden ."), "{}", weak.note);
    assert!(model.lines_from_lessons().iter().any(|line| line.contains("in the form of the lessons")), "{:?}", model.lines_from_lessons());
}

#[test]
fn a_word_that_no_lesson_pairs_is_found_through_words_of_the_same_ending_and_marked_a_guess() {
    let english: Vec<Case> = (0..320).map(rewrite_case).filter(|case| !(case.question.starts_with("passiv:") && case.answer.split_whitespace().nth(1) == Some("zorps"))).collect();
    let model = RewriteModel::learn(&english.iter().collect::<Vec<&Case>>()).expect("templates are learned");
    let plural = model.answer(&[], "passiv: the blick admired some zorps .");
    assert_eq!(plural.words, answer_words("some zorps were admired by the blick ."), "were through zorp, as feps beside fep: {}", plural.note);
    assert_eq!(plural.mark, AnswerMark::Guess, "a piece found through a related word is no proof");
    let german: Vec<Case> = (0..400).map(german_case).filter(|case| !(case.question.starts_with("passiv:") && case.question.split_whitespace().nth(2) == Some("Plone"))).collect();
    let model = RewriteModel::learn(&german.iter().collect::<Vec<&Case>>()).expect("templates are learned");
    let ending = model.answer(&[], "passiv: einige Plone zappten den Wug .");
    assert_eq!(ending.words, answer_words("der Wug wurde von einigen Plonen gezappt ."), "Plonen as Wugen and Fepen after the plural determiners: {}", ending.note);
    assert_eq!(ending.mark, AnswerMark::Guess);
}
