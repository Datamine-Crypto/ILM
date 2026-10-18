use ilm_data::sets::knowledge::text_facts::{fact_form, name_runs, corpus_sentences, corpus_tokens};
use std::collections::HashSet;

#[test]
fn a_line_of_the_text_is_cut_into_sentences_where_a_capital_letter_follows_the_end_of_a_sentence() {
    assert_eq!(corpus_sentences("Oslo is old. It lies in Norway!  The end"), vec!["Oslo is old.", "It lies in Norway!", "The end"]);
    assert_eq!(corpus_sentences("It ran from 1900 to c. 1950 in Lima."), vec!["It ran from 1900 to c. 1950 in Lima."]);
}

#[test]
fn a_sentence_is_read_as_words_with_their_inside_signs_and_marks_of_their_own() {
    assert_eq!(corpus_tokens("Port-au-Prince, the capital of Haiti."), vec!["Port-au-Prince", ",", "the", "capital", "of", "Haiti", "."]);
    assert_eq!(corpus_tokens("Côte d'Ivoire (west)"), vec!["Côte", "d'Ivoire", "(", "west", ")"]);
}

#[test]
fn a_name_is_a_run_of_words_with_capitals_joined_only_by_the_small_words_of_the_lessons() {
    let tokens = corpus_tokens("He went from the United States of America to Mexico City in May.");
    let none = HashSet::new();
    let of: HashSet<String> = ["of".to_string()].into_iter().collect();
    let shown = |runs: Vec<(usize, usize)>| -> Vec<String> { runs.into_iter().map(|(start, end)| tokens[start..end].join(" ")).collect() };
    assert_eq!(shown(name_runs(&tokens, &none)), vec!["He", "United States", "America", "Mexico City", "May"]);
    assert_eq!(shown(name_runs(&tokens, &of)), vec!["He", "United States of America", "Mexico City", "May"]);
}

#[test]
fn the_own_form_of_facts_of_two_names_is_the_longest_run_of_words_that_every_fact_holds() {
    let facts: Vec<Vec<String>> = ["ottawa is the capital of canada", "mexico city is the capital of mexico", "abu dhabi is the capital of united arab emirates"]
        .iter()
        .map(|fact| fact.split(' ').map(str::to_string).collect())
        .collect();
    let form = fact_form(&facts).expect("the facts share a form");
    assert_eq!(form.middle, vec!["is", "the", "capital", "of"]);
    assert_eq!(form.line(&("Quito".to_string(), "Ecuador".to_string())), "Quito is the capital of Ecuador.");
    let unary: Vec<Vec<String>> = ["alex is a name", "mary is a name"].iter().map(|fact| fact.split(' ').map(str::to_string).collect()).collect();
    assert!(fact_form(&unary).is_none());
}
