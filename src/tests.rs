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
fn parse_too_many_opens() {
    let src = "[[answer]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(Some(Err(String::from("Unexpected `[`!"))), question);
}

#[test]
fn parse_unclosed_answer() {
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
fn parse_unexpected_closer_in_question() {
    let src = "answer]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(Some(Err(String::from("Unexpected `]`!"))), question);
}

#[test]
fn parse_unexpected_closer_in_answer() {
    let src = "[answer]]".to_string();

    let question = Parser::new(&src).next();

    assert_eq!(Some(Err(String::from("Unexpected `]`!"))), question);
}

#[test]
fn parse_one_of_three() {
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
fn parse_one_shared_pool() {
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
fn parse_multiple_shared_pools() {
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

#[test]
fn handles_raw_answer() {
    let question = Parser::new("This [is] a [test] progra[m]").next().unwrap().unwrap();

    let answers = vec!["is", "test", "m"]
        .iter()
        .map(ToString::to_string)
        .collect();

    assert!(question.ask(answers));
}

#[test]
fn handles_one_of_answer() {
    let question = Parser::new("This [is | may be] a [test | real] progra[m | me]").next().unwrap().unwrap();

    let answers = vec!["is", "test", "m"]
        .iter()
        .map(ToString::to_string)
        .collect();

    assert!(question.ask(answers));

    let answers2 = vec!["may be", "real", "me"]
        .iter()
        .map(ToString::to_string)
        .collect();

    assert!(question.ask(answers2));
}

#[test]
#[should_panic]
// TODO: Come up with a better name for tests/handles_multiple_from_single
// It's really confusing
fn handles_multiple_from_single() {
    let question = Parser::new("[fake | answer]").next().unwrap().unwrap();


    let answer = vec!["fake | answer"]
        .iter()
        .map(ToString::to_string)
        .collect();

    assert!(question.ask(answer));
}

#[test]
fn handles_pool_items() {
    let question = Parser::new("The four seasons are: {1}, {1}, {1} and {1}. The best type of weather is either {2} or {2}; spring, summer, fall, winter; rain, snow, sun")
        .next()
        .unwrap()
        .unwrap();

    let answer = vec!["summer", "spring", "winter", "fall", "sun", "snow"]
        .iter()
        .map(ToString::to_string)
        .collect();

    assert!(question.ask(answer));
}
