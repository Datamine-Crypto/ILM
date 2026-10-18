use ilm_core::base::words::answer_words;
use ilm_core::cases::Case;
use ilm_core::tools::call::Learner;
use ilm_core::tools::explain::wording_joining::learn_joining;

const ACTIONS: &[(&str, &str)] = &[("hop", "X_HOP"), ("spin", "X_SPIN")];
const SIDES: &[(&str, &str)] = &[("", ""), ("up", "X_UP"), ("down", "X_DOWN")];
const REPEATS: &[(&str, usize)] = &[("", 1), ("again", 2), ("lots", 3)];

fn phrases() -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for (side, side_token) in SIDES {
        for (repeat, times) in REPEATS {
            for (action, token) in ACTIONS.iter().copied().chain([("face", "")]) {
                if action == "face" && side.is_empty() {
                    continue;
                }
                let once: Vec<String> = [*side_token, token].into_iter().filter(|token| !token.is_empty()).map(str::to_string).collect();
                let words: Vec<&str> = [action, side, repeat].into_iter().filter(|word| !word.is_empty()).collect();
                out.push((words.join(" "), (0..*times).flat_map(|_| once.clone()).collect()));
            }
        }
    }
    out
}

fn commands() -> Vec<Case> {
    let phrases = phrases();
    let case = |question: String, answer: Vec<String>| Case { sentences: Vec::new(), question, answer: answer.join(" ") };
    let mut out: Vec<Case> = phrases.iter().map(|(words, tokens)| case(words.clone(), tokens.clone())).collect();
    for (first, first_tokens) in &phrases {
        for (second, second_tokens) in &phrases {
            out.push(case(format!("{first} plus {second}"), [first_tokens.clone(), second_tokens.clone()].concat()));
            out.push(case(format!("{first} later {second}"), [second_tokens.clone(), first_tokens.clone()].concat()));
        }
    }
    out
}

#[test]
fn the_ways_of_joining_of_a_made_up_language_are_learned_as_lines() {
    let held = |case: &Case| case.question.starts_with("spin down lots later");
    let all = commands();
    let lessons: Vec<Case> = all.iter().filter(|case| !held(case)).cloned().collect();
    let joining = learn_joining(&lessons).expect("the learned lines read");
    assert_eq!(joining.proposed_right, lessons.len());
    for line in ["The meaning of hop is X_HOP.", "If A later B is a command and the actions of A are C and the actions of B are D then the actions of A later B are D C.", "If A lots is a part and the actions of A are B then the actions of A lots are B B B."] {
        assert!(joining.lines.iter().any(|written| written == line), "{line}");
    }
    let asked: Vec<&Case> = all.iter().filter(|case| held(case)).collect();
    assert!(!asked.is_empty());
    for case in asked {
        let said = joining.model.answer(&case.sentences, &case.question);
        assert_eq!(said.words, answer_words(&case.answer), "{}", case.question);
    }
}
