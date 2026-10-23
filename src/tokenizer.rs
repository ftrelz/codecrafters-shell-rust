use crate::UserInput;

static DELIMS: [u8; 5] = *b" \t\n\r\'";

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

pub fn parse_user_input(raw_input: &str) -> UserInput {
    let mut tokens: Vec<String> = Vec::new();

    let remaining = raw_input.as_bytes();
    let mut index = 0;
    let mut prev_char: Option<u8> = None;
    while index < remaining.len() {
        if !DELIMS.contains(&remaining[index]) {
            let mut token: &[u8] = b"";
            if prev_char.is_none_or(|c| c.is_ascii_whitespace()) {
                token = parse_token(&remaining[index..], |c| c.is_ascii_whitespace());
            } else if prev_char.unwrap() == b'\'' {
                token = parse_token(&remaining[index..], |c| *c == b'\'');
            }
            tokens.push(String::from_utf8(token.to_vec()).unwrap());
            index += token.len();
        } else {
            index += 1;
        }
        prev_char = Some(remaining[index - 1]);
    }

    let mut iter = tokens.into_iter();
    UserInput {
        command: iter.next().unwrap_or_default(),
        arguments: iter.collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{UserInput, tokenizer::parse_user_input};

    fn build_userinput(command: &str, args: &[&str]) -> UserInput {
        UserInput {
            command: command.to_string(),
            arguments: args.iter().copied().map(String::from).collect(),
        }
    }

    #[test]
    fn parses_empty_input() {
        let user_input = "";
        let expected = build_userinput("", &[]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_command() {
        let user_input = "echo";
        let expected = build_userinput("echo", &[]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_command_with_args() {
        let user_input = "echo hello world!";
        let expected = build_userinput("echo", &["hello", "world!"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_multiple_leading_trailing_spaces() {
        let user_input = " echo  hello   world!  ";
        let expected = build_userinput("echo", &["hello", "world!"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_other_ws_chars() {
        let user_input = "\techo\t\thello world!\r\n";
        let expected = build_userinput("echo", &["hello", "world!"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_quoted_command() {
        let user_input = "'hello world' x";
        let expected = build_userinput("hello world", &["x"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_single_quoted_args() {
        let user_input = "echo 'hello world!'";
        let expected = build_userinput("echo", &["hello world!"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_quotes_after_plain() {
        let user_input = "echo hello 'a b'";
        let expected = build_userinput("echo", &["hello", "a b"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_plain_after_quoted() {
        let user_input = "echo 'a b' c d";
        let expected = build_userinput("echo", &["a b", "c", "d"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_whitespace_only_input() {
        for user_input in ["   ", "\n", "\t \r\n"] {
            let expected = build_userinput("", &[]);
            assert_eq!(parse_user_input(user_input), expected, "{user_input:?}");
        }
    }

    #[test]
    fn parses_consecutive_quoted_args() {
        let user_input = "echo 'a' 'b'";
        let expected = build_userinput("echo", &["a", "b"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_multiple_spaces_inside_quotes() {
        let user_input = "echo 'a  b'";
        let expected = build_userinput("echo", &["a  b"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_other_ws_chars_inside_quotes() {
        let user_input = "echo 'a\tb'";
        let expected = build_userinput("echo", &["a\tb"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input:?}");
    }

    #[test]
    fn parses_quoted_arg_between_plain_args() {
        let user_input = "echo a 'b c' d";
        let expected = build_userinput("echo", &["a", "b c", "d"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_trailing_newline_after_quoted_arg() {
        let user_input = "echo 'a b'\n";
        let expected = build_userinput("echo", &["a b"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input:?}");
    }

    #[test]
    fn parses_empty_quoted_arg() {
        let user_input = "echo ''";
        let expected = build_userinput("echo", &[]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_unterminated_quote_as_rest_of_input() {
        let user_input = "echo 'abc def";
        let expected = build_userinput("echo", &["abc def"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    fn parses_non_ascii_args() {
        let user_input = "echo héllo 'wörld 日本'";
        let expected = build_userinput("echo", &["héllo", "wörld 日本"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    #[ignore = "adjacent quoted/unquoted text is not concatenated yet"]
    fn parses_adjacent_quoted_and_plain_text() {
        let user_input = "echo hello'world'";
        let expected = build_userinput("echo", &["helloworld"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }

    #[test]
    #[ignore = "double quotes are not supported yet"]
    fn parses_double_quoted_args() {
        let user_input = "echo \"a b\"";
        let expected = build_userinput("echo", &["a b"]);
        assert_eq!(parse_user_input(user_input), expected, "{user_input}");
    }
}
