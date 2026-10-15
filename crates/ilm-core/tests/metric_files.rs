use ilm_core::write::command::writer_command;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let place = std::env::temp_dir().join(format!("ilm_metric_files_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&place);
    std::fs::create_dir_all(&place).expect("a scratch folder");
    place
}

const ROWS: &str = "#\tset=E2E\trows=3\nF\tname\tThe Vaults\nF\tfood\tItalian\nT\tThe Vaults serves Italian food.\n\nF\tname\tThe Mill\nT\tThe Mill is a place.\n\nF\tfood\tItalian\nF\tname\tThe Vaults\nT\tItalian food is at The Vaults.\n\n";

#[test]
fn the_files_for_the_official_metrics_hold_the_references_of_each_list_and_the_written_texts() {
    let place = scratch("lists");
    let rows = place.join("rows.txt");
    let written = place.join("written.txt");
    std::fs::write(&rows, ROWS).expect("rows");
    std::fs::write(&written, "W\tThe Vaults has Italian food.\nW\tThe Mill.\n").expect("written texts");
    let references = place.join("references.txt");
    let texts = place.join("texts.txt");
    let paths = [rows, written, references.clone(), texts.clone()].map(|path| path.to_string_lossy().into_owned());
    let _ = writer_command(&["metric-files", &paths[0], &paths[1], &paths[2], &paths[3]]);
    assert_eq!(std::fs::read_to_string(&references).expect("references"), "The Vaults serves Italian food.\nItalian food is at The Vaults.\n\nThe Mill is a place.\n");
    assert_eq!(std::fs::read_to_string(&texts).expect("texts"), "The Vaults has Italian food.\nThe Mill.\n");
}

#[test]
fn no_file_is_written_when_the_written_texts_do_not_match_the_lists() {
    let place = scratch("short");
    let rows = place.join("rows.txt");
    let written = place.join("written.txt");
    std::fs::write(&rows, ROWS).expect("rows");
    std::fs::write(&written, "W\tThe Vaults has Italian food.\n").expect("written texts");
    let references = place.join("references.txt");
    let texts = place.join("texts.txt");
    let paths = [rows, written, references.clone(), texts.clone()].map(|path| path.to_string_lossy().into_owned());
    let _ = writer_command(&["metric-files", &paths[0], &paths[1], &paths[2], &paths[3]]);
    assert!(!references.exists());
    assert!(!texts.exists());
}
