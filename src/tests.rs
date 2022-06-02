#![cfg(test)]

use crate::question::*;

#[test]
fn parse_answer_only() {
    let src = "[answer]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(
        Some(Ok(Question {
            pools: Vec::new(),
            dat: vec![(
                Option::<String>::None,
                Some(Answer::Raw("answer".to_string()))
            )]
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
            pools: Vec::new(),
            dat: vec![
                (
                    Some("question ".to_string()),
                    Some(Answer::Raw("answer".to_string()))
                ),
                (
                    Some(" question ".to_string()),
                    Some(Answer::Raw("answer".to_string()))
                )
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

#[test]
fn one_of_three() {
    let src = "[a1 | a2 | a3]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(
        Some(Ok(Question {
            dat: vec![(
                None,
                Some(Answer::OneOf(vec![
                    "a1".to_string(),
                    "a2".to_string(),
                    "a3".to_string()
                ]))
            )],
            pools: Vec::new()
        })),
        question
    )
}

#[test]
fn one_shared_pool() {
    let question = Parser::new("{1}; abc").next();

    assert_eq!(
        Some(Ok(Question {
            dat: vec![(None, Some(Answer::SharedPool(0)))],
            pools: vec![vec!["abc".to_string()]]
        })),
        question
    )
}

#[test]
fn multiple_shared_pools() {
    let question = Parser::new("{1}, {1}, {2}, {2}; amogus, sus; cheese, man").next();

    assert_eq!(
        Some(Ok(Question {
            dat: vec![
                (None, Some(Answer::SharedPool(0))),
                (Some(", ".to_string()), Some(Answer::SharedPool(0))),
                (Some(", ".to_string()), Some(Answer::SharedPool(1))),
                (Some(", ".to_string()), Some(Answer::SharedPool(1)))
            ],
            pools: vec![
                vec!["amogus".to_string(), "sus".to_string()],
                vec!["cheese".to_string(), "man".to_string()]
            ]
        })),
        question
    );
}
