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
enum Macros {
    Import,
    Arr,
}

#[derive(Debug, Clone)]
enum Expr {
    Func(Box<(Types, Expr, Expr)>), // Expr1 = FuncParams, Expr2 = FuncName
    FuncName(String),
    FuncParams(Box<Vec<Expr>>), // Expr = VarName
    FuncCall(Box<(Expr, Vec<Expr>)>),

    Var(Box<(Expr, Expr)>), // Expr1 = VarName, Expr2 = Literal
    VarName((Types, String)),

    ReVar(Box<(Expr, Expr)>), // Expr1 = VarName, Expr2 = StrLit | IntLit | VarName

    Print(Box<Vec<Expr>>), // Expr = StrLit | IntLit | VarName

    StrLit(String),
    IntLit(String),

    Arr(Box<(Expr, Vec<Expr>)>), // Expr1 = VarName, Expr2 = StrLit | IntLit | VarName
    EndBlock,
    None,
}

#[derive(Debug, Clone)]
enum Token {
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

    Ident(String),
    Str(String),
    Int(String),
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
    has_intlit: bool,
    has_macro: bool,
    has_macro_name: String,
    has_pipe: bool,
    has_ref_name: Vec<Expr>,

    start_param_ix: usize,
    end_param_ix: usize,

    start_intlit_ix: usize,
    end_intlit_ix: usize,

    start_pipe_ix: usize,
    end_pipe_ix: usize,
    // expr_buffer: Vec<Expr>,

    tokens: Vec<Token>,
    current_token: usize,
    keyword_map: HashMap<String, Keyword>,
    macros_map: HashMap<String, Macros>,
    functions: Vec<String>,
    variables: Vec<Expr>,
}

impl ExprWeights {
    fn new(tokens: Vec<Token>) -> ExprWeights {
        let token_to_keyword: HashMap<String, Keyword> = HashMap::from([
            ("print".to_string(), Keyword::Print),
            ("int".to_string(), Keyword::Int),
            ("string".to_string(), Keyword::Str),
            ("_".to_string(), Keyword::Underscore), 
        ]);

        let ident_to_macro: HashMap<String, Macros> = HashMap::from([
            ("array".to_string(), Macros::Arr),
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
            has_intlit: false,
            has_macro: false,
            has_macro_name: String::new(),
            has_pipe: false,
            has_ref_name: Vec::new(),

            start_param_ix: 0,
            end_param_ix: 0,

            start_intlit_ix: 0,
            end_intlit_ix: 0,

            start_pipe_ix: 0,
            end_pipe_ix: 0,
            // expr_buffer: Vec::new(),

            tokens,
            current_token: 0,
            keyword_map: token_to_keyword,
            macros_map: ident_to_macro,
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
            self.has_intlit = false;
            self.has_macro = false;
            self.has_macro_name = String::new();
            self.has_pipe = false;
            self.has_ref_name = Vec::new();

            self.start_param_ix = 0;
            self.end_param_ix = 0;

            self.start_intlit_ix = 0;
            self.end_intlit_ix = 0;

            self.start_pipe_ix = 0;
            self.end_pipe_ix = 0;
    }

    fn check_exist_ident(&mut self, ident: String) -> bool {
        let mut found = false;
        for func in &self.functions {
            if func == &ident {
                found = true;
            }
        }

        for var in &self.variables {
            match var {
                Expr::VarName((_, name)) => {
                    if name == &ident {
                        self.has_ref_name.push(var.clone());
                        found = true;
                    }
                }
                _ => (),
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
            Keyword::Int => {
                if let Types::None = self.has_type {
                    self.has_type = Types::Int;
                }
            },
            Keyword::Str => {
                if let Types::None = self.has_type {
                    self.has_type = Types::Str;
                }
            },
            Keyword::Underscore => {
                if let Types::None = self.has_type {
                    self.has_type = Types::Void;
                }
            },
            Keyword::None => (),
        }
    }

    fn get_params(&mut self) -> Option<Vec<Expr>> {
        let params_slice = &self.tokens[self.start_param_ix..self.end_param_ix];
        let mut params: Vec<Expr> = vec![];
        let mut ty = Types::None;

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
                                ty = Types::Str;
                            },
                            _ => (),
                        }
                    } else {
                        params.push(Expr::VarName((ty.clone(), word.to_string())));
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
                                match var {
                                    Expr::VarName((_, name)) => {
                                        if name == word {
                                            params.push(var.clone());
                                        }
                                    },
                                    _ => (),
                                }
                            }
                        },
                    }
                },
                Token::Str(word) => params.push(Expr::StrLit(word.to_string())),
                Token::Int(integer) => params.push(Expr::IntLit(integer.to_string())),
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
                None => vec![Expr::VarName((Types::None, String::new()))],
            };

            for param in &params {
                match &param {
                    Expr::VarName(_) => self.variables.push(param.clone()),
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

    fn try_reassign_variable(&mut self, ident: Expr) -> Expr {
        let mut expr = Expr::None;
        let mut found_match = false;
        let mut typ = Types::None;
        let mut name = String::new();

        if let Types::None = self.has_type {
            if !self.has_macro_name.is_empty() || self.has_ref_name.is_empty() {
                return expr
            }

            for var in &self.variables {
                match var {
                    Expr::VarName((_, list_name)) => {
                        match &self.has_ref_name[0] {
                            Expr::VarName((var_typ, var_name)) => {
                                if var_name == list_name {
                                    typ = var_typ.clone();
                                    name = var_name.clone();
                                    found_match = true;
                                }
                            },
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }

        if found_match && self.has_colon && !self.has_lbrack && !self.has_rbrack && !self.has_lcurl {
            match ident {
                Expr::StrLit(ref string) => {
                    if let Types::Str = typ {
                        expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident)));
                    } else {
                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m\"{}\"\x1b[0m", typ, name, string);
                        println!("\x1b[91merror\x1b[0m: mismatch types");
                        exit(1);
                    }
                },
                Expr::IntLit(ref integer) => {
                    if let Types::Int = typ {
                        expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident)));
                    } else {
                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", typ, name, integer);
                        println!("\x1b[91merror\x1b[0m: mismatch types");
                        exit(1);
                    }
                },
                Expr::VarName((ref var_typ, ref var_name)) => {
                    match (typ.clone(), var_typ) {
                        (Types::Int, Types::Int) => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        (Types::Str, Types::Str) => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        _ => {
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", typ, name, var_name);
                            println!("\x1b[91merror\x1b[0m: mismatch types");
                            exit(1);
                        },
                    }
                },
                _ => (),
            }
        }

        expr
    }

    fn try_variable(&mut self, value: &Expr) -> Expr {
        let mut expr = Expr::None;

        match self.has_type {
            Types::None => return expr,
            _ => (),
        }

        if self.has_name.is_empty() {
            return expr
        }

        match value {
            Expr::StrLit(v) => {
                if self.has_colon && !self.has_lbrack && !self.has_rbrack && !self.has_lcurl && self.has_macro_name.is_empty() {
                    let varname = Expr::VarName((Types::Str, self.has_name.clone()));
                    if let Types::Str = self.has_type {
                        expr = Expr::Var(Box::new((varname.clone(), Expr::StrLit(v.to_string()))));
                        self.variables.push(varname);
                    } else {
                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m\"{}\"\x1b[0m", self.has_type, self.has_name, v);
                        println!("\x1b[91merror\x1b[0m: mismatch types");
                        exit(1);
                    }
                }
            },
            Expr::IntLit(v) => {
                if self.has_colon && !self.has_lbrack && !self.has_rbrack && !self.has_lcurl && self.has_macro_name.is_empty() {
                    let varname = Expr::VarName((Types::Int, self.has_name.clone()));
                    if let Types::Int = self.has_type {
                        expr = Expr::Var(Box::new((varname.clone(), Expr::IntLit(v.to_string()))));
                        self.variables.push(varname);
                    } else {
                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                        println!("\x1b[91merror\x1b[0m: mismatch types");
                        exit(1);
                    }
                }
            },
            _ => (),
        }

        expr
    }

    fn try_arr(&mut self) -> Expr {
        let mut expr = Expr::None;
        if self.has_macro_name == String::from("array") &&
            self.has_colon &&
            !self.has_name.is_empty() &&
            !self.has_lbrack &&
            !self.has_rbrack &&
            !self.has_lcurl
        {
            let values = &self.tokens[self.start_pipe_ix..self.end_pipe_ix];
            let mut expr_value: Vec<Expr> = Vec::new();

            for value in values {
                match value {
                    Token::Str(string) => expr_value.push(Expr::StrLit(string.clone())),
                    Token::Int(integer) => expr_value.push(Expr::IntLit(integer.clone())),
                    Token::Ident(ident) => {
                        let ident_num = ident.parse::<i32>();
                        match ident_num {
                            Ok(_) => {
                                expr_value.push(Expr::IntLit(ident.clone()));
                            },
                            Err(_) => {
                                expr_value.push(Expr::VarName((self.has_type.clone(), ident.clone())));
                            },
                        }
                    },
                    _ => (),
                }
            }

            expr = Expr::Arr(Box::new((Expr::VarName((self.has_type.clone(), self.has_name.clone())), expr_value)));
        }

        expr
    }

    fn try_func_call(&mut self) -> Expr {
        let mut expr = Expr::None;
        let mut name = String::new();
        let mut met_criteria = false;

        if let Types::None = self.has_type {
            for func in &self.functions {
                match &self.tokens[self.start_param_ix-1] {
                    Token::Ident(word) => {
                        if func == word && self.has_lbrack && self.has_rbrack && !self.has_colon {
                            name = word.clone();
                            met_criteria = true
                        }
                    },
                    _ => (),
                }
            }
        }

        if met_criteria {
            match self.get_params() {
                Some(param) => expr = Expr::FuncCall(Box::new((Expr::FuncName(name), param.to_vec()))),
                None => {
                    let params = &self.tokens[self.start_param_ix..self.end_param_ix];
                    let mut expr_params = Vec::new();

                    for param in params {
                        match param {
                            Token::Str(string) => {
                                expr_params.push(Expr::StrLit(string.clone()))
                            },
                            Token::Int(integer) => {
                                expr_params.push(Expr::IntLit(integer.clone()))
                            },
                            _ => (),
                        }
                    }
                    expr = Expr::FuncCall(Box::new((Expr::FuncName(name), expr_params)))
                },
            }
        }

        expr
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

                expr = self.try_func_call();
                if let Expr::None = expr {
                    match self.has_predef_func {
                        Keyword::Print => {
                            expr = self.check_print();
                        },
                        _ => (),
                    }
                }
            },
            Token::Colon => {
                self.has_colon = true;
            },
            Token::Ident(word)  => {
                // if not check for keywords
                if self.has_macro {
                    self.has_macro_name = word.to_string();
                    self.current_token += 1;
                    self.has_macro = false;
                    return expr
                }

                let int_word = word.parse::<i32>();
                let  num: (bool, Expr);
                match int_word {
                    Ok(_) => num = (true, Expr::IntLit(word.to_string())),
                    Err(_) => num = (false, Expr::None),
                }

                let keyword_res = self.keyword_map.get(word);
                let keyword: (bool, Keyword) = match keyword_res {
                    Some(k) => (true, k.clone()),
                    None => (false, Keyword::None),
                };

                if keyword.0 {
                    self.handle_keywords(keyword.1);
                } else if num. 0{
                        // if we have a number, try see if we can make a variable
                        expr = self.try_variable(&num.1);

                        if let Expr::None = expr {
                            expr = self.try_reassign_variable(num.1.clone());
                        }
                    
                } else {
                    if self.check_exist_ident(word.to_string()) {
                        if self.has_ref_name.len() == 2 {
                            if let Expr::VarName(value) = &self.has_ref_name[1] {
                                expr = self.try_reassign_variable(Expr::VarName(value.clone()));
                            }
                        }
                    }
                }
            },
            Token::Lcurl => {
                self.has_lcurl = true;
                expr = self.check_block();
            },
            Token::Rcurl => expr = Expr::EndBlock,
            Token::Str(word) => {
                self.has_strlit = true;
                let strlit = Expr::StrLit(word.to_string());
                expr = self.try_variable(&strlit);

                if let Expr::None = expr {
                    expr = self.try_reassign_variable(strlit);
                }
            },
            Token::Int(integer) => {
                self.has_intlit = true;
                let intlit = Expr::IntLit(integer.to_string());
                expr = self.try_variable(&intlit);

                if let Expr::None = expr {
                    expr = self.try_reassign_variable(intlit);
                }
            },
            Token::Macro => {
                self.has_macro = true;
            }
            Token::Lsquare => (),
            Token::Rsquare => (),
            Token::Quote => (),
            Token::Pipe => {
                if self.has_pipe {
                    self.end_pipe_ix = self.current_token;
                    self.has_pipe = false;
                    expr = self.try_arr();
                } else {
                    self.start_pipe_ix = self.current_token;
                    self.has_pipe = true;
                }
            },
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

fn generate(expressions: Vec<Expr>, out_filename: String) {
    let mut imports = String::new();
    let mut code = String::new();

    for (index, expr) in expressions.into_iter().enumerate() {
        match expr {
            Expr::FuncCall(f) => {
                let mut func_call = String::new();
                let mut first_param = true;

                match f.0 {
                    Expr::FuncName(name) => func_call.push_str(&format!("{name}(")),
                    _ => (),
                }

                for param in f.1 {
                    match param {
                        Expr::VarName((_, name)) => {
                            if first_param {
                                func_call.push_str(&format!("{name}"));
                                first_param = false;
                            } else {
                                func_call.push_str(&format!(",{name}"));
                            }
                        },
                        Expr::StrLit(string) => {
                            if first_param {
                                func_call.push_str(&format!("\"{string}\""));
                                first_param = false;
                            } else {
                                func_call.push_str(&format!(",\"{string}\""));
                            }
                        }
                        _ => (),
                    }
                }

                func_call.push_str(");");
                code.push_str(&func_call);
            },
            Expr::Arr(value) => {
                let mut arr_var = String::new();
                let mut first_elem = true;

                match value.0 {
                    Expr::VarName((typ, name)) => {
                        match typ {
                            Types::Str => arr_var.push_str(&format!("char* {name}[] = {{")),
                            Types::Int => arr_var.push_str(&format!("int {name}[] = {{")),
                            _ => (),
                        }
                    },
                    _ => (),
                }

                for v in value.1 {
                    match v {
                        Expr::StrLit(string) => {
                            if first_elem {
                                arr_var.push_str(&format!("\"{string}\""));
                                first_elem = !first_elem;
                            } else {
                                arr_var.push_str(&format!(",\"{string}\""));
                            }
                        },
                        Expr::IntLit(integer) => {
                            if first_elem {
                                arr_var.push_str(&format!("{integer}"));
                                first_elem = !first_elem;
                            } else {
                                arr_var.push_str(&format!(",{integer}"));
                            }
                        },
                        Expr::VarName((_, name)) => {
                            if first_elem {
                                arr_var.push_str(&format!("{name}"));
                                first_elem = !first_elem;
                            } else {
                                arr_var.push_str(&format!(",{name}"));
                            }
                        },
                        _ => (),
                    }
                }

                arr_var.push_str("};");
                code.push_str(&arr_var);
            },
            Expr::Var(value) => {
                let mut variable = String::new();

                match value.0 {
                    Expr::VarName((typ, name)) => {
                        match typ {
                            Types::Str => variable.push_str(&format!("char* {name}=")),
                            Types::Int => variable.push_str(&format!("int {name}=")),
                            _ => (),
                        }
                    },
                    _ => (),
                }

                match value.1 {
                    Expr::StrLit(value) => variable.push_str(&format!("\"{value}\";")),
                    Expr::IntLit(value) => variable.push_str(&format!("{value};")),
                    _ => (),
                }

                code.push_str(&variable);
            },
            Expr::ReVar(value) => {
                let mut re_var = String::new();
                let mut var_name = String::new();

                match value.0 {
                    Expr::VarName((_, name)) => {
                        var_name = name;
                    },
                    _ => (),
                }

                match value.1 {
                    Expr::StrLit(string) => {
                        re_var.push_str(&format!("{var_name} = \"{string}\";"));
                    },
                    Expr::IntLit(integer) => {
                        re_var.push_str(&format!("{var_name} = {integer};"));
                    },
                    Expr::VarName((_, name)) => {
                        re_var.push_str(&format!("{var_name} = {name};"));
                    },
                    _ => (),
                }

                code.push_str(&re_var);
            },
            Expr::Print(value) => {
                if !imports.contains("#include <stdio.h>\n") {
                    imports.push_str("#include <stdio.h>\n");
                }

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
                        Expr::VarName((typ, name)) => {
                            match typ {
                                Types::Int => {
                                    param_buf.push_str(&format!("%d"));
                                    var_buf.push_str(&format!(", {name}"));
                                },
                                Types::Str => {
                                    param_buf.push_str(&format!("%s"));
                                    var_buf.push_str(&format!(", {name}"));
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }

                print_code.push_str(&param_buf);
                print_code.push_str("\\n\"");
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
                            let mut this_typ = String::new();
                            let mut this_name = String::new();

                            match p {
                                Expr::VarName((typ, name)) => {
                                    this_name = name;
                                    match typ {
                                        Types::Str => this_typ = String::from("char*"),
                                        Types::Int => this_typ = String::from("int"),
                                        Types::Void => this_typ = String::from("void"),
                                        Types::None => (),
                                    }
                                },
                                _ => (),
                            }

                            if f_params_touched {
                                f_params.push_str(&format!(", {this_typ} {this_name}"));
                            } else {
                                f_params.push_str(&format!("{this_typ} {this_name}"));
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
                        println!("\x1b[91merror\x1b[0m: line {}, unexpected type", index + 1);
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

    let com = format!("gcc output.c -o {out_filename}");
    let out = Command::new("cmd")
        .args(["/C", &com])
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

fn build(filename: &String, out_filename: &String, keep_c: bool) {
    let file_res = fs::read_to_string(filename);
    let content = match file_res {
        Ok(content) => content,
        Err(_) => {
            println!("\x1b[91merror\x1b[0m: unable to read file");
            exit(1)
        },
    };

    let tokens = tokeniser(content);
    for token in &tokens {
        println!("{:?}", token);
    }

    let mut parse = ExprWeights::new(tokens);
    let expressions = parse.parser();
    for expr in &expressions {
        println!("{:?}", expr);
    }

    generate(expressions, out_filename.clone());

    if !keep_c {
        match fs::remove_file("output.c") {
            Ok(_) => (),
            Err(_) => {
                println!("\x1b[91merror\x1b[0m: error handling code generation.");
                exit(1)
            },
        }
    }
}

fn usage_c() {
    println!("| -c: generate c file | impulse -b FILE.imp OUTPUT_NAME |");
}

fn usage_build() {
    println!("| -b: build | impulse -b FILE.imp OUTPUT_NAME |");
}

fn usage() {
    println!("USAGE:");
    println!("impulse <COMMAND> [file.imp] <OUTPUT_NAME>");
    println!("cargo run -- <COMMAND> [file.imp] <OUTPUT_NAME>");
    println!();
    println!("-----------------------------------------------------");
    println!();
    println!("COMMANDS:");
    println!("| -h: help | impulse -h |");
    println!("| -r: run | impulse -b FILE.imp OUTPUT_NAME |");
    usage_build();
    usage_c();
    println!();
    println!("-----------------------------------------------------");
    println!();
}

fn incorrect_usage(args: &Vec<String>, usage_type: fn()) {
    if args.len() < 4 {
        println!("USAGE: ");
        usage_type();
        println!();
        println!("-----------------------------------------------------");
        println!();
        exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        usage();
        println!("\x1b[91merror\x1b[0m: invalid usage");
        exit(1)
    }

    if &args[1] == "-h" {
        usage();
    } else if &args[1] == "-b" {
        incorrect_usage(&args, usage_build);
        build(&args[2], &args[3], false);
    } else if &args[1] == "-c" {
        incorrect_usage(&args, usage_c);
        build(&args[2], &args[3], true);
    } else if &args[1] == "-r" {
        println!("run step")
    } else {
        usage();
        println!("\x1b[91merror\x1b[0m: unknown command, \x1b[93m{}\x1b[0m", &args[1]);
        exit(1)
    }
}
