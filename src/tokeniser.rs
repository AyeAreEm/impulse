use std::collections::HashMap;

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
    Return,

    Ident(String),
    Str(String),
    Int(String),
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
    ]);

    let mut tokens: Vec<Token> = Vec::new();
    let mut buf = String::new();
    let mut in_quotes = false;
    let mut in_squares = false;

    for c in file.chars() {
        if buf == String::from("return") {
            tokens.push(Token::Return);
            buf.clear();
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

        if c == '[' {
            tokens.push(Token::Lsquare);
            in_squares = true;
            continue;
        }

        if in_squares && c != ']' {
            buf.push(c);
            continue;
        }

        if c == ']' {
            in_squares = false;
            tokens.push(Token::Int(buf.clone()));
            tokens.push(Token::Rsquare);
            buf.clear();
            continue;
        }

        if c == ' ' || c == '\n' || c == '\r' {
            if buf.len() > 0 {
                tokens.push(Token::Ident(buf.clone()));
                buf.clear();
            }

            if c == '\n' {
                tokens.push(Token::Newline);
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
