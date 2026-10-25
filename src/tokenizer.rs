#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub enum TokenKind {
    #[default]
    Word,
    Space,
    Tab,
    Nl,
    Cr,
    Apostrophe,
    Quote,
    Backslash,
}

impl TokenKind {
    pub fn from_ascii(s: u8) -> TokenKind {
        match s {
            b' ' => TokenKind::Space,
            b'\t' => TokenKind::Tab,
            b'\n' => TokenKind::Nl,
            b'\r' => TokenKind::Cr,
            b'\'' => TokenKind::Apostrophe,
            b'"' => TokenKind::Quote,
            b'\\' => TokenKind::Backslash,
            _ => TokenKind::Word,
        }
    }

    pub fn is_whitespace(&self) -> bool {
        matches!(
            self,
            TokenKind::Space | TokenKind::Tab | TokenKind::Nl | TokenKind::Cr
        )
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub raw: String,
}

static DELIMS: [u8; 7] = *b" \t\n\r\'\"\\";

fn parse_token(input: &[u8], is_delim: impl Fn(&u8) -> bool) -> &[u8] {
    let mut token_len = 0;
    for char in input {
        if is_delim(char) {
            break;
        }
        token_len += 1;
    }
    &input[..token_len]
}

pub fn tokenize(raw_input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let ascii_input = raw_input.as_bytes();
    let mut index = 0;
    while index < ascii_input.len() {
        let token_kind = TokenKind::from_ascii(ascii_input[index]);
        let token_raw: String = match token_kind {
            TokenKind::Word => String::from_utf8(
                parse_token(&ascii_input[index..], |c| DELIMS.contains(c)).to_vec(),
            )
            .unwrap(),
            _ => String::from_utf8(vec![ascii_input[index]]).unwrap(),
        };
        index += token_raw.len();
        tokens.push(Token {
            kind: token_kind,
            raw: token_raw,
        });
    }

    tokens
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{Token, TokenKind, tokenize};
    use TokenKind::{Apostrophe, Backslash, Cr, Nl, Quote, Space, Tab, Word};

    fn tok(kind: TokenKind, raw: &str) -> Token {
        Token {
            kind,
            raw: raw.to_string(),
        }
    }

    #[test]
    fn tokenizes_empty_input() {
        let user_input = "";
        let expected = vec![];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_command() {
        let user_input = "echo";
        let expected = vec![tok(Word, "echo")];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_command_with_args() {
        let user_input = "echo hello world!";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "hello"),
            tok(Space, " "),
            tok(Word, "world!"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_multiple_leading_trailing_spaces() {
        let user_input = " echo  hello   world!  ";
        let expected = vec![
            tok(Space, " "),
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Space, " "),
            tok(Word, "hello"),
            tok(Space, " "),
            tok(Space, " "),
            tok(Space, " "),
            tok(Word, "world!"),
            tok(Space, " "),
            tok(Space, " "),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_other_ws_chars() {
        let user_input = "\techo\t\thello world!\r\n";
        let expected = vec![
            tok(Tab, "\t"),
            tok(Word, "echo"),
            tok(Tab, "\t"),
            tok(Tab, "\t"),
            tok(Word, "hello"),
            tok(Space, " "),
            tok(Word, "world!"),
            tok(Cr, "\r"),
            tok(Nl, "\n"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input:?}");
    }

    #[test]
    fn tokenizes_quoted_command() {
        let user_input = "'hello world' x";
        let expected = vec![
            tok(Apostrophe, "'"),
            tok(Word, "hello"),
            tok(Space, " "),
            tok(Word, "world"),
            tok(Apostrophe, "'"),
            tok(Space, " "),
            tok(Word, "x"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_single_quoted_args() {
        let user_input = "echo 'hello world!'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "hello"),
            tok(Space, " "),
            tok(Word, "world!"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_quotes_after_plain() {
        let user_input = "echo hello 'a b'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "hello"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Word, "b"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_plain_after_quoted() {
        let user_input = "echo 'a b' c d";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Word, "b"),
            tok(Apostrophe, "'"),
            tok(Space, " "),
            tok(Word, "c"),
            tok(Space, " "),
            tok(Word, "d"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_whitespace_only_input() {
        let cases = [
            (
                "   ",
                vec![tok(Space, " "), tok(Space, " "), tok(Space, " ")],
            ),
            ("\n", vec![tok(Nl, "\n")]),
            (
                "\t \r\n",
                vec![
                    tok(Tab, "\t"),
                    tok(Space, " "),
                    tok(Cr, "\r"),
                    tok(Nl, "\n"),
                ],
            ),
        ];
        for (user_input, expected) in cases {
            assert_eq!(tokenize(user_input), expected, "{user_input:?}");
        }
    }

    #[test]
    fn tokenizes_consecutive_quoted_args() {
        let user_input = "echo 'a' 'b'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "a"),
            tok(Apostrophe, "'"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "b"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_multiple_spaces_inside_quotes() {
        let user_input = "echo 'a  b'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Space, " "),
            tok(Word, "b"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_other_ws_chars_inside_quotes() {
        let user_input = "echo 'a\tb'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "a"),
            tok(Tab, "\t"),
            tok(Word, "b"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input:?}");
    }

    #[test]
    fn tokenizes_quoted_arg_between_plain_args() {
        let user_input = "echo a 'b c' d";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "b"),
            tok(Space, " "),
            tok(Word, "c"),
            tok(Apostrophe, "'"),
            tok(Space, " "),
            tok(Word, "d"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_trailing_newline_after_quoted_arg() {
        let user_input = "echo 'a b'\n";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Word, "b"),
            tok(Apostrophe, "'"),
            tok(Nl, "\n"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input:?}");
    }

    #[test]
    fn tokenizes_empty_quoted_arg() {
        let user_input = "echo ''";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_unterminated_quote_as_rest_of_input() {
        let user_input = "echo 'abc def";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "abc"),
            tok(Space, " "),
            tok(Word, "def"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_non_ascii_args() {
        let user_input = "echo héllo 'wörld 日本'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "héllo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "wörld"),
            tok(Space, " "),
            tok(Word, "日本"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_adjacent_quoted_and_plain_text() {
        let user_input = "echo hello'world'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "hello"),
            tok(Apostrophe, "'"),
            tok(Word, "world"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_adjacent_quoted() {
        let user_input = "echo 'hello''world'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "hello"),
            tok(Apostrophe, "'"),
            tok(Apostrophe, "'"),
            tok(Word, "world"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn ignores_empty_quotes() {
        let user_input = "echo hello''world";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "hello"),
            tok(Apostrophe, "'"),
            tok(Apostrophe, "'"),
            tok(Word, "world"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_double_quoted_args() {
        let user_input = "echo \"a b\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Word, "b"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_double_quoted_command() {
        let user_input = "\"hello world\" x";
        let expected = vec![
            tok(Quote, "\""),
            tok(Word, "hello"),
            tok(Space, " "),
            tok(Word, "world"),
            tok(Quote, "\""),
            tok(Space, " "),
            tok(Word, "x"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_multiple_spaces_inside_double_quotes() {
        let user_input = "echo \"a  b\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Space, " "),
            tok(Word, "b"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_other_ws_chars_inside_double_quotes() {
        let user_input = "echo \"a\tb\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "a"),
            tok(Tab, "\t"),
            tok(Word, "b"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input:?}");
    }

    #[test]
    fn tokenizes_empty_double_quoted_arg() {
        let user_input = "echo \"\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_consecutive_double_quoted_args() {
        let user_input = "echo \"a\" \"b\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "a"),
            tok(Quote, "\""),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "b"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_adjacent_double_quoted() {
        let user_input = "echo \"hello\"\"world\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "hello"),
            tok(Quote, "\""),
            tok(Quote, "\""),
            tok(Word, "world"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_adjacent_double_quoted_and_plain_text() {
        let user_input = "echo hello\"world\"!";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "hello"),
            tok(Quote, "\""),
            tok(Word, "world"),
            tok(Quote, "\""),
            tok(Word, "!"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_double_quoted_arg_between_plain_args() {
        let user_input = "echo a \"b c\" d";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "a"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "b"),
            tok(Space, " "),
            tok(Word, "c"),
            tok(Quote, "\""),
            tok(Space, " "),
            tok(Word, "d"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_apostrophe_inside_double_quotes() {
        let user_input = "echo \"it's\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "it"),
            tok(Apostrophe, "'"),
            tok(Word, "s"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_double_quote_inside_single_quotes() {
        let user_input = "echo 'say \"hi\"'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "say"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "hi"),
            tok(Quote, "\""),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_unterminated_double_quote_as_rest_of_input() {
        let user_input = "echo \"abc def";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "abc"),
            tok(Space, " "),
            tok(Word, "def"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_non_ascii_double_quoted_args() {
        let user_input = "echo \"wörld 日本\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "wörld"),
            tok(Space, " "),
            tok(Word, "日本"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_lone_backslash() {
        let user_input = "\\";
        let expected = vec![tok(Backslash, "\\")];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_backslash_as_its_own_token() {
        let user_input = "echo a\\b";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "a"),
            tok(Backslash, "\\"),
            tok(Word, "b"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_escaped_space() {
        let user_input = "echo hello\\ world";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "hello"),
            tok(Backslash, "\\"),
            tok(Space, " "),
            tok(Word, "world"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_escaped_space_in_command() {
        let user_input = "my\\ cmd arg";
        let expected = vec![
            tok(Word, "my"),
            tok(Backslash, "\\"),
            tok(Space, " "),
            tok(Word, "cmd"),
            tok(Space, " "),
            tok(Word, "arg"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_leading_backslash() {
        let user_input = "\\echo hello";
        let expected = vec![
            tok(Backslash, "\\"),
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "hello"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_trailing_backslash() {
        let user_input = "echo a\\";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "a"),
            tok(Backslash, "\\"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_consecutive_backslashes() {
        let user_input = "echo \\\\";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Backslash, "\\"),
            tok(Backslash, "\\"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_escaped_tab() {
        let user_input = "a\\\tb";
        let expected = vec![
            tok(Word, "a"),
            tok(Backslash, "\\"),
            tok(Tab, "\t"),
            tok(Word, "b"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input:?}");
    }

    #[test]
    fn tokenizes_escaped_single_quote() {
        let user_input = "echo it\\'s";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Word, "it"),
            tok(Backslash, "\\"),
            tok(Apostrophe, "'"),
            tok(Word, "s"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_escaped_double_quote() {
        let user_input = "echo \\\"a\\\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Backslash, "\\"),
            tok(Quote, "\""),
            tok(Word, "a"),
            tok(Backslash, "\\"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_escaped_regular_letters() {
        let user_input = "a\\nb\\tc";
        let expected = vec![
            tok(Word, "a"),
            tok(Backslash, "\\"),
            tok(Word, "nb"),
            tok(Backslash, "\\"),
            tok(Word, "tc"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_escaped_special_characters_as_words() {
        let user_input = "echo \\$HOME \\* \\?";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Backslash, "\\"),
            tok(Word, "$HOME"),
            tok(Space, " "),
            tok(Backslash, "\\"),
            tok(Word, "*"),
            tok(Space, " "),
            tok(Backslash, "\\"),
            tok(Word, "?"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_backslash_inside_double_quotes() {
        let user_input = "echo \"a\\nb\"";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Quote, "\""),
            tok(Word, "a"),
            tok(Backslash, "\\"),
            tok(Word, "nb"),
            tok(Quote, "\""),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_backslash_inside_single_quotes() {
        let user_input = "echo 'a\\ b'";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Apostrophe, "'"),
            tok(Word, "a"),
            tok(Backslash, "\\"),
            tok(Space, " "),
            tok(Word, "b"),
            tok(Apostrophe, "'"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }

    #[test]
    fn tokenizes_non_ascii_after_backslash() {
        let user_input = "echo \\ö\\日";
        let expected = vec![
            tok(Word, "echo"),
            tok(Space, " "),
            tok(Backslash, "\\"),
            tok(Word, "ö"),
            tok(Backslash, "\\"),
            tok(Word, "日"),
        ];
        assert_eq!(tokenize(user_input), expected, "{user_input}");
    }
}
