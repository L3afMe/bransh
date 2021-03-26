use logos::Logos;
use logos::Lexer;

use crate::OutputType;

fn parse_var(lex: &mut Lexer<Token>) -> Option<(String, bool)> {
    // Trim "$" from start
    let var = lex.slice()[1..].to_string();

    let var_name = if var.starts_with("ENV:") {
        (var.strip_prefix("ENV:").unwrap().to_string(), true)
    } else {
        (var, false)
    };

    Some(var_name)
}

impl From<&mut Lexer<'_, Token>> for OutputType {
    fn from(lex: &mut Lexer<Token>) -> Self {
        match lex.slice() {
            "|" => OutputType::Pipe,
            "&&" => OutputType::Depend,
            "||" => OutputType::DependNot,
            ">" => OutputType::Redirect,
            ">>" => OutputType::RedirectAppend,
            _ => OutputType::Ignore,
        }
    }

}

#[derive(Logos, Debug, PartialEq, Clone, Eq, Hash)]
pub enum Token {
    #[regex("#.*")]
    Comment,
    #[regex("[a-zA-Z0-9/_\\-:\\.~]+")]
    Word,
    #[regex(" +")]
    Whitespace,
    #[regex(r#""([^"\\]|\\r|\\t|\\u|\\n|\\")*""#, callback = |lex| lex.slice()[1..lex.slice().len() - 1].to_string())]
    StringLiteral(String),
    #[regex("[0-9]+", priority = 2)]
    NumberLiteral,
    #[regex("&")]
    Background,
    #[regex("(;|\\|\\||\\||&&|>>|>)", callback = |lex| OutputType::from(lex))]
    Output(OutputType),
    #[regex("\\$ENV:[a-zA-Z0-9_]+", priority = 2, callback = parse_var)]
    #[regex("\\$[a-zA-Z0-9_]+", callback = parse_var)]
    Variable((String, bool)),
    #[error]
    Error,
}

impl Default for Token {
    fn default() -> Self {
        Token::Error
    }
}

// Make cargo stop complaining about functions used for tests
#[allow(unused_imports, dead_code)]
mod test{
    use std::ops::Range;

    use logos::Logos;

    use crate::OutputType;
    use super::Token::{*, self};

    fn assert_seq(test_str: &str, expected: Vec<(Token, Range<usize>, &str)>) {
        let mut lexer = Token::lexer(test_str);

        for expected in expected {
            assert_eq!(lexer.next(), Some(expected.0));
            assert_eq!(lexer.span(), expected.1);
            assert_eq!(lexer.slice(), expected.2);
        }

        assert!(lexer.next().is_none());
    }

    #[test]
    fn variables() {
        assert_seq(
            "echo $ENV:HOME; echo $PROMPT",
            vec![
                (Word,                                       0.. 4, "echo"     ),
                (Whitespace,                                 4.. 5, " "        ),
                (Variable((String::from("HOME"), true)),     5..14, "$ENV:HOME"),
                (Output(OutputType::Ignore),                14..15, ";"        ),
                (Whitespace,                                15..16, " "        ),
                (Word,                                      16..20, "echo"     ),
                (Whitespace,                                20..21, " "        ),
                (Variable((String::from("PROMPT"), false)), 21..28, "$PROMPT"  ),
            ],
        )
    }

    #[test]
    fn piping() {
        assert_seq(
            "> | >> || && ;",
            vec![
                (Output(OutputType::Redirect),       0.. 1, ">" ),
                (Whitespace,                         1.. 2, " " ),
                (Output(OutputType::Pipe),           2.. 3, "|" ),
                (Whitespace,                         3.. 4, " " ),
                (Output(OutputType::RedirectAppend), 4.. 6, ">>"),
                (Whitespace,                         6.. 7, " " ),
                (Output(OutputType::DependNot),      7.. 9, "||"),
                (Whitespace,                         9..10, " " ),
                (Output(OutputType::Depend),        10..12, "&&"),
                (Whitespace,                        12..13, " " ),
                (Output(OutputType::Ignore),        13..14, ";" ),
            ],
        )
    }
}
