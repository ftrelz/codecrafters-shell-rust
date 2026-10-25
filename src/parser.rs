use std::{iter::Peekable, vec::IntoIter};

use crate::{
    UserInput,
    tokenizer::{Token, TokenKind, tokenize},
};

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

fn parse_term(tokens: &mut Peekable<IntoIter<Token>>) -> Option<Token> {
    let mut parsed_tokens: Vec<Token> = vec![];
    while let Some(tok) = tokens.next() {
        let new_tok = match tok.kind {
            TokenKind::Quote | TokenKind::Apostrophe => Some(parse_quoted(tokens, &tok.kind)),
            TokenKind::Word => Some(tok),
            TokenKind::Backslash => tokens.next(),
            TokenKind::Space | TokenKind::Tab | TokenKind::Nl | TokenKind::Cr => break,
        };
        if new_tok.is_some() {
            parsed_tokens.push(new_tok.unwrap());
        }
    }

    if parsed_tokens.is_empty() {
        return None;
    }

    Some(combine_tokens_into_word(&parsed_tokens))
}

pub fn parse_input(input: &str) -> UserInput {
    let mut tokens = tokenize(input).into_iter().peekable();
    skip_whitespace(&mut tokens);

    let command = parse_term(&mut tokens);
    skip_whitespace(&mut tokens);

    let mut final_tokens: Vec<Token> = vec![];
    while let Some(token) = parse_term(&mut tokens) {
        final_tokens.push(token.clone());
        skip_whitespace(&mut tokens);
    }

    UserInput {
        command: command.unwrap_or_default().raw,
        arguments: final_tokens.into_iter().map(|t| t.raw).collect(),
    }
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

        mod backslash {
            use super::*;

            // escaping special characters

            #[test]
            fn escaped_space_joins_words_into_one_arg() {
                assert_eq!(
                    parse_input("echo hello\\ world"),
                    user_input("echo", &["hello world"])
                );
            }

            #[test]
            fn escaped_space_in_command() {
                assert_eq!(parse_input("my\\ cmd arg"), user_input("my cmd", &["arg"]));
            }

            #[test]
            fn preserves_multiple_escaped_spaces() {
                assert_eq!(
                    parse_input("echo a\\ \\ \\ b"),
                    user_input("echo", &["a   b"])
                );
            }

            #[test]
            fn escaped_space_alone_is_an_arg() {
                assert_eq!(parse_input("echo \\ "), user_input("echo", &[" "]));
            }

            #[test]
            fn escaped_space_at_start_of_arg() {
                assert_eq!(
                    parse_input("echo \\ hello"),
                    user_input("echo", &[" hello"])
                );
            }

            #[test]
            fn escaped_tab_is_literal() {
                assert_eq!(parse_input("echo a\\\tb"), user_input("echo", &["a\tb"]));
            }

            #[test]
            fn escaped_single_quote_is_literal() {
                assert_eq!(parse_input("echo it\\'s"), user_input("echo", &["it's"]));
            }

            #[test]
            fn escaped_double_quote_is_literal() {
                assert_eq!(
                    parse_input("echo say\\\"hi\\\""),
                    user_input("echo", &["say\"hi\""])
                );
            }

            #[test]
            fn escaped_quote_does_not_start_a_quoted_region() {
                assert_eq!(
                    parse_input("echo \\'a b\\'"),
                    user_input("echo", &["'a", "b'"])
                );
                assert_eq!(
                    parse_input("echo \\\"a b\\\""),
                    user_input("echo", &["\"a", "b\""])
                );
            }

            #[test]
            fn escaped_backslash_is_a_single_backslash() {
                assert_eq!(parse_input("echo a\\\\b"), user_input("echo", &["a\\b"]));
            }

            #[test]
            fn escaped_backslash_does_not_escape_following_space() {
                assert_eq!(
                    parse_input("echo a\\\\ b"),
                    user_input("echo", &["a\\", "b"])
                );
            }

            #[test]
            fn escaped_dollar_is_literal() {
                assert_eq!(parse_input("echo \\$HOME"), user_input("echo", &["$HOME"]));
            }

            #[test]
            fn escaped_glob_characters_are_literal() {
                assert_eq!(parse_input("echo \\*"), user_input("echo", &["*"]));
                assert_eq!(parse_input("echo \\?"), user_input("echo", &["?"]));
                assert_eq!(
                    parse_input("echo \\*.rs file\\?"),
                    user_input("echo", &["*.rs", "file?"])
                );
            }

            #[test]
            fn escaped_shell_operators_are_literal() {
                assert_eq!(
                    parse_input("echo \\|\\;\\&\\<\\>\\#\\~\\(\\)"),
                    user_input("echo", &["|;&<>#~()"])
                );
            }

            // escaping characters with no special meaning

            #[test]
            fn escaped_letter_n_is_plain_n() {
                assert_eq!(parse_input("echo a\\nb"), user_input("echo", &["anb"]));
            }

            #[test]
            fn escaped_letter_t_is_plain_t() {
                assert_eq!(parse_input("echo a\\tb"), user_input("echo", &["atb"]));
            }

            #[test]
            fn escaped_regular_characters_are_unchanged() {
                assert_eq!(
                    parse_input("echo \\h\\e\\l\\l\\o \\1\\2\\3"),
                    user_input("echo", &["hello", "123"])
                );
            }

            #[test]
            fn escaped_first_letter_of_command() {
                assert_eq!(
                    parse_input("\\echo hello world"),
                    user_input("echo", &["hello", "world"])
                );
            }

            #[test]
            fn escaped_letters_inside_command() {
                assert_eq!(parse_input("ec\\ho hello"), user_input("echo", &["hello"]));
            }

            #[test]
            fn escaped_non_ascii_character_is_unchanged() {
                assert_eq!(parse_input("echo \\ö\\日"), user_input("echo", &["ö日"]));
            }

            // the backslash itself is removed

            #[test]
            fn backslash_is_removed_after_escaping() {
                let parsed = parse_input("echo \\a\\ \\b");
                assert_eq!(parsed, user_input("echo", &["a b"]));
                assert!(parsed.arguments.iter().all(|a| !a.contains('\\')));
            }

            // interaction with other parsing

            #[test]
            fn escaped_chars_join_with_adjacent_plain_text() {
                assert_eq!(
                    parse_input("echo pre\\ fix\\!post"),
                    user_input("echo", &["pre fix!post"])
                );
            }

            #[test]
            fn escaped_space_joins_with_single_quoted_text() {
                assert_eq!(
                    parse_input("echo 'a b'\\ c"),
                    user_input("echo", &["a b c"])
                );
            }

            #[test]
            fn escaped_space_joins_with_double_quoted_text() {
                assert_eq!(
                    parse_input("echo \"a b\"\\ c"),
                    user_input("echo", &["a b c"])
                );
            }

            #[test]
            fn escaped_and_unescaped_spaces_mix() {
                assert_eq!(
                    parse_input("echo a\\ b c\\ d  e"),
                    user_input("echo", &["a b", "c d", "e"])
                );
            }

            #[test]
            fn applies_escaping_to_non_echo_commands() {
                assert_eq!(
                    parse_input("cat my\\ file.txt"),
                    user_input("cat", &["my file.txt"])
                );
            }

            // backslashes inside quotes are NOT escapes

            #[test]
            fn backslash_is_literal_inside_double_quotes() {
                assert_eq!(
                    parse_input("echo \"hello world\\n\""),
                    user_input("echo", &["hello world\\n"])
                );
            }

            #[test]
            fn backslash_is_literal_inside_single_quotes() {
                assert_eq!(
                    parse_input("echo 'hello\\ world\\n'"),
                    user_input("echo", &["hello\\ world\\n"])
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
