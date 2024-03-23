use std::{collections::HashMap, process::exit};
use crate::tokeniser::Token;
use crate::declare_types::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Func(Box<(Types, Expr, Expr)>), // Expr1 = FuncParams, Expr2 = FuncName
    FuncName(String),
    FuncParams(Box<Vec<Expr>>), // Expr = VarName
    FuncCall(Box<(Expr, Vec<Expr>)>),

    Var(Box<(Expr, Expr)>), // Expr1 = VarName, Expr2 = Literal
    VarName((Types, String)),

    ReVar(Box<(Expr, Expr)>), // Expr1 = VarName, Expr2 = StrLit | IntLit | VarName
    ReArr(Box<(Expr, String, Expr)>), // Expr1 = VarName, Index, Expr2 = StrLit | IntLit | VarName

    Print(Box<Vec<Expr>>), // Expr = StrLit | IntLit | VarName

    StrLit(String),
    IntLit(String),

    Arr(Box<(Expr, Vec<Expr>)>), // Expr1 = VarName, Expr2 = StrLit | IntLit | VarName
    Dynam(Box<(Expr, Vec<Expr>)>), // Expr1 = VarName, Expr2 = StrLit | IntLit | VarName
    Return(Box<Expr>), // Expr = StrLit, IntLit, VarName
    EndBlock,
    None,
}

#[derive(Debug, Clone)]
pub struct ExprWeights {
    has_type: Types,
    has_lbrack: bool,
    has_rbrack: bool,
    has_colon: bool,
    has_name: String,
    has_func_name: String,
    has_lcurl: bool,
    has_predef_func: Keyword,
    has_strlit: bool,
    has_intlit: bool,
    has_macro: bool,
    has_macro_name: String,
    has_pipe: bool,
    has_ref_name: Vec<Expr>,
    has_return: bool,

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
    functions: Vec<Expr>,
    func_to_vars: HashMap<String, Vec<Expr>>,
    current_func: Expr,
    line_num: u32,
}

impl ExprWeights {
    pub fn new(tokens: Vec<Token>) -> ExprWeights {
        let token_to_keyword: HashMap<String, Keyword> = HashMap::from([
            ("print".to_string(), Keyword::Print),
            ("int".to_string(), Keyword::Int),
            ("string".to_string(), Keyword::Str),
            ("_".to_string(), Keyword::Underscore), 
        ]);

        let ident_to_macro: HashMap<String, Macros> = HashMap::from([
            ("array".to_string(), Macros::Arr),
            ("dynam".to_string(), Macros::Dynam),
        ]);

        let func_to_vars: HashMap<String, Vec<Expr>> = HashMap::new();

        ExprWeights {
            has_type: Types::None,
            has_lbrack: false,
            has_rbrack: false,
            has_colon: false,
            has_name: String::new(),
            has_func_name: String::new(),
            has_lcurl: false,
            has_predef_func: Keyword::None,
            has_strlit: false,
            has_intlit: false,
            has_macro: false,
            has_macro_name: String::new(),
            has_pipe: false,
            has_ref_name: Vec::new(),
            has_return: false,

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
            func_to_vars,
            current_func: Expr::None,
            line_num: 1,
            // variables: Vec::new(),
        }
    }

    fn clear(&mut self) {
            self.has_type = Types::None;
            self.has_lbrack = false;
            self.has_rbrack = false;
            self.has_colon = false;
            self.has_name = String::new();
            self.has_func_name = String::new();
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

    fn extract_func_name(&mut self) -> String {
        let cur_func = match &self.current_func {
            Expr::FuncName(n) => n,
            Expr::None => "",
            _ => {
                println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                println!("\x1b[91merror\x1b[0m: undefined function name");
                exit(1)
            },
        };

        cur_func.to_string()
    }

    fn check_exist_ident(&mut self, ident: String) -> bool {
        let mut found = false;
        for func in &self.functions {
            match &func {
                Expr::Func(func_info) => {
                    match &func_info.2 {
                        Expr::FuncName(func_name) => {
                            if func_name == &ident {
                                found = true;
                                break;
                            }
                        },
                        _=> (),
                    }
                },
                _ => (),
            }
        }

        if found {
            self.has_func_name = ident.clone();
        }

        let cur_func = self.extract_func_name();
        let variables_res = self.func_to_vars.get(&cur_func);
        let variables = match variables_res {
            Some(var) => var,
            None => {
                self.has_name = ident.clone();
                return found;
            },
        };

        for var in variables {
            match var {
                Expr::VarName((_, name)) => {
                    if name == &ident {
                        self.has_ref_name.push(var.clone());
                        found = true;
                    }
                },
                _ => (),
            }
        }

        if !found {
            self.has_name.push_str(&ident);
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
                if self.current_token <= 0 {
                    self.has_type = Types::Void;
                    return;
                }
                match &self.tokens[self.current_token-1] {
                    Token::Ident(_) => {
                        self.has_name.push('_');
                    }
                    _ => {
                        if let Types::None = self.has_type {
                            self.has_type = Types::Void;
                        }
                    },
                }
            },
            Keyword::None => (),
        }
    }

    fn get_params(&mut self) -> Option<Vec<Expr>> {
        let params_slice = &self.tokens[self.start_param_ix..self.end_param_ix];
        let mut params: Vec<Expr> = vec![];
        let mut ty = Types::None;
        let mut is_macro = false;
        let mut get_macro_type = false;
        let mut macro_name = Macros::None;

        for token in params_slice {
            match token {
                Token::Ident(word) => {
                    let keyword_res = self.keyword_map.get(word);
                    let keyword: (bool, Keyword) = match keyword_res {
                        Some(k) => (true, k.clone()),
                        None => (false, Keyword::None),
                    };

                    if is_macro {
                        let macro_res = self.macros_map.get(word);
                        let mac: (bool, Macros) = match macro_res {
                            Some(m) => (true, m.clone()),
                            None => (false, Macros::None),
                        };

                        if mac.0 {
                            get_macro_type = true;
                            macro_name = mac.1;
                        } else {
                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                            println!("\x1b[91merror\x1b[0m: unknown identifier, {}", word);
                            exit(1);
                        }

                        is_macro = false;
                        continue;
                    }

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
                        if get_macro_type {
                            match macro_name {
                                Macros::Arr => {
                                    params.push(Expr::VarName((Types::Arr(Box::new(ty.clone())), word.to_string())));
                                },
                                _ => (),
                            }
                            get_macro_type = false;
                        } else {
                            params.push(Expr::VarName((ty.clone(), word.to_string())));
                        }
                    }
                },
                Token::Macro => {
                    is_macro = true;
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
        let cur_func = self.extract_func_name();
        let params_slice = &self.tokens[self.start_param_ix..self.end_param_ix];
        let mut params: Vec<Expr> = Vec::new();
        for token in params_slice {
            match token {
                Token::Ident(word) => {
                    let int_word = word.parse::<i32>();
                    match int_word {
                        Ok(n) => params.push(Expr::IntLit(n.to_string())),
                        Err(_) => {
                            let variables_res = self.func_to_vars.get(&cur_func);
                            let variables = match variables_res {
                                Some(var) => var,
                                None => {
                                    println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                                    println!("\x1b[91merror\x1b[0m: unknown identifier, {}", word);
                                    exit(1);
                                }
                            };

                            let mut found_var = false;
                            for var in variables {
                                match var {
                                    Expr::VarName((_, name)) => {
                                        if name == word {
                                            params.push(var.clone());
                                            found_var = true;
                                        }
                                    },
                                    _ => (),
                                }
                            }

                            if !found_var {
                                println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                                println!("\x1b[91merror\x1b[0m: unknown identifier, {}", word);
                                exit(1);
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
            println!("\x1b[96m{}\x1b[0m", self.has_macro_name);
            if let Types::None = self.has_type {
                println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                println!("\x1b[91merror\x1b[0m: unknown type for function {}", self.has_name);
                exit(1);
            }

            let mut temp_vars = Vec::new();
            let params_res = self.get_params();
            let params = match params_res {
                Some(params) => params,
                None => vec![Expr::VarName((Types::None, String::new()))],
            };

            for param in &params {
                match &param {
                    Expr::VarName(_) => temp_vars.push(param.clone()),
                    _ => (),
                }
            }

            if self.has_macro_name.is_empty() {
                let expr = Expr::Func(Box::new((self.has_type.clone(), Expr::FuncParams(Box::new(params)), Expr::FuncName(self.has_name.clone()))));
                self.functions.push(expr.clone());
                self.current_func = Expr::FuncName(self.has_name.clone());
                self.func_to_vars.entry(self.has_name.clone()).or_insert(temp_vars);
                return Some(expr)
            } else {
                let macro_res = self.macros_map.get(&self.has_macro_name);
                let mac: (bool, Macros) = match macro_res {
                    Some(m) => (true, m.clone()),
                    None => (false, Macros::None),
                };

                if mac.0 {
                    let expr = Expr::Func(Box::new((Types::Arr(Box::new(self.has_type.clone())), Expr::FuncParams(Box::new(params)), Expr::FuncName(self.has_name.clone()))));
                    self.functions.push(expr.clone());
                    self.current_func = Expr::FuncName(self.has_name.clone());
                    self.func_to_vars.entry(self.has_name.clone()).or_insert(temp_vars);
                    return Some(expr)
                } else {
                    println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                    println!("\x1b[91merror\x1b[0m: unknown identifier {}", self.has_macro_name);
                    exit(1);
                }
            }
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

            let cur_func = self.extract_func_name();
            let variables_res = self.func_to_vars.get(&cur_func);
            let variables = match variables_res {
                Some(var) => var,
                None => {
                    println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                    println!("\x1b[91merror\x1b[0m: reassigning unassigned identifier {:?}", ident);
                    exit(1);
                }
            };

            for var in variables {
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
                    match typ {
                        Types::Str => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        Types::Arr(arr_typ) => {
                            if let Types::Str = *arr_typ {
                                expr = self.try_reassign_arr(*arr_typ, name.clone(), string);
                            }
                        },
                        _ => {
                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m\"{}\"\x1b[0m", typ, name, string);
                            println!("\x1b[91merror\x1b[0m: mismatch types");
                            exit(1);
                        }
                    }
                },
                Expr::IntLit(ref integer) => {
                    match typ {
                        Types::Int => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        Types::Arr(arr_typ) => {
                            if let Types::Int = *arr_typ {
                                expr = self.try_reassign_arr(*arr_typ, name.clone(), integer);
                            }
                        },
                        _ => {
                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", typ, name, integer);
                            println!("\x1b[91merror\x1b[0m: mismatch types");
                            exit(1);
                        }
                    }
                },
                Expr::VarName((ref var_typ, ref var_name)) => {
                    match (typ.clone(), var_typ) {
                        (Types::Int, Types::Int) => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        (Types::Str, Types::Str) => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        _ => {
                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
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

                        let cur_func = self.extract_func_name();
                        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                            vars.push(varname);
                        }
                    } else {
                        println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
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

                        let cur_func = self.extract_func_name();
                        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                            vars.push(varname);
                        }
                    } else {
                        println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
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

    fn try_reassign_arr(&mut self, typ: Types, name: String, value: &String) -> Expr {
        let mut expr = Expr::None;

        if let Types::None = self.has_type {
            let arr_index = &self.tokens[self.start_pipe_ix..self.end_pipe_ix];
            let mut pos = String::new();

            for i in arr_index {
                match i {
                    Token::Ident(index) => {
                        let ident_num = index.parse::<i32>();
                        match ident_num {
                            Ok(_) => {
                                pos = index.clone();
                            },
                            Err(_) => {
                                // do this properly, check if the variable exists instead of just
                                // letting this happen. check that variable is type int
                                pos = index.clone();
                            },
                        }
                    },
                    _ => (),
                }
            }

            let value_typ = value.parse::<i32>();
            match value_typ {
                Ok(_) => {
                    match typ {
                        Types::Int => {
                            let v = Expr::IntLit(value.clone());
                            expr = Expr::ReArr(Box::new((Expr::VarName((Types::Arr(Box::new(typ)), name)), pos, v)));
                        },
                        _ => (),
                    }
                },
                Err(_) => {
                    match typ {
                        Types::Str => {
                            let v = Expr::StrLit(value.clone());
                            expr = Expr::ReArr(Box::new((Expr::VarName((Types::Arr(Box::new(typ)), name)), pos, v)));
                        },
                        _ => {
                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", typ, self.has_name, value);
                            println!("\x1b[91merror\x1b[0m: mismatch types");
                            exit(1);
                        }
                    }
                },
            }

        }

        expr
    }

    fn try_arr(&mut self) -> Expr {
        let mut expr = Expr::None;

        let macro_res = self.macros_map.get(&self.has_macro_name);
        let mac: (bool, Macros) = match macro_res {
            Some(m) => (true, m.clone()),
            None => (false, Macros::None),
        };

        if !mac.0 {
            return expr;
        }

        if self.has_colon &&
            !self.has_name.is_empty() &&
            !self.has_lbrack &&
            !self.has_rbrack &&
            !self.has_lcurl
        {
            let values = &self.tokens[self.start_pipe_ix..self.end_pipe_ix];
            let mut expr_value: Vec<Expr> = Vec::new();

            for value in values {
                match value {
                    Token::Str(string) => {
                        if let Types::Str = self.has_type {
                            expr_value.push(Expr::StrLit(string.clone()));
                        } else {
                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                            println!("\x1b[91merror\x1b[0m: string array with mismatch elements.");
                            exit(1);
                        }
                    },
                    Token::Int(integer) => {
                        if let Types::Int = self.has_type {
                            expr_value.push(Expr::IntLit(integer.clone()))
                        } else {
                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                            println!("\x1b[91merror\x1b[0m: integer array with mismatch elements.");
                            exit(1);
                        }
                    },
                    Token::Ident(ident) => {
                        let ident_num = ident.parse::<i32>();
                        match ident_num {
                            Ok(_) => {
                                if let Types::Int = self.has_type {
                                    expr_value.push(Expr::IntLit(ident.clone()));
                                } else {
                                    println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                                    println!("\x1b[91merror\x1b[0m: integer array with mismatch elements.");
                                    exit(1);
                                }
                            },
                            Err(_) => {
                                // do this properly, check if the variable exists instead of just
                                // letting this happen
                                expr_value.push(Expr::VarName((self.has_type.clone(), ident.clone())));
                            },
                        }
                    },
                    _ => (),
                }
            }

            match mac.1 {
                Macros::Dynam => {
                    expr = Expr::Dynam(Box::new((Expr::VarName((Types::Dynam(Box::new(self.has_type.clone())), self.has_name.clone())), expr_value)));
                    let cur_func = self.extract_func_name();
                    if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                        vars.push(Expr::VarName((Types::Dynam(Box::new(self.has_type.clone())), self.has_name.clone())));
                    }
                },
                Macros::Arr => {
                    expr = Expr::Arr(Box::new((Expr::VarName((Types::Arr(Box::new(self.has_type.clone())), self.has_name.clone())), expr_value)));
                    let cur_func = self.extract_func_name();
                    if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                        vars.push(Expr::VarName((Types::Arr(Box::new(self.has_type.clone())), self.has_name.clone())));
                    }
                },
                _ => {
                    return expr;
                }
            }
        }

        expr
    }

    fn try_return(&mut self, value: Expr) -> Expr {
        let mut expr = Expr::None;
        
        if !self.has_return {
            return expr
        }

        if self.has_colon {
            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
            println!("\x1b[91merror\x1b[0m: unexpected assignment operator \x1b[96m:\x1b[0m");
            exit(1)
        }

        let cur_func = self.extract_func_name();
        for func in &self.functions {
            match func {
                Expr::Func(func_info) => {
                    let mut found = false;
                    match &func_info.2 {
                        Expr::FuncName(func_name) => {
                            if func_name == &cur_func {
                                found = true;
                            }
                        }
                        _ => (),
                    }

                    if found {
                        match func_info.0 {
                            Types::Str => {
                                match value {
                                    Expr::StrLit(ref string) => {
                                        expr = Expr::Return(Box::new(Expr::StrLit(string.to_string())));
                                    },
                                    Expr::VarName((ref typ, ref name)) => {
                                        if let Types::Str = typ {
                                            expr = Expr::Return(Box::new(Expr::VarName((typ.clone(), name.clone()))));
                                        } else {
                                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                                            println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
                                            println!("\x1b[91merror\x1b[0m: mismatch types");
                                            exit(1);
                                        }
                                    },
                                    _ => {
                                        println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                                        println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
                                        println!("\x1b[91merror\x1b[0m: mismatch types");
                                        exit(1);
                                    }
                                }
                            },
                            Types::Int => {
                                match value {
                                    Expr::IntLit(ref integer) => {
                                        expr = Expr::Return(Box::new(Expr::IntLit(integer.to_string())));
                                    },
                                    Expr::VarName((ref typ, ref name)) => {
                                        if let Types::Int = typ {
                                            expr = Expr::Return(Box::new(Expr::VarName((typ.clone(), name.clone()))));
                                        } else {
                                            println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                                            println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
                                            println!("\x1b[91merror\x1b[0m: mismatch types");
                                            exit(1);
                                        }
                                    },
                                    _ => {
                                        println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                                        println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
                                        println!("\x1b[91merror\x1b[0m: mismatch types");
                                        exit(1);
                                    }
                                }
                            },
                            _ => (),
                        }
                    }
                },
                _ => (),
            }
        }

        expr
    }

    fn try_func_call(&mut self) -> Expr {
        let mut expr = Expr::None;
        let mut name = String::new();
        let mut met_criteria = false;

        if self.has_func_name.is_empty() {
            return expr
        }

        for func in &self.functions {
            match func {
                Expr::Func(func_info) => {
                    match &func_info.2 {
                        Expr::FuncName(func_name) => {
                            if func_name == &self.has_func_name && self.has_lbrack && self.has_rbrack && !self.has_lcurl {
                                name = self.has_func_name.clone();
                                met_criteria = true;
                            }
                        },
                        _=> (),
                    }
                },
                _ => (),
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
            None => {
                println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
                println!("\x1b[91merror\x1b[0m: unknown code block declaration, {}", self.has_name);
                exit(1);
            },
        }

        expr
    }

    fn handle_var_use(&mut self, ident: String) -> Expr {
        let mut expr = Expr::None;

        if self.check_exist_ident(ident.clone()) {
            if self.has_ref_name.len() == 2 {
                if let Expr::VarName(value) = &self.has_ref_name[1] {
                    expr = self.try_reassign_variable(Expr::VarName(value.clone()));
                }
            } else if self.has_return {
                let cur_func = &self.extract_func_name();
                let variables = self.func_to_vars.get(cur_func);
                let vars = match variables {
                    Some(v) => v,
                    None => {
                        exit(1);
                    },
                };

                for v in vars.clone() {
                    if let Expr::VarName(value) = v {
                        if value.1 == ident {
                            expr = self.try_return(Expr::VarName(value));
                        }
                    }
                }
            }
        }

        expr
    }

    fn parse_to_expr(&mut self) -> Expr {
        let mut expr = Expr::None;
        match &self.tokens[self.current_token] {
            Token::Underscore => {
                self.handle_keywords(Keyword::Underscore);
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
                if let Expr::None = expr {
                    expr = self.try_func_call();
                }
            },
            Token::Colon => {
                self.has_colon = true;
            },
            Token::Ident(word)  => {
                // if not check for keywords
                if self.has_macro && self.start_param_ix == 0 {
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

                let keyword_res = self.keyword_map.get(word.as_str());
                let keyword: (bool, Keyword) = match keyword_res {
                    Some(k) => (true, k.clone()),
                    None => (false, Keyword::None),
                };

                // lord forgive me for this code
                if keyword.0 {
                    self.handle_keywords(keyword.1);
                } else if num. 0{
                        // if we have a number, try see if we can make a variable
                        expr = self.try_variable(&num.1);

                        if let Expr::None = expr {
                            expr = self.try_reassign_variable(num.1.clone());
                        }

                        if let Expr::None = expr {
                            expr = self.try_reassign_variable(num.1);
                        }
                } else {
                    expr = self.handle_var_use(word.clone());
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
                    expr = self.try_reassign_variable(strlit.clone());
                }

                if let Expr::None = expr {
                    expr = self.try_return(strlit);
                }
            },
            Token::Int(integer) => {
                self.has_intlit = true;
                let intlit = Expr::IntLit(integer.to_string());
                expr = self.try_variable(&intlit);

                if let Expr::None = expr {
                    expr = self.try_reassign_variable(intlit.clone());
                }

                if let Expr::None = expr {
                    expr = self.try_return(intlit);
                }
            },
            Token::Macro => {
                self.has_macro = true;
            }
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
            Token::Return => {
                self.has_return = true;
            },
            Token::Newline => self.line_num += 1,
            Token::Lsquare => (),
            Token::Rsquare => (),
            Token::Quote => (),
        }

        self.current_token += 1;
        expr
    }

    pub fn parser(&mut self) -> Vec<Expr> {
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
