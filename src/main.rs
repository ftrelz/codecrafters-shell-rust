mod builtins;
mod tokenizer;

use std::error::Error;
use std::io::ErrorKind;
use std::io::{self, Write};
use std::iter::Peekable;
use std::process::{Command, Stdio};
use std::vec::IntoIter;

use crate::builtins::handle_builtins;
use crate::tokenizer::{Token, TokenKind, tokenize};

#[derive(Debug, PartialEq)]
struct UserInput {
    command: String,
    arguments: Vec<String>,
}

fn skip_whitespace(tokens: &mut Peekable<IntoIter<Token>>) {
    while tokens.peek().is_some_and(|t| t.kind.is_whitespace()) {
        tokens.next();
    }
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

fn parse_quoted(tokens: &mut Peekable<IntoIter<Token>>, quote_type: &TokenKind) -> Token {
    let mut final_tokens: Vec<Token> = vec![];
    while let Some(token) = tokens.next() {
        if token.kind == *quote_type {
            if tokens.peek().is_some_and(|t| &t.kind != quote_type) {
                break;
            } else {
                tokens.next();
            }
        } else {
            final_tokens.push(token.clone());
        }
    }
    combine_tokens_into_word(&final_tokens)
}

fn parse_input(input: &str) -> UserInput {
    let mut tokens = tokenize(input).into_iter().peekable();
    skip_whitespace(&mut tokens);

    let command = match tokens.next() {
        Some(tok) => match tok.kind {
            TokenKind::Quote | TokenKind::Apostrophe => parse_quoted(&mut tokens, &tok.kind),
            TokenKind::Word => tok,
            // we just skipped all the whitespace
            TokenKind::Space | TokenKind::Tab | TokenKind::Nl | TokenKind::Cr => unreachable!(),
        },
        None => Token::default(),
    };
    skip_whitespace(&mut tokens);

    let mut intermediate_tokens: Vec<Token> = vec![];
    // first parsing pass, handles all double and single quoted args
    while let Some(token) = tokens.next() {
        match token.kind {
            TokenKind::Apostrophe | TokenKind::Quote => {
                intermediate_tokens.push(parse_quoted(&mut tokens, &token.kind))
            }
            _ => intermediate_tokens.push(token.clone()),
        }
    }

    // second parsing pass
    // at this point all apostrophes should be removed and we only have words and whitespace
    // any two tokens without whitespace between them need to be combined into a single token
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
        command: command.raw,
        arguments: final_tokens.into_iter().map(|t| t.raw).collect(),
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

    mod parse_input {
        use super::*;

        #[test]
        fn parses_empty_input() {
            assert_eq!(parse_input(""), user_input("", &[]));
        }

        #[test]
        fn parses_whitespace_only_input() {
            assert_eq!(parse_input("   "), user_input("", &[]));
        }

        #[test]
        fn parses_command_without_args() {
            assert_eq!(parse_input("ls"), user_input("ls", &[]));
        }

        #[test]
        fn parses_command_with_trailing_whitespace_only() {
            assert_eq!(parse_input("ls   "), user_input("ls", &[]));
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
        fn keeps_whitespace_inside_quotes() {
            assert_eq!(
                parse_input("echo 'hello  world'"),
                user_input("echo", &["hello  world"])
            );
        }

        #[test]
        fn parses_quoted_arg_after_plain_arg() {
            assert_eq!(
                parse_input("echo a 'b c'"),
                user_input("echo", &["a", "b c"])
            );
        }

        #[test]
        fn parses_plain_arg_after_quoted_arg() {
            assert_eq!(
                parse_input("echo 'a b' c"),
                user_input("echo", &["a b", "c"])
            );
        }

        #[test]
        fn parses_mixed_quoted_and_plain_args() {
            assert_eq!(
                parse_input("echo a 'b c' d"),
                user_input("echo", &["a", "b c", "d"])
            );
        }

        #[test]
        fn joins_adjacent_quoted_and_plain_text() {
            assert_eq!(
                parse_input("echo hello'world'"),
                user_input("echo", &["helloworld"])
            );
        }

        #[test]
        fn joins_adjacent_quoted_strings() {
            assert_eq!(
                parse_input("echo 'hello''world'"),
                user_input("echo", &["helloworld"])
            );
        }

        mod double_quotes {
            use super::*;

            #[test]
            fn parses_double_quoted_arg() {
                assert_eq!(
                    parse_input("echo \"hello world\""),
                    user_input("echo", &["hello world"])
                );
            }

            #[test]
            fn preserves_consecutive_spaces() {
                assert_eq!(
                    parse_input("echo \"hello    world\""),
                    user_input("echo", &["hello    world"])
                );
            }

            #[test]
            fn preserves_tabs() {
                assert_eq!(
                    parse_input("echo \"hello\t\tworld\""),
                    user_input("echo", &["hello\t\tworld"])
                );
            }

            #[test]
            fn preserves_mixed_spaces_and_tabs() {
                assert_eq!(
                    parse_input("echo \"a \t b\""),
                    user_input("echo", &["a \t b"])
                );
            }

            #[test]
            fn preserves_leading_and_trailing_whitespace() {
                assert_eq!(
                    parse_input("echo \"  a  \""),
                    user_input("echo", &["  a  "])
                );
            }

            #[test]
            fn parses_whitespace_only_quoted_arg() {
                assert_eq!(parse_input("echo \"   \""), user_input("echo", &["   "]));
            }

            #[test]
            fn collapses_whitespace_outside_quotes_only() {
                assert_eq!(
                    parse_input("echo   \"a   b\"    c"),
                    user_input("echo", &["a   b", "c"])
                );
            }

            #[test]
            fn treats_single_quote_literally() {
                assert_eq!(parse_input("echo \"it's\""), user_input("echo", &["it's"]));
            }

            #[test]
            fn treats_multiple_single_quotes_literally() {
                assert_eq!(
                    parse_input("echo \"'a' 'b'\""),
                    user_input("echo", &["'a' 'b'"])
                );
            }

            #[test]
            fn treats_newline_and_carriage_return_literally() {
                assert_eq!(
                    parse_input("echo \"a\nb\rc\""),
                    user_input("echo", &["a\nb\rc"])
                );
            }

            #[test]
            fn treats_shell_special_characters_literally() {
                assert_eq!(
                    parse_input("echo \"a|b;c&d>e<f*g?h#i\""),
                    user_input("echo", &["a|b;c&d>e<f*g?h#i"])
                );
            }

            #[test]
            fn parses_empty_double_quotes_as_empty_arg() {
                assert_eq!(parse_input("echo \"\""), user_input("echo", &[""]));
            }

            #[test]
            fn parses_double_quoted_arg_between_plain_args() {
                assert_eq!(
                    parse_input("echo a \"b c\" d"),
                    user_input("echo", &["a", "b c", "d"])
                );
            }

            #[test]
            fn parses_multiple_double_quoted_args() {
                assert_eq!(
                    parse_input("echo \"a b\" \"c d\""),
                    user_input("echo", &["a b", "c d"])
                );
            }

            #[test]
            fn joins_adjacent_double_quoted_strings() {
                assert_eq!(
                    parse_input("echo \"hello\"\"world\""),
                    user_input("echo", &["helloworld"])
                );
            }

            #[test]
            fn joins_adjacent_double_quoted_strings_preserving_whitespace() {
                assert_eq!(
                    parse_input("echo \"a  b\"\"c  d\""),
                    user_input("echo", &["a  bc  d"])
                );
            }

            #[test]
            fn joins_double_quoted_with_following_plain_text() {
                assert_eq!(
                    parse_input("echo \"hello\"world"),
                    user_input("echo", &["helloworld"])
                );
            }

            #[test]
            fn joins_double_quoted_with_preceding_plain_text() {
                assert_eq!(
                    parse_input("echo hello\"world\""),
                    user_input("echo", &["helloworld"])
                );
            }

            #[test]
            fn joins_double_quoted_surrounded_by_plain_text() {
                assert_eq!(
                    parse_input("echo a\"b c\"d"),
                    user_input("echo", &["ab cd"])
                );
            }

            #[test]
            fn joins_double_quoted_with_single_quoted() {
                assert_eq!(
                    parse_input("echo \"a b\"'c d'"),
                    user_input("echo", &["a bc d"])
                );
            }

            #[test]
            fn treats_double_quote_inside_single_quotes_literally() {
                assert_eq!(
                    parse_input("echo 'say \"hi\"'"),
                    user_input("echo", &["say \"hi\""])
                );
            }

            #[test]
            fn parses_double_quoted_command() {
                assert_eq!(
                    parse_input("\"my cmd\" arg"),
                    user_input("my cmd", &["arg"])
                );
            }

            #[test]
            fn applies_double_quote_parsing_to_non_echo_commands() {
                assert_eq!(
                    parse_input("cat \"my  file.txt\""),
                    user_input("cat", &["my  file.txt"])
                );
            }
        }

        #[test]
        fn applies_quote_parsing_to_non_echo_commands() {
            assert_eq!(
                parse_input("cat 'my file.txt'"),
                user_input("cat", &["my file.txt"])
            );
        }
    }
}
