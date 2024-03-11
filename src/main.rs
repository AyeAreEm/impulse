use std::{collections::HashMap, env, fs, process::exit};

#[derive(Debug, Clone)]
enum Types {
    Int,
    Str,
    Void,
    None,
}

#[derive(Debug, Clone)]
enum Keyword {
    Int,
    Str,
    Print,
    None,
}

#[derive(Debug, Clone)]
enum Expr {
    Func(Box<(Types, Expr, Expr)>), // Expr1 = FuncParams, Expr2 = FuncName
    FuncName(String),
    FuncParams(Box<Vec<(Types, Expr)>>), // Expr = VarName

    StrLit(String),
    IntLit(String),

    Var(Box<(Types, Expr, Expr)>), // Expr1 = VarName, Expr2 = Literal
    VarName(String),

    Print(Box<Expr>), // Expr = 
    EndBlock,
    None,
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

#[derive(Debug, Clone)]
struct ExprWeights {
    has_type: Types,
    has_lbrack: bool,
    has_rbrack: bool,
    has_colon: bool,
    has_name: String,
    has_lcurl: bool,
    has_predef_func: Keyword,
    has_strlit: bool,
    start_param_ix: usize,
    end_param_ix: usize,
    has_ref_name: Vec<Expr>,

    tokens: Vec<Token>,
    current_token: usize,
    keyword_map: HashMap<String, Keyword>,
    functions: Vec<String>,
    variables: Vec<String>,
}

impl ExprWeights {
    fn new(tokens: Vec<Token>) -> ExprWeights {
        let token_to_keyword: HashMap<String, Keyword> = HashMap::from([
            ("print".to_string(), Keyword::Print),
            ("int".to_string(), Keyword::Int),
            ("string".to_string(), Keyword::Str),
        ]);

        ExprWeights {
            has_type: Types::None,
            has_lbrack: false,
            has_rbrack: false,
            has_colon: false,
            has_name: String::new(),
            has_lcurl: false,
            has_predef_func: Keyword::None,
            has_strlit: false,
            start_param_ix: 0,
            end_param_ix: 0,
            has_ref_name: Vec::new(),

            tokens,
            current_token: 0,
            keyword_map: token_to_keyword,
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }

    fn clear(&mut self) {
            self.has_type = Types::None;
            self.has_lbrack = false;
            self.has_rbrack = false;
            self.has_colon = false;
            self.has_name = String::new();
            self.has_lcurl = false;
            self.has_predef_func = Keyword::None;
            self.has_strlit = false;
            self.start_param_ix = 0;
            self.end_param_ix = 0;
            self.has_ref_name = Vec::new();
    }

    fn check_exist_ident(&mut self, ident: String) -> bool {
        let mut found = false;
        for func in &self.functions {
            if func == &ident {
                self.has_ref_name.push(Expr::FuncName(ident.clone()));
                found = true; 
            }
        }

        for var in &self.variables {
            if var == &ident {
                self.has_ref_name.push(Expr::VarName(ident.clone()));
                found = true; 
            }
        }

        if !found {
            self.has_name = ident;
        }

        found
    }

    fn handle_keywords(&mut self, keyword: Keyword) {
        match keyword {
            Keyword::Print => {
                self.has_predef_func = keyword;
            },
            Keyword::Int => {
                self.has_type = Types::Int;
            },
            Keyword::Str => {
                self.has_type = Types::Str;
            },
            Keyword::None => (),
        }
    }

    fn get_params(&mut self) -> Option<Vec<(Types, Expr)>> {
        let params_slice = &self.tokens[self.start_param_ix..self.end_param_ix];
        let mut params: Vec<(Types, Expr)> = vec![];
        let mut ty = Types::None;
        let mut vn: Expr; // VarName

        for token in params_slice {
            match token {
                Token::Ident(word) => {
                    let keyword_res = self.keyword_map.get(word);
                    let keyword: (bool, Keyword) = match keyword_res {
                        Some(k) => (true, k.clone()),
                        None => (false, Keyword::None),
                    };

                    if keyword.0 {
                        match keyword.1 {
                            Keyword::Int => {
                                ty = Types::Int;
                            },
                            Keyword::Str => {
                                ty = Types::Int;
                            },
                            _ => (),
                        }
                    } else {
                        vn = Expr::VarName(word.to_string());
                        params.push((ty.clone(), vn));
                    }
                },
                _ => (),
            }
        }

        if params.is_empty() {
            None
        } else {
            Some(params)
        }
    }

    fn check_block(&mut self) -> Expr {
        // function compatability
        let mut expr = Expr::None;

        if let Types::None = self.has_type {
            return expr
        }

        if self.has_lbrack && self.has_rbrack && self.has_colon && !self.has_name.is_empty() && self.has_lcurl {
            let params_res = self.get_params();
            let params = match params_res {
                Some(params) => params,
                None => vec![(Types::None, Expr::None)],
            };

            for param in &params {
                match &param.1 {
                    Expr::VarName(name) => {
                        self.variables.push(name.to_string());
                    },
                    _ => (),
                }
            }

            expr = Expr::Func(Box::new((self.has_type.clone(), Expr::FuncParams(Box::new(params)), Expr::FuncName(self.has_name.clone()))));
            self.functions.push(self.has_name.clone());

            self.clear();
        }

        expr
    }

    fn parse_to_expr(&mut self) -> Expr {
        let mut expr = Expr::None;
        match &self.tokens[self.current_token] {
            Token::Underscore => {
                self.has_type = Types::Void;
            },
            Token::Lbrack => {
                self.has_lbrack = true; 
                self.start_param_ix = self.current_token;
            },
            Token::Rbrack => {
                self.has_rbrack = true;
                self.end_param_ix = self.current_token;
            },
            Token::Colon => {
                self.has_colon = true;
            },
            Token::Ident(word)  => {
                let keyword_res = self.keyword_map.get(word);
                let keyword: (bool, Keyword) = match keyword_res {
                    Some(k) => (true, k.clone()),
                    None => (false, Keyword::None),
                };

                if keyword.0 {
                    self.handle_keywords(keyword.1);
                } else {
                    let found_existing = self.check_exist_ident(word.to_string());

                    if let Keyword::Print = self.has_predef_func {
                        if found_existing {
                            expr = Expr::Print(Box::new(self.has_ref_name[0].clone()));
                        }
                    } 
                }
            },
            Token::Lcurl => {
                self.has_lcurl = true;
                expr = self.check_block();
            },
            Token::Str(word) => {
                self.has_strlit = true;
                match self.has_predef_func {
                    Keyword::None => (),
                    Keyword::Print => {
                        expr = Expr::Print(Box::new(Expr::StrLit(word.to_string())));
                        self.clear();
                    },
                    _ => (),
                }
            },
            Token::Quote => (),
            Token::Rcurl => {
                expr = Expr::EndBlock;
                self.clear();
            },
            Token::None => (),
        }

        self.current_token += 1;
        expr
    }

    fn parser(&mut self) -> Vec<Expr> {
        let mut program: Vec<Expr> = Vec::new();

        while self.current_token < self.tokens.len() {
            let expr = self.parse_to_expr();
            match expr {
                Expr::None => (),
                _ => program.push(expr),
            }
        }

        program
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

    // for token in &tokens {
    //     println!("{:?}", token);
    // }

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
    let mut parse = ExprWeights::new(tokens);
    let expressions = parse.parser();

    for expr in expressions {
        println!("{:?}", expr);
    }
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
