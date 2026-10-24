mod builtins;
mod tokenizer;

use std::error::Error;
use std::io::ErrorKind;
use std::io::{self, Write};
use std::iter::Peekable;
use std::process::{Command, Stdio};
use std::slice::Iter;

use crate::builtins::handle_builtins;
use crate::tokenizer::{Token, TokenKind, tokenize};

#[derive(Debug, PartialEq)]
struct UserInput {
    command: String,
    arguments: Vec<String>,
}

fn combine_tokens_into_word(tokens: &[Token]) -> Token {
    Token {
        kind: TokenKind::Word,
        raw: tokens
            .iter()
            .map(|t| t.raw.clone())
            .collect::<Vec<String>>()
            .join(""),
    }
}

fn parse_quoted(tokens: &mut Peekable<Iter<Token>>) -> Token {
    let mut final_tokens: Vec<Token> = vec![];
    while let Some(token) = tokens.next() {
        if token.kind == TokenKind::Apostrophe {
            if tokens
                .peek()
                .is_some_and(|t| t.kind != TokenKind::Apostrophe)
            {
                break;
            }
        } else {
            final_tokens.push(token.clone());
        }
    }
    combine_tokens_into_word(&final_tokens)
}

fn parse_echo_args(tokens: &Vec<Token>) -> UserInput {
    // ignore leading whitespace
    let start = tokens
        .iter()
        .position(|t| !t.kind.is_whitespace())
        .unwrap_or(tokens.len());
    let mut rest = tokens[start..].iter().peekable();
    let mut intermediate_tokens: Vec<Token> = vec![];

    // parse all quoted args
    while let Some(token) = rest.next() {
        if token.kind == TokenKind::Apostrophe {
            intermediate_tokens.push(parse_quoted(&mut rest));
        } else {
            intermediate_tokens.push(token.clone());
        }
    }

    // at this point all apostrophes should be removed and we only
    // have words and whitespace
    // any two tokens without whitespace between them need to
    // be combined into a single token
    let mut final_tokens: Vec<Token> = vec![];
    let splitted = intermediate_tokens.split(|t| t.kind.is_whitespace());
    for slice in splitted {
        if slice.len() > 1 {
            final_tokens.push(combine_tokens_into_word(slice));
        } else if slice.len() == 1 {
            final_tokens.push(slice.first().unwrap().clone());
        }
    }

    UserInput {
        command: "echo".to_string(),
        arguments: final_tokens.into_iter().map(|t| t.raw).collect(),
    }
}

fn parse_input(input: &str) -> UserInput {
    let mut tokens = tokenize(input).into_iter().peekable();
    while tokens.peek().is_some_and(|t| t.kind.is_whitespace()) {
        tokens.next();
    }
    let command = tokens.next().unwrap_or_default();
    match command.raw.as_str() {
        "echo" => parse_echo_args(&tokens.collect()),
        _ => UserInput {
            command: command.raw,
            arguments: tokens
                .filter(|t| !t.kind.is_whitespace())
                .map(|t| t.raw)
                .collect(),
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut done = false;

    while !done {
        print!("$ ");
        io::stdout().flush()?;
        let mut raw_input = String::new();

        io::stdin().read_line(&mut raw_input)?;
        let trimmed = raw_input.trim_end();

        let user_input = parse_input(trimmed);

        if !handle_builtins(&user_input, &mut done)
            && let Err(e) = Command::new(&user_input.command)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .args(&user_input.arguments)
                .status()
        {
            let err_string = match e.kind() {
                ErrorKind::NotFound => "command not found".to_string(),
                _ => e.to_string(),
            };
            println!("{}: {}", user_input.command, err_string);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind::{Apostrophe, Space, Tab, Word};

    fn tok(kind: TokenKind, raw: &str) -> Token {
        Token {
            kind,
            raw: raw.to_string(),
        }
    }

    fn user_input(command: &str, arguments: &[&str]) -> UserInput {
        UserInput {
            command: command.to_string(),
            arguments: arguments.iter().map(|a| a.to_string()).collect(),
        }
    }

    mod combine_tokens_into_word {
        use super::*;

        #[test]
        fn combines_empty_slice_into_empty_word() {
            assert_eq!(combine_tokens_into_word(&[]), tok(Word, ""));
        }

        #[test]
        fn combines_single_token() {
            let tokens = [tok(Word, "hello")];
            assert_eq!(combine_tokens_into_word(&tokens), tok(Word, "hello"));
        }

        #[test]
        fn concatenates_raw_text_in_order() {
            let tokens = [tok(Word, "hello"), tok(Word, "world")];
            assert_eq!(combine_tokens_into_word(&tokens), tok(Word, "helloworld"));
        }

        #[test]
        fn preserves_whitespace_tokens() {
            let tokens = [
                tok(Word, "a"),
                tok(Space, " "),
                tok(Tab, "\t"),
                tok(Word, "b"),
            ];
            assert_eq!(combine_tokens_into_word(&tokens), tok(Word, "a \tb"));
        }

        #[test]
        fn result_is_always_a_word() {
            let tokens = [tok(Apostrophe, "'"), tok(Space, " ")];
            assert_eq!(combine_tokens_into_word(&tokens).kind, Word);
        }
    }

    mod parse_echo_args {
        use super::*;

        fn parse(args: &str) -> UserInput {
            parse_echo_args(&tokenize(args))
        }

        #[test]
        fn command_is_always_echo() {
            assert_eq!(parse("hello").command, "echo");
        }

        #[test]
        fn parses_no_args() {
            assert_eq!(parse(""), user_input("echo", &[]));
        }

        #[test]
        fn parses_whitespace_only_args() {
            assert_eq!(parse("   "), user_input("echo", &[]));
        }

        #[test]
        fn parses_single_arg() {
            assert_eq!(parse(" hello"), user_input("echo", &["hello"]));
        }

        #[test]
        fn parses_multiple_args() {
            assert_eq!(
                parse(" hello world"),
                user_input("echo", &["hello", "world"])
            );
        }

        #[test]
        fn collapses_repeated_spaces_between_args() {
            assert_eq!(parse("  a    b"), user_input("echo", &["a", "b"]));
        }

        #[test]
        fn keeps_whitespace_inside_quotes() {
            assert_eq!(
                parse(" 'hello  world'"),
                user_input("echo", &["hello  world"])
            );
        }

        #[test]
        fn parses_quoted_arg_after_plain_arg() {
            assert_eq!(parse(" a 'b c'"), user_input("echo", &["a", "b c"]));
        }

        #[test]
        fn parses_plain_arg_after_quoted_arg() {
            assert_eq!(parse(" 'a b' c"), user_input("echo", &["a b", "c"]));
        }

        #[test]
        fn joins_adjacent_quoted_and_plain_text() {
            assert_eq!(parse(" hello'world'"), user_input("echo", &["helloworld"]));
        }

        #[test]
        fn joins_adjacent_quoted_strings() {
            assert_eq!(
                parse(" 'hello''world'"),
                user_input("echo", &["helloworld"])
            );
        }
    }

    mod parse_input {
        use super::*;

        #[test]
        fn parses_empty_input() {
            assert_eq!(parse_input(""), user_input("", &[]));
        }

        #[test]
        fn parses_command_without_args() {
            assert_eq!(parse_input("ls"), user_input("ls", &[]));
        }

        #[test]
        fn parses_command_with_args() {
            assert_eq!(
                parse_input("ls -la /tmp"),
                user_input("ls", &["-la", "/tmp"])
            );
        }

        #[test]
        fn collapses_repeated_spaces_between_args() {
            assert_eq!(
                parse_input("ls   -la    /tmp"),
                user_input("ls", &["-la", "/tmp"])
            );
        }

        #[test]
        fn ignores_leading_whitespace() {
            assert_eq!(parse_input("  ls -la"), user_input("ls", &["-la"]));
        }

        #[test]
        fn parses_echo_without_args() {
            assert_eq!(parse_input("echo"), user_input("echo", &[]));
        }

        #[test]
        fn parses_echo_with_args() {
            assert_eq!(
                parse_input("echo hello world"),
                user_input("echo", &["hello", "world"])
            );
        }

        #[test]
        fn parses_echo_with_quoted_arg() {
            assert_eq!(
                parse_input("echo 'hello  world'"),
                user_input("echo", &["hello  world"])
            );
        }

        #[test]
        fn parses_echo_with_mixed_quoted_and_plain_args() {
            assert_eq!(
                parse_input("echo a 'b c' d"),
                user_input("echo", &["a", "b c", "d"])
            );
        }
    }
}
