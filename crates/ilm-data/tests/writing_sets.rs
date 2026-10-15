use ilm_data::sets::export::Described;
use ilm_data::sets::readers::writing::{rows_file, write_rows};
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let place = std::env::temp_dir().join(format!("ilm_writing_sets_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&place);
    std::fs::create_dir_all(&place).expect("a scratch folder");
    place
}

#[test]
fn a_file_of_rows_holds_the_facts_and_the_text_with_escapes() {
    let rows = vec![(vec![("name".to_string(), "The Vaults".to_string()), ("food".to_string(), "Tab\there".to_string())], "Line one\nline two \\ end".to_string())];
    let text = rows_file("E2E", &rows, None);
    assert_eq!(text, "#\tset=E2E\trows=1\nF\tname\tThe Vaults\nF\tfood\tTab\\there\nT\tLine one\\nline two \\\\ end\n\n");
    let listed = rows_file("ViGGO", &[], Some("44.32"));
    assert_eq!(listed, "#\tset=ViGGO\trows=0\tcut=44.32\n");
}

#[test]
fn a_part_with_facts_inside_a_sort_is_read_from_its_file_of_fields() {
    let place = scratch("fields");
    let train = "\u{feff}gem_id,meaning_representation,target,references,mr,ref\n\
                 a,x,y,z,\"inform(name[Tetris], platforms[PC, Xbox], rating[good])\",\" Tetris is a good game on PC and Xbox. \"\n\
                 b,x,y,z,no facts here,Nothing to read.\n\
                 c,x,y,z,\"request(specifier[strange])\",\"Do you, too, like strange games?\"\n";
    std::fs::write(place.join("train.csv"), train).expect("the train file");
    std::fs::write(place.join("validation.csv"), "mr,ref\n\"confirm(name[Doom])\",\"So, Doom?\"\n").expect("the validation file");
    let lines = [
        "set\tViGGO".to_string(),
        "rows\tfields".to_string(),
        "facts\tinside".to_string(),
        "columns\tmr\tref".to_string(),
        "cut\t44.32".to_string(),
        format!("file\ttrain\trows\t{}", place.join("train.csv").display()),
        format!("file\tvalidation\trows\t{}", place.join("validation.csv").display()),
    ];
    std::fs::write(place.join("about.txt"), lines.join("\n") + "\n").expect("the description");
    let described = Described::read(&place.join("about.txt")).expect("a description");
    let said = write_rows(&described, &place.join("out")).expect("a set of the writer").expect("the rows are written");
    assert_eq!(said, vec!["ViGGO train: 2 rows".to_string(), "ViGGO validation: 1 rows".to_string()]);
    let written = std::fs::read_to_string(place.join("out").join("train.txt")).expect("the rows of the train part");
    assert_eq!(
        written,
        "#\tset=ViGGO\trows=2\tcut=44.32\nF\tact\tinform\nF\tname\tTetris\nF\tplatforms\tPC, Xbox\nF\trating\tgood\nT\tTetris is a good game on PC and Xbox.\n\nF\tact\trequest\nF\tspecifier\tstrange\nT\tDo you, too, like strange games?\n\n"
    );
    std::fs::write(place.join("other.txt"), "set\tbAbI\nfit\t1000\n").expect("another description");
    let other = Described::read(&place.join("other.txt")).expect("a description");
    assert!(write_rows(&other, &place.join("other")).is_none());
    let _ = std::fs::remove_dir_all(&place);
}
