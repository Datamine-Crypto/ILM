use ilm_core::cases::Case;
use ilm_core::tools::call::{AnswerMark, Learner, Lessons};
use ilm_core::base::signs::sentences_of;
use ilm_core::tools::explain::costs::{chain_ended, learning_clock, close_costs, cost_scope, open_costs};
use ilm_core::tools::explain::learn::{Library, built_on, library_proof, marked_proof, model_for, model_of};
use std::sync::Arc;
use ilm_core::tools::explain::text::{place_letter, read_topic, topic_places};

const SIZES: &str = "If A fits inside B then B is bigger than A.
If A is bigger than B and B is bigger than C then A is bigger than C.
Is A bigger than B? Yes if A is bigger than B. No if B is bigger than A.";

const WAYS: &str = "If A is north of B then the way from B to A is north.
If A is east of B then the way from B to A is east.
If the way from A to B is C and the way from B to D is E then the route from A to D is C E.
How do you go from A to B? C if the route from A to B is C.";

const KINDS: &str = "If Aes are B and C is a A then C is B.
To answer \"True or false: C is B.\": True if C is B. False otherwise.";

const OWNERS: &str = "If A owns B then B belongs to A.
Who has B? A if B belongs to A.";

fn told(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|line| line.to_string()).collect()
}

fn model(name: &str, text: &str) -> Box<dyn Learner> {
    Box::new(model_of(vec![read_topic(name, text).expect("the topic reads")]))
}

#[test]
fn rules_chain_over_the_told_sentences() {
    let sizes = model("sizes", SIZES);
    let story = told(&["The pebble fits inside the cup.", "The cup is bigger than the bead.", "The jar is bigger than the cup."]);
    assert_eq!(sizes.answer(&story, "Is the jar bigger than the pebble?").words, vec!["yes"]);
    assert_eq!(sizes.answer(&story, "Is the pebble bigger than the jar?").words, vec!["no"]);
}

#[test]
fn no_answer_when_nothing_proves_one() {
    let sizes = model("sizes", SIZES);
    let story = told(&["The pebble fits inside the cup.", "The cup is bigger than the bead."]);
    let said = sizes.answer(&story, "Is the bead bigger than the pebble?");
    assert!(said.words.is_empty());
}

#[test]
fn an_answer_holds_words_of_the_facts() {
    let ways = model("ways", WAYS);
    let story = told(&["The hall is north of the yard.", "The shed is east of the hall."]);
    assert_eq!(ways.answer(&story, "How do you go from the yard to the shed?").words, vec!["north", "east"]);
}

#[test]
fn a_plural_ending_is_read_inside_a_word() {
    let kinds = model("kinds", KINDS);
    let story = told(&["Lumpuses are blue.", "Rex is a lumpus."]);
    assert_eq!(kinds.answer(&story, "True or false: Rex is blue.").words, vec!["true"]);
    assert_eq!(kinds.answer(&story, "True or false: Rex is red.").words, vec!["false"]);
}

#[test]
fn an_open_place_takes_no_marks() {
    let owners = model("owners", OWNERS);
    assert_eq!(owners.answer(&told(&["Ann owns the red kite."]), "Who has the red kite?").words, vec!["ann"]);
    assert!(owners.answer(&told(&["[Ann] owns the red kite."]), "Who has the red kite?").words.is_empty());
}

#[test]
fn the_note_names_the_topic_and_the_answer() {
    let sizes = model("sizes", SIZES);
    let story = told(&["The cup is bigger than the bead."]);
    let said = sizes.answer(&story, "Is the cup bigger than the bead?");
    assert!(said.note.starts_with("sizes:"));
}

#[test]
fn a_line_that_is_no_rule_no_question_and_no_fact_is_a_fault() {
    assert!(read_topic("odd", "Something with no full stop").is_err());
    assert_eq!(read_topic("plain", "Something that is told.").expect("a told fact reads").facts.len(), 1);
}

#[test]
fn a_capital_letter_that_is_no_open_place_in_a_rule_or_an_answer_is_a_fault() {
    assert!(read_topic("stray", "If A sees Q then A knows Q.").is_err());
    assert!(read_topic("stray", "Does A know B? Yes if A sees Q. No otherwise.").is_err());
    let asked = read_topic("asked", "To answer \"Q: Does A know B?\": Yes if A sees B. No otherwise.
If I see A then A is near me.").expect("the Q of a question and the pronoun I read");
    assert_eq!((asked.questions.len(), asked.rules.len()), (1, 1));
}

const OPPOSITES: &str = "Tall is the opposite of short.
Wide is the opposite of narrow.
If A is the opposite of B then B is the opposite of A.
Is A the opposite of B? Yes if A is the opposite of B. No otherwise.";

const DOLLARS: &str = "Two is the number 2.
Seven is the number 7.
Nine is the number 9.
If A has B dollars and B is the number C then A has C dollars.
If A got B more dollars and B is the number C then A got C more dollars.
To answer \"A now has B dollars\": true if A has C dollars and A got D more dollars and C plus D is E and B is the number E. false otherwise.";

#[test]
fn told_facts_of_the_library_hold_in_every_story() {
    let opposites = model("opposites", OPPOSITES);
    assert_eq!(opposites.answer(&[], "Is short the opposite of tall?").words, vec!["yes"]);
    assert_eq!(opposites.answer(&[], "Is wide the opposite of tall?").words, vec!["no"]);
    let dollars = model("dollars", DOLLARS);
    let story = told(&["Kip has two dollars.", "Kip got seven more dollars."]);
    assert_eq!(dollars.answer(&story, "Kip now has nine dollars").words, vec!["true"]);
    assert_eq!(dollars.answer(&story, "Kip now has seven dollars").words, vec!["false"]);
}

const ROUGH: &str = "Is A bigger than B? Yes if A is bigger than B. No otherwise.";

const EXACT: &str = "If A is bigger than B and B is bigger than C then A is bigger than C.
Is A bigger than B? Yes if A is bigger than B. No if B is bigger than A.";

#[test]
fn the_topic_with_the_largest_share_answers_first() {
    let story = told(&["The jar is bigger than the cup.", "The cup is bigger than the bead."]);
    let lesson = |question: &str, answer: &str| Case { sentences: story.clone(), question: question.to_string(), answer: answer.to_string() };
    let lessons: Vec<Case> = (0..6)
        .flat_map(|_| [lesson("Is the jar bigger than the bead?", "yes"), lesson("Is the bead bigger than the jar?", "no"), lesson("Is the jar bigger than the cup?", "yes")])
        .collect();
    let library = vec![read_topic("a_rough", ROUGH).expect("the topic reads"), read_topic("b_exact", EXACT).expect("the topic reads")];
    let model = model_for(Arc::new(Library::of(library).expect("the library builds")), &Lessons::of(&lessons), 2).expect("both topics fit");
    let said = model.answer(&story, "Is the jar bigger than the bead?");
    assert_eq!(said.words, vec!["yes"]);
    assert!(said.note.starts_with("b_exact:"));
}

#[test]
fn topics_that_tie_on_their_sample_are_judged_on_more_lessons() {
    let story = told(&["The jar is bigger than the cup.", "The cup is bigger than the bead."]);
    let lesson = |question: &str, answer: &str| Case { sentences: story.clone(), question: question.to_string(), answer: answer.to_string() };
    let lessons: Vec<Case> = std::iter::repeat_n(lesson("Is the jar bigger than the cup?", "yes"), 60).chain(std::iter::repeat_n(lesson("Is the jar bigger than the bead?", "yes"), 40)).collect();
    let library = vec![read_topic("a_rough", ROUGH).expect("the topic reads"), read_topic("b_exact", EXACT).expect("the topic reads")];
    let model = model_for(Arc::new(Library::of(library).expect("the library builds")), &Lessons::of(&lessons), 2).expect("both topics fit");
    let said = model.answer(&story, "Is the jar bigger than the bead?");
    assert_eq!(said.words, vec!["yes"]);
    assert!(said.note.starts_with("b_exact:"));
}

const BIRDS: &str = "If A is a penguin then A is flightless.
If A is a bird and not A is flightless then A can fly.
Can A fly? Yes if A can fly. No otherwise.";

const HOME: &str = "Is A at home? Yes if A came home and not A went out. No otherwise.";

#[test]
fn a_condition_with_not_holds_when_no_fact_fits() {
    let birds = model("birds", BIRDS);
    let story = told(&["Tweety is a bird.", "Pingu is a bird.", "Pingu is a penguin."]);
    assert_eq!(birds.answer(&story, "Can Tweety fly?").words, vec!["yes"]);
    assert_eq!(birds.answer(&story, "Can Pingu fly?").words, vec!["no"]);
    let home = model("home", HOME);
    assert_eq!(home.answer(&told(&["Ann came home."]), "Is Ann at home?").words, vec!["yes"]);
    assert_eq!(home.answer(&told(&["Bob came home.", "Bob went out."]), "Is Bob at home?").words, vec!["no"]);
}

const PLACES: &str = "If A moved to B then A is in B.
If A went back to B then A is in B.
Where is A? B if latest A is in B.
Is A in B? Yes if latest A is in B. No otherwise.";

#[test]
fn the_latest_fact_about_a_thing_wins() {
    let places = model("places", PLACES);
    let story = told(&["Ann moved to the hall.", "Bob moved to the yard.", "Ann moved to the shed.", "Ann went back to the hall.", "Bob moved to the hall."]);
    assert_eq!(places.answer(&story, "Where is Ann?").words, vec!["the", "hall"]);
    assert_eq!(places.answer(&story, "Where is Bob?").words, vec!["the", "hall"]);
    assert_eq!(places.answer(&story[..3], "Where is Ann?").words, vec!["the", "shed"]);
    assert_eq!(places.answer(&story, "Is Ann in the shed?").words, vec!["no"]);
    assert_eq!(places.answer(&story, "Is Ann in the hall?").words, vec!["yes"]);
    let again = told(&["Ann moved to the hall.", "Ann moved to the shed.", "Ann moved to the hall."]);
    assert_eq!(places.answer(&again, "Where is Ann?").words, vec!["the", "hall"]);
}

const COINS: &str = "If A had B coins and A got C coins and B plus C is D then A has D coins.
If A has B coins and C has D coins and B is more than D then A is richer than C.
Is A richer than B? Yes if A is richer than B. No otherwise.
How many coins does A have? B if A has B coins.";

const PUPILS: &str = "How many pupils does A teach? B if B is the number of C where C studies with A.
Does A teach at least B pupils? Yes if C is the number of D where D studies with A and C is at least B. No otherwise.";

#[test]
fn numbers_add_compare_and_count() {
    let coins = model("coins", COINS);
    let story = told(&["Kip had 4 coins.", "Kip got 7 coins.", "Lu had 9 coins.", "Lu got 1 coins."]);
    assert_eq!(coins.answer(&story, "How many coins does Kip have?").words, vec!["11"]);
    assert_eq!(coins.answer(&story, "Is Kip richer than Lu?").words, vec!["yes"]);
    assert_eq!(coins.answer(&story, "Is Lu richer than Kip?").words, vec!["no"]);
    let pupils = model("pupils", PUPILS);
    let story = told(&["Ada studies with Bo.", "Cy studies with Bo.", "Di studies with Ed.", "Ada studies with Bo."]);
    assert_eq!(pupils.answer(&story, "How many pupils does Bo teach?").words, vec!["2"]);
    assert_eq!(pupils.answer(&story, "Does Bo teach at least 2 pupils?").words, vec!["yes"]);
    assert_eq!(pupils.answer(&story, "Does Ed teach at least 2 pupils?").words, vec!["no"]);
}

const SCALES: &str = "If A is above B on C and B is above D on C then A is above D on C.";

const HEIGHTS: &str = "Builds on scales.
If A is taller than B then A is above B on height.
Is A taller than B? Yes if A is above B on height. No otherwise.";

#[test]
fn a_topic_uses_the_rules_of_the_topics_it_builds_on() {
    let library = vec![read_topic("scales", SCALES).expect("the topic reads"), read_topic("heights", HEIGHTS).expect("the topic reads")];
    assert_eq!(library[1].builds_on, vec!["scales"]);
    let whole = built_on(&library).expect("every topic it builds on is there");
    let heights = model_of(vec![whole[1].clone()]);
    let story = told(&["Kip is taller than Lu.", "Lu is taller than Mo."]);
    assert_eq!(heights.answer(&story, "Is Kip taller than Mo?").words, vec!["yes"]);
    let alone = model_of(vec![library[1].clone()]);
    assert_eq!(alone.answer(&story, "Is Kip taller than Mo?").words, vec!["no"]);
    assert!(built_on(&library[1..]).is_err());
}

const SCALE_WORDS: &str = "If A is B and C is Ber then C is above A on B.
If A is Best then A is at the top on B.
Is A above B on C? Yes if A is above B on C. No otherwise.
Who is at the top on A? B if B is at the top on A.
What is A doing? B if A is Bing.";

#[test]
fn endings_of_comparison_and_of_verbs_are_read_inside_a_word() {
    let scales = model("scale words", SCALE_WORDS);
    let story = told(&["Ann is tall.", "Bob is taller.", "Cy is big.", "Di is bigger.", "Ed is happy.", "Flo is happier.", "Gus is nice.", "Hal is nicer.", "Ivy is tallest.", "Jo is singing."]);
    assert_eq!(scales.answer(&story, "Is Bob above Ann on tall?").words, vec!["yes"]);
    assert_eq!(scales.answer(&story, "Is Di above Cy on big?").words, vec!["yes"]);
    assert_eq!(scales.answer(&story, "Is Flo above Ed on happy?").words, vec!["yes"]);
    assert_eq!(scales.answer(&story, "Is Hal above Gus on nice?").words, vec!["yes"]);
    assert_eq!(scales.answer(&story, "Is Ann above Bob on tall?").words, vec!["no"]);
    assert_eq!(scales.answer(&story, "Who is at the top on tall?").words, vec!["ivy"]);
    assert_eq!(scales.answer(&story, "What is Jo doing?").words, vec!["sing"]);
}

const NOTED: &str = "Note: a line for the reader? It holds a question mark and stays out.
Builds on: scales.
If A is taller than B then A is above B on height.
Is A taller than B? Yes if A is above B on height. No otherwise.";

const FLIGHT: &str = "Tweety is a bird.
If A is a bird and not A is flightless then A can fly.
If A is a penguin then A is flightless.
Can A fly? Yes if A can fly. No otherwise.";

const KNOWN_PLACES: &str = "The key is in the box.
Where is A? B if latest A is in B.";

#[test]
fn notes_are_skipped_and_known_facts_come_before_the_story() {
    let noted = read_topic("noted", NOTED).expect("the topic reads");
    assert_eq!((noted.questions.len(), noted.builds_on.clone(), noted.lines), (1, vec!["scales".to_string()], 3));
    let library = vec![read_topic("scales", SCALES).expect("the topic reads"), noted];
    let heights = model_of(vec![built_on(&library).expect("scales is there")[1].clone()]);
    assert_eq!(heights.answer(&told(&["Kip is taller than Lu.", "Lu is taller than Mo."]), "Is Kip taller than Mo?").words, vec!["yes"]);
    let flight = model("flight", FLIGHT);
    assert_eq!(flight.answer(&[], "Can Tweety fly?").words, vec!["yes"]);
    assert_eq!(flight.answer(&told(&["Tweety is a penguin."]), "Can Tweety fly?").words, vec!["no"]);
    let places = model("known places", KNOWN_PLACES);
    assert_eq!(places.answer(&[], "Where is the key?").words, vec!["the", "box"]);
    assert_eq!(places.answer(&told(&["The key is in the bag."]), "Where is the key?").words, vec!["the", "bag"]);
}

const MOVES: &str = "If A went to B then A is in B.
Reply to \"Where is A?\" with B: A is in B.
Reply to \"Is A in the B?\" with yes: Yes, A is in the B.";

const PLACES_SAID: &str = "Builds on: moves.
Where is A? B if latest A is in B.
Is A in the B? Yes if latest A is in the B. No otherwise.
Reply to \"Is A in the B?\" with no: No, A is not in the B.";

#[test]
fn a_reply_line_says_the_answer_as_a_sentence() {
    let library = vec![read_topic("moves", MOVES).expect("the topic reads"), read_topic("places said", PLACES_SAID).expect("the topic reads")];
    assert_eq!((library[0].reply_lines.len(), library[1].reply_lines.len()), (2, 1));
    let places = model_of(vec![built_on(&library).expect("moves is there")[1].clone()]);
    let story = told(&["Mary went to the kitchen.", "Mary went to the garden."]);
    let said = places.answer(&story, "Where is Mary?");
    assert_eq!(said.words, vec!["the", "garden"]);
    assert!(said.note.starts_with("Mary is in the garden. (places said:"), "{}", said.note);
    let said = places.answer(&story, "Is Mary in the kitchen?");
    assert_eq!((said.words, said.note.starts_with("No, Mary is not in the kitchen. (")), (vec!["no".to_string()], true));
    assert!(places.answer(&story, "Is Mary in the garden?").note.starts_with("Yes, Mary is in the garden. ("));
    assert!(read_topic("odd", "Reply to \"Where is A?\" B.").is_err());
}

const CARRYING: &str = "If A went to the B then the place of A is B.
If A got the B then the holder of the B is A.
If A dropped the B then the holder of the B is nobody.
If A got the B and at that time the place of A is C then the place of the B is C.
If A dropped the B and at that time the place of A is C then the place of the B is C.
If the holder of the B is A and A went to the C and at that time the holder of the B is A then the place of the B is C.
Where is the A? B if latest the place of the A is B.
If the holder of the B is A and A went to the C and at that time the holder of the B is A then the B came to C.
Where was the A before the B? C if the A came to B and before that time the place of the A is C.";

#[test]
fn at_that_time_reads_the_time_of_the_clause_before_it() {
    let carrying = model("carrying", CARRYING);
    let story = told(&["Mary went to the kitchen.", "Mary went to the office.", "Mary got the ball.", "Mary went to the garden.", "Mary dropped the ball.", "Mary went to the hall."]);
    assert_eq!(carrying.answer(&story, "Where is the ball?").words, vec!["garden"]);
    assert_eq!(carrying.answer(&story, "Where was the ball before the garden?").words, vec!["office"]);
    assert!(carrying.answer(&story, "Where was the ball before the office?").words.is_empty());
    let again = told(&["Mary went to the kitchen.", "Mary got the ball.", "Mary went to the garden.", "Mary went to the kitchen.", "Mary dropped the ball.", "Mary went to the hall."]);
    assert_eq!(carrying.answer(&again, "Where is the ball?").words, vec!["kitchen"]);
    assert_eq!(carrying.answer(&again, "Where was the ball before the garden?").words, vec!["kitchen"]);
    assert_eq!(carrying.answer(&again, "Where was the ball before the kitchen?").words, vec!["garden"]);
    let twice = told(&["Mary went to the kitchen.", "Mary got the ball.", "Mary went to the garden.", "Mary went to the kitchen.", "Mary went to the hall.", "Mary went to the kitchen."]);
    assert_eq!(carrying.answer(&twice, "Where was the ball before the kitchen?").words, vec!["hall"]);
}

const WATCHING: &str = "If A left the B then the room of A is outside.
If A came into the B then the room of A is B.
If A put the C in the D then the place of the C is D.
Where does A think that B looks for the C? D if latest the room of B is outside and at that time the room of A is outside and at that time the place of the C is D. D if latest the room of A is outside and at that time the room of B is outside and at that time the place of the C is D. D if latest the room of B is outside and at that time the place of the C is D. D if latest the room of A is outside and at that time the place of the C is D. D if latest the place of the C is D.";

#[test]
fn at_that_time_after_at_that_time_reads_the_older_time() {
    let watching = model("watching", WATCHING);
    let story = told(&["Ann came into the hall.", "Bob came into the hall.", "Ann put the key in the box.", "Ann left the hall.", "Bob put the key in the bag.", "Bob put the key in the cup.", "Bob left the hall."]);
    assert_eq!(watching.answer(&story, "Where does Bob think that Ann looks for the key?").words, vec!["box"]);
    assert_eq!(watching.answer(&story, "Where does Ann think that Bob looks for the key?").words, vec!["box"]);
    let stayed = told(&["Ann came into the hall.", "Bob came into the hall.", "Ann put the key in the box.", "Bob left the hall.", "Ann put the key in the bag."]);
    assert_eq!(watching.answer(&stayed, "Where does Ann think that Bob looks for the key?").words, vec!["box"]);
    assert_eq!(watching.answer(&stayed, "Where does Bob think that Ann looks for the key?").words, vec!["box"]);
}

const PLAIN_TRUTH: &str = "Is A B? Yes if A is B. No otherwise.";

#[test]
fn an_answer_that_holds_otherwise_waits_for_every_reading_of_the_question() {
    let truth = model("plain truth", PLAIN_TRUTH);
    let story = told(&["The bear is big."]);
    assert_eq!(truth.answer(&story, "Is the bear big?").words, vec!["yes"]);
    assert_eq!(truth.answer(&story, "Is the bear small?").words, vec!["no"]);
}

const JOINED: &str = "If D, A, E, C And B are from K, F, J, H And G respectively then B is from G.
If A's B is C then A has a B.
Where is B from? G if B is from G.
What is A? B if A is B.
Does A have a B? Yes if A has a B. No otherwise.
Is A in the B? Yes if A is in the B. No otherwise.";

#[test]
fn words_are_joined_and_parted_as_english_writes_them() {
    let joined = model("joined", JOINED);
    let story = told(&["Ann, Bob, Cy, Dee and Eve are from Lima, Oslo, Rome, Bern and Port-au-Prince respectively.", "Kim is self-assured.", "Kim\u{2019}s dog is brown."]);
    assert_eq!(joined.answer(&story, "Where is Eve from?").words, vec!["port", "-", "au", "-", "prince"]);
    assert_eq!(joined.answer(&story, "What is Kim?").words, vec!["self", "-", "assured"]);
    assert_eq!(joined.answer(&story, "Does Kim have a dog?").words, vec!["yes"]);
    assert_eq!(joined.answer(&told(&["Kim is in the green bottle."]), "Is Kim in the green_bottle?").words, vec!["yes"]);
    assert_eq!(joined.answer(&told(&["Kim is in the green_bottle."]), "Is Kim in the green bottle?").words, vec!["yes"]);
}

const LISTS: &str = "If A And B are C then A is C.
If A And B are C then B is C.
If A is R And W then A is R.
If A is R And W then A is W.
Is A B? Yes if A is B. No otherwise.";

#[test]
fn a_place_of_one_word_takes_no_phrase() {
    let lists = model("lists", LISTS);
    let story = told(&["Nick and Laura are from France and Canada respectively.", "Kate is calm and plain-speaking."]);
    assert_eq!(lists.answer(&story, "Is Laura from France?").words, vec!["no"]);
    assert_eq!(lists.answer(&story, "Is Kate plain-speaking?").words, vec!["yes"]);
    assert_eq!(lists.answer(&story, "Is Kate calm?").words, vec!["yes"]);
}

#[test]
fn the_places_are_named_by_their_letters() {
    use ilm_core::tools::explain::text::{OPEN_PLACES, place_letter};
    let letters: Vec<String> = (0..OPEN_PLACES).map(place_letter).collect();
    assert_eq!(letters.concat(), "ABCDEFGHJKLMNOPRSTUVWXYZAABBCCDDEEFFGGHHJJKKLLMMNNOOPPAAABBBCCCDDDEEEFFFGGGHHHJJJKKKLLLMMMNNNOOOPPPRRSSTTUUAAAABBBBCCCCDDDD");
}

const SIGNS: &str = "( is closed by ).
{ is closed by }.
Write AAAA RR SS BBBB as AAAA BBBB if RR is closed by SS.
Write AAAA RR SS as AAAA if RR is closed by SS.
Is it matched? yes if Sequence :. no if Sequence : AAAA.
What is left open? AAAA if Sequence : AAAA.
What is last? R if Sequence : AAAA R.";

#[test]
fn places_of_signs_hold_brackets_that_do_not_close() {
    let signs = model("signs", SIGNS);
    assert_eq!(signs.answer(&told(&["Sequence: ( { } ( ) )"]), "Is it matched?").words, vec!["yes"]);
    assert_eq!(signs.answer(&told(&["Sequence: ( { ) }"]), "Is it matched?").words, vec!["no"]);
    assert_eq!(signs.answer(&told(&["Sequence: ( ( { } x"]), "What is left open?").words, vec!["(", "(", "x"]);
    assert!(signs.answer(&told(&["Sequence: x ( ("]), "What is last?").words.is_empty());
    assert_eq!(signs.answer(&told(&["Sequence: ( ( x"]), "What is last?").words, vec!["x"]);
}

const SAID_WITH_MARKS: &str = "If AA said that BB then AA told BB.
What did AA tell? BB if AA told BB.";

#[test]
fn a_place_written_twice_holds_marks_but_not_the_brackets_of_a_name() {
    let said = model("said", SAID_WITH_MARKS);
    let story = told(&["Kip, the tall one, said that rain, sadly, came.", "(Lu) said that Mo's hat fell."]);
    assert_eq!(said.answer(&story, "What did Kip, the tall one, tell?").words, vec!["rain", "sadly", "came"]);
    assert!(said.answer(&story, "What did (Lu) tell?").words.is_empty());
    let plain = model("plain", "If A said that B then A told B.\nWhat did A tell? B if A told B.");
    assert!(plain.answer(&story, "What did Kip, the tall one, tell?").words.is_empty());
}

const SEEING: &str = "If A came into the B then the room of A is B.
If A left the B then the room of A is outside.
If the room of A is B then A is a person.
If A put the C in the D in the B then the C is in D in the room B.
If D is a person and the C is in B in the room R and at that time the room of D is R then D saw the C in the B.
If D came into the R and at that time the C is in B in the room R then D saw the C in the B.
Where does D look for the C? B if latest D saw the C in the B.";

#[test]
fn an_outcome_at_that_time_keeps_the_time_of_the_clause_it_points_to() {
    let seeing = model("seeing", SEEING);
    let story = told(&["Ann came into the hall.", "Bob came into the hall.", "Ann put the key in the box in the hall.", "Bob left the hall.", "Ann put the key in the bag in the hall.", "Bob came into the hall."]);
    assert_eq!(seeing.answer(&story, "Where does Bob look for the key?").words, vec!["bag"]);
    assert_eq!(seeing.answer(&story, "Where does Ann look for the key?").words, vec!["bag"]);
    let away = told(&["Ann came into the hall.", "Bob came into the hall.", "Ann put the key in the box in the hall.", "Bob left the hall.", "Ann put the key in the bag in the hall.", "Bob came into the yard."]);
    assert_eq!(seeing.answer(&away, "Where does Bob look for the key?").words, vec!["box"]);
}

const DIFFERENT: &str = "If A is next to B and not A is the same as B then A stands apart from B.
If A has B coins and A has C coins and not B is at least C then A has two amounts.
Does A stand apart from B? Yes if A stands apart from B. No otherwise.
Does A have two amounts? Yes if A has two amounts. No otherwise.";

#[test]
fn not_before_a_worked_out_condition_holds_when_it_does_not_hold() {
    let different = model("different", DIFFERENT);
    let story = told(&["Kip is next to Lu.", "Mo is next to Mo.", "Kip has 3 coins.", "Lu has 3 coins.", "Lu has 4 coins."]);
    assert_eq!(different.answer(&story, "Does Kip stand apart from Lu?").words, vec!["yes"]);
    assert_eq!(different.answer(&story, "Does Mo stand apart from Mo?").words, vec!["no"]);
    assert_eq!(different.answer(&story, "Does Kip have two amounts?").words, vec!["no"]);
    assert_eq!(different.answer(&story, "Does Lu have two amounts?").words, vec!["yes"]);
}

const ASKED_HERE: &str = "If the question is Is there a A here then A is wanted.
If the A sits here then A is near.
Is there a A here? Yes if A is wanted and A is near. No otherwise.";

const SAID: &str = "If A is B then A was said.
Was A said? Yes if A was said. No otherwise.";

#[test]
fn the_question_is_told_only_to_a_topic_whose_rules_read_it() {
    let asked = model("asked here", ASKED_HERE);
    let story = told(&["The cat sits here."]);
    assert_eq!(asked.answer(&story, "Is there a cat here?").words, vec!["yes"]);
    assert_eq!(asked.answer(&story, "Is there a dog here?").words, vec!["no"]);
    let said = model("said", SAID);
    assert_eq!(said.answer(&told(&["The sky is blue."]), "Was the question said?").words, vec!["no"]);
    assert_eq!(said.answer(&told(&["The sky is blue."]), "Was the sky said?").words, vec!["yes"]);
}

const OWNED: &str = "If X owns a A and C is the number of B where X owns a B then X owns C things.
If the D of W is E and C is the number of B where W has a B then W has C parts.
How many things does X own? C if X owns C things.
How many parts does W have? C if W has C parts.";

#[test]
fn a_count_is_read_after_the_conditions_that_fill_its_places() {
    let owned = model("owned", OWNED);
    let story = told(&["Ann owns a cat.", "Ann owns a dog.", "Bob owns a fox.", "The colour of Kip is red.", "Kip has a wheel.", "Lu has a door."]);
    assert_eq!(owned.answer(&story, "How many things does Ann own?").words, vec!["2"]);
    assert_eq!(owned.answer(&story, "How many things does Bob own?").words, vec!["1"]);
    assert_eq!(owned.answer(&story, "How many parts does Kip have?").words, vec!["1"]);
}

const KEPT: &str = "If A went to the B then the place of A is B.
If A got the B and at that time the place of A is C then the place of the B is C.
If A got the B and not the place of the B is hall then the B is lost.
If C is the number of B where the place of the B is hall then the hall holds C things.
Is the B lost? Yes if the B is lost. No otherwise.
How many things does the hall hold? C if the hall holds C things.";

#[test]
fn not_and_counts_wait_for_the_rules_that_read_facts_by_their_time() {
    let kept = model("kept", KEPT);
    let story = told(&["Ann went to the hall.", "Ann got the ball.", "Ann got the cup."]);
    assert_eq!(kept.answer(&story, "Is the ball lost?").words, vec!["no"]);
    assert_eq!(kept.answer(&story, "How many things does the hall hold?").words, vec!["2"]);
}

const NAMED: &str = "Big is a size.
Small is a size.
Blue is a colour.
Black is a colour.
Square is a shape.
Circle is a shape.
A is a determiner.
Write A Y S T U B as A S T U number one B if Y is a determiner and S is a size and T is a colour and U is a shape.
Write A Y S T U as A S T U number one if Y is a determiner and S is a size and T is a colour and U is a shape.
Does the story say A? Yes if A. No otherwise.";

#[test]
fn a_line_that_writes_a_sentence_again_keeps_only_the_new_form() {
    let named = model("named", NAMED);
    let story = told(&["Block A contains a big blue square and a small black circle."]);
    assert_eq!(named.answer(&story, "Does the story say block A contains big blue square number one and small black circle number one?").words, vec!["yes"]);
    assert_eq!(named.answer(&story, "Does the story say block A contains a big blue square and a small black circle?").words, vec!["no"]);
    assert_eq!(named.answer(&story, "Does the story say block A contains big blue square number one and a small black circle?").words, vec!["no"]);
    assert!(read_topic("stray", "Write A sees Q as Q sees A.").is_err());
}

const TOLD_RULES: &str = "Read told rules with something, someone, it and they for any thing.
If all A things are B then if something is A then it is B.
To answer \"True or false: A.\": True if A. False otherwise.";

const NO_TOLD_RULES: &str = "To answer \"True or false: A.\": True if A. False otherwise.";

#[test]
fn a_topic_that_reads_told_rules_chains_the_rules_of_the_story() {
    let told_rules = model("told rules", TOLD_RULES);
    let story = told(&["The fox is kind.", "If something is kind then it is calm.", "If someone is calm and it sees the owl then the owl is glad.", "The fox sees the owl.", "If something is glad and not it is calm then it sings.", "All green things are round.", "The elk is green."]);
    assert_eq!(told_rules.answer(&story, "True or false: the fox is calm.").words, vec!["true"]);
    assert_eq!(told_rules.answer(&story, "True or false: the owl is glad.").words, vec!["true"]);
    assert_eq!(told_rules.answer(&story, "True or false: the owl sings.").words, vec!["true"]);
    assert_eq!(told_rules.answer(&story, "True or false: the fox sings.").words, vec!["false"]);
    assert_eq!(told_rules.answer(&story, "True or false: the owl is calm.").words, vec!["false"]);
    assert_eq!(told_rules.answer(&story, "True or false: the elk is round.").words, vec!["true"]);
    let plain = model("no told rules", NO_TOLD_RULES);
    assert_eq!(plain.answer(&story, "True or false: the fox is calm.").words, vec!["false"]);
}

const TOTALS: &str = "How many cakes were served? N if N is the sum of B where A served B cakes.
How many cakes were served by A? N if N is the sum of B where A served B cakes.";

#[test]
fn a_total_adds_the_numbers_of_every_fact_of_a_form() {
    let totals = model("totals", TOTALS);
    let story = told(&["Kip served 3 cakes.", "Lu served 4 cakes.", "Kip served 5 cakes.", "Mo served 3 cakes."]);
    assert_eq!(totals.answer(&story, "How many cakes were served?").words, vec!["15"]);
    assert_eq!(totals.answer(&story, "How many cakes were served by Kip?").words, vec!["8"]);
    assert_eq!(totals.answer(&story, "How many cakes were served by Bo?").words, vec!["0"]);
    let none = told(&["Kip served many cakes."]);
    assert!(totals.answer(&none, "How many cakes were served by Kip?").words.is_empty());
}

const FLOWS: &str = "If an edge from node A to node B has capacity C then A goes to B with capacity C.
What is the maximum flow from node A to node B? N if N is the flow from A to B where C goes to D with capacity E.";

#[test]
fn a_flow_is_the_largest_flow_over_the_told_edges() {
    let flows = model("flows", FLOWS);
    let story = told(&["An edge from node 0 to node 1 has capacity 3.", "An edge from node 1 to node 2 has capacity 2.", "An edge from node 0 to node 2 has capacity 1.", "An edge from node 1 to node 3 has capacity 4.", "An edge from node 3 to node 2 has capacity 5."]);
    assert_eq!(flows.answer(&story, "What is the maximum flow from node 0 to node 2?").words, vec!["4"]);
    assert_eq!(flows.answer(&story, "What is the maximum flow from node 2 to node 0?").words, vec!["0"]);
    assert_eq!(flows.answer(&story, "What is the maximum flow from node 1 to node 2?").words, vec!["6"]);
    assert_eq!(flows.answer(&story, "What is the maximum flow from node 4 to node 2?").words, vec!["0"]);
}

const LAYERED_NOT: &str = "If A is cold and not A is round then A is smart.
If A is big and not A is red then A is round.
Is A smart? Yes if A is smart. No otherwise.";

const CYCLE_NOT: &str = "If A is a bird and not A is a penguin then A flies.
If A swims and not A flies then A is a penguin.";

#[test]
fn a_rule_with_not_waits_for_the_rules_that_can_make_what_it_reads_as_absent() {
    use ilm_core::tools::explain::chain::not_strata;
    let layered = model("layered not", LAYERED_NOT);
    let story = told(&["Kip is cold.", "Kip is big.", "Lu is cold."]);
    assert_eq!(layered.answer(&story, "Is Kip smart?").words, vec!["no"]);
    assert_eq!(layered.answer(&story, "Is Lu smart?").words, vec!["yes"]);
    let rules = read_topic("layered not", LAYERED_NOT).expect("the topic reads").rules;
    assert_eq!(not_strata(&rules), (vec![1, 0], false));
    let cycle = read_topic("cycle not", CYCLE_NOT).expect("the topic reads").rules;
    assert_eq!(not_strata(&cycle), (vec![0, 0], true));
}

const ANNOUNCED: &str = "To answer \"A\": entailment if it follows that A. not_entailment otherwise.";

#[test]
fn a_statement_follows_when_it_holds_in_every_world_the_announcements_leave() {
    let worlds = model("announced", ANNOUNCED);
    let three = told(&["Ann is a person.", "Bo is a person.", "Cy is a person.", "Every person sees the facts of the others.", "It is announced that some person holds."]);
    assert_eq!(worlds.answer(&three, "Ann knows whether Ann holds").words, vec!["not", "entailment"]);
    assert_eq!(worlds.answer(&three, "Ann knows whether some person holds").words, vec!["entailment"]);
    let mut more = three.clone();
    more.push("It is announced that Ann does not know whether Ann holds.".to_string());
    assert_eq!(worlds.answer(&more, "Ann does not know whether Ann holds").words, vec!["entailment"]);
    assert_eq!(worlds.answer(&more, "Bo knows whether Bo holds").words, vec!["not", "entailment"]);
    let two = told(&["Ann is a person.", "Bo is a person.", "Every person sees the facts of the others.", "It is announced that some person holds.", "It is announced that Ann does not know whether Ann holds."]);
    assert_eq!(worlds.answer(&two, "Bo holds").words, vec!["entailment"]);
    assert_eq!(worlds.answer(&two, "Bo knows that Bo holds").words, vec!["entailment"]);
    assert_eq!(worlds.answer(&two, "Ann holds").words, vec!["not", "entailment"]);
    let cards = told(&["Ann is a person.", "Bo is a person.", "Every person sees the person's own fact.", "Ann sees the fact of Bo."]);
    assert_eq!(worlds.answer(&cards, "Ann knows whether Bo holds").words, vec!["entailment"]);
    assert_eq!(worlds.answer(&cards, "Bo knows whether Ann holds").words, vec!["not", "entailment"]);
    assert_eq!(worlds.answer(&cards, "Bo knows whether Ann knows whether Bo holds").words, vec!["entailment"]);
}

const KINDS_OF_WORDS: &str = "red is a colour word.
blue is a colour word.
Is AA plain? yes if AA is told and not AA holds a colour word. no if AA is told and AA holds a colour word.
What is B backward? C if C is B backward.";

#[test]
fn a_run_holds_a_word_of_a_kind_and_reads_backward() {
    let kinds = model("kinds of words", KINDS_OF_WORDS);
    assert_eq!(kinds.answer(&told(&["the car is fast"]), "Is the car is fast plain?").words, vec!["yes"]);
    assert_eq!(kinds.answer(&told(&["the car is red"]), "Is the car is red plain?").words, vec!["no"]);
    assert_eq!(kinds.answer(&[], "What is up and down backward?").words, vec!["down", "and", "up"]);
}

const LIST_FUNCTIONS: &str = "Write the question is AAAA as the value is AAAA , end.
Write AAAA V B , CCCC as AAAA B B , CCCC if V says a list twice and not B holds a function word.
Write AAAA V B , CCCC as AAAA C , CCCC if V says a list backward and not B holds a function word and C is B backward.
Write AAAA V B , C , CCCC as AAAA B C , CCCC if V joins two lists and not B holds a function word and not C holds a function word.";

#[test]
fn the_meanings_of_function_words_are_learned_over_a_text_of_list_functions() {
    use ilm_core::tools::explain::wording_meanings::meanings_topic;
    use ilm_core::tools::explain::learn::model_of;
    let functions = read_topic("list functions", LIST_FUNCTIONS).expect("the text reads");
    let lesson = |question: &str, answer: &str| Case { sentences: Vec::new(), question: question.to_string(), answer: answer.to_string() };
    let lessons = vec![lesson("dup a b", "a b a b"), lesson("flip a b c", "c b a"), lesson("tie_up a b , c", "a b c"), lesson("dup flip a b", "b a b a"), lesson("flip tie_up a , b c", "c b a")];
    let cases: Vec<&Case> = lessons.iter().collect();
    let meant = model_of(vec![meanings_topic(&[functions.clone()], &cases).expect("meanings are learned")]);
    assert_eq!(meant.answer(&[], "flip dup x y").words, vec!["y", "x", "y", "x"]);
    assert_eq!(meant.answer(&[], "tie_up flip x y , dup z").words, vec!["y", "x", "z", "z"]);
    let told_lessons = vec![Case { sentences: vec!["a story".to_string()], ..lesson("dup a b", "a b a b") }];
    assert!(meanings_topic(&[functions], &told_lessons.iter().collect::<Vec<&Case>>()).is_none());
}

const LESSON_MEANINGS: &str = "Read the lessons as the answers of their questions.
If the answer of R is B then the meaning of R is B.
If the answer of R twice is B B then the meaning of R is B.
If the question is R twice then R twice is asked.
To answer \"R twice\": B B if R twice is asked and the meaning of R is B.";

const NO_LESSON_MEANINGS: &str = "If the answer of R is B then the meaning of R is B.
If the question is R twice then R twice is asked.
To answer \"R twice\": B B if R twice is asked and the meaning of R is B.";

#[test]
fn a_topic_that_reads_the_lessons_takes_the_meanings_of_words_from_them() {
    let lesson = |question: &str, answer: &str| Case { sentences: Vec::new(), question: question.to_string(), answer: answer.to_string() };
    let lessons: Vec<Case> = std::iter::repeat_n([lesson("hop twice", "up up"), lesson("sit", "down"), lesson("hop twice", "up up")], 4).flatten().collect();
    let reading = model_for(Arc::new(Library::of(vec![read_topic("lesson meanings", LESSON_MEANINGS).expect("the topic reads")]).expect("the library builds")), &Lessons::of(&lessons), 2).expect("the topic fits");
    assert_eq!(reading.answer(&[], "sit twice").words, vec!["down", "down"]);
    assert_eq!(reading.answer(&[], "hop twice").words, vec!["up", "up"]);
    assert!(reading.answer(&[], "run twice").words.is_empty());
    let deaf = model_for(Arc::new(Library::of(vec![read_topic("no lesson meanings", NO_LESSON_MEANINGS).expect("the topic reads")]).expect("the library builds")), &Lessons::of(&lessons), 2);
    assert!(deaf.is_none());
}

#[test]
fn what_a_run_keeps_for_a_set_of_known_facts_follows_the_set_and_never_its_place_in_memory() {
    use ilm_core::tools::explain::chain::{chain_facts, knowledge_base};
    use ilm_core::tools::explain::text::fact_numbers;
    use ilm_core::tools::explain::wording::known_holders;
    use std::collections::HashSet;
    let rooms = |place: &str| read_topic("rooms", &format!("The {place} is a room.\nIf A went to the B and the B is a room then A is in the B.")).expect("the text reads");
    let kitchen = fact_numbers("kitchen")[0];
    let mut numbers: HashSet<u64> = HashSet::new();
    for round in 0..300 {
        let place = if round % 2 == 0 { "kitchen" } else { "garden" };
        let topic = rooms(place);
        let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
        assert!(numbers.insert(base.id()), "round {round}: every set of facts has its own number");
        assert_eq!(known_holders(&base).contains_key(&kitchen), place == "kitchen", "round {round}: the holders of a set made after another was let go");
    }
    let told = ["Ann went to the kitchen.".to_string()];
    for round in 0..40 {
        let place = if round % 2 == 0 { "kitchen" } else { "garden" };
        let topic = rooms(place);
        let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
        let facts = chain_facts(&told, &base, &topic.rules, topic.rules_key);
        assert_eq!(facts.holds(&fact_numbers("Ann is in the kitchen.")), place == "kitchen", "round {round}: a kept chain of one set is not given for another");
    }
}

#[test]
fn a_told_sentence_is_told_and_a_made_fact_is_not() {
    use ilm_core::tools::explain::chain::{chain_facts, knowledge_base};
    use ilm_core::tools::explain::text::fact_numbers;
    let topic = read_topic("made", "If A is tall then A is big.").expect("a rule reads");
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
    let facts = chain_facts(&["Ann is tall.".to_string()], &base, &topic.rules, topic.rules_key);
    assert!(facts.is_told(&fact_numbers("Ann is tall.")));
    assert!(facts.holds(&fact_numbers("Ann is big.")));
    assert!(!facts.is_told(&fact_numbers("Ann is big.")));
    let again = chain_facts(&["Ann is tall.".to_string()], &base, &topic.rules, topic.rules_key);
    assert!(again.is_told(&fact_numbers("Ann is tall.")));
    assert!(!again.is_told(&fact_numbers("Ann is big.")));
}

#[test]
fn a_learned_rule_fills_its_places_with_different_things_and_the_same_line_of_the_library_need_not() {
    use ilm_core::tools::explain::chain::{chain_facts, empty_filling, knowledge_base, rules_key};
    use ilm_core::tools::explain::derive::{answer_proof, proof_lines};
    use ilm_core::tools::explain::text::fact_numbers;
    let given = read_topic("siblings", "If A is a parent of B and A is a parent of C then B is a sibling of C.
If B is a sibling of C then C knows B.").expect("the rules read");
    let mut learned = given.rules.clone();
    for rule in &mut learned {
        rule.learned = true;
    }
    let learned_key = rules_key(&learned);
    assert_ne!(learned_key, given.rules_key, "the mark is in the key of the rules, so the chains of the two are kept apart");
    let story = told(&["Ann is a parent of Bob.", "Ann is a parent of Cid."]);
    let base = Arc::new(knowledge_base(&given.facts, &given.rules, false));
    let written = chain_facts(&story, &base, &given.rules, given.rules_key);
    assert!(written.holds(&fact_numbers("Bob is a sibling of Cid.")));
    assert!(written.holds(&fact_numbers("Bob is a sibling of Bob.")), "a line of the library may fill two places with one thing");
    let learned_base = Arc::new(knowledge_base(&given.facts, &learned, false));
    let made = chain_facts(&story, &learned_base, &learned, learned_key);
    assert!(made.holds(&fact_numbers("Bob is a sibling of Cid.")));
    assert!(made.holds(&fact_numbers("Cid is a sibling of Bob.")));
    assert!(made.holds(&fact_numbers("Cid knows Bob.")));
    assert!(!made.holds(&fact_numbers("Bob is a sibling of Bob.")), "a learned rule makes no child its own sibling");
    assert!(!made.holds(&fact_numbers("Bob knows Bob.")));
    let proof = proof_lines(&answer_proof(&learned, &made, given.facts.len(), &learned[1].conditions, &learned[1].absent, &empty_filling()));
    assert!(!proof.is_empty());
    assert!(proof.iter().all(|line| !line.contains("bob is a sibling of bob") && !line.contains("cid is a sibling of cid")), "{proof:?}");
}

const WORD_PLACES: &str = "Kip is a name.
Kind is a quality.
Calm is a quality.
If GG and GG is told and GG holds the word X and X is a name and GG holds the word S and S is a quality then X is S.
If GG and GG is told and GG holds the word S and S is a quality and T comes after S in GG and T is a quality then T comes after S.
Is X S? yes if X is S. no otherwise.
Does T come after S? yes if T comes after S. no otherwise.";

#[test]
fn a_condition_reads_the_words_of_a_sentence_by_their_places() {
    let places = model("word places", WORD_PLACES);
    let story = told(&["Kip, the tall one, is surely kind and calm."]);
    assert_eq!(places.answer(&story, "Is Kip kind?").words, vec!["yes"]);
    assert_eq!(places.answer(&story, "Is Kip calm?").words, vec!["yes"]);
    assert_eq!(places.answer(&story, "Is Kip tall?").words, vec!["no"]);
    assert_eq!(places.answer(&story, "Does calm come after kind?").words, vec!["yes"]);
    assert_eq!(places.answer(&story, "Does kind come after calm?").words, vec!["no"]);
}

const FLOUR: &str = "How much flour was used? N if N is the sum of B where A used B cups of flour.
What is A plus B? C if A plus B is C.";

#[test]
fn a_number_with_a_point_is_one_word_and_an_exact_amount() {
    let flour = model("flour", FLOUR);
    let story = told(&["Kip used 0.25 cups of flour.", "Lu used 3.5 cups of flour.", "Mo used 2 cups of flour."]);
    assert_eq!(flour.answer(&story, "How much flour was used?").words, ilm_core::base::words::answer_words("5.75"));
    assert_eq!(flour.answer(&[], "What is 0.5 plus 0.25?").words, ilm_core::base::words::answer_words("0.75"));
    assert_eq!(ilm_core::tools::explain::text::text_words("At 7.30 the end."), vec!["At", "7.30", "the", "end", "."]);
}

const CLASSICAL: &str = "To answer \"AA\": True if it is proved that AA. False if it is disproved that AA. Unknown otherwise.";

#[test]
fn a_learned_rule_of_one_condition_fills_two_places_with_one_thing_when_its_sentence_names_it_twice() {
    use ilm_core::tools::explain::chain::{chain_facts, knowledge_base, rules_key};
    use ilm_core::tools::explain::text::fact_numbers;
    let given = read_topic("either", "If A is either in the B or the C then the place of A is B or the C.").expect("the rule reads");
    let mut learned = given.rules.clone();
    for rule in &mut learned {
        rule.learned = true;
    }
    let learned_key = rules_key(&learned);
    let base = Arc::new(knowledge_base(&given.facts, &learned, false));
    let made = chain_facts(&told(&["Fred is either in the kitchen or the kitchen.", "Ann is either in the hall or the yard."]), &base, &learned, learned_key);
    assert!(made.holds(&fact_numbers("The place of fred is kitchen or the kitchen.")), "the sentence names one place twice, and the rule says it again");
    assert!(made.holds(&fact_numbers("The place of ann is hall or the yard.")));
}

#[test]
fn told_clauses_are_read_in_every_direction_and_a_question_by_refutation() {
    let logic = model("classical", CLASSICAL);
    let truth = |story: &[&str], question: &str| logic.answer(&told(story), question).words;
    let rule = ["It holds that if (the cat is red) then (the dog is blue).", "It is false that (the dog is blue)."];
    assert_eq!(truth(&rule, "the cat is red"), vec!["false"]);
    assert_eq!(truth(&rule, "it is false that the cat is red"), vec!["true"]);
    assert_eq!(truth(&rule, "the owl is green"), vec!["unknown"]);
    let either = ["It holds that (the cat is red) or (the dog is blue).", "It is false that (the cat is red)."];
    assert_eq!(truth(&either, "the dog is blue"), vec!["true"]);
    let not_both = ["It holds that it is false that ((the cat is red) and (the dog is blue)).", "The cat is red."];
    assert_eq!(truth(&not_both, "the dog is blue"), vec!["false"]);
    let any = ["Kip is a thing.", "Lu is a thing.", "It holds that for any thing x , if (x is big) then (x is red).", "Kip is big."];
    assert_eq!(truth(&any, "kip is red"), vec!["true"]);
    assert_eq!(truth(&any, "lu is red"), vec!["unknown"]);
}

const LETTERS: &str = "If R plays with S and the first letter of R is T and the first letter of S is T then R And S share a letter.
Do R And S share a letter? yes if R And S share a letter. no otherwise.
What is the first letter of R? T if the first letter of R is T.";

#[test]
fn a_condition_reads_the_first_letter_of_a_word() {
    let letters = model("letters", LETTERS);
    let story = told(&["Kip plays with Kim.", "Lu plays with Mo."]);
    assert_eq!(letters.answer(&story, "Do Kip and Kim share a letter?").words, vec!["yes"]);
    assert_eq!(letters.answer(&story, "Do Lu and Mo share a letter?").words, vec!["no"]);
    assert_eq!(letters.answer(&[], "What is the first letter of Kip?").words, vec!["k"]);
}

const PLACED_LETTERS: &str = "If R is a code and letter N of R is L and N plus 1 is M and not letter M of R is K then R has N letters.
What is letter N of R? L if letter N of R is L.
How many letters has R? N if R has N letters.
Where does R hold L? N if letter N of R is L.";

#[test]
fn a_condition_reads_the_letters_of_a_word_and_the_digits_of_a_number_by_their_places() {
    let letters = model("placed letters", PLACED_LETTERS);
    assert_eq!(letters.answer(&[], "What is letter 2 of Kip?").words, vec!["i"]);
    assert_eq!(letters.answer(&[], "What is letter 4 of Kip?").words, Vec::<String>::new());
    assert_eq!(letters.answer(&[], "What is letter 3 of 2026?").words, vec!["2"]);
    assert_eq!(letters.answer(&[], "Where does Kip hold p?").words, vec!["3"]);
    assert_eq!(letters.answer(&told(&["Abcde is a code."]), "How many letters has Abcde?").words, vec!["5"]);
}

const DIVISIONS: &str = "What is A divided by B? C if A divided by B is C.
What is A divided by B and then times B? D if A divided by B is C and C times B is D.
If Pair R S and R divided by S is C and C is less than 0.5 then the pair R S is small.
Is the pair R S small? yes if the pair R S is small. no otherwise.";

#[test]
fn a_division_is_exact_and_an_amount_with_no_end_of_its_decimal_is_one_word() {
    let divisions = model("divisions", DIVISIONS);
    assert_eq!(divisions.answer(&[], "What is 1 divided by 4?").words, ilm_core::base::words::answer_words("0.25"));
    assert_eq!(divisions.answer(&[], "What is 1 divided by 3?").words, ilm_core::base::words::answer_words("1/3"));
    assert_eq!(divisions.answer(&[], "What is -2 divided by 6?").words, ilm_core::base::words::answer_words("-1/3"));
    assert_eq!(divisions.answer(&[], "What is 2 divided by 0?").words, Vec::<String>::new());
    assert_eq!(divisions.answer(&[], "What is 1 divided by 3 and then times 3?").words, vec!["1"]);
    assert_eq!(divisions.answer(&told(&["Pair 1 3."]), "Is the pair 1 3 small?").words, vec!["yes"]);
    assert_eq!(divisions.answer(&told(&["Pair 2 3."]), "Is the pair 2 3 small?").words, vec!["no"]);
}

const PRODUCTS: &str = "If Big R S and R times S is C then the product of R and S is C.
If Big R S and R divided by S is C then the share of R and S is C.";

#[test]
fn a_step_whose_result_leaves_the_width_stops_the_chain_at_a_bound() {
    use ilm_core::tools::explain::chain::{chain_facts, knowledge_base};
    let topic = read_topic("products", PRODUCTS).expect("the topic reads");
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
    let wide = told(&["Big 100000000000000000000 100000000000000000000."]);
    assert!(chain_facts(&wide, &base, &topic.rules, topic.rules_key).stopped_at_bound());
    let small = told(&["Big 6 4."]);
    assert!(!chain_facts(&small, &base, &topic.rules, topic.rules_key).stopped_at_bound());
    let by_nought = told(&["Big 6 0."]);
    assert!(!chain_facts(&by_nought, &base, &topic.rules, topic.rules_key).stopped_at_bound());
}

const GROWING: &str = "If the T count N then the T count N more.
How many Bs do I have? N if the B count N and not N holds the word more.";

#[test]
fn a_rule_whose_outcome_holds_its_own_condition_with_more_words_stops_soon() {
    use ilm_core::tools::explain::chain::{chain_facts, knowledge_base};
    let topic = read_topic("growing", GROWING).expect("the topic reads");
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
    let facts = chain_facts(&told(&["the apple count 1."]), &base, &topic.rules, topic.rules_key);
    assert!(facts.stopped_at_bound());
    assert!(facts.added_to(&base) < 5_000);
    let growing = model("growing", GROWING);
    assert_eq!(growing.answer(&told(&["the pear count 2."]), "How many pears do I have?").words, vec!["2"]);
    assert!(growing.bound_stops() > 0);
}

const WHOLE_PARTS: &str = "What is the whole part of A? B if the whole part of A is B.
What is A over B, rounded down? D if A divided by B is C and the whole part of C is D.
What is left of A by B? F if A divided by B is C and the whole part of C is D and D times B is E and A minus E is F.";

#[test]
fn the_whole_part_of_a_number_rounds_down_on_both_sides_of_nought() {
    use ilm_core::base::words::answer_words;
    let parts = model("whole parts", WHOLE_PARTS);
    assert_eq!(parts.answer(&[], "What is the whole part of 2.5?").words, answer_words("2"));
    assert_eq!(parts.answer(&[], "What is the whole part of -2.5?").words, answer_words("-3"));
    assert_eq!(parts.answer(&[], "What is the whole part of 7?").words, answer_words("7"));
    assert_eq!(parts.answer(&[], "What is the whole part of -7?").words, answer_words("-7"));
    assert_eq!(parts.answer(&[], "What is 1 over 3, rounded down?").words, answer_words("0"));
    assert_eq!(parts.answer(&[], "What is -7 over 2, rounded down?").words, answer_words("-4"));
    assert_eq!(parts.answer(&[], "What is left of 17 by 5?").words, answer_words("2"));
    assert_eq!(parts.answer(&[], "What is left of -17 by 5?").words, answer_words("3"));
}

#[test]
fn some_thing_names_a_witness_and_grounding_stops_at_its_bound() {
    let logic = model("classical", CLASSICAL);
    let some = told(&["It holds that for some thing x , (x is big).", "It holds that for any thing x , if (x is big) then (x is red)."]);
    assert_eq!(logic.answer(&some, "for some thing x , x is red").words, vec!["true"]);
    assert_eq!(logic.answer(&some, "kip is red").words, vec!["unknown"]);
    let crowd = |count: usize| {
        let mut story: Vec<String> = (1..=count).map(|at| format!("T{at} is a thing.")).collect();
        story.push("It holds that for any thing x , for any thing y , if (x likes y) then (y likes x).".to_string());
        story.push("T1 likes T2.".to_string());
        story
    };
    let started = std::time::Instant::now();
    assert_eq!(logic.answer(&crowd(60), "t2 likes t1").words, vec!["true"]);
    assert!(started.elapsed().as_secs() < 10);
    assert_eq!(logic.answer(&crowd(70), "t2 likes t1").words, vec!["unknown"]);
}

const NESTED: &str = "Write the fact that AAA does not hold as it is false that (AAA).
Write AAA ( the fact that BBB is not incorrect ) as AAA ( BBB ).
Write AAA ( if BBB then CCC ) as AAA ( if (BBB) then (CCC) ).
To answer \"AA\": True if it is proved that AA. False if it is disproved that AA. Unknown otherwise.";

#[test]
fn a_place_written_three_times_holds_whole_groups_so_lines_rewrite_step_by_step() {
    let nested = model("nested", NESTED);
    let story = told(&["The fact that the fact that the cat runs is not incorrect does not hold."]);
    assert_eq!(nested.answer(&story, "the cat runs").words, vec!["false"]);
    let rule = told(&["The fact that if the cat runs then the dog barks does not hold."]);
    assert_eq!(nested.answer(&rule, "the cat runs").words, vec!["true"]);
    assert_eq!(nested.answer(&rule, "the dog barks").words, vec!["false"]);
}

const PREFERENCES: &str = "Read told rules with something and someone for any thing.
To answer \"AA\": proved if AA. disproved if it is false that (AA). unknown otherwise.";

#[test]
fn a_named_told_rule_is_defeated_by_a_preferred_rule_with_the_opposite_outcome() {
    let prefer = model("preferences", PREFERENCES);
    let rules = ["The cat is hungry.", "Rule1: If the cat is hungry then the cat eats.", "Rule2: If the cat is tired then it is false that (the cat eats)."];
    let story = |more: &[&str]| told(&rules.iter().chain(more.iter()).copied().collect::<Vec<&str>>());
    assert_eq!(prefer.answer(&story(&["The cat is tired.", "Rule2 is preferred over Rule1."]), "the cat eats").words, vec!["disproved"]);
    assert_eq!(prefer.answer(&story(&["The cat is tired.", "Rule1 is preferred over Rule2."]), "the cat eats").words, vec!["proved"]);
    assert_eq!(prefer.answer(&story(&["Rule2 is preferred over Rule1."]), "the cat eats").words, vec!["proved"]);
}

const SAID_FORMULAS: &str = "Write it is said that AAA as it holds that ( say AAA ).
Write the question is AAA as the asked formula is ( say AAA ).
Write inside say AAA if BBB as if ( say BBB ) then ( say AAA ).
Write inside say the fact that AAA does not hold as it is false that ( say AAA ).
Write inside say AAA as AAA.
To answer \"AA\": True if the asked formula is ( BBB ) and it is proved that BBB. False if the asked formula is ( BBB ) and it is disproved that BBB. Unknown otherwise.";

#[test]
fn a_line_that_writes_inside_brackets_builds_a_nested_formula_part_by_part() {
    let said = model("said", SAID_FORMULAS);
    let story = told(&["It is said that the dog barks if the cat runs.", "It is said that the cat runs.", "It is said that the fact that the fox naps does not hold if the cat runs."]);
    assert_eq!(said.answer(&story, "the dog barks").words, vec!["true"]);
    assert_eq!(said.answer(&story, "the fox naps").words, vec!["false"]);
    assert_eq!(said.answer(&story, "the fact that the fox naps does not hold").words, vec!["true"]);
    assert_eq!(said.answer(&story, "the hen sings").words, vec!["unknown"]);
    let whole_only = model("said", &SAID_FORMULAS.replace("Write inside", "Write"));
    assert_eq!(whole_only.answer(&story, "the dog barks").words, vec!["unknown"]);
}

const STEMS: &str = "The clause block reads the stems of words.
To answer \"AA\": True if it is proved that AA. False if it is disproved that AA. Unknown otherwise.";

#[test]
fn a_text_can_ask_the_clause_block_to_compare_sentences_by_the_stems_of_their_words() {
    let stems = model("stems", STEMS);
    let exact = model("classical", CLASSICAL);
    let story = told(&["It holds that if (the kissing occurs) then (the activeness occurs).", "The kiss occurs.", "The non-deflationariness occurs.", "The bus stops."]);
    assert_eq!(stems.answer(&story, "the active occurs").words, vec!["true"]);
    assert_eq!(exact.answer(&story, "the active occurs").words, vec!["unknown"]);
    assert_eq!(stems.answer(&story, "the deflationariness occurs").words, vec!["false"]);
    assert_eq!(stems.answer(&story, "the bus stop").words, vec!["true"]);
    assert_eq!(stems.answer(&story, "the bu stop").words, vec!["unknown"]);
}

#[test]
fn a_text_can_ask_the_clause_block_to_take_things_where_the_sentences_meet() {
    let met = model("met", &format!("The clause block takes things from the sentences.\n{CLASSICAL}"));
    let exact = model("classical", CLASSICAL);
    let rule = "It holds that for any thing x , if (x thrones) then (x sings).";
    let story = told(&[rule, "The caustic thrones."]);
    assert_eq!(met.answer(&story, "the caustic sings").words, vec!["true"]);
    assert_eq!(exact.answer(&story, "the caustic sings").words, vec!["unknown"]);
    let back = told(&[rule, "It is false that (the owl sings)."]);
    assert_eq!(met.answer(&back, "the owl thrones").words, vec!["false"]);
    let asked = told(&["It holds that for any thing x , (x sings)."]);
    assert_eq!(met.answer(&asked, "the owl sings").words, vec!["true"]);
    let red = told(&["It holds that for any thing x , (x red).", "It holds that for any thing x , (x dog)."]);
    assert_eq!(met.answer(&red, "the cat red a dog").words, vec!["unknown"]);
    let some = model("some", "The clause block takes things from the sentences.\nTo answer \"AAA\": True if it is proved that AAA. False if it is disproved that AAA. Unknown otherwise.");
    assert_eq!(some.answer(&told(&["It is false that (the cat red a dog)."]), "for some thing x , it is false that ((x red) and (x dog))").words, vec!["unknown"]);
    let both = model("met", &format!("The clause block takes things from the sentences.\nThe clause block reads the stems of words.\n{CLASSICAL}"));
    assert_eq!(both.answer(&told(&[rule, "The caustic throne."]), "the caustic singing").words, vec!["true"]);
}

#[test]
fn a_false_formula_said_of_any_thing_is_read_for_a_new_witness() {
    let logic = model("classical", "To answer \"AAA\": True if it is proved that AAA. False if it is disproved that AAA. Unknown otherwise.");
    let truth = |story: &[&str], question: &str| logic.answer(&told(story), question).words;
    assert_eq!(truth(&["The cat is red."], "for any thing x , x is blue"), vec!["unknown"]);
    assert_eq!(truth(&["Kip is a thing.", "Kip is warm."], "for any thing x , x is warm"), vec!["unknown"]);
    let rules = ["It holds that for any thing x , if (x is a cat) then (x is furry).", "It holds that for any thing x , if (x is furry) then (x is warm)."];
    assert_eq!(truth(&rules, "for any thing x , if (x is a cat) then (x is warm)"), vec!["true"]);
    assert_eq!(truth(&rules, "for any thing x , if (x is warm) then (x is a cat)"), vec!["unknown"]);
    assert_eq!(truth(&["It is false that (for any thing x , (x is red))."], "the moon is cheese"), vec!["unknown"]);
}

const DOUBLING: &str = "If Start with R then the value is R.
If Double the value and before that time the value is R and R times 2 is S then the value is S.
If Add one to the value and before that time the value is R and R plus 1 is S then the value is S.
What is the value? R if latest the value is R.";

#[test]
fn rules_that_read_an_earlier_time_read_the_story_in_its_order() {
    let doubling = model("doubling", DOUBLING);
    let value = |story: &[&str]| doubling.answer(&told(story), "What is the value?").words;
    assert_eq!(value(&["Start with 3.", "Double the value.", "Add one to the value.", "Double the value."]), vec!["14"]);
    assert_eq!(value(&["Start with 3.", "Double the value.", "Double the value.", "Double the value."]), vec!["24"]);
}

#[test]
fn a_chain_that_ends_at_a_bound_is_counted() {
    let endless = model("endless", "If the count is A and A plus 1 is B then the count is B.\nWhat is the count? A if the count is A.");
    let _ = endless.answer(&told(&["The count is 1."]), "What is the count?");
    assert!(endless.bound_stops() > 0);
    let sizes = model("sizes", SIZES);
    let _ = sizes.answer(&told(&["The pebble fits inside the cup."]), "Is the cup bigger than the pebble?");
    assert_eq!(sizes.bound_stops(), 0);
}

#[test]
fn a_write_that_changes_nothing_is_no_fit_and_endless_writing_is_counted() {
    let said = model("said", "Write it is said that AAA as ( say AAA ).\nWrite inside say AAA . as say AAA.\nWrite inside say AAA as AAA.\nIs the cat red? yes if ( the cat is red ). no otherwise.");
    assert_eq!(said.answer(&told(&["It is said that the cat is red."]), "Is the cat red?").words, vec!["yes"]);
    assert_eq!(said.bound_stops(), 0);
    let turning = model("turning", "Write A is up as A is down.\nWrite A is down as A is up.\nIs B up? yes if B is up. no otherwise.");
    let _ = turning.answer(&told(&["The lid is up."]), "Is the lid up?");
    assert!(turning.bound_stops() > 0);
}

#[test]
fn a_condition_that_it_follows_reads_the_facts_that_say_so() {
    let follows = model("follows", "If Premise : BB then it follows that BB.\nIf it follows that AA then probe one AA.\nTo answer \"Relation:\": yes if probe one AA. no otherwise.");
    assert_eq!(follows.answer(&told(&["Premise: the cat is red."]), "Relation:").words, vec!["yes"]);
}

const BELOW_ZERO: &str = "If the start is B and the drop is C and B minus C is A then the end is A.
What is the end? A if the end is A.
What is B times C? A if B times C is A.
Is B below C? yes if B is less than C. no otherwise.";

#[test]
fn a_number_below_nothing_is_one_word_that_steps_and_compares() {
    let below = model("below", BELOW_ZERO);
    let ask = |story: &[&str], question: &str| below.answer(&told(story), question).words;
    assert_eq!(ask(&["The start is 0.", "The drop is 0.4."], "What is the end?"), ilm_core::base::words::answer_words("-0.4"));
    assert_eq!(ask(&[], "What is -0.4 times -2?"), ilm_core::base::words::answer_words("0.8"));
    assert_eq!(ask(&[], "Is -1 below 0.5?"), vec!["yes"]);
    assert_eq!(ask(&[], "Is -0.5 below -2?"), vec!["no"]);
    assert_eq!(ilm_core::tools::explain::text::text_words("From -0.4 to 3-4 and a well-known x - 2."), vec!["From", "-0.4", "to", "3", "-", "4", "and", "a", "well-known", "x", "-", "2", "."]);
}

const SAME_POINTS: &str = "If Point R S and Point T U and R S is the same as T U then probe R S T U.
Is R S met by T U? yes if probe R S T U. no otherwise.";

#[test]
fn runs_of_places_compare_as_words_and_nothing_below_nothing_is_nothing() {
    let points = model("points", SAME_POINTS);
    let story = told(&["Point 1 2.", "Point 1 3.", "Point -0.0 3.", "Point 0.0 3."]);
    assert_eq!(points.answer(&story, "Is 1 2 met by 1 2?").words, vec!["yes"]);
    assert_eq!(points.answer(&story, "Is 1 2 met by 1 3?").words, vec!["no"]);
    assert_eq!(points.answer(&story, "Is 0.0 3 met by 0.0 3?").words, vec!["yes"]);
    assert_eq!(ilm_core::tools::explain::text::text_words("At -0.0 and -0.5 ."), vec!["At", "0.0", "and", "-0.5", "."]);
}

const SMALL_NUMBERS: &str = "Is A small? yes if A times 1 is C and C is less than 0.5. no otherwise.
Is A tiny? yes if A times 1 is C and 0.5 is more than C. no otherwise.";

#[test]
fn a_number_with_a_point_in_the_conditions_of_an_answer_stays_one_number() {
    let small = model("small", SMALL_NUMBERS);
    assert_eq!(small.answer(&[], "Is 0.25 small?").words, vec!["yes"]);
    assert_eq!(small.answer(&[], "Is 0.75 small?").words, vec!["no"]);
    assert_eq!(small.answer(&[], "Is 0.25 tiny?").words, vec!["yes"]);
}

const MARKS_READ: &str = "If Number R and letter K of R is . then R has a point.
If GG and GG is told and GG holds the word ? then GG asks.
Has R a point? yes if R has a point. no otherwise.
Does it ask? yes if GG asks. no otherwise.";

#[test]
fn a_condition_may_read_a_mark_that_ends_a_sentence_as_a_letter_or_a_word() {
    let marks = model("marks", MARKS_READ);
    assert_eq!(marks.answer(&told(&["Number 2.5."]), "Has 2.5 a point?").words, vec!["yes"]);
    assert_eq!(marks.answer(&told(&["Number 25."]), "Has 25 a point?").words, vec!["no"]);
    assert_eq!(marks.answer(&told(&["Is it red ? Yes."]), "Does it ask?").words, vec!["yes"]);
    assert_eq!(marks.answer(&told(&["It is red."]), "Does it ask?").words, vec!["no"]);
}

#[test]
fn a_told_condition_with_an_open_place_reads_every_told_sentence_and_a_cut_rule_is_a_fault() {
    let told_ones = model("told ones", "If AA is told then AA was said.\nWas AA said? yes if AA was said. no otherwise.");
    assert_eq!(told_ones.answer(&told(&["The lid is red."]), "Was the lid is red said?").words, vec!["yes"]);
    assert_eq!(told_ones.answer(&told(&["The lid is red."]), "Was the lid is blue said?").words, vec!["no"]);
    assert!(read_topic("cut", "If Step and then stop is read then probe five.").is_err());
    assert!(read_topic("kept", "If Step And Then stop is read then probe six.").is_ok());
}

#[test]
fn an_answer_that_holds_otherwise_is_known_as_a_default() {
    let red = model("red", "Is A red? yes if A is red. no otherwise.");
    let proved = red.answer(&told(&["The cat is red."]), "Is the cat red?");
    let guessed = red.answer(&told(&["The cat is red."]), "Is the dog red?");
    assert_eq!(proved.mark, AnswerMark::Proof);
    assert_eq!(guessed.mark, AnswerMark::Default);
}

const MOVES_PROOF: &str = "If A got the B then the holder of the B is A.
If A picked up the B then A got the B.
If the holder of the B is A and A went to the C and at that time the holder of the B is A then the place of the B is C.
Where is the B? C if latest the place of the B is C.";

const KIN_PROOF: &str = "If A is the mother of B then A is a parent of B.
If A is the father of B then A is a parent of B.
If A is a parent of B and B is a parent of C then A is a grandparent of C.
Is A a grandparent of C? yes if A is a grandparent of C. no otherwise.";

#[test]
fn a_missing_answer_names_the_first_condition_that_the_story_lacks() {
    let kin = model("kin", KIN_PROOF);
    let half = told(&["Ann is the mother of Bob."]);
    let missing = kin.missing(&half, "Is Ann a grandparent of Cid?", &["yes".to_string()]);
    assert!(missing.iter().any(|line| line.trim() == "missing: ann is a grandparent of cid"), "{missing:?}");
    assert!(missing.iter().any(|line| line.trim() == "missing, as the text writes it: kin: A is a grandparent of C"), "{missing:?}");
    let family = told(&["Ann is the mother of Bob.", "Bob is the father of Cid."]);
    let held = kin.missing(&family, "Is Ann a grandparent of Cid?", &["no".to_string()]);
    assert!(held.iter().any(|line| line.contains("an earlier answer of the library comes first")), "{held:?}");
    assert!(kin.missing(&family, "Where is Ann?", &["home".to_string()]).is_empty());
}

#[test]
fn an_answer_is_explained_by_a_proof_from_the_told_sentences() {
    let moves = model("moves", MOVES_PROOF);
    let story = told(&["Mary picked up the apple.", "Mary went to the hallway."]);
    let proof = moves.proof(&story, "Where is the apple?");
    assert!(proof.iter().any(|line| line.contains("the place of the apple is hallway") && line.contains("by the rule")));
    assert!(proof.iter().any(|line| line.trim() == "told: mary picked up the apple"));
    assert!(proof.iter().any(|line| line.trim() == "told: mary went to the hallway"));
    let kin = model("kin", KIN_PROOF);
    let family = told(&["Ann is the mother of Bob.", "Bob is the father of Cid."]);
    let proof = kin.proof(&family, "Is Ann a grandparent of Cid?");
    assert!(proof.iter().any(|line| line.contains("ann is a grandparent of cid by the rule")));
    assert!(proof.iter().any(|line| line.trim() == "told: bob is the father of cid"));
    let guessed = kin.proof(&family, "Is Bob a grandparent of Ann?");
    assert!(guessed.iter().any(|line| line.contains("no step proves it")));
    let logic = model("classical", CLASSICAL);
    let rule = told(&["It holds that if (the cat is red) then (the dog is blue).", "The cat is red."]);
    let proof = logic.proof(&rule, "the dog is blue");
    assert!(proof.iter().any(|line| line.contains("proved by refutation: the dog is blue")));
    assert!(proof.iter().any(|line| line.trim() == "told: the cat is red"));
    let sum = model("sum", "If A has B coins and A got C coins and B plus C is D then A ends with D coins.\nHow many coins has A? D if A ends with D coins.");
    let proof = sum.proof(&told(&["Kip has 3 coins.", "Kip got 4 coins."]), "How many coins has Kip?");
    assert!(proof.iter().any(|line| line.trim() == "worked out: 3 plus 4 is 7"));
}

#[test]
fn words_compare_in_the_order_of_the_alphabet_and_a_count_may_count_a_place_of_one_word() {
    let order = model("order", "Does V come first? yes if V comes before W in the alphabet and W is a word. no otherwise.\nHow many words? N if N is the number of W where W is a word.");
    let story = told(&["Pear is a word."]);
    assert_eq!(order.answer(&story, "Does apple come first?").words, vec!["yes"]);
    assert_eq!(order.answer(&story, "Does zebra come first?").words, vec!["no"]);
    assert_eq!(order.answer(&told(&["Pear is a word.", "Fig is a word."]), "How many words?").words, vec!["2"]);
}

#[test]
fn an_answer_that_starts_with_every_says_the_words_of_each_filling_in_order() {
    let stops = model("stops", "To answer \"Which stops?\": every C if C is a stop. none otherwise.\nTo answer \"Which stop?\": C if C is a stop. none otherwise.");
    let story = told(&["Boston is a stop.", "Denver is a stop.", "Boston is a stop."]);
    assert_eq!(stops.answer(&story, "Which stops?").words, vec!["boston", "denver"]);
    assert_eq!(stops.answer(&story, "Which stop?").words, vec!["boston"]);
    assert_eq!(stops.answer(&told(&["Rain fell."]), "Which stops?").words, vec!["none"]);
    let trips = model("trips", "To answer \"From where?\": every R if Flights go AA from R to BB. none otherwise.");
    assert_eq!(trips.answer(&told(&["Flights go now from boston to denver or from dallas to miami."]), "From where?").words, vec!["boston", "dallas"]);
}

#[test]
fn a_proof_says_whether_a_condition_held_at_a_time_read_the_state_of_its_thing_or_the_newest_fact_of_any_thing() {
    let story = told(&["Kip took the harp.", "Kip left the harp.", "Kip took the drum.", "Kip went to the attic."]);
    let start = "If A took the B then the holder of the B is A.
If A left the B then the holder of the B is no more.
";
    let asked = "
Where is the A? B if latest the A is in the B.";
    let short = model("short", &format!("{start}If A went to the B and at that time the holder of the C is A then the C is in the B.{asked}"));
    let proof = short.proof(&story, "Where is the drum?");
    assert!(proof.iter().any(|line| line.contains("at that time the holder of the drum is kip") && line.contains("the newest fact of its sentence")), "{proof:?}");
    let bound = model("bound", &format!("{start}If the holder of the C is A and A went to the B and at that time the holder of the C is A then the C is in the B.{asked}"));
    let proof = bound.proof(&story, "Where is the drum?");
    assert!(proof.iter().any(|line| line.contains("at that time the holder of the drum is kip") && line.contains("the latest value of this state")), "{proof:?}");
}

#[test]
fn a_proof_marks_a_condition_at_a_time_read_on_facts_that_keep_no_history() {
    use ilm_core::tools::explain::chain::{chain_facts, empty_filling, knowledge_base};
    use ilm_core::tools::explain::derive::{answer_proof, proof_lines};
    let topic = read_topic("held", "If A took the B then the holder of the B is A.
If A went to the B and at that time the holder of the C is A then the C is in the B.").expect("the topic reads");
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
    let facts = chain_facts(&told(&["Kip took the drum.", "Kip went to the attic."]), &base, &topic.rules, topic.rules_key);
    let rule = &topic.rules[1];
    let proof = proof_lines(&answer_proof(&topic.rules, &facts, topic.facts.len(), &rule.conditions, &rule.absent, &empty_filling()));
    assert!(proof.iter().any(|line| line.contains("at that time the holder of the drum is kip") && line.contains("the facts keep no history")), "{proof:?}");
}

#[test]
fn the_missing_step_says_which_fact_a_condition_held_at_a_time_read() {
    let taken = model("taken", "If A moved to the B then A went to the B.
Where is the A? C if latest B took the A and at that time B went to the C.");
    let story = told(&["Kip moved to the porch.", "Kip took the drum.", "Kip moved to the attic."]);
    let lines = taken.missing(&story, "Where is the drum?", &["attic".to_string()]);
    assert!(lines.iter().any(|line| line.contains("holds: latest kip took the drum") && line.contains("not the state of one thing")), "{lines:?}");
    assert!(lines.iter().any(|line| line.contains("holds: at that time kip went to the porch") && line.contains("the latest value of this state")), "{lines:?}");
}

#[test]
fn a_proof_of_the_library_answers_from_told_sentences_alone() {
    let sentences = sentences_of("  Kip has two dollars. Kip got seven more dollars.   Kip now has nine dollars ");
    assert_eq!(sentences, vec!["Kip has two dollars.", "Kip got seven more dollars.", "Kip now has nine dollars"]);
    let library = Library::of(vec![read_topic("dollars", DOLLARS).expect("the topic reads")]).expect("the library builds");
    let (question, told) = sentences.split_last().expect("a question");
    let (words, proof) = library_proof(&library, told, question).expect("the library answers");
    assert_eq!(words, vec!["true"]);
    assert!(proof[0].starts_with("proof of the answer dollars: true"), "{proof:?}");
    assert!(proof.iter().any(|line| line.trim() == "told: kip got seven more dollars"), "{proof:?}");
    assert_eq!(library_proof(&library, told, "Kip now has seven dollars"), None);
}

#[test]
fn the_costs_of_a_learning_name_each_way_and_the_chains_by_text() {
    assert_eq!(close_costs().line, "");
    open_costs();
    {
        let mut way = cost_scope(Some("label list"), Some("words"));
        chain_ended(learning_clock(), true);
        chain_ended(learning_clock(), false);
        way.taken();
    }
    let line = close_costs().line;
    assert!(line.starts_with("ways: label list "), "{line}");
    assert!(line.contains(" taken | chains: 2 made, 1 ran away"), "{line}");
    assert!(line.contains("label list on words: 2 made, 1 ran away"), "{line}");
    assert_eq!(close_costs().line, "");
    chain_ended(learning_clock(), true);
    assert_eq!(close_costs().line, "");
}

#[test]
fn a_proof_marks_each_rule_and_learned_fact_as_discovered_or_given() {
    let proof: Vec<String> = ["proof of the answer t: x", "  a is c by the rule: If A is B then A is C", "    told: a is b", "  known: two is the number 2", "  known: three is the number 3", "  c is d by the rule: If C is D then C is E"].iter().map(|line| line.to_string()).collect();
    let rules = vec![("If A is B then A is C".to_string(), false), ("If C is D then C is E".to_string(), true)];
    let marked = marked_proof(&proof, &rules, &["Two is the number 2.".to_string()], "babi01");
    assert_eq!(marked[0], "proof of the answer t: x");
    assert_eq!(marked[1], "  a is c by the rule: If A is B then A is C [discovered on babi01]");
    assert_eq!(marked[2], "    told: a is b");
    assert_eq!(marked[3], "  known: two is the number 2 [discovered on babi01]");
    assert_eq!(marked[4], "  known: three is the number 3");
    assert_eq!(marked[5], "  c is d by the rule: If C is D then C is E [given]");
}

#[test]
fn the_places_of_a_text_show_a_name_that_the_reader_takes_as_places() {
    let read = read_topic("capitals", "If A lives in Washington, D.C. then A lives in the United States.").expect("the line reads");
    let letters: Vec<String> = topic_places(&read).into_iter().map(place_letter).collect();
    assert_eq!(letters, vec!["A", "D", "C"]);
    let fact = read_topic("capitals", "Washington is the capital of the United States.").expect("the line reads");
    assert!(topic_places(&fact).is_empty());
}

#[test]
fn a_condition_that_no_fact_fits_shows_the_known_facts_that_the_rules_which_would_make_it_read() {
    let text = "Kip is the step 1.
Mol is the step 2.
Nat is the step 3.
If R rests at T and T is the step X then R rests at step X.
If R comes at T and T is the step X and R rests at step V and V is less than X then R comes late.
If R comes at T and not R comes late then R comes in time.
Does A come in time? yes if A comes in time.";
    let paced = model("paced", text);
    let proof = paced.proof(&told(&["Wren rests at nat.", "Wren comes at mol."]), "Does Wren come in time?");
    let absent = proof.iter().position(|line| line.trim() == "no fact fits: wren comes late").expect("the absent condition is shown");
    let under: Vec<&String> = proof[absent..].iter().take(4).collect();
    assert!(under.iter().any(|line| line.trim() == "known: mol is the step 2"), "{proof:?}");
    assert!(under.iter().any(|line| line.trim() == "known: nat is the step 3"), "{proof:?}");
}

const USUAL_SEXES: &str = "John is a male name.
Mary is a female name.
Alex is a male name.
Alex is a female name.
Son means a male child.
Daughter means a female child.
If A is a male name then usually A is male.
If A is a female name then usually A is female.
If A is mother of B then A is female.
If A is father of B then A is male.
Is A male? yes if A is male. no otherwise.";

#[test]
fn a_rule_that_holds_usually_gives_way_to_a_told_fact() {
    let sexes = model("sexes", USUAL_SEXES);
    assert_eq!(sexes.answer(&told(&["John went home."]), "Is John male?").words, vec!["yes"]);
    assert_eq!(sexes.answer(&told(&["John is mother of Ann."]), "Is John male?").words, vec!["no"]);
    assert_eq!(sexes.answer(&told(&["Mary is father of Tom."]), "Is Mary male?").words, vec!["yes"]);
    assert_eq!(sexes.answer(&told(&["Mary went home."]), "Is Mary male?").words, vec!["no"]);
    assert_eq!(sexes.answer(&told(&["Alex went home."]), "Is Alex male?").words, vec!["no"], "two rival defaults give neither");
    let proof = sexes.proof(&told(&["John went home."]), "Is John male?");
    assert!(proof.iter().any(|line| line.contains("then usually A is male")), "{proof:?}");
    let overruled = sexes.proof(&told(&["John is mother of Ann."]), "Is John male?");
    assert!(overruled.iter().any(|line| line.contains("a default is overruled: john is male (If A is a male name then usually A is male)")), "{overruled:?}");
    assert!(overruled.iter().any(|line| line.contains("john is female by the rule: If A is mother of B then A is female")), "{overruled:?}");
    assert!(overruled.iter().any(|line| line.contains("told: john is mother of ann")), "{overruled:?}");
}

const KIN_NOT_CYCLE: &str = "If A is a parent of B and B is a parent of C then A is a grandparent of C.
If A is a parent of B and B is married to C then C is a child-in-law of A.
If A is a grandparent of B and C is a parent of B and not C is a child-in-law of A then A is a parent of C.
Is A a parent of C? yes if A is a parent of C. no otherwise.";

#[test]
fn the_proof_of_a_rule_that_must_not_hold_a_fact_that_its_own_outcome_feeds_stays_on_a_small_stack() {
    let kin = model("kin", KIN_NOT_CYCLE);
    let story: Vec<String> = (0..20).flat_map(|at| [format!("Ann is a grandparent of B{at}."), format!("C{at} is a parent of B{at}.")]).collect();
    let small = std::thread::Builder::new().stack_size(2 << 20).spawn(move || (kin.answer(&story, "Is Ann a parent of C0?").words, kin.rules_used(&story, "Is Ann a parent of C0?").len())).expect("the thread starts");
    let (said, rules) = small.join().expect("the proof ends on a stack of two megabytes");
    assert_eq!(said, vec!["yes".to_string()]);
    assert!(rules > 0);
}
