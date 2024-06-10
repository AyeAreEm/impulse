use std::{fs, collections::HashMap, process::exit};
use crate::tokeniser::{tokeniser, Token}; use crate::declare_types::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Func {
        typ: Types,
        params: Vec<Expr>,
        name: String,
    },
    MacroFunc {
        typ: Types,
        params: Vec<Expr>,
        name: String,
    },
    FuncCall {
        name: String,
        gave_params: Vec<Expr>,
    },
    VariableName {
        typ: Types,
        name: String,
        reassign: bool,
        field_data: (bool, bool), // is_field, is_pointer
    },
    Variable {
        info: Box<Expr>, // Expr = VariableName
        value: Box<Expr>, // Expr = VariableName | StrLit | IntLit | FuncCall
    },
    DerefPointer(Box<Expr>),
    Address(Box<Expr>),

    If(Vec<Expr>),
    OrIf(Vec<Expr>),
    Else,

    Loop {
        condition: Vec<Expr>,
        modifier: Box<Expr>, // Expr = IntLit
    },

    Equal,
    SmallerThan,
    BiggerThan,
    Exclaim,

    And,
    Or,

    True,
    False,

    IntLit(String),
    StrLit {
        content: String,
        is_cstr: bool,
    },

    Import(String),
    CEmbed(String),

    Return(Box<Expr>),
    Break,
    Continue,

    StructName(String),
    StructDef {
        struct_name: Box<Expr>, // Expr = StructName
        struct_fields: Vec<Expr>, // Expr = VarName
    },
    EndStruct(String),

    MacroStructName {
        name: String,
        generics: Vec<Expr>,
    },
    MacroStructDef {
        struct_name: Box<Expr>, // Expr = StructName
        struct_fields: Vec<Expr>, // Expr = VarName
    },
    MacroEndStruct(String),

    ArrayLit(Vec<Expr>),
    // ArrayIndex {
    //     array_variable: Box<Expr>, // Expr = VarName
    //     index_at: Box<Expr>, // Expr = IntLit
    // },

    StartBlock,
    EndBlock,
    None,
}

#[derive(Debug, Clone)]
pub struct ExprWeights {
    token_stack: Vec<Token>,
    expr_stack: Vec<Expr>,

    tokens: Vec<Token>,
    current_token: usize,
    keyword_map: HashMap<String, Keyword>,
    macros_map: HashMap<String, Macros>,

    pub functions: Vec<Expr>,
    structures: Vec<Expr>,
    func_to_vars: HashMap<String, Vec<Vec<Expr>>>,

    current_func: String,
    current_scope: usize,

    in_scope: bool,
    in_func: bool,
    in_struct_def: bool,

    line_num: u32,
    
    imports: Vec<String>,
    program: Vec<(Expr, String, u32)>,
    filename: String,
}

impl ExprWeights {
    pub fn new(tokens: Vec<Token>, filename: &String) -> ExprWeights {
        let func_to_vars: HashMap<String, Vec<Vec<Expr>>> = HashMap::new();
        let keyword_map: HashMap<String, Keyword> = HashMap::from([
            // ("println".to_string(), Keyword::Println),
            // ("print".to_string(), Keyword::Print),
            // ("readin".to_string(), Keyword::ReadIn),

            ("i32".to_string(), Keyword::I32),
            ("int".to_string(), Keyword::Int),

            ("u8".to_string(), Keyword::U8),
            ("i8".to_string(), Keyword::I8),
            ("char".to_string(), Keyword::Char),

            ("usize".to_string(), Keyword::Usize),

            ("bool".to_string(), Keyword::Bool),

            ("typeid".to_string(), Keyword::TypeId),

            ("if".to_string(), Keyword::If),
            ("orif".to_string(), Keyword::OrIf),
            ("else".to_string(), Keyword::Else),

            ("or".to_string(), Keyword::Or),
            ("and".to_string(), Keyword::And),

            ("loop".to_string(), Keyword::Loop),

            ("return".to_string(), Keyword::Return),
            ("break".to_string(), Keyword::Break),
            ("continue".to_string(), Keyword::Continue),

            ("struct".to_string(), Keyword::Struct),
        ]);

        let macros_map: HashMap<String, Macros> = HashMap::from([
            ("c".to_string(), Macros::C),
            ("import".to_string(), Macros::Import),
            ("array".to_string(), Macros::Arr),
            // ("dynam".to_string(), Macros::Dynam),
        ]);

        ExprWeights {
            token_stack: Vec::new(),
            expr_stack: Vec::new(),

            tokens,
            current_token: 0,
            keyword_map,
            macros_map,

            functions: Vec::new(),
            func_to_vars,
            structures: Vec::new(),

            current_func: String::new(),
            current_scope: 0,

            in_scope: false,
            in_func: false,
            in_struct_def: false,

            line_num: 1,
            
            imports: Vec::new(),
            program: Vec::new(),
            filename: filename.to_owned(),
        }
    }

    fn comp_err(&self, error_msg: &str) {
        println!("\x1b[91merror\x1b[0m: {}:{}", self.filename, self.line_num);
        println!("\x1b[91merror\x1b[0m: {error_msg}");
    }

    fn program_push(&mut self, expr: Expr) {
        self.program.push((expr, self.filename.clone(), self.line_num));
    }

    fn error_if_stack_not_empty(&self) {
        if !self.token_stack.is_empty() {
            self.comp_err("might be a missing `;`. stack is not empty when it should be");
            exit(1);
        }
    }

    fn keyword_to_type(&self, kw: Keyword) -> Types {
        match kw {
            Keyword::I32 => Types::I32,
            Keyword::U8 => Types::U8,
            Keyword::I8 => Types::I8,
            Keyword::Char => Types::Char,
            Keyword::Usize => Types::Usize,
            Keyword::Int => Types::Int,
            Keyword::Bool => Types::Bool,
            Keyword::TypeId => Types::TypeId,
            Keyword::Generic(typ) => Types::Generic(typ),
            Keyword::TypeDef { type_name, generics } =>  Types::TypeDef { type_name, generics },
            Keyword::Pointer(pointer_to, _) => Types::Pointer(Box::new(pointer_to)),
            Keyword::Address => Types::Address,
            _ => {
                self.comp_err(&format!("can't convert {:?} to a type. type might not be reimplemented yet or defined.", kw));
                exit(1);
            },
        }
    }

    fn new_scope(&mut self, new_var: Expr) {
        self.in_scope = true;

        if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
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
        if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
            if self.in_scope && self.current_scope == 0 {
                self.in_scope = false;
            } else if self.in_scope && self.current_scope > 0 {
                // self.in_scope = false; This might slight be broken
                vars[self.current_scope].pop();
                self.current_scope -= 1;
            } else {
                self.in_func = false;
                vars[self.current_scope].pop();
            }
        }

        if !self.in_func {
            self.func_to_vars.remove(&self.current_func);
        }
    }

    fn check_intlit(&self, intlit: String) -> Expr {
        // TODO: MAKE THIS SUPPORT OTHER NUMBER TYPES LIKE U8

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
            ('%', Token::Mod),
            // ('.', Token::Dot),
        ]);

        let mut tokens: Vec<Token> = Vec::new();
        let mut buf = String::new();

        for (i, ch) in intlit.chars().enumerate() {
            if ch == ' ' || ch == '\n' || ch == '\r' {
                if buf.len() > 0 {
                    tokens.push(Token::Ident(buf.clone()));
                    buf.clear();
                }

                continue;
            }

            if i == intlit.len()-1 {
                buf.push(ch);
                tokens.push(Token::Ident(buf.clone()));
                buf.clear();
                break;
            }

            let integer_res = ch.to_digit(10);
            match integer_res {
                Some(_) => {
                    buf.push(ch);
                    continue;
                },
                None => (),
            }

            let token_res = symb_to_token.get(&ch);
            match token_res {
                Some(token) => {
                    if buf.len() > 0 {
                        tokens.push(Token::Ident(buf.clone()));
                        buf.clear();
                    }
                    tokens.push(token.clone());
                },
                None => {
                    buf.push(ch);
                },
            }
        }

        let mut clean = String::new();
        let mut has_func = Expr::None;
        let mut params = Vec::new();
        let mut brack_rc = 0;

        for token in tokens {
            match token {
                Token::Rbrack => {
                    brack_rc -= 1;
                    if brack_rc > 0 {
                        params.push(token);
                        continue;
                    } else {
                        match has_func {
                            Expr::None => {
                                self.comp_err(&format!("found () without a function name. did you mean to use []?"));
                                exit(1);
                            },
                            Expr::Func { .. } => self.create_func_call(&has_func, params.clone()),
                            _ => {
                                self.comp_err(&format!("unexpected expression in integer literal: {:?}", has_func));
                                exit(1);
                            },
                        }
                    };
                },
                _ => {
                    if brack_rc > 0 {
                        params.push(token.clone());
                        continue;
                    }
                },
            }

            match token {
                Token::Plus => clean.push('+'),
                Token::Minus => clean.push('-'),
                Token::Multiple => clean.push('*'),
                Token::Divide => clean.push('/'),
                Token::Lsquare => clean.push('('),
                Token::Rsquare => clean.push(')'),
                Token::Mod => clean.push('%'),
                Token::Ident(ident) => {
                    // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                    let ident_num = ident.parse::<i32>();
                    let is_num = match ident_num {
                        Ok(_) => true,
                        Err(_) => false,
                    };

                    if is_num {
                        clean.push_str(&ident);
                        continue;
                    }

                    let expr = self.find_ident(ident.to_string());
                    match expr {
                        Expr::None => {
                            self.comp_err(&format!("unknown identifier: {}", ident));
                            exit(1);
                        },
                        Expr::VariableName { typ, name, field_data, .. } => {
                            if field_data.0 && field_data.1 {
                                match typ {
                                    Types::I32 | Types::U8 | Types::I8 | Types::Int |
                                    Types::Usize | Types::Pointer(_) | Types::Generic(_) => {
                                        let new_name = ident.replace(".", "->");
                                        clean.push_str(&new_name)
                                    } ,
                                    _ => {
                                        self.comp_err(&format!("variable {name} is not an integer. {typ:?}:{name}"));
                                        exit(1);
                                    },
                                }
                            } else {
                                match typ {
                                    Types::I32 | Types::U8 | Types::I8 | Types::Int |
                                    Types::Usize | Types::Pointer(_) | Types::Generic(_) => clean.push_str(&ident),
                                    _ => {
                                        self.comp_err(&format!("variable {name} is not an integer. {typ:?}:{name}"));
                                        exit(1);
                                    }
                                }
                            }
                        },
                        Expr::Func { typ, params, name } => {
                            match typ {
                                Types::I32 => {
                                    has_func = Expr::Func { typ, params, name };
                                },
                                _ => {
                                    self.comp_err(&format!("function {name} does not return integer. {typ:?}:{name}"));
                                    exit(1);
                                },
                            }
                        },
                        _ => {
                            self.comp_err(&format!("unexpected expression in integer literal: {}", ident));
                            exit(1);
                        }
                    }
                },
                Token::Lbrack => {
                    brack_rc += 1;
                },
                _ => (),
            }
        }

        Expr::IntLit(clean)
    }

    fn create_keyword_pointer(&self, typ: Types, pointer_counter: i32) -> (Keyword, i32) {
        let mut tmp = typ.clone();

        for _ in 0..pointer_counter-1 {
            tmp = Types::Pointer(Box::new(tmp));
        }

        return (Keyword::Pointer(tmp, typ), 0)
    }

    fn create_func(&mut self, typ: Types, params: Vec<Token>, name: String) {
        let mut variables = Vec::new();
        let mut expr_param = Vec::new();
        let mut typeid_names: Vec<String> = Vec::new();
        let mut kw_buf = Keyword::None;
        let mut pointer_counter = 0;
        let mut is_macro_func = false;
        let mut is_generic = false;

        for (i, param) in params.iter().enumerate() {
            match param {
                Token::Ident(ident) => {
                    if let Keyword::None = kw_buf {
                        let keyword_res = self.keyword_map.get(ident);
                        match keyword_res {
                            Some(kw) => {
                                if pointer_counter > 0 {
                                    (kw_buf, pointer_counter) = self.create_keyword_pointer(self.keyword_to_type(kw.clone()), pointer_counter);
                                } else if is_generic {
                                    self.comp_err(&format!("declaring generic but {} is already an identifier", ident));
                                    exit(1);
                                } else {
                                    kw_buf = kw.clone();
                                    if let Keyword::TypeId = kw_buf {is_macro_func = true};
                                }
                            },
                            None => {
                                let found_struct = self.find_structure(ident);
                                if let Expr::StructDef { .. } = found_struct {
                                    if pointer_counter > 0 {
                                        (kw_buf, pointer_counter) = self.create_keyword_pointer(Types::TypeDef {
                                            type_name: ident.to_owned(),
                                            generics: vec![],
                                        }, pointer_counter);
                                    } else {
                                        kw_buf = Keyword::TypeDef {
                                            type_name: ident.to_string(),
                                            generics: vec![],
                                        };
                                    }
                                } else if let Expr::MacroStructDef { .. } = found_struct {
                                    let mut pass_typs = Vec::new();
                                    let mut name_buf = String::new();

                                    if i == params.len() {
                                        self.comp_err(&format!("expected more tokens when creating function"));
                                        exit(1);
                                    }

                                    if let Token::Int(typs) = &params[i+1] {
                                        for (index, ch) in typs.chars().enumerate() {
                                            if ch == ' ' || ch == '\n' || index == typs.len()-1 {
                                                if index == typs.len() - 1 {
                                                    name_buf.push(ch);
                                                }
                                                let keyword_rs = self.keyword_map.get(&name_buf);
                                                match keyword_rs {
                                                    Some(keyword) => {
                                                        let _ = self.keyword_to_type(keyword.clone());
                                                        pass_typs.push(name_buf.clone());
                                                    },
                                                    None => {
                                                        let mut found = false;
                                                        for name in &typeid_names {
                                                            if name == &name_buf {
                                                                pass_typs.push(name_buf.clone());
                                                                found = true;
                                                                break;
                                                            }
                                                        }

                                                        if !found {
                                                            self.comp_err(&format!("expected a defined typeid, found undefined `{name_buf}`"));
                                                            exit(1);
                                                        }
                                                    },
                                                }

                                                name_buf.clear();
                                            } else {
                                                name_buf.push(ch);
                                            }
                                        }
                                    } else {
                                        self.comp_err(&format!("expected a typ after struct type, e.g. `my_struct[type]`"));
                                        exit(1);
                                    }

                                    kw_buf = Keyword::TypeDef {
                                        type_name: ident.to_string(),
                                        generics: pass_typs,
                                    };

                                    if pointer_counter > 0 {
                                        (kw_buf, pointer_counter) = self.create_keyword_pointer(self.keyword_to_type(kw_buf), pointer_counter);
                                    }
                                } else if is_generic {
                                    for name in &typeid_names {
                                        if name == ident {
                                            kw_buf = Keyword::Generic(ident.to_owned());
                                            is_generic = false;
                                            break;
                                        }
                                    }

                                    if is_generic {
                                        self.comp_err(&format!("generic type {ident} hasn't been defined as a typeid yet."));
                                        exit(1);
                                    }
                                } else {
                                    self.comp_err(&format!("expected a type, got {}", ident));
                                    exit(1);
                                }
                            },
                        }
                    } else {
                        let typ = self.keyword_to_type(kw_buf.clone());
                        if let Types::TypeDef {type_name: ref user_def, .. } = typ {
                            if let Expr::StructDef { struct_fields, .. } | Expr::MacroStructDef { struct_fields, .. } = self.find_structure(&user_def) {
                                for field in struct_fields {
                                    match field {
                                        Expr::VariableName { typ: vartyp, name: varname, .. } => {
                                            let new_name = format!("{ident}.{varname}");
                                            let new_expr = Expr::Variable {
                                                info: Box::new(Expr::VariableName {
                                                    typ: vartyp,
                                                    name: new_name,
                                                    reassign: false,
                                                    field_data: (true, false),
                                                }),
                                                value: Box::new(Expr::None),
                                            };

                                            expr_param.push(new_expr);
                                        },
                                        _ => (),
                                    }
                                }
                            }
                        } else if let Keyword::Pointer(.., last) = kw_buf {
                            if let Types::TypeDef { type_name: ref user_def, .. } = last {
                                if let Expr::StructDef { struct_fields, .. } | Expr::MacroStructDef { struct_fields, .. } = self.find_structure(&user_def) {
                                    for field in struct_fields {
                                        match field {
                                            Expr::VariableName { typ: vartyp, name: varname, .. } => {
                                                let new_name = format!("{ident}.{varname}");
                                                let new_expr = Expr::Variable {
                                                    info: Box::new(Expr::VariableName {
                                                        typ: Types::Pointer(Box::new(vartyp)),
                                                        name: new_name,
                                                        reassign: false,
                                                        field_data: (true, true),
                                                    }),
                                                    value: Box::new(Expr::None),
                                                };

                                                expr_param.push(new_expr);
                                            },
                                            _ => (),
                                        }
                                    }
                                }
                            }
                        } else if let Types::TypeId = typ {
                            typeid_names.push(ident.to_owned());
                        }

                        let final_expr = Expr::VariableName {
                            typ,
                            name: ident.to_owned(),
                            reassign: false,
                            field_data: (false, false),
                        };

                        variables.push(final_expr.clone());
                        expr_param.push(
                            Expr::Variable {
                                info: Box::new(final_expr),
                                value: Box::new(Expr::None),
                            }
                        );
                        kw_buf = Keyword::None;
                    }
                },
                Token::Caret => {
                    pointer_counter += 1;
                },
                Token::Underscore => {
                    if pointer_counter > 0 {
                        (kw_buf, pointer_counter) = self.create_keyword_pointer(Types::Void, pointer_counter);
                    } else {
                        self.comp_err(&format!("can't use void as type. void pointers work tho."));
                        exit(1);
                    }
                },
                Token::Dollar => {
                    is_generic = true;
                },
                Token::Int(_) => (),
                _ => {
                    self.comp_err(&format!("unexpected token in function argument: {param:?}"));
                    exit(1);
                }
            }
        }

        let found_func = self.find_func(&name);
        if let Expr::Func { .. } = found_func {
            self.comp_err(&format!("identifier {name} already declared as another function"));
            exit(1);
        } else if let Expr::MacroFunc { .. } = found_func {
            self.comp_err(&format!("identifier {name} already declared as another function"));
            exit(1);
        } else if let Expr::StructDef { .. } = self.find_structure(&name) {
            self.comp_err(&format!("identifier {name} already declared as struct"));
            exit(1);
        }

        let expr = if is_macro_func {
            Expr::MacroFunc { typ, params: variables, name: name.clone() }
        } else {
            Expr::Func { typ, params: variables, name: name.clone() }
        };
        self.functions.push(expr.clone());
        self.current_func = name.clone();
        self.func_to_vars.entry(name.clone()).or_insert(vec![expr_param]);
        self.in_func = true;

        self.program_push(expr);
        self.token_stack.clear();
    }

    // this starts defining a struct
    fn create_struct_def(&mut self, name: String, generics: Expr) {
        if self.in_func {
            self.comp_err(&format!("cannot make struct {name} inside a function"));
            exit(1);
        }

        // don't need to check if num, already checked when getting name param
        let keyword_res = self.keyword_map.get(&name);
        match keyword_res {
            Some(kw) => {
                self.comp_err(&format!("can't use keyword {kw:?} as struct name"));
                exit(1);
            },
            None => (),
        }

        let found_func = self.find_func(&name);
        if let Expr::Func { .. } = found_func {
            self.comp_err(&format!("identifier {name} already declared as another function"));
            exit(1);
        } else if let Expr::MacroFunc { .. } = found_func {
            self.comp_err(&format!("identifier {name} already declared as another function"));
            exit(1);
        } else if let Expr::StructDef { .. } = self.find_structure(&name) {
            self.comp_err(&format!("identifier {name} already declared as struct"));
            exit(1);
        }

        let mut pass_generic = Vec::new();
        match generics {
            Expr::None => (),
            Expr::IntLit(generic) => {
                let mut name_buf = String::new();
                for (i, ch) in generic.chars().enumerate() {
                    if ch == ' ' || ch == '\n' || i == generic.len()-1 {
                        if i == generic.len() - 1 {
                            name_buf.push(ch);
                        }

                        let gen_var = Expr::Variable {
                            info: Box::new(Expr::VariableName {
                                typ: Types::TypeId,
                                name: name_buf.clone(),
                                reassign: false,
                                field_data: (false, false),
                            }),
                            value: Box::new(Expr::None)
                        };
                        pass_generic.push(gen_var);
                        name_buf.clear();
                    } else {
                        name_buf.push(ch);
                    }
                }
            },
            unexpected => {
                self.comp_err(&format!("unexpected expression {unexpected:?} when making struct {name}"));
                exit(1);
            },
        }

        self.current_func = name.clone();
        self.func_to_vars.entry(name.clone()).or_insert(vec![pass_generic.clone()]);
        self.in_struct_def = true;

        if pass_generic.is_empty() {
            self.expr_stack.push(Expr::StructName(name));
        } else {
            self.expr_stack.push(Expr::MacroStructName{name, generics: pass_generic});
        }
        self.token_stack.clear();
    }

    // this creates the final struct after defining reaches }
    fn create_struct(&mut self) {
        let name = self.expr_stack.remove(0);
        let exprs = self.expr_stack.clone();
        self.expr_stack.clear();

        // push to a list of structs
        let expr = if let Expr::MacroStructName { .. } = name {
            Expr::MacroStructDef { struct_name: Box::new(name.clone()), struct_fields: exprs }
        } else {
            Expr::StructDef { struct_name: Box::new(name.clone()), struct_fields: exprs }
        };
        self.program_push(expr.clone());
        match name {
            Expr::StructName(struct_name) => {
                self.program_push(Expr::EndStruct(struct_name.clone()));
                self.func_to_vars.remove(&struct_name);
            },
            Expr::MacroStructName { name, .. } => {
                self.program_push(Expr::MacroEndStruct(name));
            },
            _ => {
                self.comp_err(&format!("unexpected expression when creating struct"));
                exit(1);
            }
        }

        self.in_struct_def = false;
        self.structures.push(expr);
    }

    fn boolean_conditions(&self, params: &Vec<Token>, is_loop: bool) -> (Vec<Expr>, Expr) {
        if params.is_empty() {
            self.comp_err(&format!("expected expressions in boolean condition, got nothing"));
            exit(1);
        }

        let mut expr_params = Vec::new();
        let mut side_affect = Expr::None;
        let mut had_index = false;

        for (i, param) in params.iter().enumerate() {
            match param {
                Token::Equal => expr_params.push(Expr::Equal),
                Token::SmallerThan => expr_params.push(Expr::SmallerThan),
                Token::BiggerThan => expr_params.push(Expr::BiggerThan),
                Token::Exclaim => expr_params.push(Expr::Exclaim),
                Token::True => expr_params.push(Expr::True),
                Token::False => expr_params.push(Expr::False),
                Token::Ident(ident) => {
                    // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                    let ident_num = ident.parse::<i32>();
                    let is_num = match ident_num {
                        Ok(_) => true,
                        Err(_) => false,
                    };

                    if is_num {
                        expr_params.push(Expr::IntLit(ident.to_string()));
                        continue;
                    }

                    let keyword_res = self.keyword_map.get(ident);
                    let keyword = match keyword_res {
                        Some(k) => k.clone(),
                        None => Keyword::None,
                    };

                    match keyword {
                        Keyword::Or => {
                            expr_params.push(Expr::Or);
                            continue;
                        },
                        Keyword::And => {
                            expr_params.push(Expr::And);
                            continue;
                        }
                        Keyword::None => (),
                        _ => {
                            self.comp_err(&format!("unexpected keyword {keyword:?} in boolean condition"));
                            exit(1);
                        },
                    }

                    let expr = self.find_ident(ident.to_string());
                    match expr {
                        Expr::Func { name, .. } => {
                            if let Token::Lbrack = params[i+1] {
                                self.comp_err(&format!("function call inside conditions not reimplemented yet"));
                                exit(1);
                            } else {
                                self.comp_err(&format!("can't compare to function. did you mean to call {name}?. do `{name}()`"));
                                exit(1);
                            }
                        },
                        Expr::VariableName { typ, name, reassign, field_data } => {
                            if i != params.len()-1 {
                                if let Token::Int(index) = &params[i+1] {
                                    had_index = true;
                                    expr_params.push(Expr::VariableName {
                                        typ: Types::ArrIndex {
                                            arr_typ: Box::new(typ),
                                            index_at: index.to_owned(),
                                        },
                                        name,
                                        reassign,
                                        field_data
                                    });
                                } else {
                                    expr_params.push(Expr::VariableName { typ, name, reassign, field_data });
                                }
                            } else {
                                expr_params.push(Expr::VariableName { typ, name, reassign, field_data });
                            }
                        },
                        Expr::None => {
                            if is_loop && expr_params.is_empty() {
                                let new_varname = Expr::VariableName {
                                    typ: Types::I32,
                                    name: ident.to_owned(),
                                    reassign: true,
                                    field_data: (false, false),
                                };

                                expr_params.push(new_varname.clone());
                                side_affect = Expr::Variable {
                                    info: Box::new(new_varname),
                                    value: Box::new(Expr::IntLit(String::from("0"))),
                                };
                            } else {
                                self.comp_err(&format!("unknown identifier: {}", ident));
                                exit(1);
                            }
                        },
                        _ => {
                            self.comp_err(&format!("unexpected expression: {:?}", expr));
                            exit(1);
                        }
                    }
                },
                Token::Int(intlit) => {
                    if had_index {
                        had_index = false;
                        continue;
                    }
                    expr_params.push(self.check_intlit(intlit.to_owned()));
                },
                _ => {
                    self.comp_err(&format!("unexpected token in boolean condition: {:?}", param));
                    exit(1);
                },
            }
        }

        match expr_params[expr_params.len()-1] {
            Expr::Equal | Expr::SmallerThan |
            Expr::BiggerThan | Expr::Exclaim |
            Expr::And | Expr::Or => {
                self.comp_err("incomplete condition");
                exit(1);
            },
            _ => (),
        }

        (expr_params, side_affect)
    }

    fn create_branch(&mut self, branch_typ: Keyword, params: Vec<Token>) {
        let expr_params = self.boolean_conditions(&params, false).0; // only getting the array

        match branch_typ {
            Keyword::If => {
                self.token_stack.clear();
                self.new_scope(Expr::None);
                self.program_push(Expr::If(expr_params));
            },
            Keyword::OrIf => {
                self.token_stack.clear();
                self.new_scope(Expr::None);
                self.program_push(Expr::OrIf(expr_params));
            },
            _ => (),
        }
    }

    fn create_loop(&mut self, params: Vec<Token>, mut modifier: String) {
        let expr_params = self.boolean_conditions(&params, true);

        if modifier.is_empty() {
            modifier = "_".to_string();
        }

        match modifier.as_str() {
            "+" | "-" | "_" => {
                self.token_stack.clear();
                match expr_params.1 {
                    Expr::None | Expr::Variable { .. } => self.new_scope(expr_params.1),
                    _ => {
                        self.comp_err(&format!("unexpected expression in loop. {:?}", expr_params.1));
                        exit(1);
                    },
                }

                self.program_push(Expr::Loop {
                    condition: expr_params.0,
                    modifier: Box::new(Expr::IntLit(modifier.to_owned())),
                });
            },
            _ => {
                self.comp_err(&format!("unexpected loop modifier: {modifier}"));
                exit(1);
            }
        }
    }

    fn handle_lcurl(&mut self) {
        let mut typ: Types = Types::None;
        let mut name = String::new();
        let mut keyword = Keyword::None;

        let mut brack_rc = 0;
        let mut in_bracks = false;
        let mut seen_colon = 0;
        let mut pointer_counter = 0;

        let mut params = Vec::new();

        let mut create_branch = false;

        let mut create_loop = false;
        let mut loop_modifier = String::new();

        let mut create_struct = false;
        let mut create_generic = false;
        let mut generic_subtype = Expr::None;

        for (i, token) in self.token_stack.iter().enumerate() {
            match token {
                Token::Dollar => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        create_generic = true;
                    }
                },
                Token::Caret => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        pointer_counter += 1;
                    }
                },
                // TODO: NOT SURE IF THIS IS NEEDED, WILL REMOVE
                Token::Underscore => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        if name.is_empty() || pointer_counter > 0 {
                            typ = Types::Void;

                            if pointer_counter > 0 {
                                let tmp_kw = self.create_keyword_pointer(Types::Void, pointer_counter).0;
                                typ = self.keyword_to_type(tmp_kw);
                            }
                        } else if let Token::Ident(_) = self.token_stack[i+1] {
                            name.push('_');
                        }
                    }
                },
                Token::Ident(ident) => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                        let ident_num = ident.parse::<i32>();
                        match ident_num {
                            Ok(_) => {
                                self.comp_err(&format!("can't use a number as an identifier. try `name{ident}`?"));
                                exit(1);
                            },
                            Err(_) => (),
                        }

                        if let Keyword::None = keyword {
                            let keyword_res = self.keyword_map.get(ident);
                            keyword = match keyword_res {
                                Some(kw) => kw.clone(),
                                None => Keyword::None,
                            };

                            if let Expr::StructDef { .. } | Expr::MacroStructDef { .. } = self.find_structure(ident) {
                                keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: vec![] };
                            }

                            match keyword {
                                Keyword::If | Keyword::OrIf | Keyword::Else => create_branch = true,
                                Keyword::Loop => create_loop = true,
                                Keyword::Struct => create_struct = true,
                                _ => {
                                    if let Types::None = typ {
                                        typ = if create_generic {
                                            // TODO: check if ident is already declared
                                            Types::Generic(ident.to_owned()) // might cause errors idk lmao
                                        } else {self.keyword_to_type(keyword.clone())};
                                        if pointer_counter > 0 {
                                            let tmp_kw = self.create_keyword_pointer(typ, pointer_counter).0;
                                            typ = self.keyword_to_type(tmp_kw);
                                        }
                                    } else {
                                        name.push_str(&ident);
                                    }
                                },
                            }
                        } else {
                            name.push_str(&ident);
                        }
                    }
                },
                Token::Lbrack => {
                    brack_rc += 1;
                    if brack_rc > 0 {
                        in_bracks = true;
                    }
                },
                Token::Rbrack => {
                    brack_rc -= 1;
                    if brack_rc == 0 {
                        in_bracks = false;
                    }
                },
                Token::Colon => {
                    seen_colon += 1;
                },
                Token::Equal => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err("`=` needs to be inside brackets");
                        exit(1);
                    }
                },
                Token::SmallerThan => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err("`<` needs to be inside brackets");
                        exit(1);
                    }
                },
                Token::BiggerThan => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err("`>` needs to be inside brackets");
                        exit(1);
                    }
                },
                Token::Exclaim => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err("`!` needs to be inside brackets");
                        exit(1);
                    }
                },
                Token::True => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err("expected `true` in brackets");
                        exit(1);
                    }
                },
                Token::False => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err("expected `false` in brackets");
                        exit(1);
                    }
                },
                Token::Int(symbols) => {
                    if create_loop {
                        loop_modifier = symbols.to_owned();
                        break;
                    } else if in_bracks {
                        params.push(token.clone());
                    } else if create_struct {
                        generic_subtype = Expr::IntLit(symbols.to_string());
                    } else {
                        self.comp_err(&format!("unexpected integer literal in block definition: {symbols}"));
                        exit(1);
                    }
                },
                Token::Lsquare => (),
                Token::Rsquare => (), // these might break stuff idk
                unexpected => {
                    self.comp_err(&format!("unexpected token: {unexpected:?}"));
                    exit(1);
                },
            }
        }

        if create_loop {
            if self.in_struct_def {
                self.comp_err("can't use loops inside structs");
                exit(1);
            }

            self.create_loop(params, loop_modifier);
            return
        }

        if create_branch {
            if self.in_struct_def {
                self.comp_err("can't use branches inside structs");
                exit(1);
            }

            match keyword {
                Keyword::Else => {
                    if !params.is_empty() {
                        self.comp_err("expected no conditions for else branch, got one");
                        exit(1);
                    }

                    self.token_stack.clear();
                    self.program_push(Expr::Else);
                    self.new_scope(Expr::None);
                    return
                }
                _ => self.create_branch(keyword, params),
            }
            return
        }

        if create_struct {
            if self.in_struct_def {
                self.comp_err("can't use create a struct inside structs");
                exit(1);
            }

            if seen_colon != 1 {
                self.comp_err(&format!("expected assigment operator `:`. did you mean `struct {name}: {{`?"));
                exit(1);
            } else {
                self.create_struct_def(name.clone(), generic_subtype);
                return
            }
        }

        if seen_colon == 1 {
            if self.in_struct_def {
                self.comp_err("can't use create a function inside structs");
                exit(1);
            }

            self.create_func(typ.clone(), params.clone(), name.clone());
            return
        } else if seen_colon > 1 {
            self.comp_err("unexpected assignment operator `:`");
            exit(1);
        }

        self.new_scope(Expr::None);
        self.program_push(Expr::StartBlock);
    }

    fn find_variable(&self, ident: &String) -> Expr {
        let variables_res = self.func_to_vars.get(&self.current_func);
        let variables = match variables_res {
            Some(vars) => vars.clone(),
            None => return Expr::None,
        };

        for vars in &variables[self.current_scope] {
            match vars {
                Expr::Variable { info, .. } => {
                    match *info.clone() {
                        Expr::VariableName { name, .. } => {
                            if &name == ident {
                                return *info.clone();
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }

        Expr::None
    }

    fn find_func(&self, ident: &String) -> Expr {
        for func in &self.functions {
            match func {
                Expr::Func { name, .. } | Expr::MacroFunc { name, .. } => {
                    if name == ident {
                        return func.clone()
                    }
                },
                _ => (),
            }
        }

        Expr::None
    }

    fn find_structure(&self, ident: &String) -> Expr {
        for struc in &self.structures {
            match struc {
                Expr::StructDef { struct_name, .. } => {
                    match *struct_name.clone() {
                        Expr::StructName(acc_name) => {
                            if &acc_name == ident {
                                return struc.clone()
                            }
                        },
                        _ => (),
                    }
                },
                Expr::MacroStructDef { struct_name, .. } => {
                    match *struct_name.clone() {
                        Expr::MacroStructName { name: acc_name, .. } => {
                            if &acc_name == ident {
                                return struc.clone()
                            }
                        },
                        Expr::StructName(acc_name) => {
                            if &acc_name == ident {
                                return struc.clone()
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }

        Expr::None
    }

    fn find_ident(&self, ident: String) -> Expr {
        let mut found: Expr;
        found = self.find_variable(&ident);

        if let Expr::None = found {}
        else {
            return found
        }

        found = self.find_func(&ident);
        if let Expr::None = found {}
        else {
            return found
        }

        found = self.find_structure(&ident);
        if let Expr::None = found {}
        else {
            return found
        }

        
        found
    }

    fn create_func_call(&self, expr: &Expr, params: Vec<Token>) -> Expr {
        if !self.in_func {
            self.comp_err(&format!("cannot call funcion outside of a scope. function: {:?}", expr));
            exit(1);
        }

        let mut nested_func = Expr::None;
        let mut nested_brack_rs = 0;
        let mut nested_params = Vec::new();
        let mut square_rc = 0;
        let mut intlit_buf = String::new();
        let mut found_amper = false;

        let mut expr_params = Vec::new();
        for (i, param) in params.iter().enumerate() {
            match param {
                Token::Int(intlit) => {
                    if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                    } else if square_rc > 0 {
                        intlit_buf.push_str(intlit);
                    } else {
                        expr_params.push(self.check_intlit(intlit.to_owned()));
                    }
                },
                Token::Str(strlit) => {
                    if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                    } else if square_rc > 0 {
                        self.comp_err(&format!("expected integers inside [], found token {:?}", param));
                        exit(1);
                    } else {
                        expr_params.push(Expr::StrLit {
                            content: strlit.to_owned(),
                            is_cstr: false
                        });
                    }
                },
                Token::Ident(ident) => {
                    if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                        continue;
                    }

                    if square_rc > 0 {
                        intlit_buf.push_str(ident);
                        continue;
                    }

                    if found_amper {
                        expr_params.push(self.create_address(ident));
                        found_amper = false;
                        continue;
                    }

                    // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                    let ident_num = ident.parse::<i32>();
                    let is_num = match ident_num {
                        Ok(_) => true,
                        Err(_) => false,
                    };

                    if is_num {
                        expr_params.push(Expr::IntLit(ident.to_string()));
                        continue;
                    }

                    let expr = self.find_ident(ident.to_string());
                    if let Expr::None = expr {
                        let keyword_rs = self.keyword_map.get(ident);
                        match keyword_rs {
                            Some(keyword) => {
                                let _ = self.keyword_to_type(keyword.clone());
                                expr_params.push(Expr::VariableName { typ: Types::TypeId, name: ident.clone(), reassign: false, field_data: (false, false) })
                            },
                            None => {
                                self.comp_err(&format!("unknown identifier: {}", ident));
                                exit(1);
                            }
                        }
                    } else if let Expr::Func { .. } = expr {
                        nested_func = expr;
                    } else if let Expr::MacroFunc { .. } = expr {
                        nested_func = expr;
                    } else if let Expr::VariableName {ref typ, reassign, field_data, ..} = expr {
                        if field_data.0 && field_data.1 {
                            let new_name = ident.replace(".", "->");
                            expr_params.push(Expr::VariableName { typ: typ.clone(), name: new_name, reassign: reassign.clone(), field_data })
                        } else {
                            expr_params.push(expr.clone());
                        }
                    } else {
                        expr_params.push(expr);
                    }
                },
                Token::Lsquare => {
                    square_rc += 1;
                },
                Token::Rsquare => {
                    square_rc -= 1;
                    if square_rc == 0 {
                        let intlit = self.check_intlit(intlit_buf.clone());
                        expr_params.push(intlit);
                    }
                },
                Token::Lbrack => {
                    if i != 0 {
                        nested_brack_rs += 1;
                    }
                },
                Token::Rbrack => {
                    if i > 0 && nested_brack_rs > 0 {
                        nested_brack_rs -= 1;
                        if nested_brack_rs == 0 {
                            expr_params.push(self.create_func_call(&nested_func, nested_params.clone()));
                            nested_params.clear();
                        }
                    } else {
                        nested_brack_rs -= 1;
                    }
                },
                Token::Minus => {
                    if square_rc > 0 {
                        intlit_buf.push('-');
                    } else {
                        self.comp_err(&format!("can't have `-` operator outside of []"));
                        exit(1);
                    }
                },
                Token::Plus => {
                    if square_rc > 0 {
                        intlit_buf.push('+');
                    } else {
                        self.comp_err(&format!("can't have `+` operator outside of []"));
                        exit(1);
                    }
                },
                Token::Multiple => {
                    if square_rc > 0 {
                        intlit_buf.push('*');
                    } else {
                        self.comp_err(&format!("can't have `*` operator outside of []"));
                        exit(1);
                    }
                },
                Token::Divide => {
                    if square_rc > 0 {
                        intlit_buf.push('/');
                    } else {
                        self.comp_err(&format!("can't have `/` operator outside of []"));
                        exit(1);
                    }
                },
                Token::Ampersand => {
                    found_amper = true;
                },
                Token::True => {
                    if square_rc > 0 {
                        self.comp_err(&format!("can't have `true` inside integer literal"));
                        exit(1);
                    } else if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                    } else {
                        expr_params.push(Expr::True);
                    }
                },
                Token::False => {
                    if square_rc > 0 {
                        self.comp_err(&format!("can't have `false` inside integer literal"));
                        exit(1);
                    } else if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                    } else {
                        expr_params.push(Expr::False);
                    }
                },
                Token::Quote => (),
                _ => {
                    self.comp_err(&format!("unexpected token: {:?}", param));
                    exit(1);
                } 
            }
        }

        match expr {
            Expr::Func { name, .. } | Expr::MacroFunc { name, .. } => {
                return Expr::FuncCall { name:name.to_owned(), gave_params: expr_params }
            },
            _ => {
                self.comp_err(&format!("expected a function, got {:?}", expr));
                exit(1);
            },
        }
    }

    fn handle_import_macro(&mut self, path: &String) -> Expr {
        if path.chars().nth(path.len()-1).unwrap() == 'h' {
            self.imports.push(path.to_string());
            let no_extension = path.split_at(path.len()-2);
            return Expr::Import(no_extension.0.to_string())
        }

        if !self.imports.is_empty() {
            for imp in &self.imports {
                if imp == path {
                    return Expr::None
                }
            }

            self.imports.push(path.to_string());
        } else {
            self.imports.push(path.to_string());
        }

        let file_res = fs::read_to_string(path);
        let content = match file_res {
            Ok(content) => content,
            Err(_) => {
                self.comp_err("unable to read file");
                exit(1);
            },
        };

        let tokens = tokeniser(content);
        let mut parse = ExprWeights::new(tokens, path);
        parse.functions = self.functions.clone();
        parse.imports = self.imports.clone();
        let mut expressions = parse.parser();

        if self.imports.len() != parse.imports.len() {
            let mut new = parse.imports[self.imports.len()..].to_vec();
            self.imports.append(&mut new);
        }

        if self.functions.len() != parse.functions.len() {
            let mut new = parse.functions[self.functions.len()..].to_vec();
            self.functions.append(&mut new);
        }

        // might need to do the same thing to structures as done with imports and functions
        self.structures.append(&mut parse.structures);
        self.program.append(&mut expressions);
        return Expr::None
    }

    fn handle_array_macro(&mut self, length: &String, tokens: Vec<Token>) -> Expr {
        if tokens.len() < 1 {
            self.comp_err(&format!("expected keyword and identifer (int x), got {tokens:?}"));
            exit(1);
        }

        let keyword: Keyword;
        match &tokens[0] {
            Token::Ident(ident) => {
                let keyword_res = self.keyword_map.get(ident);
                match keyword_res {
                    Some(kw) => keyword = kw.clone(),
                    None => {
                        if let Expr::StructDef { .. } = self.find_structure(ident) {
                            keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: vec![] };
                        } else {
                            self.comp_err(&format!("expected keyword, got {ident}"));
                            exit(1);
                        }
                    }
                }
            },
            unexpected => {
                self.comp_err(&format!("expected identifier, got {unexpected:?}"));
                exit(1);
            }
        }

        let name: String;
        match &tokens[1] {
            Token::Ident(ident) => {
                match self.find_ident(ident.to_owned()) {
                    Expr::None => name = ident.to_owned(),
                    declared => {
                        self.comp_err(&format!("identifier {ident} already declared as {declared:?}"));
                        exit(1);
                    },
                }
            },
            unexpected => {
                self.comp_err(&format!("expected identifier, got {unexpected:?}"));
                exit(1);
            }
        }

        let typ = self.keyword_to_type(keyword);
        Expr::VariableName {
            typ: Types::Arr {
                typ: Box::new(typ),
                length: length.to_owned(),
            },
            name,
            reassign: false,
            field_data: (false, false),
        }
    }

    fn handle_macros(&mut self, ident: &String, index: &usize, value: &Vec<Token>) -> Expr {
        let macro_res = self.macros_map.get(ident);
        let mac = match macro_res {
            Some(m) => m.clone(),
            None => Macros::None,
        };

        if let Macros::None = mac {
            self.comp_err(&format!("macro {ident} does not exist"));
            exit(1);
        }
        
        match mac {
            Macros::C => {
                if let Token::Int(embed) = &value[index+2] {
                    let mut offset_line_num = 0;
                    for ch in embed.chars() {
                        if ch == '\n' {
                            offset_line_num += 1;
                        }
                    }

                    self.line_num += offset_line_num;
                    let cleaned = embed.trim().replace("\r", "")/* .replace("    ", "") */;
                    return Expr::CEmbed(cleaned)
                } else {
                    self.comp_err(&format!("expected [_code_], got {:?}", value[index+2]));
                    exit(1);
                }
            },
            Macros::Import => {
                if let Token::Str(path) = &value[index+2] {
                    return self.handle_import_macro(path);
                } else {
                    self.comp_err(&format!("expected \"path_to_file\", got {:?}", value[index+2]));
                    exit(1);
                }
            },
            Macros::Arr => {
                if let Token::Int(intlit) = &value[index+2] {
                    return self.handle_array_macro(intlit, value[index+4..].to_vec());
                } else {
                    self.comp_err(&format!("expected [__num__] to specify length of array"));
                    exit(1);
                }
            },
            _ => {
                self.comp_err(&format!("macro {mac:?} not reimplemented yet"));
                exit(1);
            }
        }
    }

    fn handle_left_assign(&mut self, var_info: Vec<Token>) -> Expr {
        let mut keyword = Keyword::None;
        match &var_info[0] {
            Token::Ident(ident) => {
                let keyword_res = self.keyword_map.get(ident);
                match keyword_res {
                    Some(kw) => keyword = kw.clone(),
                    None => {
                        match self.find_structure(ident) {
                            Expr::StructDef { .. } => {
                                keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: vec![] }
                            },
                            Expr::MacroStructDef { .. } => {
                                let mut pass_typs = Vec::new();
                                if let Token::Int(typs) = &var_info[2] {
                                    let mut name_buf = String::new();
                                    for (i, ch) in typs.chars().enumerate() {
                                        if ch == ' ' || ch == '\n' || i == typs.len()-1 {
                                            if i == typs.len() - 1 {
                                                name_buf.push(ch);
                                            }
                                            let keyword_rs = self.keyword_map.get(&name_buf);
                                            match keyword_rs {
                                                Some(keyword) => {
                                                    let _ = self.keyword_to_type(keyword.clone());
                                                    pass_typs.push(name_buf.clone());
                                                },
                                                None => {
                                                    let found_var = self.find_variable(&name_buf);
                                                    if let Expr::VariableName { typ, .. } = found_var {
                                                        if let Types::TypeId = typ {
                                                            pass_typs.push(name_buf.clone());
                                                            continue;
                                                        }
                                                    }
                                                    self.comp_err(&format!("expected type, got {name_buf}"));
                                                    exit(1);
                                                },
                                            }

                                            name_buf.clear();
                                        } else {
                                            name_buf.push(ch);
                                        }
                                    }
                                } else {
                                    self.comp_err(&format!("expected type for generic struct"));
                                    exit(1);
                                }
                                keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: pass_typs }
                            },
                            _ => (),
                        };

                        if let Keyword::None = keyword {
                            match self.find_variable(ident) {
                                Expr::None => {
                                    self.comp_err(&format!("undeclared identifier: {ident}"));
                                    exit(1);
                                },
                                Expr::VariableName { typ, name, field_data, .. } => {
                                    if var_info.len() > 1 {
                                        if let Token::Caret = &var_info[1] {
                                            let mut deref = Expr::DerefPointer(Box::new(
                                                Expr::VariableName { 
                                                    typ,
                                                    name,
                                                    reassign: true,
                                                    field_data 
                                                }
                                            ));

                                            for token in &var_info[2..] {
                                                if let Token::Caret = token {
                                                    deref = Expr::DerefPointer(Box::new(deref));
                                                } else {
                                                    self.comp_err(&format!("unexpected token after ^"));
                                                    exit(1);
                                                }
                                            }

                                            return deref
                                        }
                                    }

                                    if var_info.len() > 3 {
                                        if let Token::Int(intlit) = &var_info[2] {
                                            let at_expr = self.check_intlit(intlit.to_owned());
                                            let at = match at_expr {
                                                Expr::IntLit(lit) => lit,
                                                _ => String::new(),
                                            };
                                            return Expr::VariableName { 
                                                typ: Types::ArrIndex {
                                                    arr_typ: Box::new(typ.clone()),
                                                    index_at: at,
                                                },
                                                name: name.to_owned(),
                                                reassign: true, 
                                                field_data
                                            }
                                        } else {
                                            self.comp_err(&format!("expected integer to index array, got {:?}", &var_info[2]));
                                            exit(1);
                                        }
                                    }

                                    return Expr::VariableName { typ: typ.clone(), name: name.to_owned(), reassign: true, field_data }
                                }
                                expr => {
                                    return expr
                                },
                            }
                        }
                    },
                }
            },
            Token::Macro => {
                if var_info.len() < 4 {
                    self.comp_err(&format!("expected more tokens after macro"));
                    exit(1);
                }

                if let Token::Ident(ident) = &var_info[1] {
                    let macro_res = self.macros_map.get(ident);
                    let mac = match macro_res {
                        Some(m) => m.clone(),
                        None => {
                            self.comp_err(&format!("expected macro, got {ident}"));
                            exit(1);
                        }
                    };

                    match mac {
                        Macros::Arr => {
                            if let Token::Int(intlit) = &var_info[3] {
                                return self.handle_array_macro(intlit, var_info[5..].to_vec());
                            } else {
                                // TODO_TYPECHECK: RMBER TO CHECK THE ACTUAL LENGTH IN TYPE CHECKER
                                return self.handle_array_macro(&String::from("-1"), var_info[2..].to_vec());
                            }
                        },
                        unexpected => {
                            self.comp_err(&format!("macro {unexpected:?} not reimplemented yet"));
                            exit(1);
                        }
                    }
                }
            },
            Token::Underscore => {
                self.comp_err(&format!("variables with type void are not supported. you can use a void pointer."));
                exit(1);
            },
            Token::Caret => {
                if var_info.len() < 3 {
                    self.comp_err(&format!("expected more tokens after pointer"));
                    exit(1);
                }

                if let Token::Ident(ident) = &var_info[1] {
                    let keyword_res = self.keyword_map.get(ident);
                    match keyword_res {
                        Some(kw) => {
                            let typ = self.keyword_to_type(kw.clone());
                            keyword = Keyword::Pointer(typ.clone(), typ);
                        },
                        None => {
                            match self.find_structure(ident) {
                                Expr::StructDef { struct_name, .. } => {
                                    match *struct_name {
                                        Expr::StructName(name) => {
                                            let tmp = Types::TypeDef { type_name: name.clone(), generics: vec![] };
                                            keyword = Keyword::Pointer(tmp.clone(), tmp);
                                        },
                                        _ => {
                                            self.comp_err(&format!("expected a type after ^, found {ident} instead"));
                                            exit(1);
                                        },
                                    }
                                },
                                Expr::MacroStructDef { struct_name, .. } => {
                                    match *struct_name {
                                        Expr::MacroStructName { name, .. } => {
                                            if var_info.len() < 6 {
                                                self.comp_err(&format!("expected more tokens after pointer"));
                                                exit(1);
                                            }

                                            let mut pass_typs = Vec::new();
                                            if let Token::Int(typs) = &var_info[3] {
                                                let mut name_buf = String::new();
                                                for (i, ch) in typs.chars().enumerate() {
                                                    if ch == ' ' || ch == '\n' || i == typs.len()-1 {
                                                        if i == typs.len() - 1 {
                                                            name_buf.push(ch);
                                                        }
                                                        let keyword_rs = self.keyword_map.get(&name_buf);
                                                        match keyword_rs {
                                                            Some(keyword) => {
                                                                let _ = self.keyword_to_type(keyword.clone());
                                                                pass_typs.push(name_buf.clone());
                                                            },
                                                            None => {
                                                                let found_var = self.find_variable(&name_buf);
                                                                if let Expr::VariableName { typ, .. } = found_var {
                                                                    if let Types::TypeId = typ {
                                                                        pass_typs.push(name_buf.clone());
                                                                        continue;
                                                                    }
                                                                }
                                                                self.comp_err(&format!("expected type, got {name_buf}"));
                                                                exit(1);
                                                            },
                                                        }

                                                        name_buf.clear();
                                                    } else {
                                                        name_buf.push(ch);
                                                    }
                                                }
                                            } else {
                                                self.comp_err(&format!("expected type for generic {name}, got {:?} . e.g. `{name}[type]`", var_info[3]));
                                                exit(1);
                                            }

                                            let tmp = Types::TypeDef { type_name: name.clone(), generics: pass_typs };
                                            keyword = Keyword::Pointer(tmp.clone(), tmp);
                                        },
                                        _ => {
                                            self.comp_err(&format!("expected a type after ^, found {ident} instead"));
                                            exit(1);
                                        }
                                    }
                                },
                                _ => {
                                    self.comp_err(&format!("expected a type after ^, found {ident} instead"));
                                    exit(1);
                                }
                            }
                        }
                    }
                } else if let Token::Caret = &var_info[1] {
                    let mut pointer_counter = 2;
                    for index in 2..var_info.len()-1 {
                        match &var_info[index] {
                            Token::Caret => {
                                pointer_counter += 1;
                            },
                            Token::Ident(ident) => {
                                let keyword_res = self.keyword_map.get(ident);
                                match keyword_res {
                                    Some(kw) => {
                                        (keyword, pointer_counter) = self.create_keyword_pointer(self.keyword_to_type(kw.clone()), pointer_counter);
                                    },
                                    None => {
                                        match self.find_structure(ident) {
                                            Expr::StructDef { struct_name, .. } => {
                                                match *struct_name {
                                                    Expr::StructName(name) => {
                                                        (keyword, pointer_counter) = self.create_keyword_pointer(Types::TypeDef {
                                                            type_name: name.clone(),
                                                            generics: vec![],
                                                        }, pointer_counter);
                                                    },
                                                    _ => {
                                                        self.comp_err(&format!("expected a type after ^, found {ident} instead"));
                                                        exit(1);
                                                    },
                                                }
                                            },
                                            _ => {
                                                self.comp_err(&format!("expected a type after ^, found {ident} instead"));
                                                exit(1);
                                            }
                                        }
                                    }
                                }
                            },
                            Token::Underscore => {
                                (keyword, pointer_counter) = self.create_keyword_pointer(Types::Void, pointer_counter);
                            },
                            unexpected => {
                                self.comp_err(&format!("expected identifier after ^, got {unexpected:?}"));
                                exit(1);
                            }
                        }
                    }
                } else if let Token::Underscore = &var_info[1] {
                    keyword = Keyword::Pointer(Types::Void, Types::Void);
                } else {
                    self.comp_err(&format!("expected identifier after ^, got {:?}", &var_info[2]));
                    exit(1);
                }
            },
            Token::Dollar => {
                if var_info.len() < 3 {
                    self.comp_err(&format!("expected more tokens after `$`"));
                    exit(1);
                }

                if let Token::Ident(typeid_ident) = &var_info[1] {
                    let found_var = self.find_variable(typeid_ident);
                    if let Expr::VariableName { typ, name, .. } = found_var {
                        if let Types::TypeId = typ {
                            keyword = Keyword::Generic(name);
                        } else {
                            self.comp_err(&format!("expected {typeid_ident:?} to be typeid. instead is {typ:?}"));
                            exit(1);
                        }
                    } else {
                        self.comp_err(&format!("undefined typeid: {typeid_ident}"));
                        exit(1);
                    }
                }

                let typ = self.keyword_to_type(keyword.clone());
                if let Token::Ident(varname) = &var_info[2] {
                    return Expr::VariableName { typ, name: varname.to_owned(), reassign: false, field_data: (false, false) }
                }
            },
            _ => {
                self.comp_err(&format!("unexpected token: {:?}", var_info[0]));
                exit(1);
            }
        }

        let name = match &var_info[var_info.len()-1] {
            Token::Ident(ident) => ident,
            unexpected => {
                self.comp_err(&format!("unexpected token in variable name: {:?}", unexpected));
                exit(1);
            }
        };

        let keyword_res = self.keyword_map.get(name);
        match keyword_res {
            Some(k) => {
                self.comp_err(&format!("expected identifier, found keyword {k:?}"));
                exit(1);
            },
            None => (),
        };

        if let Expr::StructDef { .. } = self.find_structure(name) {
            self.comp_err(&format!("expected identifier, found struct name: {name}"));
            exit(1);
        }

        if name == &String::from(".") {
            self.comp_err(&format!("can't name a variable `.`"));
            exit(1);
        }

        let found_expr = self.find_ident(name.clone());
        if let Expr::None = found_expr {
            match keyword {
                Keyword::TypeDef {type_name: ref user_def, ..} => {
                    self.propagate_struct_fields(name.to_string(), user_def.to_string(), false);
                },
                Keyword::Pointer(.., ref last_typ) => {
                    if let Types::TypeDef {type_name: user_def, ..} = last_typ {
                        self.propagate_struct_fields(name.to_string(), user_def.to_string(), true);
                    }
                },
                _ => (),
            }

            let typ = self.keyword_to_type(keyword.clone());
            return Expr::VariableName { typ, name: name.to_owned(), reassign: false, field_data: (false, false)};
        } else if let Expr::VariableName { typ, name, field_data, .. } = found_expr {
            match keyword {
                Keyword::None => {
                    return Expr::VariableName { typ, name, reassign: true, field_data};
                },
                _ => {
                    self.comp_err(&format!("variable {:?} already declared", name));
                    exit(1);
                }
            }
        } else if let Expr::StructDef {..}  = found_expr {
            self.comp_err(&format!("{name} already declared as a struct"));
            exit(1);
        } else {
            self.comp_err(&format!("unexpected expression: {:?}", found_expr));
            exit(1);
        }
    }

    fn check_arraylit(&self, params: &Vec<Token>) -> Vec<Expr> {
        let mut expr_params = Vec::new();
        let mut brack_rc = 0;
        let mut has_func = Expr::None;
        
        let mut in_func_params = false;
        let mut func_params = Vec::new();

        for param in params {
            match param {
                Token::Ident(ident) => {
                    if in_func_params {
                        func_params.push(param.clone());
                        continue;
                    }

                    // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                    let ident_num = ident.parse::<i32>();
                    match ident_num {
                        Ok(_) => {
                            expr_params.push(Expr::IntLit(ident.to_owned()));
                            continue;
                        },
                        Err(_) => (),
                    }

                    let keyword_res = self.keyword_map.get(ident);
                    match keyword_res {
                        Some(kw) => {
                            self.comp_err(&format!("expected literal or identifier, found keyword {kw:?}"));
                            exit(1);
                        },
                        None => (),
                    }

                    match self.find_ident(ident.to_owned()) {
                        Expr::None => {
                            self.comp_err(&format!("expected declared identifier, got none: {ident}"));
                            exit(1);
                        },
                        Expr::StructDef { .. } => {
                            self.comp_err(&format!("expected declared identifier, got struct definiton: {ident}"));
                            exit(1);
                        },
                        Expr::Func { typ, params, name } => {
                            has_func = Expr::Func { typ, params, name }
                        },
                        found => {
                            expr_params.push(found);
                        }
                    }
                },
                Token::Int(intlit) => {
                    if in_func_params {
                        func_params.push(param.clone());
                        continue;
                    }
                    expr_params.push(self.check_intlit(intlit.to_owned()));
                },
                Token::Lbrack => {
                    brack_rc += 1;
                    if let Expr::Func { .. } = has_func {
                        in_func_params = true;
                    }
                },
                Token::Rbrack => {
                    brack_rc -= 1;
                    if brack_rc == 0 {
                        expr_params.push(self.create_func_call(&has_func, func_params.clone()));
                        in_func_params = false;
                    }
                }
                unexpected => {
                    self.comp_err(&format!("unexpected token: {unexpected:?}"));
                    exit(1);
                }
            }
        }

        expr_params
    }

    fn create_address(&self, ident: &String) -> Expr {
        let found_ident = self.find_ident(ident.clone());
        match found_ident {
            Expr::VariableName { .. } => {
                return Expr::Address(Box::new(found_ident))
            },
            unexpected => {
                self.comp_err(&format!("unexpected expression {unexpected:?} when getting address of identifier: {ident:?}"));
                exit(1);
            }
        }
    }

    fn handle_right_assign(&mut self, value: Vec<Token>, is_right: bool) -> Expr {
        let mut buffer = Vec::new();
        let mut params = Vec::new();
        let mut brack_rc = 0;
        let mut pipe_rc = 0;
        let mut pointer_counter = 0;
        let mut found_macro = false;
        let mut found_amper = false;

        let mut returning = false;
        let mut create_generic = false;

        for (i, token) in value.iter().enumerate() {
            // handle right bracket occurences
            match token {
                Token::Lbrack => {
                    brack_rc += 1;
                    params.push(token.clone());
                },
                Token::Rbrack => {
                    brack_rc -= 1;
                    params.push(token.clone());

                    if brack_rc > 0 {
                        continue;
                    } else if pipe_rc == 1 {
                        continue;
                    } else {
                        if buffer.len() > 1 {
                            self.comp_err(&format!("unexpected multiple expressions outside of parameters: {:?}", buffer));
                            exit(1);
                        } else if self.in_struct_def {
                            self.comp_err(&format!("can't call function {:?} inside struct", buffer[0]));
                            exit(1);
                        } else if returning {
                            buffer[0] = self.create_func_call(&buffer[0], params.clone());
                        } else {
                            return self.create_func_call(&buffer[0], params);
                        }
                    }
                },
                Token::Pipe => {
                    pipe_rc += 1;
                },
                Token::Caret => {
                    pointer_counter += 1;
                },
                _ => {
                    if brack_rc > 0 {
                        params.push(token.clone());
                        continue;
                    } else if pipe_rc == 1 {
                        params.push(token.clone());
                        continue;
                    }
                },
            }

            match token {
                Token::Int(intlit) => {
                    // TODO: ARRAYS INDEXING WITH [i+1] WILL NOT WORK WITH THIS
                    buffer.push(self.check_intlit(intlit.to_string()));
                },
                Token::Str(strlit) => {
                    buffer.push(Expr::StrLit {
                        content: strlit.to_string(),
                        is_cstr: false,
                    });
                },
                Token::Ident(ident) => {
                    if found_macro {
                        return self.handle_macros(ident, &i, &value);
                    } else if found_amper {
                        buffer.push(self.create_address(ident));
                        found_amper = false;
                        continue;
                    }

                    // check if this is an expr without a :
                    if !is_right {
                        let keyword_res = self.keyword_map.get(ident);
                        match keyword_res {
                            Some(k) => {
                                let mut keyword = k.clone();

                                if let Keyword::Break = k {
                                    return Expr::Break
                                } else if let Keyword::Continue = k {
                                    return Expr::Continue;
                                } else if let Keyword::Return = k {
                                    returning = true;
                                    continue;
                                } else if pointer_counter > 0 {
                                    (keyword, _) = self.create_keyword_pointer(self.keyword_to_type(k.clone()), pointer_counter);
                                }

                                // anything that needs an identifier after the keyword is handled
                                // below
                                if i + 1 == value.len() {
                                    self.comp_err(&format!("expected identifier after keyword {k:?}, got nothing"));
                                    exit(1);
                                }

                                // similar syntax between defining var and returning, check which
                                if self.in_struct_def {
                                    let expr = self.create_define_var(keyword, value[i+1].clone(), vec![]);
                                    self.expr_stack.push(expr);
                                    return Expr::None
                                } else {
                                    return self.create_define_var(keyword, value[i+1].clone(), vec![]);
                                }
                            },
                            None => {
                                let found_ident = self.find_ident(ident.to_owned());
                                if returning {}
                                else if create_generic {
                                    if let Expr::VariableName { typ, name, .. } = found_ident {
                                        if let Types::TypeId = typ {
                                            let keyword = Keyword::Generic(name);

                                            if i + 1 == value.len() {
                                                self.comp_err(&format!("expected identifier after keyword {keyword:?}, got nothing"));
                                                exit(1);
                                            }

                                            if self.in_struct_def {
                                                let expr = if i + 2 < value.len() {
                                                    let slice = value[i+1..value.len()-1].to_vec();
                                                    let last = value.last().unwrap().clone();
                                                    self.create_define_var(keyword, last, slice)
                                                } else {
                                                    self.create_define_var(keyword, value[i+1].clone(), vec![])
                                                };
                                                self.expr_stack.push(expr);
                                                return Expr::None
                                            } else {
                                                return self.create_define_var(keyword, value[i+1].clone(), vec![]);
                                            }
                                        } else {
                                            self.comp_err(&format!("expected typeid after `$`, got {typ:?}"));
                                            exit(1);
                                        }
                                    }
                                } else if let Expr::StructDef { .. } = found_ident {
                                    let mut k = Keyword::TypeDef {
                                        type_name: ident.clone(), 
                                        generics: vec![]
                                    };

                                    if pointer_counter > 0 {
                                        (k, _) = self.create_keyword_pointer(self.keyword_to_type(k), pointer_counter);
                                    }

                                    if self.in_struct_def {
                                        let expr = self.create_define_var(k, value[i+1].clone(), vec![]);
                                        self.expr_stack.push(expr);
                                        return Expr::None;
                                    }

                                    return self.create_define_var(k, value[i+1].clone(), vec![]);
                                } else if let Expr::MacroStructDef { .. } = found_ident {
                                    let mut k = Keyword::TypeDef {
                                        type_name: ident.clone(), 
                                        generics: vec![]
                                    };

                                    if pointer_counter > 0 {
                                        (k, _) = self.create_keyword_pointer(self.keyword_to_type(k), pointer_counter);
                                    }

                                    let slice = value[i+1..value.len()-1].to_vec();
                                    let last = value.last().unwrap().clone();
                                    // TODO: figure out how this messes things up
                                    if self.in_struct_def {
                                        let expr = self.create_define_var(k, value[i+1].clone(), vec![]);
                                        self.expr_stack.push(expr);
                                        return Expr::None;
                                    }

                                    return self.create_define_var(k, last, slice)
                                } else if let Expr::Func { .. } = found_ident {
                                    return self.create_func_call(&found_ident, value[i+1..].to_vec());
                                } else if let Expr::MacroFunc { .. } = found_ident {
                                    return self.create_func_call(&found_ident, value[i+1..].to_vec());
                                } else {
                                    if self.in_struct_def && !self.current_func.is_empty() && ident == &self.current_func {
                                        let mut k = Keyword::TypeDef {
                                            type_name: format!("struct {}", self.current_func), 
                                            generics: vec![]
                                        };
                                        if pointer_counter > 0 {
                                            k = self.create_keyword_pointer(self.keyword_to_type(k), pointer_counter).0;
                                        }

                                        let expr = self.create_define_var(k, value[i+1].clone(), vec![]);
                                        self.expr_stack.push(expr);
                                        return Expr::None
                                    }
                                    self.comp_err(&format!("unknown identifier: {ident}"));
                                    exit(1);
                                }
                            },
                        }
                    }

                    // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                    let ident_num = ident.parse::<i32>();
                    let is_num = match ident_num {
                        Ok(_) => true,
                        Err(_) => false,
                    };

                    if is_num {
                        buffer.push(Expr::IntLit(ident.to_string()));
                        continue;
                    }

                    let expr = self.find_ident(ident.to_string());
                    match expr {
                        Expr::VariableName { ref typ, ref name, reassign, field_data } => {
                            if field_data.0 && field_data.1 {
                                let new_name = name.replace(".", "->");
                                buffer.push(Expr::VariableName { typ: typ.clone(), name: new_name, reassign, field_data })
                            } else {
                                buffer.push(expr)
                            }
                        }
                        Expr::None => {
                            self.comp_err(&format!("unknown identifier: {}", ident));
                            exit(1);
                        },
                        _ => buffer.push(expr),
                    }
                },
                Token::Lbrack => {
                    // handled above
                },
                Token::Rbrack => {
                    // handled above
                },
                Token::Pipe => {
                    // handled above
                },
                Token::Macro => {
                    found_macro = true;
                },
                Token::Dollar => {
                    create_generic = true;
                },
                Token::Quote => (),
                Token::Lsquare => (),
                Token::Rsquare => (),
                Token::Caret => {
                    if is_right {
                        let top_expr_res = buffer.pop();
                        let top_expr = match top_expr_res {
                            Some(ex) => ex,
                            None => {
                                self.comp_err(&format!("unexpected ^ operator as there's no expression in front it."));
                                exit(1);
                            }
                        };

                        buffer.push(Expr::DerefPointer(Box::new(top_expr)));
                    }
                },
                Token::Ampersand => {
                    found_amper = true;
                },
                Token::Underscore => {
                    if !is_right {
                        if pointer_counter > 0 {
                            let keyword = self.create_keyword_pointer(Types::Void, pointer_counter).0;
                            if i + 1 == value.len() {
                                self.comp_err(&format!("expected identifier after keyword {keyword:?}, got nothing"));
                                exit(1);
                            }

                            if self.in_struct_def {
                                let expr = self.create_define_var(keyword, value[i+1].clone(), vec![]);
                                self.expr_stack.push(expr);
                                return Expr::None
                            } else {
                                return self.create_define_var(keyword, value[i+1].clone(), vec![]);
                            }
                        } else {
                            self.comp_err(&format!("can't make void type. void pointers are allowed"));
                            exit(1);
                        }
                    } else {
                        self.comp_err(&format!("can't make void type. void pointers are allowed"));
                        exit(1);
                    }
                },
                Token::True => {
                    buffer.push(Expr::True);
                },
                Token::False => {
                    buffer.push(Expr::False);
                },
                _ => {
                    self.comp_err(&format!("unexpected token: {:?}", token));
                    exit(1);
                }
            }
        }

        if buffer.is_empty() && params.is_empty() {
            if returning {
                return Expr::Return(Box::new(Expr::None));
            }

            self.comp_err(&format!("expected a token, got none."));
            exit(1);
        }

        if returning && buffer.len() <= 1 {
            if self.in_struct_def {
                self.comp_err(&format!("can't use return inside struct"));
                exit(1);
            }

            return Expr::Return(Box::new(buffer[0].clone()))
        }
        
        if buffer.len() > 1 {
            match &buffer[0] {
                Expr::Address(varname) => {
                    if let Expr::VariableName { typ, name, .. } = *varname.clone() {
                        if let Types::Arr { .. } = typ {
                        } else if let Types::Pointer { .. } = typ {
                        } else {
                            self.comp_err(&format!("couldn't handle expressions: {:?}", buffer));
                            exit(1);
                        }

                        if let Expr::IntLit(intlit) = &buffer[1] {
                            let expr = Expr::Address(Box::new(Expr::VariableName {
                                typ: Types::ArrIndex {
                                    arr_typ: Box::new(typ.clone()),
                                    index_at: intlit.to_owned()
                                },
                                name: name.to_owned(),
                                reassign: false,
                                field_data: (false, false)
                            }));
                            if returning {
                                return Expr::Return(Box::new(expr))
                            } else {
                                return expr;
                            }
                        } else {
                            self.comp_err(&format!("couldn't handle expressions: {:?}", buffer));
                            exit(1);
                        }
                    }
                }
                Expr::VariableName { typ, name, .. } => {
                    if let Types::Arr { .. } = typ {
                    } else if let Types::Pointer { .. } = typ {
                    } else {
                        self.comp_err(&format!("couldn't handle expressions: {:?}", buffer));
                        exit(1);
                    }

                    if let Expr::IntLit(intlit) = &buffer[1] {
                        let expr = Expr::VariableName {
                            typ: Types::ArrIndex {
                                arr_typ: Box::new(typ.clone()),
                                index_at: intlit.to_owned()
                            },
                            name: name.to_owned(),
                            reassign: false,
                            field_data: (false, false),
                        };
                        if returning {
                            return Expr::Return(Box::new(expr))
                        } else {
                            return expr;
                        }
                    } else {
                        self.comp_err(&format!("couldn't handle expressions: {:?}", buffer));
                        exit(1);
                    }
                },
                _ => {
                    self.comp_err(&format!("couldn't handle expressions: {:?}", buffer));
                    exit(1);
                },
            }
        }

        if !params.is_empty() {
            let arrlit = self.check_arraylit(&params);
            return Expr::ArrayLit(arrlit)
        }

        buffer[0].clone()
    }

    fn propagate_struct_fields(&mut self, fname: String, user_def: String, is_ptr: bool) {
        match self.find_structure(&user_def) {
            Expr::StructDef { struct_fields, .. } | Expr::MacroStructDef { struct_fields, .. } => {
                for field in struct_fields {
                    match field {
                        Expr::VariableName { typ, name, .. } => {
                            let new_name = format!("{fname}.{name}");
                            let new_expr = Expr::Variable {
                                info: Box::new(Expr::VariableName {
                                    typ,
                                    name: new_name,
                                    reassign: false,
                                    field_data: (true, is_ptr),
                                }),
                                value: Box::new(Expr::None),
                            };
                            if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
                                vars[self.current_scope].push(new_expr);
                            }
                        },
                        _ => (),
                    }
                }
            },
            _ => {
                self.comp_err("unexpected expression during field propagation");
                exit(1);
            },
        }
    }

    fn create_define_var(&mut self, kw: Keyword, ident: Token, generics: Vec<Token>) -> Expr {
        println!("kw: {kw:?}, ident: {ident:?}, generics: {generics:?}");
        let expr: Expr;
        let fname: String;
        let mut pass_typs: Vec<String> = Vec::new();

        for gen in generics {
            match gen {
                Token::Lsquare => (),
                Token::Rsquare => (),
                Token::Int(typs) => {
                    let mut name_buf = String::new();
                    for (i, ch) in typs.chars().enumerate() {
                        if ch == ' ' || ch == '\n' || i == typs.len()-1 {
                            if i == typs.len() - 1 {
                                name_buf.push(ch);
                            }
                            let keyword_rs = self.keyword_map.get(&name_buf);
                            match keyword_rs {
                                Some(keyword) => {
                                    let _ = self.keyword_to_type(keyword.clone());
                                    pass_typs.push(name_buf.clone());
                                },
                                None => {
                                    let found_var = self.find_variable(&name_buf);
                                    if let Expr::VariableName { typ, .. } = found_var {
                                        if let Types::TypeId = typ {
                                            pass_typs.push(name_buf.clone());
                                            continue;
                                        }
                                    }
                                    self.comp_err(&format!("expected type, got {name_buf}"));
                                    exit(1);
                                },
                            }

                            name_buf.clear();
                        } else {
                            name_buf.push(ch);
                        }
                    }
                },
                unexpected => {
                    self.comp_err(&format!("unexpected token {unexpected:?}. expecting `typedef[__type__] varname;`"));
                    exit(1);
                },
            }
        }

        match ident {
            Token::Ident(word) => {
                // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                let ident_num = word.parse::<i32>();
                let is_num = match ident_num {
                    Ok(_) => true,
                    Err(_) => false,
                };

                if is_num {
                    self.comp_err(&format!("expected identifier, found number {word}"));
                    exit(1);
                }

                let keyword_res = self.keyword_map.get(&word);
                match keyword_res {
                    Some(k) => {
                        self.comp_err(&format!("expected identifier, found keyword {k:?}"));
                        exit(1);
                    },
                    None => (),
                };

                let found_expr = self.find_ident(word.clone());
                match found_expr {
                    Expr::None => {
                        let mut typ = self.keyword_to_type(kw.clone());
                        fname = word.clone();

                        if let Types::TypeDef { ref mut generics, .. } = typ {
                            *generics = pass_typs;
                        }
                        expr = Expr::VariableName { typ, name: word, reassign: false, field_data: (false, false) };
                    },
                    _ => {
                        self.comp_err(&format!("identifier {:?} already declared", word));
                        exit(1);
                    },
                }
            },
            _ => {
                self.comp_err(&format!("unexpected token {ident:?} after a keyword {kw:?}"));
                exit(1);
            },
        }

        match kw {
            Keyword::I32 | Keyword::I8 | Keyword::U8 |
            Keyword::Char | Keyword::Usize | Keyword::Bool => (),
            Keyword::Generic(_) => (),
            Keyword::Pointer(.., last) => {
                if let Types::TypeDef { type_name: user_def, .. } = last {
                    self.propagate_struct_fields(fname, user_def.to_string(), true);
                }
            },
            Keyword::TypeDef { type_name: ref user_def, .. } => {
                self.propagate_struct_fields(fname, user_def.to_string(), false);
            },
            _ => {
                self.comp_err(&format!("unexpected keyword: {kw:?}"));
                exit(1);
            },
        }

        expr
    }

    fn create_variable(&mut self, left: Vec<Token>, right: Vec<Token>) {
        if !self.in_func {
            self.comp_err(&format!("cannot create variable outside of a scope. variable: {:?}", left));
            exit(1);
        }

        let left_expr = self.handle_left_assign(left);
        let right_expr = self.handle_right_assign(right, true);

        // if let Expr::VariableName { typ, name, reassign, field_data } = left_expr {
        //     // CHECK IF TYPE IS A CSTR OR STRING, THEN CHANGE THE STRLIT IS_CSTR FIELD
        // }

        let expr = Expr::Variable { info: Box::new(left_expr), value: Box::new(right_expr) };
        if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
            vars[self.current_scope].push(expr.clone());
        }

        self.token_stack.clear();
        self.program_push(expr);
    }

    fn handle_semicolon(&mut self) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut seen_colon = 0;

        for (_i, token) in self.token_stack.iter().enumerate() {
            match token {
                Token::Colon => seen_colon += 1,
                _ => {
                    if seen_colon == 0 {
                        left.push(token.clone());
                    } else if seen_colon == 1 {
                        right.push(token.clone());
                    } else {
                        self.comp_err("unexpected assignment operator `:`");
                        exit(1);
                    }
                },
            }
        }

        if !right.is_empty() {
            if self.in_struct_def {
                self.comp_err("can't initalise members inside a struct");
                exit(1);
            }
            self.create_variable(left, right);
            return
        }

        // this could be a funccall or whatever but if its variable def, we need to handle it.
        let expr = self.handle_right_assign(left, false);
        self.token_stack.clear();
        match expr {
            Expr::VariableName { .. } => {
                let new_var = Expr::Variable { info: Box::new(expr.clone()), value: Box::new(Expr::None) };
                if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
                    vars[self.current_scope].push(new_var);
                }
            },
            // struct stuff
            Expr::None => {
                return
            },
            _ => (),
        }

        self.program_push(expr)
    }

    pub fn parser(&mut self) -> Vec<(Expr, String, u32)> {
        let mut curl_rc = 0;

        while self.current_token < self.tokens.len() {
            match self.tokens[self.current_token] {
                Token::Lcurl => {
                    curl_rc += 1;
                    self.handle_lcurl();
                },
                Token::Rcurl => {
                    self.error_if_stack_not_empty();

                    curl_rc -= 1;
                    self.prev_scope();
                    if self.in_struct_def {
                        self.create_struct();
                    } else {
                        self.program_push(Expr::EndBlock);
                    }
                },
                Token::SemiColon => {
                    self.handle_semicolon();
                },
                Token::Newline => {
                    self.error_if_stack_not_empty();
                    self.line_num += 1;
                },
                _ => self.token_stack.push(self.tokens[self.current_token].clone()),
            }

            self.current_token += 1;
        }

        if curl_rc != 0 {
            self.comp_err("missing a `{` or `}` somewhere");
            exit(1);
        }

        self.program.clone()
    }
}
