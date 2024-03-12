use std::{collections::HashMap, env, fs, process::{exit, Command}};

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
    Underscore,
    None,
}

#[derive(Debug, Clone)]
enum Expr {
    Func(Box<(Types, Expr, Expr)>), // Expr1 = FuncParams, Expr2 = FuncName
    FuncName(String),
    FuncParams(Box<Vec<(Types, Expr)>>), // Expr = VarName


    Var(Box<(Types, Expr, Expr)>), // Expr1 = VarName, Expr2 = Literal
    VarName(String),

    Print(Box<Vec<Expr>>), // Expr = 

    StrLit(String),
    IntLit(String),
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

    // expr_buffer: Vec<Expr>,

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
            ("_".to_string(), Keyword::Underscore), 
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

            // expr_buffer: Vec::new(),

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
            Keyword::Print => self.has_predef_func = keyword,
            Keyword::Int => self.has_type = Types::Int,
            Keyword::Str => self.has_type = Types::Str,
            Keyword::Underscore => self.has_type = Types::Void,
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

    fn check_print(&mut self) -> Expr {
        let expr = Expr::None;
        let mut met_criteria = false;

        match self.has_type {
            Types::None => {
                if self.has_lbrack && self.has_rbrack && !self.has_colon && !self.has_lcurl {
                    // expr = Expr::Print(Box::new(self.expr_buffer));
                    met_criteria = true;
                }
            },
            _ => (),
        }
        if !met_criteria {
            return expr
        }

        let params_slice = &self.tokens[self.start_param_ix..self.end_param_ix];
        let mut params: Vec<Expr> = Vec::new();
        for token in params_slice {
            match token {
                Token::Ident(word) => {
                    let int_word = word.parse::<i32>();
                    match int_word {
                        Ok(n) => params.push(Expr::IntLit(n.to_string())),
                        Err(_) => {
                            for var in &self.variables {
                                if var == word {
                                    params.push(Expr::VarName(word.to_string()));
                                }
                            }
                        },
                    }
                },
                Token::Str(word) => params.push(Expr::StrLit(word.to_string())),
                _ => (),
            }
        }

        Expr::Print(Box::new(params))
    }

    fn check_func(&mut self) -> Option<Expr> {
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

            let expr = Expr::Func(Box::new((self.has_type.clone(), Expr::FuncParams(Box::new(params)), Expr::FuncName(self.has_name.clone()))));
            self.functions.push(self.has_name.clone());
            return Some(expr)
        }

        if !self.has_name.is_empty() {
            println!("\x1b[93mwarning\x1b[0m: tried to make a function without a name.");
        }

        None
    }

    fn check_block(&mut self) -> Expr {
        // function compatability
        let mut expr = Expr::None;

        if let Types::None = self.has_type {
            return expr
        }

        match self.check_func() {
            Some(e) => expr = e,
            None => (),
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

                match self.has_predef_func {
                    Keyword::Print => {
                        expr = self.check_print();
                    },
                    _ => (),
                }
            },
            Token::Colon => {
                self.has_colon = true;
            },
            Token::Ident(word)  => {
                // first convert to int
                // let mut num = Expr::None;
                // let int_word = word.parse::<i32>();
                // match int_word {
                //     Ok(n) => num = Expr::IntLit(n.to_string()),
                //     Err(_) => (),
                // }

                // if not check for keywords
                let keyword_res = self.keyword_map.get(word);
                let keyword: (bool, Keyword) = match keyword_res {
                    Some(k) => (true, k.clone()),
                    None => (false, Keyword::None),
                };

                if keyword.0 {
                    self.handle_keywords(keyword.1);
                } else {
                    // found variable or constant, print that thang
                    self.check_exist_ident(word.to_string());
                    // let found_existing = self.check_exist_ident(word.to_string());
                    // match self.has_predef_func {
                    //     Keyword::Print => {
                    //         if found_existing {
                    //             for ref_name in &self.has_ref_name {
                    //                 self.expr_buffer.push(ref_name.clone());
                    //             }
                    //         } else {
                    //             self.expr_buffer.push(num);
                    //         }
                    //     },
                    //     _ => (),
                    // }
                }
            },
            Token::Lcurl => {
                self.has_lcurl = true;
                expr = self.check_block();
            },
            Token::Str(word) => {
                self.has_strlit = true;

                // add variable making thing
            },
            Token::Quote => (),
            Token::Rcurl => {
                expr = Expr::EndBlock;
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
                _ => {
                    program.push(expr);
                    self.clear();
                },
            }
        }

        program
    }
}

fn generate(expressions: Vec<Expr>) {
    let mut imports = String::new();
    let mut code = String::new();

    for (index, expr) in expressions.into_iter().enumerate() {
        match expr {
            Expr::Print(value) => {
                imports.push_str("#include <stdio.h>\n");

                let mut print_code = String::from(format!("printf(\""));
                let mut param_buf = String::new();
                let mut var_buf = String::new();

                for v in *value {
                    match v {
                        Expr::StrLit(string) => {
                                param_buf.push_str(&format!("{string}"));
                        },
                        Expr::IntLit(integer) => {
                                param_buf.push_str(&format!("%d"));
                                var_buf.push_str(&format!(", {integer}"));
                        },
                        Expr::VarName(_) => (),
                        _ => (),
                    }

                    // match v {
                    //     Expr::StrLit(str) => code.push_str(&format!("printf(\"{str}\");")),
                    //     Expr::IntLit(integer) => code.push_str(&format!("printf(\"%d\", {integer});")),
                    //     _ => (),
                    // }
                }

                print_code.push_str(&param_buf);
                print_code.push('"');
                print_code.push_str(&var_buf);
                print_code.push_str(");");
                code.push_str(&print_code);
            }
            Expr::EndBlock => {
                code.push_str("}");
            },
            Expr::Func(f) => {
                let ty = &f.0;
                let params = &f.1;
                let name = &f.2;
                let f_ty: String;
                let mut f_params = String::new();
                let mut f_params_touched = false;
                let mut f_name = String::new();

                match params {
                    Expr::FuncParams(ps) => {
                        for p in *ps.clone() {
                            let mut this_ty = String::new();
                            let mut this_name = String::new();

                            match p.0 {
                                Types::Void => this_ty = String::from("void"),
                                Types::Int => this_ty = String::from("int"),
                                Types::Str => this_ty = String::from("char*"),
                                Types::None => (),
                            }

                            match p.1 {
                                Expr::VarName(vn) => this_name = vn,
                                Expr::None => (),
                                _ => {
                                    println!("\x1b[91merror\x1b[0m: line {index}, unexpected expression");
                                    exit(1)
                                },
                            }

                            if f_params_touched {
                                f_params.push_str(&format!(", {this_ty} {this_name}"));
                            } else {
                                f_params.push_str(&format!("{this_ty} {this_name}"));
                                f_params_touched = true;
                            }
                        }
                    },
                    _ => (),
                }

                match name {
                    Expr::FuncName(n) => f_name = n.to_string(),
                    _ => (),
                }

                match ty {
                    Types::Void => {
                        if f_name == String::from("main") {
                            f_ty = String::from("int");
                        } else {
                            f_ty = String::from("void");
                        }
                    },
                    Types::Int => f_ty = String::from("int"),
                    Types::Str => f_ty = String::from("char*"),
                    Types::None => {
                        println!("\x1b[91merror\x1b[0m: line {index}, unexpected type");
                        exit(1)
                    },
                }

                let func = format!("{f_ty} {f_name}({f_params}) {{");
                code.push_str(&func);
            },
            _ => (),
        }
    }

    let c_code = format!("{imports}{code}");
    match fs::write("./output.c", c_code) {
        Ok(_) => (),
        Err(err) => {
            println!("{err}");
            exit(1);
        }
    }

    let out = Command::new("cmd")
        .args(["/C", "gcc output.c -o output"])
        .output();

    match out {
        Ok(_) => (),
        Err(err) => {
            println!("\x1b[91merror\x1b[0m: failed to compile");
            println!("{err}");
            exit(1)
        },
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
    let mut parse = ExprWeights::new(tokens);
    let expressions = parse.parser();
    for expr in &expressions {
        println!("{:?}", expr);
    }

    generate(expressions);
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

    if &args[1] == "-b" {
        build(&args[2]);
    } else if &args[1] == "-h" {
        println!("help step");
    } else {
        usage();
        println!("\x1b[91merror\x1b[0m: unknown command, \x1b[93m{}\x1b[0m", &args[1]);
        exit(1)
    }
}
