use ilm_data::controls::file::ChangedCase;
use ilm_data::controls::made::made_changes;
use ilm_core::cases::Case;
use ilm_data::sets::made::{made_answer, made_family, made_kind};

#[test]
fn held_families_are_not_made() {
    assert!(made_family(50, false).is_err());
    assert!(made_family(99, false).is_err());
    assert_eq!(made_kind("made_12"), Some((12, false)));
    assert_eq!(made_kind("made_12_deep"), Some((12, true)));
    assert_eq!(made_kind("babi_1"), None);
}

#[test]
fn the_family_of_seed_one() {
    let family = made_family(1, false).unwrap();
    assert_eq!((family.lessons.len(), family.read.len(), family.deep.len()), (1000, 400, 400));
    assert_eq!((family.lessons[0].text.as_str(), family.lessons[0].answer.as_str()), ("koliv buju", "sozid"));
    assert_eq!((family.read[0].text.as_str(), family.read[0].answer.as_str()), ("vasa ( vofod vafu ( kafik viki koja ) )", "zunar jada"));
    assert_eq!(family.deep[0].answer, "kolis bajut pova jobib");
    for case in family.lessons.iter().chain(&family.read).chain(&family.deep) {
        assert_eq!(made_answer(&family, &case.text).as_deref(), Some(case.answer.as_str()));
    }
    assert_eq!(made_answer(&family, "( ("), None);
}

#[test]
fn changed_texts_of_the_family_of_seed_one() {
    let family = made_family(1, false).unwrap();
    let lessons: Vec<Case> = family.lessons.iter().map(|case| Case { sentences: Vec::new(), question: case.text.clone(), answer: case.answer.clone() }).collect();
    let changes = made_changes(&lessons, false, "made_1").unwrap();
    assert_eq!(changes.len(), 1721);
    let first = ChangedCase { place: Some(667), sort: "item".to_string(), sentences: Vec::new(), question: "koliv ( kafik buju jipa )".to_string(), truth: "jada".to_string() };
    assert_eq!(changes[0], first);
    let last = changes.last().unwrap();
    assert_eq!((last.place, last.sort.as_str(), last.truth.as_str()), (Some(999), "out", "sozid kulo jada pova"));
    assert_eq!(made_changes(&lessons, true, "made_1").unwrap().first().and_then(|change| change.place), Some(1));
    assert!(made_changes(&lessons[1..], false, "made_1").is_err());
}
