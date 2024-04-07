use std::{process::exit, collections::HashMap};

#[derive(Debug, Clone)]
pub enum Token {
    Quote,
    Macro,
    Lbrack,
    Rbrack,

    Lsquare,
    Rsquare,

    Lcurl,
    Rcurl,
    Underscore,
    Colon,
    Pipe,
    Newline,

    Equal,
    SmallerThan,
    BiggerThan,
    Exclaim,

    Plus,
    Minus,
    Multiple,
    Divide,

    Ident(String),
    Str(String),
    Int(String),
    Digit(char), // this is a single digit inside an Int
}

pub fn tokeniser(file: String) -> Vec<Token> {
    let symb_to_token: HashMap<char, Token> = HashMap::from([
        ('(', Token::Lbrack),
        (')', Token::Rbrack),
        ('{', Token::Lcurl),
        ('}', Token::Rcurl),
        ('[', Token::Lsquare),
        (']', Token::Rsquare),
        ('"', Token::Quote),
        ('_', Token::Underscore),
        (':', Token::Colon),
        ('@', Token::Macro),
        ('|', Token::Pipe),
        ('=', Token::Equal),
        ('<', Token::SmallerThan),
        ('>', Token::BiggerThan),
        ('!', Token::Exclaim),
    ]);

    let mut line_num: i64 = -1;

    let mut tokens: Vec<Token> = Vec::new();
    let mut buf = String::new();

    let mut in_quotes = false;
    let mut in_squares = false;
    let mut square_occurences = 0;
    let mut comment_line = false;

    for c in file.chars() {
        if comment_line {
            continue;
        }

        if c == '"' {
            in_quotes = !in_quotes;

            if !in_quotes {
                tokens.push(Token::Str(buf.clone()));
                buf.clear();
            }
            tokens.push(Token::Quote);
            continue;
        }

        if in_quotes {
            buf.push(c);
            continue;
        }

        if c == '_' {
            if !buf.is_empty() {
                buf.push(c);
            } else {
                tokens.push(Token::Underscore);
            }
            continue;
        }

        if c == '#' {
            comment_line = true;
            continue;
        }

        if c == '[' {
            square_occurences += 1;
            if !in_squares {
                in_squares = true;
                tokens.push(Token::Lsquare);
            } else {
                buf.push(c);
            }

            continue;
        }

        if in_squares && c != ']' {
            buf.push(c);
            continue;
        }

        if c == ']' {
            square_occurences -= 1;
            if in_squares && square_occurences == 0 {
                in_squares = false;
                tokens.push(Token::Int(buf.clone()));
                tokens.push(Token::Rsquare);
                buf.clear();
            } else if in_squares {
                buf.push(c);
            }

            continue;
        }

        if c == ' ' || c == '\n' || c == '\r' {
            if buf.len() > 0 {
                tokens.push(Token::Ident(buf.clone()));
                buf.clear();
            }

            if c == '\n' {
                if comment_line {
                    comment_line = false;
                }
                tokens.push(Token::Newline);
                line_num += 1;
            }
            continue;
        }

        let token_res = symb_to_token.get(&c);
        let token: (bool, Token);
        match token_res {
            Some(t) => token = (true, t.clone()),
            None => {
                buf.push(c);
                token = (false, Token::Quote)
            }
        }

        if token.0 {
            if buf.len() > 0 {
                tokens.push(Token::Ident(buf.clone()));
                buf.clear();
            }
            tokens.push(token.1);
        }
    }

    tokens
}
