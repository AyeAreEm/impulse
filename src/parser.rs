use std::collections::HashSet;
use std::{fs, collections::HashMap, process::exit};
use crate::tokeniser::{tokeniser, Token}; use crate::declare_types::*;

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

    CImport(String),
    CEmbed(String),

    Println(Box<Vec<Expr>>),
    Print(Box<Vec<Expr>>), // Expr = StrLit | IntLit | VarName
    ReadIn,

    StrLit(String),
    IntLit(String),

    If(Box<Expr>), // Expr = Condition
    OrIf(Box<Expr>),
    Else,
    Condition(Box<Vec<Expr>>), // Literally anything

    Or,
    And,

    Loop(Box<(Expr, Expr)>), // Expr1 = Condition, Expr2 = Loopmod
    LoopMod(String),

    Array(Box<Vec<Expr>>),
    Dynamic(Box<Vec<Expr>>),

    ArrIndex(Box<(Expr, Expr)>), // Exp1 = VarName, Expr2 = IntLit
    Return(Box<Expr>), // Expr = StrLit | IntLit | VarName

    // DEFINE A STRUCT
    StructDef(String),
    StructField(Box<Expr>), // Expr = VarName
    Struct(Box<(Expr, Vec<Expr>)>), // Expr1 = StructDef, Expr2 = StructField
    EndStruct,

    // DEFINE A VAR STRUCT
    StructVarField(Box<Expr>), // Expr = VarName,
    EndStructVar,

    Equal,
    SmallerThan,
    BiggerThan,
    Exclaim,

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
    intlit_content: String,
    has_macro: bool,
    has_macro_name: String,
    has_pipe: bool,
    has_ref_name: Vec<Expr>,
    has_return: bool,
    has_if: bool,
    has_orif: bool,
    has_else: bool,
    has_loop: bool,
    has_cembed_macro: bool,
    has_def_struct: bool,
    has_dot: bool,
    has_semicolon: bool,

    start_param_ix: usize,
    end_param_ix: usize,

    start_pipe_ix: usize,
    end_pipe_ix: usize,
    expr_buffer: Vec<Expr>,

    tokens: Vec<Token>,
    current_token: usize,
    keyword_map: HashMap<String, Keyword>,
    macros_map: HashMap<String, Macros>,

    pub functions: Vec<Expr>,
    func_to_vars: HashMap<String, Vec<Vec<Expr>>>,

    current_func: Expr,
    current_scope: usize,
    current_var_struct: String,

    struct_def: Expr,
    structures: Vec<Expr>,

    user_def_types: Vec<Types>,

    in_scope: bool,
    in_func: bool,
    in_struct_var: bool,

    line_num: u32,
    
    program: Vec<Expr>,
}

impl ExprWeights {
    pub fn new(tokens: Vec<Token>) -> ExprWeights {
        let token_to_keyword: HashMap<String, Keyword> = HashMap::from([
            ("println".to_string(), Keyword::Println),
            ("print".to_string(), Keyword::Print),
            ("readin".to_string(), Keyword::ReadIn),

            ("int".to_string(), Keyword::Int),
            ("string".to_string(), Keyword::Str),

            ("if".to_string(), Keyword::If),
            ("orif".to_string(), Keyword::OrIf),
            ("else".to_string(), Keyword::Else),

            ("or".to_string(), Keyword::Or),
            ("and".to_string(), Keyword::And),

            ("loop".to_string(), Keyword::Loop),
            ("_".to_string(), Keyword::Underscore), 

            ("return".to_string(), Keyword::Return),
            ("struct".to_string(), Keyword::Struct),
        ]);

        let ident_to_macro: HashMap<String, Macros> = HashMap::from([
            ("c".to_string(), Macros::C),
            ("import".to_string(), Macros::Import),
            ("array".to_string(), Macros::Arr),
            ("dynam".to_string(), Macros::Dynam),
        ]);

        let func_to_vars: HashMap<String, Vec<Vec<Expr>>> = HashMap::new();

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
            intlit_content: String::new(),
            has_macro: false,
            has_macro_name: String::new(),
            has_pipe: false,
            has_ref_name: Vec::new(),
            has_return: false,
            has_if: false,
            has_orif: false,
            has_else: false,
            has_loop: false,
            has_cembed_macro: false,
            has_def_struct: false,
            has_dot: false,
            has_semicolon: false,

            start_param_ix: 0,
            end_param_ix: 0,

            start_pipe_ix: 0,
            end_pipe_ix: 0,
            expr_buffer: Vec::new(),

            tokens,
            current_token: 0,
            keyword_map: token_to_keyword,
            macros_map: ident_to_macro,

            functions: Vec::new(),
            func_to_vars,
            current_func: Expr::None,
            current_scope: 0,
            current_var_struct: String::new(),

            struct_def: Expr::None,
            structures: Vec::new(),

            user_def_types: Vec::new(),
            in_scope: false,
            in_func: false,
            in_struct_var: false,

            line_num: 1,

            program: Vec::new(),
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
            self.intlit_content = String::new();
            self.has_macro = false;
            self.has_macro_name = String::new();
            self.has_pipe = false;
            self.has_ref_name = Vec::new();
            self.has_return = false;
            self.has_if = false;
            self.has_orif = false;
            self.has_else = false;
            self.has_loop = false;
            self.has_cembed_macro = false;
            self.has_dot = false;
            self.has_semicolon = false;
            // self.has_def_struct = false;

            self.start_param_ix = 0;
            self.end_param_ix = 0;

            self.start_pipe_ix = 0;
            self.end_pipe_ix = 0;
    }

    fn comp_err(&self, error_msg: &str) {
        println!("\x1b[91merror\x1b[0m: line {}", self.line_num);
        println!("\x1b[91merror\x1b[0m: {error_msg}");
    }

    fn new_scope(&mut self, new_var: Expr) {
        self.in_scope = true;

        let cur_func = self.extract_func_name();
        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
            let mut old_scope = vars[self.current_scope].clone();
            self.current_scope += 1;
            vars.push(vec![]);
            vars[self.current_scope].append(&mut old_scope);

            match new_var {
                Expr::None => (),
                _ => vars[self.current_scope].push(new_var),
            }
        }
    }

    fn prev_scope(&mut self) {
        let cur_func = self.extract_func_name();
        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
            if self.in_scope && self.current_scope == 0 {
                self.in_scope = false;
            } else if self.in_scope && self.current_scope > 0 {
                self.in_scope = false;
                vars[self.current_scope].pop();
                self.current_scope -= 1;
            } else {
                self.in_func = false;
                vars[self.current_scope].pop();
            }
        }
    }

    fn extract_func_name(&self) -> String {
        let cur_func = match &self.current_func {
            Expr::FuncName(n) => n,
            Expr::None => "",
            _ => {
                self.comp_err("undefined function name");
                exit(1);
            },
        };

        cur_func.to_string()
    }

    fn handle_import(&mut self, location: &Expr) -> Option<Expr> {
        match location {
            Expr::StrLit(loc) => {
                if loc.chars().nth(loc.len()-1).unwrap() == 'h' {
                    let no_extension = loc.split_at(loc.len()-2);
                    self.has_macro_name.clear();
                    return Some(Expr::CImport(no_extension.0.to_string()))
                } else {
                    let file_res = fs::read_to_string(loc);
                    let content = match file_res {
                        Ok(content) => content,
                        Err(_) => {
                            self.comp_err("unable to read file");
                            exit(1);
                        },
                    };

                    if self.program.is_empty() {
                        let tokens = tokeniser(content);
                        let mut parse = ExprWeights::new(tokens);
                        let mut expressions = parse.parser();

                        self.functions.append(&mut parse.functions);
                        self.program.append(&mut expressions);
                    } else {
                        self.comp_err("imports need to be before writing your program");
                        exit(1);
                    }
                }
            },
            _ => (),
        }

        self.has_macro_name.clear();
        None
    }

    fn make_arr_index(&self, arr_name: Expr, index: &String) -> Expr {
        let mut expr = Expr::None;
        let index_num_res = index.parse::<usize>();
        let index_num = match index_num_res {
            Ok(val) => (true, val),
            Err(_) => {
                let cur_func = self.extract_func_name();
                let variables_res = self.func_to_vars.get(&cur_func);
                let variables = match variables_res {
                    Some(var) => var,
                    None => {
                        self.comp_err(&format!("undefined variable {index}"));
                        exit(1);
                    },
                };

                let mut found = false;
                let mut value = 0;
                for var in &variables[self.current_scope] {
                    match var {
                        Expr::Var(var_info) => {
                            match &var_info.0 {
                                Expr::VarName((typ, name)) => {
                                    if let Types::Int = typ {
                                        if name == index {
                                            found = true;
                                        }
                                    }
                                },
                                _ => (),
                            }

                            if found {
                                match &var_info.1 {
                                    Expr::IntLit(num) => {
                                        match num.parse::<usize>() {
                                            Ok(n) => value = n,
                                            Err(_) => {
                                                self.comp_err("array index must be a number");
                                                exit(1);
                                            }
                                        }
                                    },
                                    Expr::VarName(_) => {
                                        // ADD SUPPORT FOR THIS
                                        self.comp_err("currently unsupported. nested variable for index in array");
                                        exit(1);
                                    },
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                }

                if found {
                    (true, value)
                } else {
                    (false, 0)
                }
            }
        };

        if !index_num.0 {
            self.comp_err(&format!("undefined variable {index}"));
            exit(1);
        }

        if let Expr::Var(var) = arr_name {
            match var.1 {
                Expr::Array(arr) => {
                    if arr.len() > index_num.1 {
                        expr = Expr::ArrIndex(Box::new((var.0, Expr::IntLit(index.clone()))));
                    } else {
                        self.comp_err("index out of array size.");
                        exit(1);
                    }
                },
                _ => (),
            }
        }

        expr
    }

    fn check_intlit(&mut self, intlit: String) -> Expr {
        // check for func names, arrays, variables, etc.

        let symb_to_token: HashMap<char, Token> = HashMap::from([
            ('(', Token::Lbrack),
            (')', Token::Rbrack),
            ('[', Token::Lsquare),
            (']', Token::Rsquare),
            ('+', Token::Plus),
            ('-', Token::Minus),
            ('*', Token::Multiple),
            ('/', Token::Divide),
            ('|', Token::Pipe),
            ('.', Token::Dot),
        ]);
        let mut tokens: Vec<Token> = Vec::new();
        let mut buf = String::new();
        let mut clean = String::new();

        for c in intlit.chars() {
            if c == ' ' || c == '\n' || c == '\r' {
                if buf.len() > 0 {
                    tokens.push(Token::Ident(buf.clone()));
                    buf.clear();
                }

                continue;
            }

            let integer_res = c.to_digit(10);
            match integer_res {
                Some(_) => {
                    tokens.push(Token::Digit(c));
                    continue;
                },
                None => (),
            }

            let token_res = symb_to_token.get(&c);
            match token_res {
                Some(token) => {
                    if buf.len() > 0 {
                        tokens.push(Token::Ident(buf.clone()));
                        buf.clear();
                    }
                    tokens.push(token.clone());
                },
                None => {
                    buf.push(c);
                }
            }
        }

        let mut local_dot = false;
        let mut local_name = String::new();
        for token in tokens {
            match token {
                Token::Plus => clean.push('+'),
                Token::Minus => clean.push('-'),
                Token::Multiple => clean.push('*'),
                Token::Divide => clean.push('/'),
                Token::Digit(num) => clean.push(num),
                Token::Lbrack => {
                    self.has_lbrack = true; 
                    self.start_param_ix = self.current_token;
                    clean.push('(');
                },
                Token::Rbrack => {
                    self.has_rbrack = true;
                    self.end_param_ix = self.current_token;
                    let expr = self.try_func_call();
                    if let Expr::None = expr {
                        self.comp_err("incorrect usage of function call, maybe undefined function name");
                        exit(1);
                    }

                    clean.push(')');
                },
                Token::Lsquare => clean.push('('),
                Token::Rsquare => clean.push(')'),
                Token::Pipe => {
                    if self.has_pipe {
                        self.end_pipe_ix = self.current_token;
                        self.has_pipe = false;

                        let name_expr = &self.has_ref_name[self.has_ref_name.len()-1];
                        let name = match name_expr {
                            Expr::Var(ref var_info) => {
                                match &var_info.0 {
                                    Expr::VarName((_, n)) => {
                                        n.to_owned()
                                    },
                                    _ => String::new(),
                                }
                            }
                            _ => String::new(),
                        };

                        if name.is_empty() {
                            self.comp_err(&format!("unknown identifier {:?}", name_expr));
                            exit(1);
                        }
                    } else {
                        self.start_pipe_ix = self.current_token;
                        self.has_pipe = true;
                    }
                    clean.push('|');
                },
                Token::Ident(ident) => {
                    if local_dot {
                        let cur_func = self.extract_func_name();
                        let variables_res = self.func_to_vars.get(&cur_func);
                        let variables = match variables_res {
                            Some(vars) => vars.clone(),
                            None => Vec::new(),
                        };

                        let mut fields = Vec::new();
                        for vars in &variables[self.current_scope] {
                            match vars {
                                Expr::Var(var_info) => {
                                    match &var_info.0 {
                                        Expr::VarName((_, name)) => {
                                            if name == &local_name {
                                                match &var_info.1 {
                                                    Expr::Condition(getter) => {
                                                        for get in *getter.clone() {
                                                            match get {
                                                                Expr::StructField(struct_field) => {
                                                                    fields.push(*struct_field);
                                                                },
                                                                _ => (),
                                                            }
                                                        }
                                                    },
                                                    _ => ()
                                                }
                                            }
                                        }
                                        _ => (),
                                    }
                                },
                                _ => (),
                            }
                        }

                        let mut found = false;
                        for field in fields {
                            match field {
                                Expr::VarName((typ, name)) => {
                                    if let Types::Int = typ {
                                        if name == ident {
                                            found = true;
                                        }
                                    }
                                },
                                _ => (),
                            }
                        }

                        if !found {
                            self.comp_err(&format!("this unknown identifier {:?}", ident));
                            exit(1);
                        }

                        local_dot = false;
                    } else if self.check_exist_ident(ident.clone()) {
                        local_name = ident.clone();
                    } else {
                        self.comp_err(&format!("unknown identifier {:?}", ident));
                        exit(1);
                    }

                    clean.push_str(&ident);
                },
                Token::Dot => {
                    local_dot = true;
                    clean.push('.');
                },
                _ => (),
            }
        }

        Expr::IntLit(clean)
    }

    fn check_exist_ident(&mut self, ident: String) -> bool {
        let mut found = false;

        for typ in &self.user_def_types {
            match typ {
                Types::UserDef(typ_name) => {
                    if typ_name == &ident {
                        found = true;
                        break;
                    }
                },
                _ => (),
            }
        }

        if found {
            self.has_type = Types::UserDef(ident.clone());
            self.current_var_struct = ident.clone();
            return found
        }

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

        for var in &variables[self.current_scope] {
            match var {
                Expr::Var(var_info) => {
                    match &var_info.0 {
                        Expr::VarName((_, name)) => {
                            if name == &ident {
                                self.has_ref_name.push(Expr::Var(var_info.clone()));
                                found = true;
                            }
                        },
                        _ => (),
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
            Keyword::Println => self.has_predef_func = keyword,
            Keyword::Print => self.has_predef_func = keyword,
            Keyword::ReadIn => self.has_predef_func = keyword,
            Keyword::Int => {
                if let Types::None = self.has_type {
                    self.has_type = Types::Int;
                } else if self.has_def_struct {
                    self.has_type = Types::Int;
                }
            },
            Keyword::Str => {
                if let Types::None = self.has_type {
                    self.has_type = Types::Str;
                } else if self.has_def_struct {
                    self.has_type = Types::Str;
                }
            },
            Keyword::Return => {
                self.has_return = true;
            },
            Keyword::Loop => {
                self.has_loop = true;
            },
            Keyword::If => {
                self.has_if = true;
            },
            Keyword::OrIf => {
                self.has_orif = true;
            },
            Keyword::Else => {
                self.has_else = true;
            }
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
            Keyword::Struct => {
                self.has_def_struct = true;
            },
            Keyword::Or => (),
            Keyword::And => (),
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
                            self.comp_err(&format!("unknown identifier {:?}", word));
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
                                    params.push(Expr::Var(Box::new((Expr::VarName((Types::Arr(Box::new(ty.clone())), word.to_string())), Expr::None))));
                                },
                                _ => (),
                            }
                            get_macro_type = false;
                        } else {
                            params.push(Expr::Var(Box::new((Expr::VarName((ty.clone(), word.to_string())), Expr::None))));
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

    fn check_readin(&mut self) -> Expr {
        let mut expr = Expr::None;

        match self.has_type {
            Types::Str => {
                if self.has_colon && !self.has_lcurl && self.has_lbrack {
                    expr = Expr::Var(Box::new((Expr::VarName((self.has_type.clone(), self.has_name.clone())), Expr::ReadIn)));
                    let cur_func = self.extract_func_name();
                    if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                        vars[self.current_scope].push(expr.clone());
                    }
                }
            },
            _ => {
                self.comp_err(&format!("function readin returns string but {} is type {:?}", self.has_name, self.has_type));
                exit(1);
            },
        }

        expr
    }

    fn check_print(&mut self, is_line: bool) -> Expr {
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
        if params_slice.is_empty() && self.expr_buffer.is_empty() {
            return Expr::Print(Box::new(params));
        }

        let mut buf = Expr::None;
        for token in params_slice {
            match token {
                Token::Ident(word) => {
                    let int_word = word.parse::<i32>();
                    match int_word {
                        Ok(n) => {
                            match buf {
                                Expr::None => params.push(Expr::IntLit(n.to_string())),
                                _ => {
                                    if self.has_pipe {
                                        let temp = self.make_arr_index(buf.clone(), word);
                                        params.push(temp);
                                    } else {
                                        self.comp_err(&format!("can't print all values in arr: {}", word));
                                        println!("\x1b[93merror\x1b[0m: did you mean {:?}|{}|", buf, word);
                                        exit(1);
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            let variables_res = self.func_to_vars.get(&cur_func);
                            let variables = match variables_res {
                                Some(var) => var,
                                None => {
                                    self.comp_err(&format!("unknown identifier, {}", word));
                                    exit(1);
                                }
                            };

                            let mut found_var = false;
                            for var in &variables[self.current_scope] {
                                match var {
                                    Expr::Var(var_info) => {
                                        match &var_info.0 {
                                            Expr::VarName((typ, name)) => {
                                                if name == word {
                                                    found_var = true;

                                                    if let Types::UserDef(_) = typ {
                                                        match &var_info.1 {
                                                            Expr::Condition(_fields) => {
                                                                self.comp_err("can't print structs yet. not implemented");
                                                                exit(1);
                                                            },
                                                            _ => (),
                                                        }
                                                    } else if let Types::Arr(_arr_typ) = typ {
                                                        buf = Expr::Var(var_info.clone());
                                                    } else if let Types::Int = typ {
                                                        match buf {
                                                            Expr::None => {
                                                                params.push(Expr::Var(var_info.clone()));
                                                            },
                                                            _ => {
                                                                if self.has_pipe {
                                                                    let temp = self.make_arr_index(buf.clone(), name);
                                                                    params.push(temp);
                                                                    found_var = true;
                                                                } else {
                                                                    self.comp_err(&format!("can't print all values in array. maybe use a loop? array: {}", word));
                                                                    exit(1);
                                                                }
                                                                buf = Expr::None;
                                                            },
                                                        }
                                                    } else {
                                                        params.push(Expr::Var(var_info.clone()));
                                                    }
                                                }
                                            },
                                            _ => (),
                                        }
                                    },
                                    _ => (),
                                }
                            }

                            if !found_var {
                                self.comp_err(&format!("unknown identifier {word}"));
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

        if is_line {
            return Expr::Println(Box::new(params))
        }
        Expr::Print(Box::new(params))
    }

    fn check_func(&mut self) -> Expr {
        let mut expr = Expr::None;

        if let Types::None = self.has_type {
            return expr
        }

        if self.has_lbrack && self.has_rbrack && self.has_colon && !self.has_name.is_empty() && self.has_lcurl {
            if let Types::None = self.has_type {
                self.comp_err(&format!("unknown type for function {}", self.has_name));
                exit(1);
            }

            self.in_scope = true;
            let mut temp_vars = Vec::new();
            let params_res = self.get_params();
            let params = match params_res {
                Some(params) => params,
                None => vec![],
            };

            if !params.is_empty() {
                for param in &params {
                    match &param {
                        Expr::Var(_) => temp_vars.push(param.clone()),
                        _ => (),
                    }
                }
            }

            if self.has_macro_name.is_empty() {
                expr = Expr::Func(Box::new((self.has_type.clone(), Expr::FuncParams(Box::new(params)), Expr::FuncName(self.has_name.clone()))));
                self.functions.push(expr.clone());
                self.current_func = Expr::FuncName(self.has_name.clone());
                self.func_to_vars.entry(self.has_name.clone()).or_insert(vec![temp_vars]);
            } else {
                let macro_res = self.macros_map.get(&self.has_macro_name);
                let mac: (bool, Macros) = match macro_res {
                    Some(m) => (true, m.clone()),
                    None => (false, Macros::None),
                };

                if mac.0 {
                    match mac.1 {
                        Macros::Arr => {
                            expr = Expr::Func(Box::new((Types::Arr(Box::new(self.has_type.clone())), Expr::FuncParams(Box::new(params)), Expr::FuncName(self.has_name.clone()))));
                        },
                        Macros::Dynam => {
                            expr = Expr::Func(Box::new((Types::Dynam(Box::new(self.has_type.clone())), Expr::FuncParams(Box::new(params)), Expr::FuncName(self.has_name.clone()))));
                        }
                        _ => (),
                    }
                    self.functions.push(expr.clone());
                    self.current_func = Expr::FuncName(self.has_name.clone());
                    self.func_to_vars.entry(self.has_name.clone()).or_insert(vec![temp_vars]);
                } else {
                    self.comp_err(&format!("unknown identifier {}", self.has_macro_name));
                    exit(1);
                }
            }
        }

        self.in_func = true;
        expr
    }

    fn check_conditional(&mut self, is_loop: bool) -> Vec<Expr> {
        let cur_func = self.extract_func_name();
        let params = &self.tokens[self.start_param_ix..self.end_param_ix];
        let mut arr_expr = Vec::new();

        if params.is_empty() {
            self.comp_err("empty conditional");
            exit(1);
        }

        for param in params.to_owned() {
            match param {
                Token::Ident(ident) => {
                    let int_word = ident.parse::<i32>();
                    let num = match int_word {
                        Ok(_) => (true, Expr::IntLit(ident.to_string())),
                        Err(_) => (false, Expr::None),
                    };

                    if num.0 {
                        arr_expr.push(num.1);
                        continue;
                    }

                    let keyword_res = self.keyword_map.get(&ident);
                    let keyword: (bool, Keyword) = match keyword_res {
                        Some(k) => (true, k.clone()),
                        None => (false, Keyword::None),
                    };

                    if keyword.0 {
                        match keyword.1 {
                            Keyword::Or => {
                                arr_expr.push(Expr::Or);
                                continue;
                            },
                            Keyword::And => {
                                arr_expr.push(Expr::And);
                                continue;
                            },
                            _ => {
                                self.comp_err(&format!("forbidden keyword in statement block {:?}", keyword.1));
                                exit(1);
                            },
                        }
                    }

                    let variables_res = self.func_to_vars.get(&cur_func);
                    let variables = match variables_res {
                        Some(var) => var,
                        None => {
                            self.comp_err(&format!("unknown identifier {ident}"));
                            exit(1);
                        }
                    };

                    let mut found = false;
                    for var in &variables[self.current_scope] {
                        match var {
                            Expr::Var(var_info) => {
                                match &var_info.0 {
                                    Expr::VarName((_, varname)) => {
                                        if &ident == varname {
                                            arr_expr.push(var_info.0.clone());
                                            found = true;
                                        }
                                    },
                                    _ => (),
                                }
                            },
                            _ => (),
                        }
                    }

                    if !found && is_loop {
                        let expr = Expr::Var(Box::new((Expr::VarName((Types::Int, ident.to_owned())), Expr::IntLit(String::from("0")))));
                        arr_expr.push(expr.clone());
                        self.new_scope(expr);
                    }
                },
                Token::Equal => arr_expr.push(Expr::Equal),
                Token::SmallerThan => arr_expr.push(Expr::SmallerThan),
                Token::BiggerThan => arr_expr.push(Expr::BiggerThan),
                Token::Exclaim => arr_expr.push(Expr::Exclaim),
                _ => (),
            }
        }

        match arr_expr[arr_expr.len()-1] {
            Expr::Equal => {
                self.comp_err("incomplete condition");
                exit(1);
            },
            Expr::SmallerThan => {
                self.comp_err("incomplete condition");
                exit(1);
            },
            Expr::BiggerThan => {
                self.comp_err("incomplete condition");
                exit(1);
            },
            Expr::Exclaim => {
                self.comp_err("incomplete condition");
                exit(1);
            },
            Expr::Or => {
                self.comp_err("incomplete condition");
                exit(1);
            },
            Expr::And => {
                self.comp_err("incomplete condition");
                exit(1);
            },
            _ => (),
        }

        if !is_loop {
            self.new_scope(Expr::None);
        }

        arr_expr
    }

    fn check_if(&mut self) -> Expr {
        let mut expr = Expr::None;
        let stmnt = if self.has_if {
            Keyword::If
        } else if self.has_orif {
            Keyword::OrIf
        } else {
            Keyword::Else
        };

        if self.has_colon {
            return expr
        }

        if let Types::None = self.has_type {
            if let Keyword::Else = stmnt {
                expr = Expr::Else;
                return expr
            }
            let arr_expr = self.check_conditional(false);

            match stmnt {
                Keyword::If => expr = Expr::If(Box::new(Expr::Condition(Box::new(arr_expr)))),
                Keyword::OrIf => expr = Expr::OrIf(Box::new(Expr::Condition(Box::new(arr_expr)))),
                _ => (),
            }

        }

        expr
    }

    fn check_loop(&mut self) -> Expr {
        let mut expr = Expr::None;
        if !self.has_macro_name.is_empty() {
            return expr
        }

        if let Types::None = self.has_type {
            let arr_expr =  self.check_conditional(true);
            let mut loop_mod = Expr::None;

            if self.intlit_content.is_empty() || self.intlit_content.len() > 1 {
                self.comp_err("incorrect use of increment / decrement statement. only one character inside []");
                exit(1);
            }

            let mut varname = String::new();
            match &arr_expr[0] {
                Expr::Var(var_info) => {
                    match &var_info.0 {
                        Expr::VarName((_, vn)) => {
                            varname = vn.to_owned();
                        },
                        _ => (),
                    }
                },
                Expr::VarName((typ, vn)) => {
                    if let Types::Int = typ {
                        varname = vn.to_owned();
                    } else {
                        self.comp_err(&format!("{:?} {} is not a number", typ, varname));
                        exit(1);
                    }
                },
                _ => {
                    self.comp_err(&format!("first token in loop must be a variable, {:?} is not a variable", arr_expr[0]));
                    exit(1);
                },
            }
            
            match self.intlit_content.chars().nth(0) {
                Some(token) => {
                    match token {
                        '+' => loop_mod = Expr::LoopMod(format!("{varname}++")),
                        '-' => loop_mod = Expr::LoopMod(format!("{varname}--")),
                        _ => {
                            self.comp_err("modifier restricted to + or -");
                            exit(1);
                        },
                    }
                },
                None => (),
            }

            expr = Expr::Loop(Box::new((Expr::Condition(Box::new(arr_expr)), loop_mod)));
        }
        expr 
    }

    fn try_reassign_variable(&mut self, ident: Expr) -> Expr {
        let mut expr = Expr::None;

        if !self.expr_buffer.is_empty() {
            match &self.expr_buffer[0] {
                Expr::VarName((typ, varname)) => {
                    match typ {
                        Types::Str => {
                            println!("typ: {:?}, ident: {:?}", typ, ident);
                            match ident {
                                Expr::StrLit(ref _string) => {
                                    expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), varname.to_owned())), ident.clone())));
                                },
                                _ => {
                                    self.comp_err(&format!("expected string, got {:?}", typ));
                                    exit(1);
                                },
                            }
                        },
                        Types::Int => {
                            match ident {
                                Expr::IntLit(ref _integer) => {
                                    expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), varname.to_owned())), ident.clone())));
                                },
                                _ => {
                                    self.comp_err(&format!("expected int, got {:?}", typ));
                                    exit(1);
                                },
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
            self.expr_buffer.clear();
            return expr
        }

        if !self.has_macro_name.is_empty() || self.has_ref_name.is_empty() {
            return expr
        }

        let mut found_match = false;
        let mut typ = Types::None;
        let mut name = String::new();
        if let Types::None = self.has_type {
            let cur_func = self.extract_func_name();
            let variables_res = self.func_to_vars.get(&cur_func);
            let variables = match variables_res {
                Some(var) => var,
                None => {
                    self.comp_err(&format!("reassigning unassigned identifier {:?}", ident));
                    exit(1);
                }
            };

            if self.has_dot {
                match &self.has_ref_name[self.has_ref_name.len()-1] {
                    Expr::Var(ref_info) => {
                        match &ref_info.1 {
                            Expr::Condition(getter) => {
                                for get in *getter.clone() {
                                    match get {
                                        Expr::StructField(struct_field) => {
                                            match *struct_field {
                                                Expr::VarName((field_typ, field_name)) => {
                                                    if field_name == self.has_name {
                                                        typ = field_typ.clone();
                                                        name = field_name.clone();
                                                        found_match = true;
                                                        break;
                                                    }
                                                },
                                                _ => (),
                                            }
                                        },
                                        _ => (),
                                    } 
                                }
                            },
                            _ => (),
                        }
                    },
                    _ => (),
                }
            } else {
                for var in &variables[self.current_scope] {
                    match var {
                        Expr::Var(var_info) => {
                            match &var_info.0 {
                                Expr::VarName((_, list_name)) => {
                                    match &self.has_ref_name[0] {
                                        Expr::Var(var_info) => {
                                            match &var_info.0 {
                                                Expr::VarName((var_typ, var_name)) => {
                                                    if var_name == list_name {
                                                        typ = var_typ.clone();
                                                        name = var_name.clone();
                                                        found_match = true;
                                                    }
                                                },
                                                _ => (),
                                            }
                                        },
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }
            }
        }

        if found_match && self.has_colon && !self.has_lbrack && !self.has_rbrack && !self.has_lcurl {
            if self.has_dot {
                let mut fullname = String::new();
                for prop_name in &self.has_ref_name {
                    if let Expr::Var(ref_info) = prop_name {
                        if let Expr::VarName((_, partial_name)) = &ref_info.0 {
                            fullname.push_str(&format!("{partial_name}."));
                        }
                    }
                }
                fullname.push_str(&format!("{}", name));

                match ident {
                    Expr::StrLit(ref _string) => {
                        match typ {
                            Types::Str => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), fullname)), ident.clone()))),
                            _ => (),
                        }
                    },
                    _ => (),
                }
                return expr
            }

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
                            self.comp_err("mismatch types");
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m\"{}\"\x1b[0m", typ, name, string);
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
                            self.comp_err("mismatch types");
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", typ, name, integer);
                            exit(1);
                        }
                    }
                },
                Expr::VarName((ref var_typ, ref var_name)) => {
                    match (typ.clone(), var_typ) {
                        (Types::Int, Types::Int) => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        (Types::Str, Types::Str) => expr = Expr::ReVar(Box::new((Expr::VarName((typ.clone(), name.clone())), ident))),
                        _ => {
                            self.comp_err("mismatch types");
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", typ, name, var_name);
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

        if self.expr_buffer.len() == 2 {
            match &self.expr_buffer[0] {
                Expr::VarName((left_typ, varname)) => {
                    let mut found_semicolon = false;
                    let mut fullname = String::new();
                    let mut fields: Vec<Expr> = Vec::new();
                    match &self.expr_buffer[1] {
                        Expr::VarName((_, exist_var)) => {
                            fullname.push_str(&exist_var);

                            let cur_func = self.extract_func_name();
                            let variables_res = self.func_to_vars.get(&cur_func);
                            let variables = match variables_res {
                                Some(vars) => vars.clone(),
                                None => Vec::new(),
                            };

                            for vars in &variables[self.current_scope] {
                                match vars {
                                    Expr::Var(var_info) => {
                                        match &var_info.0 {
                                            Expr::VarName((_, list_var)) => {
                                                let first_name: &str = varname.split('.').collect::<Vec<&str>>()[0];
                                                if first_name == list_var {
                                                    match &var_info.1 {
                                                        Expr::Condition(getter) => {
                                                            for get in *getter.clone() {
                                                                match get {
                                                                    Expr::StructField(struct_field) => {
                                                                        fields.push(*struct_field.clone());
                                                                    },
                                                                    _ => (),
                                                                }
                                                            }
                                                        },
                                                        _ => (),
                                                    }
                                                }
                                            },
                                            _ => (),
                                        }
                                    },
                                    _ => (),
                                }
                            }
                        },
                        _ => (),
                    }

                    let mut right_typ = &Types::None;
                    for (i, token) in self.tokens[self.current_token+1..self.tokens.len()-1].iter().enumerate() {
                        if let Token::SemiColon = token {
                            found_semicolon = true;
                            self.current_token += i;
                            break;
                        } else if let Token::Dot = token {
                            fullname.push('.');
                        } else if let Token::Ident(field_name) = token {
                            let mut found = false;
                            for field in &fields {
                                match field {
                                    Expr::VarName((this_typ, name)) => {
                                        if field_name == name {
                                            fullname.push_str(&format!("{field_name}"));
                                            found = true;
                                            right_typ = this_typ;
                                            break;
                                        }
                                    },
                                    _ => (),
                                }
                            }

                            if !found {
                                self.comp_err(&format!("{field_name} does not exist as a struct field"));
                                exit(1);
                            }
                        }
                    }
                    if !found_semicolon {
                        self.comp_err("need ; at the end of getting struct field from variable");
                        exit(1);
                    }

                    match (left_typ, right_typ) {
                        (Types::Str, Types::Str) => {
                            expr = Expr::ReVar(Box::new((self.expr_buffer[0].clone(), Expr::VarName((Types::None, fullname)))));
                        },
                        (Types::Int, Types::Int) => {
                            expr = Expr::ReVar(Box::new((self.expr_buffer[0].clone(), Expr::VarName((Types::None, fullname)))));
                        },
                        _ => {
                            self.comp_err("mistmatch types");
                            exit(1);
                        },
                    }

                    let cur_func = self.extract_func_name();
                    if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                        vars[self.current_scope].push(expr.clone());
                    }
                },
                _ => (),
            }

            self.expr_buffer.clear();
            return expr
        }

        match self.has_type {
            Types::None => return expr,
            _ => (),
        }

        if self.has_name.is_empty() {
            return expr
        }

        if self.has_colon && !self.has_lcurl && !self.has_lbrack && self.has_macro_name.is_empty() {
            match value {
                Expr::StrLit(v) => {
                    let varname = Expr::VarName((Types::Str, self.has_name.clone()));
                    if let Types::Str = self.has_type {
                        expr = Expr::Var(Box::new((varname.clone(), Expr::StrLit(v.to_string()))));

                        let cur_func = self.extract_func_name();
                        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                            vars[self.current_scope].push(expr.clone());
                        }
                    } else {
                        self.comp_err("mistmatch types");
                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m\"{}\"\x1b[0m", self.has_type, self.has_name, v);
                        exit(1);
                    }
                },
                Expr::IntLit(v) => {
                    // even tho it's an int, it might be an int from an array, e.g. arr|0|
                    // don't need to check if the variable exists. self.has_ref_name checks.
                    // GOD PLEASE FORGIVE ME
                    let varname = Expr::VarName((Types::Int, self.has_name.clone()));
                    if !self.has_ref_name.is_empty() && self.has_pipe {
                        let arr_index = self.make_arr_index(self.has_ref_name[0].clone(), v);
                        let new_var = Expr::VarName((self.has_type.clone(), self.has_name.clone()));

                        if let Expr::ArrIndex(ref arr_info) = arr_index {
                            if let Expr::VarName((var_typ, _)) = &arr_info.0 {
                                if let Types::Arr(arr_typ) = var_typ {
                                    match **arr_typ {
                                        Types::Int => {
                                            if let Types::Int = self.has_type {
                                                expr = Expr::Var(Box::new((new_var, arr_index)));
                                                
                                                let cur_func = self.extract_func_name();
                                                if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                                    vars[self.current_scope].push(expr.clone());
                                                }
                                            } else {
                                                self.comp_err("mistmatch types");
                                                println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                                                exit(1);
                                            }
                                        },
                                        Types::Str => {
                                            if let Types::Str = self.has_type {
                                                expr = Expr::Var(Box::new((new_var, arr_index)));
                                                
                                                let cur_func = self.extract_func_name();
                                                if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                                    vars[self.current_scope].push(expr.clone());
                                                }
                                            } else {
                                                self.comp_err("mistmatch types");
                                                println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                                                exit(1);
                                            }
                                        },
                                        _ => {
                                            self.comp_err("unsupported type rn lols mb");
                                            exit(1);
                                        },
                                    }
                                }
                            }
                        }
                    } else if let Types::Int = self.has_type {
                        expr = Expr::Var(Box::new((varname, Expr::IntLit(v.to_string()))));

                        let cur_func = self.extract_func_name();
                        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                            vars[self.current_scope].push(expr.clone());
                        }
                    } else {
                        self.comp_err("mismatch types");
                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                        exit(1);
                    }
                },
                Expr::VarName((typ, v)) => {
                    match typ {
                        Types::Int => {
                            let varname = Expr::VarName((Types::Int, self.has_name.clone()));
                            if !self.has_ref_name.is_empty() && self.has_pipe {
                                let arr_index = self.make_arr_index(self.has_ref_name[0].clone(), v);
                                let new_var = Expr::VarName((self.has_type.clone(), self.has_name.clone()));

                                if let Expr::ArrIndex(ref arr_info) = arr_index {
                                    if let Expr::VarName((var_typ, _)) = &arr_info.0 {
                                        if let Types::Arr(arr_typ) = var_typ {
                                            match **arr_typ {
                                                Types::Int => {
                                                    if let Types::Int = self.has_type {
                                                        expr = Expr::Var(Box::new((new_var, arr_index)));
                                                        
                                                        let cur_func = self.extract_func_name();
                                                        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                                            vars[self.current_scope].push(expr.clone());
                                                        }
                                                    } else {
                                                        self.comp_err("mismatch types");
                                                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                                                        exit(1);
                                                    }
                                                },
                                                Types::Str => {
                                                    if let Types::Str = self.has_type {
                                                        expr = Expr::Var(Box::new((new_var, arr_index)));
                                                        
                                                        let cur_func = self.extract_func_name();
                                                        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                                            vars[self.current_scope].push(expr.clone());
                                                        }
                                                    } else {
                                                        self.comp_err("mismatch types");
                                                        println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                                                        exit(1);
                                                    }
                                                },
                                                _ => {
                                                    self.comp_err("unsupported type rn lols mb");
                                                    exit(1);
                                                },
                                            }
                                        }
                                    }
                                }
                            } else if let Types::Int = self.has_type {
                                expr = Expr::Var(Box::new((varname, Expr::VarName((Types::Int, v.clone())))));

                                let cur_func = self.extract_func_name();
                                if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                    vars[self.current_scope].push(expr.clone());
                                }
                            } else {
                                self.comp_err("unsupported type rn lols mb");
                                println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                                exit(1);
                            }
                        },
                        Types::Str => {
                            let varname = Expr::VarName((Types::Str, self.has_name.clone()));
                            println!("has_type: {:?}", self.has_type);
                            if let Types::Str = self.has_type {
                                expr = Expr::Var(Box::new((varname, Expr::VarName((Types::Str, v.clone())))));

                                let cur_func = self.extract_func_name();
                                if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                    vars[self.current_scope].push(expr.clone());
                                }
                            } else {
                                self.comp_err("unsupported type rn lols mb");
                                println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                                exit(1);
                            }
                        },
                        Types::UserDef(typedef) => {
                            // FIX THIS, MAKE IT ACTUALLY CHECK INSIDE OF ASSUMING THE USER IS
                            // RIGHT
                            let varname = Expr::VarName((Types::UserDef(typedef.clone()), self.has_name.clone()));
                            let mut found_semicolon = false;
                            let mut fullname = String::new();

                            let cur_func = self.extract_func_name();
                            let variables_res = self.func_to_vars.get(&cur_func);
                            let variables = match variables_res {
                                Some(vars) => vars.to_owned(),
                                None => Vec::new(),
                            };

                            let mut fields = Vec::new();
                            for vars in &variables[self.current_scope] {
                                let mut this_one = false;
                                match vars {
                                    Expr::Var(var_info) => {
                                        match &var_info.0 {
                                            Expr::VarName((_exist_typ, exist_name)) => {
                                                if v == exist_name {
                                                    this_one = true;
                                                }
                                            },
                                            _ => (),
                                        }

                                        if this_one {
                                            match &var_info.1 {
                                                Expr::Condition(getter) => {
                                                    for get in *getter.clone() {
                                                        match get {
                                                            Expr::StructField(struct_field) => {
                                                                fields.push(*struct_field.clone());
                                                            },
                                                            _ => (),
                                                        }
                                                    }
                                                },
                                                _ => (),
                                            }
                                            break;
                                        }
                                    },
                                    _ => (),
                                }

                            }

                            fullname.push_str(v);

                            let mut right_typ = Types::None;
                            for (i, token) in self.tokens[self.current_token+1..self.tokens.len()-1].iter().enumerate() {
                                if let Token::SemiColon = token {
                                    found_semicolon = true;
                                    self.current_token += i;
                                    break;
                                } else if let Token::Dot = token {
                                    fullname.push('.');
                                } else if let Token::Ident(field_name) = token {
                                    let mut found = false;
                                    for field in &fields {
                                        match field {
                                            Expr::VarName((this_typ, name)) => {
                                                if field_name == name {
                                                    fullname.push_str(&format!("{field_name}"));
                                                    found = true;
                                                    right_typ = this_typ.clone();
                                                    break;
                                                }
                                            },
                                            _ => (),
                                        }
                                    }

                                    if !found {
                                        self.comp_err(&format!("{field_name} does not exist as a struct field"));
                                        exit(1);
                                    }
                                }
                            }

                            if !found_semicolon {
                                self.comp_err("need ; at the end of getting struct field from variable");
                                exit(1);
                            }

                            if found_semicolon {
                                let new_var = Expr::VarName((self.has_type.clone(), self.has_name.clone()));
                                match (self.has_type.clone(), right_typ.clone()) {
                                    (Types::Str, Types::Str) => {
                                        expr = Expr::Var(Box::new((new_var, Expr::VarName((right_typ, fullname)))));
                                    },
                                    (Types::Int, Types::Int) => {
                                        expr = Expr::Var(Box::new((new_var, Expr::VarName((right_typ, fullname)))));
                                    },
                                    _ => {
                                        self.comp_err("mistmatch types");
                                        exit(1);
                                    },
                                }

                                let cur_func = self.extract_func_name();
                                if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                    vars[self.current_scope].push(expr.clone());
                                }
                            } else if let Types::UserDef(_) = self.has_type {
                                expr = Expr::Var(Box::new((varname, Expr::VarName((Types::UserDef(typedef.clone()), v.clone())))));
                                let cur_func = self.extract_func_name();
                                if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                                    vars[self.current_scope].push(expr.clone());
                                }
                            } else {
                                self.comp_err("mismatch types");
                                println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", self.has_type, self.has_name, v);
                                exit(1);
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
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
                            self.comp_err("mismatch types");
                            println!("\x1b[93m{:?}\x1b[0m \x1b[96m{}\x1b[0m: \x1b[96m{}\x1b[0m", typ, self.has_name, value);
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

        if !self.has_pipe {
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
                            self.comp_err("string array with mismatch elements.");
                            exit(1);
                        }
                    },
                    Token::Int(integer) => {
                        if let Types::Int = self.has_type {
                            expr_value.push(Expr::IntLit(integer.clone()))
                        } else {
                            self.comp_err("integer array with mismatch elements.");
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
                                    self.comp_err("integer array with mismatch elements.");
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
                    let expr_name = Expr::VarName((Types::Dynam(Box::new(self.has_type.clone())), self.has_name.clone()));
                    expr = Expr::Var(Box::new((expr_name, Expr::Dynamic(Box::new(expr_value)))));
                    let cur_func = self.extract_func_name();
                    if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                        vars[self.current_scope].push(expr.clone());
                    }
                },
                Macros::Arr => {
                    let expr_name = Expr::VarName((Types::Arr(Box::new(self.has_type.clone())), self.has_name.clone()));
                    expr = Expr::Var(Box::new((expr_name, Expr::Array(Box::new(expr_value)))));
                    let cur_func = self.extract_func_name();
                    if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                        vars[self.current_scope].push(expr.clone());
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
            self.comp_err("unexpected assignment operator \x1b[96m:\x1b[0m");
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
                                            self.comp_err("mismatch types");
                                            println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
                                            exit(1);
                                        }
                                    },
                                    _ => {
                                        self.comp_err("mismatch types");
                                        println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
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
                                            self.comp_err("mismatch types");
                                            println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
                                            exit(1);
                                        }
                                    },
                                    _ => {
                                        self.comp_err("mismatch types");
                                        println!("\x1b[93mneed type: {:?}\x1b[0m \x1b[96mgot: {:?}\x1b[0m", func_info.0, value);
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

    fn try_return_value(&mut self, name: String, typ: Types, params: Vec<Expr>) -> Expr {
        let expr: Expr;

        match self.has_type {
            Types::Int => {
                match typ {
                    Types::Int => (),
                    _ => {
                        self.comp_err("mismatch types");
                        exit(1);
                    },
                }
            },
            Types::Str => {
                match typ {
                    Types::Str => (),
                    _ => {
                        self.comp_err("mismatch types");
                        exit(1);
                    },
                }
            },
            _ => {
                self.comp_err("unsupported type rn lol soz");
                exit(1);
            },
        }
        expr = Expr::Var(Box::new(
            (
                Expr::VarName((typ, self.has_name.clone())),
                Expr::FuncCall(Box::new((Expr::FuncName(name), params)))
            )
        ));

        let cur_func = self.extract_func_name();
        if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
            vars[self.current_scope].push(expr.clone());
        }

        expr
    }

    fn try_func_call(&mut self) -> Expr {
        // FIND A WAY TO ERROR IF THE FUNC NAME DOESN'T EXIST

        let mut expr = Expr::None;
        let mut typ = Types::None;
        let mut name = String::new();

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
                                if self.has_colon {
                                    typ = func_info.0.clone();
                                    break;
                                }
                                break;
                            }
                        },
                        _=> (),
                    }
                },
                _ => (),
            }
        }

        match self.get_params() {
            Some(param) => {
                if let Types::None = typ {
                    expr = Expr::FuncCall(Box::new((Expr::FuncName(name), param)));
                } else {
                    expr = self.try_return_value(name, typ, param)
                }
            },
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

                if let Types::None = typ {
                    expr = Expr::FuncCall(Box::new((Expr::FuncName(name), expr_params)))
                } else {
                    expr = self.try_return_value(name, typ, expr_params)
                }
            },
        }

        expr
    }

    fn struct_field(&mut self) {
        if let Types::None = self.has_type {
            return;
        }

        if !self.has_name.is_empty() {
            self.expr_buffer.push(Expr::StructField(Box::new(Expr::VarName((self.has_type.clone(), self.has_name.clone())))));
        } else {
            self.comp_err(&format!("incorrect struct field, {}", self.has_name));
            exit(1);
        }
    }

    fn handle_var_struct_field(&mut self, value: &Expr) -> Expr {
        let expr: Expr;

        if self.has_colon {
            let mut fields = Vec::new();

            for struc in &self.structures {
                match struc {
                    Expr::Struct(struct_info) => {
                        match &struct_info.0 {
                            Expr::StructDef(struct_name) => {
                                if struct_name == &self.current_var_struct {
                                    fields = struct_info.1.clone();
                                    break;
                                }
                            },
                            _ => (),
                        }
                    },
                    _ => (),
                }
            }

            let mut found = false;
            let mut this_typ = Types::None;
            for field in fields {
                match field {
                    Expr::StructField(struct_field) => {
                        match *struct_field {
                            Expr::VarName((typ, name)) => {
                                if name == self.has_name {
                                    found = true;
                                    this_typ = typ.clone();
                                    break;
                                }
                            },
                            _ => (),
                        }
                    },
                    _ => (),
                }
            }

            if !found {
                self.comp_err(&format!("\x1b[91m{}\x1b[0m field does not exist in struct \x1b[93m{}\x1b[0m", self.has_name, self.current_var_struct));
                exit(1);
            }

            match this_typ {
                Types::Str => {
                    if let Expr::StrLit(_) = value {
                        expr = Expr::StructVarField(Box::new(Expr::Var(Box::new((Expr::VarName((this_typ, self.has_name.clone())), value.clone())))));
                    } else {
                        self.comp_err(&format!("mismatch types for struct field \x1b[93m{}\x1b[0m, expected \x1b[93m{:?}\x1b[0m, got \x1b[93m{:?}\x1b[0m", self.has_name, this_typ, value));
                        exit(1);
                    }
                },
                Types::Int => {
                    if let Expr::IntLit(_) = value {
                        expr = Expr::StructVarField(Box::new(Expr::Var(Box::new((Expr::VarName((this_typ, self.has_name.clone())), value.clone())))));
                    } else {
                        self.comp_err(&format!("mismatch types for struct field \x1b[93m{}\x1b[0m, expected \x1b[93m{:?}\x1b[0m, got \x1b[93m{:?}\x1b[0m", self.has_name, this_typ, value));
                        exit(1);
                    }
                },
                _ => {
                    self.comp_err("unsupported rn lol soz");
                    exit(1);
                },
            }
        } else {
            self.comp_err(&format!("incorrect struct field in variable, {}", self.has_name));
            exit(1);
        }

        expr
    }

    fn check_struct(&mut self) -> Expr {
        if !self.has_colon || !self.has_func_name.is_empty() || self.has_loop || self.has_name.is_empty() ||
            self.in_func || (self.has_if || self.has_orif || self.has_else) {
            self.comp_err(&format!("incorrect struct definintion, {}", self.has_name));
            exit(1);
        }

        if self.user_def_types.len() > 0 {
            for typ in &self.user_def_types {
                match typ {
                    Types::UserDef(name) => {
                        if name == &self.has_name {
                            self.comp_err(&format!("struct {} already defined", self.has_name));
                            exit(1);
                        }
                    }
                    _ => (),
                }
            }
        }

        let expr = Expr::StructDef(self.has_name.clone());
        self.struct_def = expr.clone();
        self.user_def_types.push(Types::UserDef(self.has_name.clone()));
        expr
    }

    fn check_block(&mut self) -> Expr {
        let mut expr = Expr::None;

        if let Types::UserDef(struct_name) = &self.has_type {
            // check if they making a variable with struct
            if self.has_colon && !self.has_lbrack && !self.has_rbrack {
                self.in_struct_var = true;

                let mut found = false;
                let mut struct_fields = Vec::new();
                for struc in &self.structures {
                    match struc {
                        Expr::Struct(struct_info) => {
                            match &struct_info.0 {
                                Expr::StructDef(struct_def) => {
                                    if struct_def == struct_name {
                                        struct_fields = struct_info.1.clone();
                                        found = true;
                                        break;
                                    }
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }

                if found {
                    expr = Expr::Var(Box::new((Expr::VarName((self.has_type.clone(), self.has_name.clone())), Expr::Condition(Box::new(struct_fields)))));
                    let cur_func = self.extract_func_name();
                    if let Some(vars) = self.func_to_vars.get_mut(&cur_func) {
                        vars[self.current_scope].push(expr.clone());
                    }
                }
            } else {
                self.comp_err(&format!("incorrect variable struct definition, {}", self.has_name));
                exit(1);
            }
        } else if self.has_def_struct {
            // check if they defining a new struct
            expr = self.check_struct();
        } else if self.has_if || self.has_orif || self.has_else {
            expr = self.check_if();
        } else if self.has_loop {
            expr = self.check_loop();
        } else {
            expr = self.check_func();
        }

        if let Expr::None = expr {
            self.comp_err(&format!("unknown code block declaration, {}", self.has_name));
            exit(1);
        }

        expr
    }

    fn handle_var_use(&mut self, ident: String) -> Expr {
        let mut expr = Expr::None;

        if self.has_dot {
            let cur_func = self.extract_func_name();
            let variables = self.func_to_vars.get(&cur_func);
            let vars = match variables {
                Some(v) => v,
                None => {
                    exit(1);
                },
            };

            for var in &vars[self.current_scope] {
                match var {
                    Expr::Var(var_info) => {
                        match &var_info.0 {
                            Expr::VarName((_, name)) => {
                                if !self.expr_buffer.is_empty() {
                                    if name == &ident {
                                        self.expr_buffer.push(var_info.0.clone());
                                        expr = self.try_variable(&var_info.0.clone());
                                        return expr
                                    }
                                }
                            },
                            _ => (),
                        }
                    },
                    _ => (),
                }
            }

            // MY GOD PLEASE FORGIVE THIS MONSTROSITY
            let mut found_property = false;
            for var in &vars[self.current_scope] {
                match var {
                    Expr::Var(var_info) => {
                        match &var_info.0 {
                            Expr::VarName((_, name)) => {
                                match &self.has_ref_name[self.has_ref_name.len()-1] {
                                    Expr::Var(ref_info) => {
                                        match &ref_info.0 {
                                            Expr::VarName((_, ref_name)) => {
                                                if name == ref_name {
                                                    match &ref_info.1 {
                                                        Expr::Condition(getter) => {
                                                            for get in *getter.clone() {
                                                                match get {
                                                                    Expr::StructField(struct_field) => {
                                                                        match *struct_field {
                                                                            Expr::VarName((field_typ, field_name)) => {
                                                                                if field_name == ident {
                                                                                    let fullname = format!("{name}.{field_name}");
                                                                                    self.expr_buffer.push(Expr::VarName((field_typ.clone(), fullname)));
                                                                                    found_property = true;
                                                                                    break;
                                                                                }
                                                                            },
                                                                            _ => (),
                                                                        }
                                                                    },
                                                                    _ => (),
                                                                }
                                                            }
                                                        },
                                                        _ => (),
                                                    }
                                                }
                                            },
                                            _ => (),
                                        }
                                    },
                                    _ => (),
                                }
                            },
                            _ => (),
                        }
                    },
                    _ => (),
                }
            }

            if !found_property {
                self.comp_err(&format!("struct \x1b[93m{}\x1b[0m doesn't have field \x1b[93m{}\x1b[0m", self.has_name, ident));
                exit(1);
            }

            return expr;
        }

        if self.has_def_struct {
            self.has_name = ident;
            self.struct_field();
            return expr;
        }

        if self.check_exist_ident(ident.clone()) {
            if self.in_struct_var {
                match self.has_type {
                    Types::None => (),
                    _ => {
                        self.comp_err(&format!("types are not allowed when defining a variable struct, {:?}", self.has_type));
                        exit(1);
                    },
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

                for v in vars[self.current_scope].clone() {
                    match v {
                        Expr::Var(var_info) => {
                            if let Expr::VarName(value) = &var_info.0 {
                                if value.1 == ident {
                                    expr = self.try_return(Expr::VarName(value.clone()));
                                }
                            }
                        }
                        _ => (),
                    }
                }
            } else if !self.has_ref_name.is_empty() {
                match &self.has_ref_name[self.has_ref_name.len()-1] {
                    Expr::Var(var_info) => {
                        if let Expr::VarName(value) = &var_info.0 {
                            if let Types::None = self.has_type {
                                expr = self.try_reassign_variable(Expr::VarName(value.clone()));
                            } else {
                                // here
                                expr = self.try_variable(&Expr::VarName(value.clone()));
                            }
                        }
                    },
                    _ => (),
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
                    Keyword::Println => expr = self.check_print(true),
                    Keyword::Print => expr = self.check_print(false),
                    Keyword::ReadIn => expr = self.check_readin(),
                    Keyword::None => expr = self.try_func_call(),
                    _ => (),
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

                    let macro_res = self.macros_map.get(&self.has_macro_name);
                    match macro_res {
                        Some(mac) => {
                            match mac {
                                Macros::C => self.has_cembed_macro = true,
                                _ => (),
                            }
                        },
                        None => (),
                    }
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
                        
                    if self.in_struct_var {
                        expr = self.handle_var_struct_field(&Expr::IntLit(word.to_owned()));
                    } else {
                        expr = self.try_return(num.1.clone());
                    }

                    if let Expr::None = expr {
                        expr = self.try_variable(&num.1);
                    }
                    if let Expr::None = expr {
                        expr = self.try_reassign_variable(num.1.clone());
                    }
                } else {
                    expr = self.handle_var_use(word.clone());
                }
            },
            Token::Lcurl => {
                self.has_lcurl = true;
                expr = self.check_block();
            },
            Token::Rcurl => {
                if self.has_def_struct {
                    let mut field_names = HashSet::new();
                    let mut found = false;
                    let mut line = 0;
                    for (i, struct_values) in self.expr_buffer.iter().enumerate() {
                        match struct_values {
                            Expr::StructField(field) => {
                                match *field.clone() {
                                    Expr::VarName((_, name)) => {
                                        if !field_names.insert(name) {
                                            line = i;
                                            found = true;
                                            break;
                                        }
                                    },
                                    _ => (),
                                }
                            },
                            _ => (),
                        }
                    }

                    if found {
                        self.comp_err(&format!("struct has repeated fields, struct field line: {}", line));
                        exit(1);
                    }

                    for elem in &self.expr_buffer {
                        println!("{:?}", elem);
                    }
                    self.structures.push(Expr::Struct(Box::new((self.struct_def.clone(), self.expr_buffer.clone()))));
                    self.program.append(&mut self.expr_buffer);

                    self.has_def_struct = false;
                    self.struct_def = Expr::None;
                    self.expr_buffer.clear();

                    expr = Expr::EndStruct;
                } else if self.in_struct_var {
                    self.in_struct_var = false;

                    expr = Expr::EndStructVar;
                } else {
                    self.prev_scope();
                    expr = Expr::EndBlock;
                }
            },
            Token::Str(word) => {
                self.has_strlit = true;
                let strlit = Expr::StrLit(word.to_string());

                if !self.has_macro_name.is_empty() {
                    match self.handle_import(&strlit) {
                        Some(value) => expr = value,
                        None => (),
                    }
                } else if self.in_struct_var {
                    expr = self.handle_var_struct_field(&strlit);
                }

                if let Expr::None = expr {
                    expr = self.try_variable(&strlit);
                }

                if let Expr::None = expr {
                    expr = self.try_reassign_variable(strlit.clone());
                }

                if let Expr::None = expr {
                    expr = self.try_return(strlit);
                }
            },
            Token::Int(integer) => {
                self.has_intlit = true;
                self.intlit_content.push_str(integer);

                if self.has_cembed_macro {
                    let trimmed = integer.trim().replace("\r", "").replace("    ", "");
                    expr = Expr::CEmbed(trimmed);
                } else {
                    let intlit = self.check_intlit(integer.to_string());

                    if self.in_struct_var {
                        expr = self.handle_var_struct_field(&intlit);
                    } else {
                        expr = self.try_variable(&intlit);
                    }

                    if let Expr::None = expr {
                        expr = self.try_reassign_variable(intlit.clone());
                    }

                    if let Expr::None = expr {
                        expr = self.try_return(intlit);
                    }
                }
                
            },
            Token::Macro => {
                self.has_macro = true;
            }
            Token::Pipe => {
                if self.has_pipe {
                    self.end_pipe_ix = self.current_token;
                    expr = self.try_arr();
                } else {
                    self.start_pipe_ix = self.current_token;
                    self.has_pipe = true;
                }
            },
            Token::Newline => self.line_num += 1,
            Token::Dot => {
                self.has_dot = true;
            },
            Token::SemiColon => {
                self.has_semicolon = true;
            },
            Token::Equal => (),
            Token::SmallerThan => (),
            Token::BiggerThan => (),
            Token::Exclaim => (),
            Token::Lsquare => (),
            Token::Rsquare => (),
            Token::Quote => (),
            Token::Divide => (),
            Token::Multiple => (),
            Token::Plus => (),
            Token::Minus => (),
            Token::Digit(_) => (),
        }

        self.current_token += 1;
        expr
    }

    pub fn parser(&mut self) -> Vec<Expr> {
        while self.current_token < self.tokens.len() {
            let expr = self.parse_to_expr();
            match expr {
                Expr::None => (),
                _ => {
                    println!("{:?}", expr);
                    self.program.push(expr);
                    self.clear();
                },
            }
        }

        self.program.clone()
    }
}
