use ilm_commands::command::{COMMAND_WIDE_OPTIONS, REFUSING_COMMANDS, refused_options};

#[test]
fn every_option_of_every_command_passes_the_check_of_a_command_that_refuses_options() {
    for (command, own) in REFUSING_COMMANDS {
        assert!(refused_options(command, COMMAND_WIDE_OPTIONS).is_empty(), "{command} refuses an option of every command");
        assert!(refused_options(command, own).is_empty(), "{command} refuses an option of its own");
        assert_eq!(refused_options(command, &["no-such-option"]), vec!["no-such-option".to_string()]);
    }
}

#[test]
fn every_option_that_the_program_reads_before_a_command_is_an_option_of_every_command() {
    let source = include_str!("../src/command.rs");
    let start = source.find("pub fn run_command()").expect("the program runs its commands");
    let end = start + source[start..].find("match part.as_str()").expect("the program picks a command");
    let wide = source.lines().find(|line| line.starts_with("pub const COMMAND_WIDE_OPTIONS")).expect("the options of every command");
    let read: Vec<&str> = source[start..end].split(|letter: char| !(letter.is_ascii_uppercase() || letter == '_')).filter(|word| word.ends_with("_OPTION")).collect();
    assert!(!read.is_empty(), "the program reads some option before every command");
    for option in read {
        assert!(wide.contains(option), "{option} is read before every command but is not an option of every command");
    }
}
