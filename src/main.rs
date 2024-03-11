use std::{collections::HashMap, env, fs, process::exit};

#[derive(Debug)]
enum Expr {
    Func(Func),
    FuncParam(String),
    Print(Box<Expr>),
    StrLit(String),
    VarName(String),
    IntLit(String),
    EndBlock,
}

#[derive(Debug, Clone, PartialEq)]
enum Types {
    Int,
    Str,
    Void,
    None,
}

#[derive(Debug, Clone, PartialEq)]
enum Keyword {
    Int,
    Str,
    Void,
    Print,
    None,
}

#[derive(Debug)]
struct Func {
    type_of: Types,
    name: String,
    parameters: Option<Vec<(Types, Expr)>>
}

#[derive(Debug, Clone)]
enum Token {
    Quote,
    Lbrack,
    Rbrack,

    Lcurl,
    Rcurl,
    Underscore,
    Colon,
    Ident(String),
    Str(String),
    None,
}

#[derive(Debug)]
struct Properties {
    has_name: String,
    has_type: Types,
    has_lbrack: bool,
    has_rbrack: bool,
    has_lcurl: bool,
    has_colon: bool,
    has_pass_params: (Token, String),
    has_seen_print: bool,
    calling_print: bool,
    start_params_index: usize,
    end_params_index: usize,
}

impl Properties {
    fn new() -> Properties {
        Properties{
            has_type: Types::None,
            has_name: String::from(""),
            has_colon: false,
            has_lcurl: false,
            has_lbrack: false,
            has_rbrack: false,
            has_pass_params: (Token::None, String::from("")),
            has_seen_print: false,
            calling_print: false,
            start_params_index: 0,
            end_params_index: 0,
        }
    }

    fn try_make_func(&mut self, params: Option<Vec<(Types, Expr)>>) -> Option<Func> {
        if self.has_colon && self.has_lbrack && self.has_rbrack && self.has_lcurl && !self.has_name.is_empty() && self.has_type != Types::None {
            return Some(Func {
                type_of: self.has_type.clone(),
                name: self.has_name.clone(),
                parameters: params,
            })
        }
        None
    }

    fn try_make_params(&mut self, arr: &Vec<Token>) -> Option<Vec<(Types, Expr)>> {
        if self.start_params_index >= self.end_params_index {
            return None
        }

        let param_slice = &arr[self.start_params_index..self.end_params_index];
        let mut params: Vec<(Types, Expr)> = Vec::new();
        let mut type_of = Types::None;
        let mut name: Expr;

        for token in param_slice {
            match token {
                Token::Ident(identif) => {
                    if identif == "int" {
                        type_of = Types::Int;
                    } else if identif == "string" {
                        type_of = Types::Str;
                    } else {
                        name = Expr::VarName(identif.to_string());
                        params.push((type_of.clone(), name));
                    }
                },
                // Token::Str(word) => {
                //     println!("{}", word);
                // },
                _ => (),
            }
        }

        if params.len() >= 1 {
            Some(params)
        } else {
            None
        }

    }
}

fn parser(tokens: Vec<Token>) {
    let token_to_keyword: HashMap<String, Keyword> = HashMap::from([
        ("print".to_string(), Keyword::Print),
    ]);

    let mut exprs: Vec<Expr> = Vec::new();
    let mut current_token: usize = 0;
    let mut props = Properties::new();

    while current_token < tokens.len() {
        match &tokens[current_token] {
            Token::Underscore => {
                props.has_type = Types::Void;
            },
            Token::Lbrack => {
                props.has_lbrack = true;
                if props.has_seen_print {
                    props.calling_print = true;
                    current_token += 1;
                    continue;
                }

                props.start_params_index = current_token;
            },
            Token::Rbrack => {
                props.has_rbrack = true;
                if props.calling_print {
                    props.calling_print = !false;
                    current_token += 1;
                    continue;
                }

                props.end_params_index = current_token;
            },
            Token::Colon => {
                props.has_colon = true;
            },
            Token::Ident(identif) => {
                let keyword_res = token_to_keyword.get(identif);
                let keyword: (bool, Keyword);
                match keyword_res {
                    Some(k) => keyword = (true, k.clone()),
                    None => keyword = (false, Keyword::None),
                }

                if keyword.0 {
                     match keyword.1 {
                         Keyword::Print => {
                            props.has_seen_print = true;
                         },
                         _ => println!("haven;t done yet")
                     }
                } else {
                    props.has_name = String::from(identif);

                    if let Token::Lcurl = &tokens[current_token+1] {
                        let params = props.try_make_params(&tokens);
                        props.has_lcurl = true;

                        let func_res = props.try_make_func(params);
                        props = Properties::new();
                        let func: (bool, Func);
                        match func_res {
                            Some(f) => func = (true, f),
                            None => func = (false, Func {
                                type_of: Types::None,
                                name: String::from(""),
                                parameters: None,
                            }),
                        }

                        if func.0 {
                            exprs.push(Expr::Func(func.1));
                        }

                        current_token += 1;
                    }
                }
            },
            Token::Lcurl => {
                ()
            },
            Token::Str(words) => {
                if props.calling_print {
                    exprs.push(Expr::Print(Box::new(Expr::StrLit(words.to_string()))));
                    props.calling_print = false;
                    props.has_seen_print = false;
                }
            },
            Token::Quote => (),
            Token::Rcurl => exprs.push(Expr::EndBlock),
            Token::None => (),
        }

        current_token += 1;
    }

    for expr in exprs {
        println!("{:?}", expr);
    }
}

fn tokeniser(file: String) -> Vec<Token> {
    let symb_to_token: HashMap<char, Token> = HashMap::from([
        ('(', Token::Lbrack),
        (')', Token::Rbrack),
        ('{', Token::Lcurl),
        ('}', Token::Rcurl),
        ('"', Token::Quote),
        ('_', Token::Underscore),
        (':', Token::Colon),
    ]);

    let mut tokens: Vec<Token> = Vec::new();
    let mut buf = String::new();
    let mut in_quotes = false;

    for c in file.chars() {
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

        if c == ' ' || c == '\n' || c == '\r' {
            if buf.len() > 0 {
                tokens.push(Token::Ident(buf.clone()));
                buf.clear();
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

fn build(filename: &String) {
    let file_res = fs::read_to_string(filename);
    let content = match file_res {
        Ok(content) => content,
        Err(_) => {
            println!("\x1b[91merror\x1b[0m: unable to read file");
            exit(1)
        },
    };

    let tokens = tokeniser(content);
    let parsed = parser(tokens);
}

fn usage() {
    println!("cargo run <COMMAND> [file.imp]")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("\x1b[91merror\x1b[0m: invalid usage");
        exit(1)
    }

    if &args[1] == "build" {
        build(&args[2]);
    } else if &args[1] == "help" {
        println!("help step");
    } else {
        usage();
        println!("\x1b[91merror\x1b[0m: unknown command, \x1b[93m{}\x1b[0m", &args[1]);
        exit(1)
    }
}
