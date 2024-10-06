use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Token {
    Quote,
    SingleQuote,

    Macro,
    Lbrack,
    Rbrack,

    Lsquare,
    Rsquare,

    Lcurl,
    Rcurl,
    Underscore,
    Colon,
    SemiColon,
    Pipe,
    Newline,

    Equal,
    SmallerThan,
    BiggerThan,
    Exclaim,

    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,

    Caret,
    Ampersand,
    Dollar,

    True,
    False,

    Ident(String),
    Str(String),
    Char(String),
    Int(String),
    // Digit(char), // this is a single digit inside an Int
}

fn check_ident(word: &String) -> Token {
    match word.as_str() {
        "true" => Token::True,
        "false" => Token::False,
        _ => Token::Ident(word.to_owned())
    }
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
        ('^', Token::Caret),
        ('&', Token::Ampersand),
        ('$', Token::Dollar),
        // ('.', Token::Dot),
    ]);

    let mut tokens: Vec<Token> = Vec::new();
    let mut buf = String::new();

    let mut in_quotes = false;
    let mut in_single_quotes = false;
    let mut in_squares = false;
    let mut square_occurences = 0;
    let mut comment_line = false;

    for c in file.chars() {
        if c == '#' && !in_squares && !in_quotes && !in_single_quotes {
            comment_line = true;
            continue;
        }

        if comment_line && c == '\n' {
            tokens.push(Token::Newline);
            comment_line = false;
            continue;
        }

        if comment_line {
            continue;
        }

        if c == '[' && !in_quotes && !in_single_quotes {
            square_occurences += 1;
            if !in_squares {
                if buf.len() > 0 {
                    tokens.push(check_ident(&buf));
                    buf.clear();
                }

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

        if c == ']' && !in_quotes && !in_single_quotes {
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

        if c == '\'' && !in_quotes {
            in_single_quotes = !in_single_quotes;

            if !in_single_quotes {
                tokens.push(Token::Char(buf.clone()));
                buf.clear();
            }
            tokens.push(Token::SingleQuote);
            continue;
        }

        if in_single_quotes {
            buf.push(c);
            continue;
        }

        if c == '"' && !in_single_quotes {
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

        if c == ' ' || c == '\n' || c == '\r' || c == ';' || c == '\t' {
            if buf.len() > 0 {
                tokens.push(check_ident(&buf));
                buf.clear();
            }

            if c == '\n' {
                tokens.push(Token::Newline);
            } else if c == ';' {
                tokens.push(Token::SemiColon);
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
                tokens.push(check_ident(&buf));
                buf.clear();
            }
            tokens.push(token.1);
        }
    }

    tokens
}
