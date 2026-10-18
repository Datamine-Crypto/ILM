use ilm_data::sets::export::Described;
use ilm_data::sets::readers::inflect::{inflect_cases, inflect_large};

fn line(fields: &[&str]) -> Vec<String> {
    fields.iter().map(|field| field.to_string()).collect()
}

#[test]
fn letters_and_features_of_made_words() {
    let folder = std::env::temp_dir().join(format!("ilm_inflect_reader_{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let train = folder.join("made-train");
    std::fs::write(&train, "kab\tk\u{e4}be\tV;PST;3\n\tx\tV\nmo\tmos\tN;PL\r\n").unwrap();
    let dev = folder.join("made-dev");
    std::fs::write(&dev, "zu\tzuk\tADJ;\u{dc}/x\n").unwrap();
    let train_place = train.display().to_string();
    let dev_place = dev.display().to_string();
    let described = Described { lines: vec![line(&["file", "infl_made", "train", &train_place]), line(&["file", "infl_made", "read", &dev_place]), line(&["file", "infl_made", "high", &dev_place])] };
    let found = inflect_cases(&described, "infl_made").unwrap();
    let cases: Vec<(&str, &str)> = found.cases.iter().map(|case| (case.question.as_str(), case.answer.as_str())).collect();
    assert_eq!(cases, [("k a b tv tpst t3", "k u00e4 b e"), ("m o tn tpl", "m o s"), ("z u tadj txdcx2fx", "z u k")]);
    assert_eq!(found.read_from, Some(2));
    assert_eq!(inflect_large(&described, "infl_made").unwrap().len(), 1);
}
