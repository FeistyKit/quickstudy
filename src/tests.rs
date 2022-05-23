#![cfg(test)]

use crate::*;

#[test]
fn parse_answer_only() {
    let src = "[answer]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(
        Some(Ok(Question {
            dat: vec![(Option::<String>::None, Some("answer".to_string()))]
        })),
        question
    );
}

#[test]
fn too_many_opens() {
    let src = "[[answer]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(Some(Err(String::from("Unexpected `[`!"))), question);
}

#[test]
fn unclosed_answer() {
    let src = "[answer".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(
        Some(Err(String::from("Unexpected end of answer!"))),
        question
    );
}

#[test]
fn parse_valid_question() {
    let src = "question [answer] question [answer]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(
        Some(Ok(Question {
            dat: vec![
                (Some("question ".to_string()), Some("answer".to_string())),
                (Some(" question ".to_string()), Some("answer".to_string()))
            ]
        })),
        question
    );
}

#[test]
fn unexpected_closer_in_question() {
    let src = "answer]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(Some(Err(String::from("Unexpected `]`!"))), question);
}

#[test]
fn unexpected_closer_in_answer() {
    let src = "[answer]]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(Some(Err(String::from("Unexpected `]`!"))), question);
}
