use std::{path::Path, fs, collections::HashMap, process::exit};
use crate::tokeniser::{tokeniser, Token}; use crate::{declare_types::*, Gen};
use fs_extra::{dir::CopyOptions, copy_items};

const CUR_PATH: &str = env!("current_path");

#[derive(Debug, Clone)]
pub enum Expr {
    Func {
        typ: Types,
        params: Vec<Expr>,
        name: String,
        is_inline: bool,
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
        constant: bool,
        field_data: (bool, bool), // is_field, is_pointer
        // dots_to_arrows: Vec<bool>,
    },
    Variable {
        info: Box<Expr>, // Expr = VariableName
        value: Box<Expr>, // Expr = VariableName | StrLit | IntLit | FuncCall
    },
    DerefPointer(Box<Expr>),
    Address(Box<Expr>),

    If(Vec<Expr>, Box<Expr>), // last expr is for the capture
    OrIf(Vec<Expr>, Box<Expr>),
    Else,

    Switch(Vec<Expr>),
    Case(Vec<Expr>),
    Fall(Vec<Expr>),

    Loop {
        condition: Vec<Expr>,
        modifier: Box<Expr>, // Expr = IntLit
    },
    For {
        for_this: Box<Expr>,
        in_this: Box<Expr>,
        iterator: String,
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
    CharLit(String),
    StrLit {
        content: String,
        is_cstr: bool, // TODO: REMOVE, ALL STRING LITERALS ARE CSTR BY DEFAULT
    },

    Import(String),
    CEmbed(String),

    Return(Box<Expr>),
    Break,
    Continue,

    StructName {
        name: String,
        is_shared: bool,
    },
    StructDef {
        struct_name: Box<Expr>, // Expr = StructName
        struct_fields: Vec<Expr>, // Expr = VarName
    },
    EndStruct(String),

    EnumName(String),
    EnumDef {
        enum_name: Box<Expr>,
        enum_fields: Vec<Expr>,
    },

    MacroStructName {
        name: String,
        generics: Vec<Expr>,
        is_shared: bool,
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

struct DeferInfo {
    scope: usize,
    exprs: Vec<Expr>,
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
    enums: Vec<Expr>,
    enums_fields: Vec<Expr>, // VarNames
    func_to_vars: HashMap<String, Vec<Vec<Expr>>>,
    global_vars: Vec<Expr>,

    current_func: String,
    current_scope: usize,
    previous_func: String,

    in_scope: bool,
    in_func: bool,
    in_struct_def: bool,
    in_enum_def: bool,

    in_defer: bool,
    one_defer: bool,
    defer_scope: usize,

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

            ("u8".to_string(), Keyword::U8),
            ("i8".to_string(), Keyword::I8),
            ("char".to_string(), Keyword::Char),

            ("u16".to_string(), Keyword::U16),
            ("i16".to_string(), Keyword::I16),

            ("u32".to_string(), Keyword::U32),
            ("i32".to_string(), Keyword::I32),

            ("uint".to_string(), Keyword::UInt),
            ("int".to_string(), Keyword::Int),

            ("u64".to_string(), Keyword::U64),
            ("i64".to_string(), Keyword::I64),

            ("usize".to_string(), Keyword::Usize),

            ("f32".to_string(), Keyword::F32),
            ("f64".to_string(), Keyword::F64),

            ("bool".to_string(), Keyword::Bool),

            ("typeid".to_string(), Keyword::TypeId),
            ("any".to_string(), Keyword::Any),

            ("if".to_string(), Keyword::If),
            ("orif".to_string(), Keyword::OrIf),
            ("else".to_string(), Keyword::Else),

            ("switch".to_string(), Keyword::Switch),
            ("case".to_string(), Keyword::Case),
            ("fall".to_string(), Keyword::Fall),

            ("or".to_string(), Keyword::Or),
            ("and".to_string(), Keyword::And),

            ("loop".to_string(), Keyword::Loop),
            ("for".to_string(), Keyword::For),

            ("return".to_string(), Keyword::Return),
            ("break".to_string(), Keyword::Break),
            ("continue".to_string(), Keyword::Continue),

            ("struct".to_string(), Keyword::Struct),
            ("enum".to_string(), Keyword::Enum),

            ("defer".to_string(), Keyword::Defer),
        ]);

        let macros_map: HashMap<String, Macros> = HashMap::from([
            ("c".to_string(), Macros::C),
            ("import".to_string(), Macros::Import),
            ("inline".to_string(), Macros::Inline),
            ("shared".to_string(), Macros::Shared),
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
            global_vars: Vec::new(),
            structures: Vec::new(),
            enums: Vec::new(),
            enums_fields: Vec::new(),

            current_func: String::new(),
            current_scope: 0,
            previous_func: String::new(),

            in_scope: false,
            in_func: false,
            in_struct_def: false,
            in_enum_def: false,
            in_defer: false,
            one_defer: false,
            defer_scope: 0,

            line_num: 1,
            
            imports: Vec::new(),
            program: Vec::new(),
            filename: filename.to_owned(),
        }
    }

    // fn comp_warn(&self, warning_msg: &str) {
    //     println!("\x1b[93mwarning\x1b[0m: {}:{}", self.filename, self.line_num);
    //     println!("\x1b[93mwarning\x1b[0m: {warning_msg}");
    // }

    fn comp_err(&self, error_msg: &str) {
        println!("\x1b[91merror\x1b[0m: {}:{}", self.filename, self.line_num);
        println!("\x1b[91merror\x1b[0m: {error_msg}");
    }

    fn program_push(&mut self, expr: Expr) {
        self.program.push((expr, self.filename.clone(), self.line_num));
    }

    fn error_if_token_stack_not_empty(&self) {
        if !self.token_stack.is_empty() {
            self.comp_err("might be a missing `;`. stack is not empty when it should be");
            exit(1);
        }
    }

    fn keyword_to_type(&self, kw: Keyword) -> Types {
        match kw {
            Keyword::U8 => Types::U8,
            Keyword::I8 => Types::I8,
            Keyword::Char => Types::Char,

            Keyword::U16 => Types::U16,
            Keyword::I16 => Types::I16,

            Keyword::U32 => Types::U32,
            Keyword::I32 => Types::I32,

            Keyword::U64 => Types::U64,
            Keyword::I64 => Types::I64,

            Keyword::UInt => Types::UInt,
            Keyword::Int => Types::Int,

            Keyword::Usize => Types::Usize,

            Keyword::F32 => Types::F32,
            Keyword::F64 => Types::F64,
            Keyword::Bool => Types::Bool,
            Keyword::TypeId => Types::TypeId,
            Keyword::Any => Types::Any,
            Keyword::Generic(typ) => Types::Generic(typ),
            Keyword::TypeDef { type_name, generics } =>  Types::TypeDef { type_name, generics },
            Keyword::Pointer(pointer_to, _) => Types::Pointer(Box::new(pointer_to)),
            Keyword::Arr { typ, length } => Types::Arr { typ, length },
            Keyword::Address => Types::Address,
            Keyword::None => {
                if self.in_enum_def {
                    Types::None
                } else {
                    self.comp_err(&format!("can't convert {:?} to a type. type might not be reimplemented yet or defined.", kw));
                    exit(1);
                }
            },
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

    fn new_scope_vars(&mut self, new_vars: Vec<Expr>) {
        self.in_scope = true;

        if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
            let mut old_scope = vars[self.current_scope].clone();
            self.current_scope += 1;
            vars.push(vec![]);
            vars[self.current_scope].append(&mut old_scope);

            for var in new_vars {
                match var {
                    Expr::None => (),
                    _ => vars[self.current_scope].push(var),
                }
            }
        }
    }

    fn prev_scope(&mut self) {
        if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
            if self.in_scope && self.current_scope == 0 {
                self.in_scope = false;
            } else if self.in_scope && self.current_scope > 0 {
                vars.pop();
                self.current_scope -= 1;

                if self.current_scope == 0 {
                    self.in_scope = false;
                }
            } else {
                self.in_func = false;
                vars.pop();
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
            ('*', Token::Multiply),
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
                        tokens.push(Token::Ident(buf.clone()));
                        buf.clear();
                    },
                }

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
                            Expr::Func { .. } | Expr::MacroFunc { .. } => {
                                let func_call = self.create_func_call(&has_func, params.clone());

                                // we need to make a new generator instance and only call the
                                // `handle_funccall` function to get back a clean string of this
                                // func call, this does not affect the compilation except for maybe
                                // error messages but not sure.
                                let mut gen = Gen::new(self.filename.clone(), String::from("output.c"), true, false, Lang::C);
                                let clean_func_call = gen.handle_funccall(func_call);
                                params.clear();
                                clean.push_str(&clean_func_call)
                            },
                            ref unexpected => {
                                self.comp_err(&format!("unexpected expression {unexpected:?} in integer literal: {:?}", has_func));
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
                Token::Multiply => clean.push('*'),
                Token::Divide => clean.push('/'),
                Token::Lsquare => clean.push('('),
                Token::Rsquare => clean.push(')'),
                Token::Mod => clean.push('%'),
                Token::Ident(ident) => {
                    // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                    let ident_num = ident.parse::<f64>();
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
                            match typ {
                                Types::I32 | Types::U32 | Types::U8 | Types::I8 | Types::UInt | Types::Int | Types::U16 | Types::I16 |
                                Types::U64 | Types::I64 |Types::Usize | Types::Pointer(_) | Types::Generic(_) |
                                Types::F32 | Types::F64 => {
                                    let mut sanitised = name.clone();
                                    if !field_data.0 && name.contains(".") {
                                        sanitised = name.replace(".", "_");
                                    }
                                    clean.push_str(&sanitised)
                                },
                                _ => {
                                    self.comp_err(&format!("variable {name} is not an integer. {typ:?}:{name}"));
                                    exit(1);
                                }
                            }
                        },
                        Expr::Func { typ, params, name, is_inline } => {
                            match typ {
                                Types::I32 | Types::U32 | Types::U8 | Types::I8 | Types::UInt | Types::Int | Types::U16 | Types::I16 |
                                Types::U64 | Types::I64 |Types::Usize | Types::Pointer(_) | Types::Generic(_) |
                                Types::F32 | Types::F64 => {
                                    has_func = Expr::Func { typ, params, name, is_inline };
                                },
                                _ => {
                                    self.comp_err(&format!("function {name} does not return integer. {typ:?}:{name}"));
                                    exit(1);
                                },
                            }
                        },
                        Expr::MacroFunc { typ, params, name } => {
                            match typ {
                                Types::I32 | Types::U32 | Types::U8 | Types::I8 | Types::UInt | Types::Int | Types::U16 | Types::I16 |
                                Types::U64 | Types::I64 |Types::Usize | Types::Pointer(_) | Types::Generic(_) |
                                Types::F32 | Types::F64 => {
                                    has_func = Expr::MacroFunc { typ, params, name };
                                },
                                _ => {
                                    self.comp_err(&format!("function {name} does not return integer. {typ:?}:{name}"));
                                    exit(1);
                                },
                            }
                        }
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

    fn parse_generics_to_pass(&mut self, typs: &String, generic_not_defined_yet: bool) -> Vec<Types> {
        let mut name_buf = String::new();
        let mut pass_typs = Vec::new();
        let mut pointer_counter = 0;

        for (i, ch) in typs.chars().enumerate() {
            if ch == ' ' || ch == '\n' || i == typs.len()-1 {
                if i == typs.len() - 1 {
                    name_buf.push(ch);
                }

                let keyword_rs = self.keyword_map.get(&name_buf);
                match keyword_rs {
                    Some(keyword) => {
                        let typ = self.keyword_to_type(keyword.clone());
                        if pointer_counter > 0 {
                            let kw; (kw, pointer_counter) = self.create_keyword_pointer(typ, pointer_counter);
                            pass_typs.push(self.keyword_to_type(kw));
                        } else {
                            pass_typs.push(typ);
                        }
                    },
                    None => {
                        if name_buf.chars().last().unwrap() == '_' {
                            if pointer_counter > 0 {
                                let kw; (kw, pointer_counter) = self.create_keyword_pointer(Types::Void, pointer_counter);
                                pass_typs.push(self.keyword_to_type(kw));
                                name_buf.clear();
                                continue;
                            } else {
                                self.comp_err("can't have a _ (void) generic parameter, did you mean ^_ (void pointer)?");
                                exit(1);
                            }
                        }

                        let found_var = self.find_variable(&name_buf);
                        if let Expr::VariableName { typ, name, .. } = found_var {
                            if let Types::TypeId = typ {
                                let typedef = Types::TypeDef { type_name: name, generics: None };
                                if pointer_counter > 0 {
                                    let kw; (kw, pointer_counter) = self.create_keyword_pointer(typedef, pointer_counter);
                                    pass_typs.push(self.keyword_to_type(kw));
                                    name_buf.clear();
                                    continue;
                                }
                                pass_typs.push(typedef);
                                name_buf.clear();
                                continue;
                            }
                        } else if generic_not_defined_yet {
                            let typedef = Types::TypeDef { type_name: name_buf.clone(), generics: None };
                            if pointer_counter > 0 {
                                let kw; (kw, pointer_counter) = self.create_keyword_pointer(typedef, pointer_counter);
                                pass_typs.push(self.keyword_to_type(kw));
                                name_buf.clear();
                                continue;
                            }
                            pass_typs.push(typedef);
                            name_buf.clear();
                            continue;
                        } else {
                            // this is done for the second function declared in a
                            // struct that uses a generic type. like option[T] at :: () {}
                            // the "T" is defined by the struct above it in the scope
                            if !self.previous_func.is_empty() {
                                let temp = self.current_func.clone();
                                self.current_func = self.previous_func.clone();
                                let found_var_again = self.find_variable(&name_buf);
                                self.current_func = temp;
                                if let Expr::VariableName { typ, name, .. } = found_var_again {
                                    if let Types::TypeId = typ {
                                        let typedef = Types::TypeDef { type_name: name, generics: None };
                                        if pointer_counter > 0 {
                                            let kw; (kw, pointer_counter) = self.create_keyword_pointer(typedef, pointer_counter);
                                            pass_typs.push(self.keyword_to_type(kw));
                                            name_buf.clear();
                                            continue;
                                        }
                                        pass_typs.push(typedef);
                                        name_buf.clear();
                                        continue;
                                    }
                                }
                            }
                        }

                        let found_typ = self.find_structure(&name_buf);
                        if let Expr::StructDef { struct_name, .. } |
                            Expr::MacroStructDef { struct_name, .. } = found_typ {
                                if let Expr::StructName { name, .. } = *struct_name {
                                    let typedef = Types::TypeDef { type_name: name, generics: None };
                                    if pointer_counter > 0 {
                                        let kw; (kw, pointer_counter) = self.create_keyword_pointer(typedef, pointer_counter);
                                        pass_typs.push(self.keyword_to_type(kw));
                                        name_buf.clear();
                                        continue;
                                    }
                                    pass_typs.push(typedef);
                                    name_buf.clear();
                                    continue;
                                } else if let Expr::MacroStructName { .. } = *struct_name {
                                    self.comp_err("generic struct with type of a generic struct not supported yet");
                                    exit(1);
                                }
                        } else {
                            self.comp_err(&format!("expected type, got {name_buf}"));
                            exit(1);
                        }
                    },
                }

                name_buf.clear();
            } else if ch == '^' {
                pointer_counter += 1;
            } else {
                name_buf.push(ch);
            }
        }

        pass_typs
    }

    fn create_keyword_pointer(&self, typ: Types, pointer_counter: i32) -> (Keyword, i32) {
        let mut tmp = typ.clone();

        for _ in 0..pointer_counter-1 {
            tmp = Types::Pointer(Box::new(tmp));
        }

        return (Keyword::Pointer(tmp, typ), 0)
    }

    fn create_func_propagate_field(&mut self, fname: String, user_def: String, is_ptr: bool) -> Vec<Expr> {
        let mut expr_param = Vec::new();

        if let Expr::StructDef { struct_fields, .. } | Expr::MacroStructDef { struct_fields, .. } = self.find_structure(&user_def) {
            for field in struct_fields {
                match field {
                    Expr::VariableName { typ: vartyp, name: varname, constant, field_data, .. } => {
                        let new_name = if is_ptr {
                            format!("{fname}->{varname}")
                        } else {
                            format!("{fname}.{varname}")
                        };
                        if let Types::TypeDef { ref type_name, .. } = vartyp {
                            expr_param.append(&mut self.create_func_propagate_field(new_name.clone(), type_name.to_owned(), field_data.1));
                        }
                        let new_expr = Expr::Variable {
                            info: Box::new(Expr::VariableName {
                                typ: Types::Pointer(Box::new(vartyp)),
                                name: new_name,
                                reassign: false,
                                constant,
                                field_data: (true, is_ptr),
                            }),
                            value: Box::new(Expr::None),
                        };

                        expr_param.push(new_expr);
                    },
                    _ => (),
                }
            }
        }
        return expr_param;
    }

    fn create_func(&mut self, typ: Types, params: Vec<Token>, name: String, is_inline: bool) {
        let mut variables = Vec::new();
        let mut expr_param = Vec::new();
        let mut array_lens: Vec<String> = Vec::new();

        let mut typeid_names: Vec<String> = Vec::new();
        let mut kw_buf = Keyword::None;
        let mut pointer_counter = 0;
        let mut is_macro_func = false;
        let mut is_generic = false;

        match typ {
            Types::Any | Types::TypeId => is_macro_func = true,
            _ => (),
        }

        for (i, param) in params.iter().enumerate() {
            match param {
                Token::Ident(ident) => {
                    if let Keyword::None = kw_buf {
                        let keyword_res = self.keyword_map.get(ident);
                        match keyword_res {
                            Some(kw) => {
                                let mut updated_kw_buf = false;
                                if !array_lens.is_empty() {
                                    // TODO: make this support mutli-dimensional arrays
                                    // kw_buf = Keyword::Arr { typ: Box::new(self.keyword_to_type(kw.clone())), length: array_lens[0].clone() };
                                    let typ = self.keyword_to_type(kw.clone());
                                    kw_buf = Keyword::TypeDef { type_name: String::from("array"), generics: Some(vec![typ]) };
                                    array_lens.clear();
                                    updated_kw_buf = true;
                                }

                                if pointer_counter > 0 {
                                    (kw_buf, pointer_counter) = self.create_keyword_pointer(self.keyword_to_type(kw.clone()), pointer_counter);
                                    updated_kw_buf = true;
                                }

                                if is_generic {
                                    self.comp_err(&format!("declaring generic but {} is already an identifier", ident));
                                    exit(1);
                                }

                                if !updated_kw_buf {
                                    kw_buf = kw.clone();
                                    if let Keyword::TypeId | Keyword::Any = kw_buf {is_macro_func = true}
                                }
                            },
                            None => {
                                let found_ident = self.find_ident(ident.to_string());
                                match found_ident {
                                    Expr::StructDef { .. } => {
                                        if !array_lens.is_empty() {
                                            // TODO: make this support mutli-dimensional arrays
                                            // kw_buf = Keyword::Arr { typ: Box::new(self.keyword_to_type(kw.clone())), length: array_lens[0].clone() };
                                            let typ = Types::TypeDef {
                                                type_name: ident.to_string(),
                                                generics: None,
                                            };
                                            kw_buf = Keyword::TypeDef { type_name: String::from("array"), generics: Some(vec![typ]) };
                                            array_lens.clear();
                                        } else if pointer_counter > 0 {
                                            (kw_buf, pointer_counter) = self.create_keyword_pointer(Types::TypeDef {
                                                type_name: ident.to_owned(),
                                                generics: Some(vec![]),
                                            }, pointer_counter);
                                        } else {
                                            kw_buf = Keyword::TypeDef {
                                                type_name: ident.to_string(),
                                                generics: Some(vec![]),
                                            };
                                        }
                                    },
                                    Expr::MacroStructDef { .. } => {
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
                                                            let typ = self.keyword_to_type(keyword.clone());
                                                            pass_typs.push(typ);
                                                        },
                                                        None => {
                                                            let mut found = false;
                                                            for name in &typeid_names {
                                                                if name == &name_buf {
                                                                    pass_typs.push(Types::TypeDef { type_name: name_buf.clone(), generics: None });
                                                                    found = true;
                                                                    break;
                                                                }
                                                            }

                                                            if !found {
                                                                let found_struc = self.find_structure(&name_buf);
                                                                if let Expr::StructDef { .. } = found_struc {
                                                                    pass_typs.push(Types::TypeDef { type_name: name_buf.clone(), generics: None });
                                                                    found = true;
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
                                            generics: Some(pass_typs),
                                        };

                                        if pointer_counter > 0 {
                                            (kw_buf, pointer_counter) = self.create_keyword_pointer(self.keyword_to_type(kw_buf), pointer_counter);
                                        }
                                    },
                                    Expr::EnumDef { .. } => {
                                        if pointer_counter > 0 {
                                            (kw_buf, pointer_counter) = self.create_keyword_pointer(Types::TypeDef {
                                                type_name: ident.to_owned(),
                                                generics: None,
                                            }, pointer_counter);
                                        } else {
                                            kw_buf = Keyword::TypeDef {
                                                type_name: ident.to_string(),
                                                generics: None,
                                            };
                                        }
                                    },
                                    _ => {
                                        if is_generic {
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
                                    }
                                }
                            },
                        }
                    } else {
                        let typ = self.keyword_to_type(kw_buf.clone());
                        if let Types::TypeDef {type_name: ref user_def, .. } = typ {
                            // THIS NEEDS TO BE DONE HERE BECAUSE WE ARE ENTERING A NEW SCOPE, THE
                            // "propagate_struct_fields" FUNCTION WORKS FOR THE CURRENT SCOPE
                            expr_param.append(&mut self.create_func_propagate_field(ident.clone(), user_def.to_owned(), false));
                        } else if let Keyword::Pointer(.., last) = kw_buf {
                            if let Types::TypeDef { type_name: ref user_def, .. } = last {
                                expr_param.append(&mut self.create_func_propagate_field(ident.clone(), user_def.to_owned(), true));
                            }
                        } else if let Types::TypeId = typ {
                            typeid_names.push(ident.to_owned());
                        }

                        let final_expr = Expr::VariableName {
                            typ,
                            name: ident.to_owned(),
                            reassign: false,
                            constant: false,
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
                Token::Int(intlit_str) => {
                    if intlit_str.is_empty() {
                        array_lens.push(intlit_str.clone());
                        continue;
                    }

                    let intlit_res = intlit_str.parse::<f64>();
                    match intlit_res {
                        Ok(_) => {
                            array_lens.push(intlit_str.clone());
                        },
                        Err(_) => continue,
                    }
                },
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
            Expr::Func { typ, params: variables, name: name.clone(), is_inline }
        };
        self.functions.push(expr.clone());
        self.current_func = name.clone();
        self.func_to_vars.entry(name.clone()).or_insert(vec![expr_param]);
        self.in_func = true;

        let sanitised_expr = match expr {
            Expr::Func { ref typ, ref params, ref name, is_inline } => {
                let san_name = name.replace(".", "__");
                Expr::Func { typ: typ.clone(), params: params.clone(), name: san_name, is_inline }
            },
            Expr::MacroFunc { ref typ, ref params, ref name } => {
                let san_name = name.replace(".", "__");
                Expr::MacroFunc { typ: typ.clone(), params: params.clone(), name: san_name }
            }
            _ => unreachable!(),
        };
        self.program_push(sanitised_expr);
        self.token_stack.clear();
    }

    // just definition, not the body of the enum yet
    fn create_enum_def(&mut self, name: String) {
        if self.in_func {
            self.comp_err(&format!("cannot make enum {name} inside a function"));
            exit(1);
        }

        let keyword_res = self.keyword_map.get(&name);
        match keyword_res {
            Some(kw) => {
                self.comp_err(&format!("can't use keyword {kw:?} as enum name"));
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

        self.current_func = name.clone();
        self.func_to_vars.entry(name.clone()).or_insert(vec![vec![]]);
        self.in_enum_def = true;

        self.expr_stack.push(Expr::EnumName(name));
        self.token_stack.clear();
    }

    // create the final enum with the body
    fn create_enum(&mut self) {
        let name = self.expr_stack.remove(0);
        let exprs = self.expr_stack.clone();
        self.expr_stack.clear();

        for ex in &exprs {
            self.enums_fields.push(ex.clone());
        }

        let sanitised_name = if let Expr::EnumName(ref enumname) = name {
            enumname.replace(".", "__")
        } else {unreachable!()};

        self.program_push(Expr::EnumDef {
            enum_name: Box::new(Expr::EnumName(sanitised_name.clone())),
            enum_fields: exprs.clone()
        });


        match name {
            Expr::EnumName(ref enum_name) => {
                self.program_push(Expr::EndStruct(sanitised_name.clone()));
                self.func_to_vars.remove(enum_name);
            },
            _ => {
                self.comp_err(&format!("unexpected expression when creating struct"));
                exit(1);
            }
        }

        self.in_enum_def = false;
        let expr = Expr::EnumDef { enum_name: Box::new(name.clone()), enum_fields: exprs.clone() };
        self.enums.push(expr);

        let field_count = Expr::Variable {
            info: Box::new(Expr::VariableName {
                typ: Types::Usize,
                name: format!("{sanitised_name}.field_count"),
                reassign: false,
                constant: true,
                field_data: (false, false),
            }),
            value: Box::new(Expr::IntLit(format!("{}", exprs.len()))),
        };
        self.enums_fields.push(field_count.clone());
        self.program_push(field_count);
    }

    // this starts defining a struct
    fn create_struct_def(&mut self, name: String, generics: Expr, is_shared: bool) {
        if self.in_func {
            self.comp_err(&format!("cannot make struct {name} inside a function"));
            exit(1);
        }

        // don't need to check if num, already checked when getting name param
        // edit: no clue what i meant by "if num", but it's fine... right?
        let keyword_res = self.keyword_map.get(&name);
        match keyword_res {
            Some(kw) => {
                self.comp_err(&format!("can't use keyword {kw:?} as struct name"));
                exit(1);
            },
            None => (),
        }

        let found_func = self.find_ident(name.clone());
        if let Expr::Func { .. } = found_func {
            self.comp_err(&format!("identifier {name} already declared as function"));
            exit(1);
        } else if let Expr::MacroFunc { .. } = found_func {
            self.comp_err(&format!("identifier {name} already declared as function"));
            exit(1);
        } else if let Expr::StructDef { .. } = found_func {
            self.comp_err(&format!("identifier {name} already declared as struct"));
            exit(1);
        } else if let Expr::MacroStructDef { .. } = found_func {
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
                                constant: false,
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
            self.expr_stack.push(Expr::StructName{ name, is_shared });
        } else {
            self.expr_stack.push(Expr::MacroStructName { name, generics: pass_generic, is_shared });
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
        self.structures.push(expr.clone());

        let sanitised_name: String;
        let sanitised_expr = match expr {
            Expr::StructDef { ref struct_name, ref struct_fields } => {
                if let Expr::StructName { name: sname, is_shared } = *struct_name.clone() {
                    sanitised_name = sname.replace(".", "__");
                    Expr::StructDef { struct_name: Box::new(Expr::StructName{name: sanitised_name.clone(), is_shared}), struct_fields: struct_fields.to_vec() }
                } else {
                    unreachable!()
                }
            },
            Expr::MacroStructDef { ref struct_name, ref struct_fields } => {
                if let Expr::StructName {name: sname, is_shared} = *struct_name.clone() {
                    sanitised_name = sname.replace(".", "__");
                    Expr::MacroStructDef { struct_name: Box::new(Expr::StructName { name: sanitised_name.clone(), is_shared }), struct_fields: struct_fields.to_vec() }
                } else if let Expr::MacroStructName { name: sname, generics, is_shared } = *struct_name.clone() {
                    sanitised_name = sname.replace(".", "__");
                    Expr::MacroStructDef { struct_name: Box::new(Expr::MacroStructName { name: sanitised_name.clone(), generics, is_shared }), struct_fields: struct_fields.to_vec() }
                } else {
                    unreachable!()
                }
            },
            _ => unreachable!(),
        };

        self.program_push(sanitised_expr);
        match name {
            Expr::StructName { name: struct_name, .. } => {
                self.program_push(Expr::EndStruct(sanitised_name));
                self.func_to_vars.remove(&struct_name);
            },
            Expr::MacroStructName { .. } => {
                self.program_push(Expr::MacroEndStruct(sanitised_name));
            },
            _ => {
                self.comp_err(&format!("unexpected expression when creating struct"));
                exit(1);
            }
        }


        self.in_struct_def = false;
    }

    fn boolean_conditions(&self, params: &Vec<Token>, is_loop: bool) -> (Vec<Expr>, Expr) {
        let mut func_call = Expr::None;
        let mut func_just_got = false;
        let mut func_brack_rc = 0;
        let mut func_params = Vec::new();

        let mut expr_params = Vec::new();
        let mut side_affect = Expr::None;
        let mut had_index = false;

        let mut pointer_counter = 0;

        if params.is_empty() {
            expr_params.push(Expr::IntLit(String::from(";")));
        }

        for (i, param) in params.iter().enumerate() {
            match param {
                Token::Equal => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::Equal)
                    } else {
                        func_params.push(param.clone());
                    }
                },
                Token::SmallerThan => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::SmallerThan)
                    } else {
                        func_params.push(param.clone());
                    }
                },
                Token::BiggerThan => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::BiggerThan)
                    } else {
                        func_params.push(param.clone());
                    }
                } ,
                Token::Exclaim => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::Exclaim)
                    } else {
                        func_params.push(param.clone());
                    }
                },
                Token::True => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::True)
                    } else {
                        func_params.push(param.clone());
                    }
                },
                Token::False => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::False)
                    } else {
                        func_params.push(param.clone());
                    }
                },
                Token::Caret => {
                    if func_brack_rc == 0 {
                        pointer_counter += 1;
                    } else {
                        func_params.push(param.clone());
                    }
                },
                Token::Ident(ident) => {
                    if func_brack_rc != 0 {
                        func_params.push(param.clone());
                        continue;
                    }

                    // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                    let ident_num = ident.parse::<f64>();
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

                    // TODO: add dereferncing inside of boolean condition
                    let expr = self.find_ident(ident.to_string());
                    match expr {
                        Expr::Func { ref name, .. } | Expr::MacroFunc { ref name, .. } => {
                            if let Token::Lbrack = params[i+1] {
                                func_call = expr;
                                func_brack_rc += 1;
                                func_just_got = true;
                            } else {
                                self.comp_err(&format!("can't compare to function. did you mean to call {name}?. do `{name}()`"));
                                exit(1);
                            }
                        },
                        Expr::VariableName { typ, name, reassign, constant, field_data } => {
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
                                        constant,
                                        field_data
                                    });
                                } else {
                                    expr_params.push(Expr::VariableName { typ, name, reassign, constant, field_data });
                                }
                            } else {
                                expr_params.push(Expr::VariableName { typ, name, reassign, constant, field_data });
                            }
                        },
                        Expr::None => {
                            if is_loop && expr_params.is_empty() {
                                let new_varname = Expr::VariableName {
                                    typ: Types::Usize,
                                    name: ident.to_owned(),
                                    reassign: true,
                                    constant: false,
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
                    if func_brack_rc != 0 {
                        func_params.push(param.clone());
                        continue;
                    }

                    if had_index {
                        had_index = false;
                        continue;
                    }
                    expr_params.push(self.check_intlit(intlit.to_owned()));
                },
                Token::Lbrack => {
                    if func_just_got {
                        func_just_got = false;
                        continue;
                    }
                    func_brack_rc += 1;
                },
                Token::Rbrack => {
                    if func_brack_rc > 0 {
                        func_brack_rc -= 1;
                        if func_brack_rc == 0 {
                            expr_params.push(self.create_func_call(&func_call, func_params.clone()));
                            func_call = Expr::None;
                            func_params.clear();
                        }
                    }
                },
                Token::Str(word) => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::StrLit { content: word.to_owned(), is_cstr: true })
                    } else {
                        func_params.push(param.clone());
                    }
                },
                Token::Char(charlit) => {
                    if func_brack_rc == 0 {
                        expr_params.push(Expr::CharLit(charlit.clone()))
                    } else {
                        func_params.push(param.clone());
                    }
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

    fn create_case(&mut self, params: &Vec<Token>, is_fall: bool) {
        self.token_stack.clear();
        self.new_scope(Expr::None);

        let expr_params = if params.is_empty() {
            vec![Expr::None]
        } else {
            self.boolean_conditions(&params, false).0
        };

        if self.in_defer {
            if is_fall {
                self.expr_stack.push(Expr::Fall(expr_params));
            } else {
                self.expr_stack.push(Expr::Case(expr_params));
            }
        } else {
            if is_fall {
                self.program_push(Expr::Fall(expr_params));
            } else {
                self.program_push(Expr::Case(expr_params));
            }
        }
    }

    fn create_branch(&mut self, branch_typ: Keyword, params: Vec<Token>, capture: &String) {
        self.token_stack.clear();

        let captured = if params.len() == 1 && !capture.is_empty() {
            if let Token::Ident(variable) = &params[0] {
                let found = self.find_variable(&variable);
                match found {
                    Expr::VariableName { typ, name, .. } => {
                        if let Types::TypeDef { type_name, generics: generics_op } = typ {
                            if type_name == String::from("option") {
                                let generics = match generics_op {
                                    Some(g) => g,
                                    None => {
                                        self.comp_err(&format!("{type_name} unsupported in for loops currently"));
                                        exit(1);
                                    },
                                };

                                let subtyp = generics[0].clone();

                                match subtyp {
                                    Types::TypeDef { ref type_name, .. } => self.propagate_struct_fields(capture.to_owned(), type_name.to_string(), false, true),
                                    _ => (),
                                }

                                let value = self.find_variable(&format!("{name}.value"));
                                if let Expr::None = value {
                                    self.comp_err(&format!("expected variable {name}.value to exist when it doesn't. can't be used in if statement capture"));
                                    exit(1);
                                }

                                Expr::Variable {
                                    info: Box::new(Expr::VariableName {
                                        typ: subtyp,
                                        name: capture.to_owned(),
                                        reassign: false,
                                        constant: false,
                                        field_data: (false, false)
                                    }),
                                    value: Box::new(value),
                                }
                            } else {
                                self.comp_err(&format!("{type_name} unsupported in if statement capture"));
                                exit(1);
                            }
                        } else {
                            self.comp_err(&format!("{typ:?} unsupported in if statement capture"));
                            exit(1);
                        }
                    },
                    _ => unreachable!(),
                }
            } else {
                self.comp_err(&format!("expected one identifier in condition, found {:?}", params[0]));
                exit(1);
            }
        } else {
            Expr::None
        };

        match branch_typ {
            Keyword::If => {
                let expr_params = self.boolean_conditions(&params, false).0; // only getting the array
                self.new_scope(captured.clone());
                if self.in_defer {
                    self.expr_stack.push(Expr::If(expr_params, Box::new(captured)));
                } else {
                    self.program_push(Expr::If(expr_params, Box::new(captured)));
                }
            },
            Keyword::OrIf => {
                let expr_params = self.boolean_conditions(&params, false).0; // only getting the array
                self.new_scope(captured.clone());
                if self.in_defer {
                    self.expr_stack.push(Expr::OrIf(expr_params, Box::new(captured)));
                } else {
                    self.program_push(Expr::OrIf(expr_params, Box::new(captured)));
                }
            },
            Keyword::Else => {
                if !params.is_empty() {
                    self.comp_err("expected no conditions for else branch, got one");
                    exit(1);
                }

                if !capture.is_empty() {
                    self.comp_err("expected no capture for else branch, got one");
                    exit(1);
                }

                self.new_scope(Expr::None);
                if self.in_defer {
                    self.expr_stack.push(Expr::Else);
                } else {
                    self.program_push(Expr::Else);
                }
            },
            Keyword::Switch => {
                let expr_params = self.boolean_conditions(&params, false).0; // only getting the array
                self.new_scope(Expr::None);
                if self.in_defer {
                    self.expr_stack.push(Expr::Switch(expr_params));
                } else {
                    self.program_push(Expr::Switch(expr_params));
                }
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

                if self.in_defer {
                    self.expr_stack.push(Expr::Loop {
                        condition: expr_params.0,
                        modifier: Box::new(Expr::IntLit(modifier.to_owned())),
                    });
                } else {
                    self.program_push(Expr::Loop {
                        condition: expr_params.0,
                        modifier: Box::new(Expr::IntLit(modifier.to_owned())),
                    });
                }
            },
            _ => {
                self.comp_err(&format!("unexpected loop modifier: {modifier}"));
                exit(1);
            }
        }
    }

    fn create_for(&mut self, params: Vec<Token>, modifier: String) {
        if params.len() > 1 {
            self.comp_err("expected only one expression to loop through");
            exit(1);
        }

        if modifier.is_empty() {
            self.comp_err("expected new variable names e.g. `[elem]` or `[elem i]`");
            exit(1);
        }

        let new_varnames: Vec<&str> = modifier.split(' ').collect();
        if new_varnames.len() > 2 {
            self.comp_err(&format!("expected at most two variables to extract in for loop, got {}", new_varnames.len()));
            exit(1);
        }

        let mut expr = Expr::None;
        let mut side_effects = Vec::new();
        let in_this = self.boolean_conditions(&params, false);

        let for_this_typ = match &in_this.0[0] {
            Expr::VariableName { typ, constant, .. } => {
                if let Types::Arr { typ: subtyp, .. } = typ {
                    if let Types::TypeDef { type_name, .. } = *subtyp.clone() {
                        self.propagate_struct_fields(new_varnames[0].to_owned(), type_name, false, *constant);
                    }
                    *subtyp.clone()
                } else if let Types::TypeDef { type_name, generics: generics_op } = typ {
                    let generics = match generics_op {
                        Some(g) => g,
                        None => {
                            self.comp_err(&format!("{type_name} unsupported in for loops currently"));
                            exit(1);
                        },
                    };

                    if type_name == &String::from("dyn") || type_name == &String::from("array") {
                        let subtyp = generics[0].clone();
                        match subtyp {
                            Types::TypeDef { ref type_name, .. } => self.propagate_struct_fields(new_varnames[0].to_owned(), type_name.to_string(), false, *constant),
                            _ => (),
                        }
                        subtyp
                    } else if type_name == &String::from("string") || type_name == &String::from("str") {
                        Types::Char
                    } else {
                        self.comp_err(&format!("{type_name} unsupported in for loops currently"));
                        exit(1);
                    }
                } else {
                    self.comp_err(&format!("{typ:?} unsupported in for loops currently"));
                    exit(1);
                }
            },
            unexpected => {
                self.comp_err(&format!("{unexpected:?} unsupported in for loops currently"));
                exit(1);
            }
        };

        let found_var = self.find_variable(&new_varnames[0].to_owned());
        match found_var {
            Expr::VariableName { constant, .. } => {
                if !constant {
                    self.comp_err(&format!("variable {} is constant. can't be used in for loop", new_varnames[0]));
                }
                if new_varnames.len() == 2 {
                    expr = Expr::For { 
                        for_this: Box::new(found_var.clone()),
                        in_this: Box::new(in_this.0[0].clone()),
                        iterator: new_varnames[1].to_string()
                    };
                    side_effects.push(Expr::Variable {
                        info: Box::new(Expr::VariableName {
                            typ: Types::Usize,
                            name: new_varnames[1].to_string(),
                            reassign: false,
                            constant: false,
                            field_data: (false, false),
                        }),
                        value: Box::new(Expr::IntLit(String::from("0")))
                    });
                } else {
                    expr = Expr::For { 
                        for_this: Box::new(found_var.clone()),
                        in_this: Box::new(in_this.0[0].clone()),
                        iterator: String::new()
                    };
                }
            },
            Expr::None => {
                let new_var = Expr::VariableName {
                    typ: for_this_typ,
                    name: new_varnames[0].to_string(),
                    reassign: false,
                    constant: false,
                    field_data: (false, false)
                };

                if new_varnames.len() == 2 {
                    expr = Expr::For {
                        for_this: Box::new(new_var.clone()),
                        in_this: Box::new(in_this.0[0].clone()),
                        iterator: new_varnames[1].to_string()
                    };
                    side_effects.push(Expr::Variable { info: Box::new(new_var), value: Box::new(Expr::None) });
                    side_effects.push(Expr::Variable {
                        info: Box::new(Expr::VariableName {
                            typ: Types::Usize,
                            name: new_varnames[1].to_string(),
                            reassign: false,
                            constant: false,
                            field_data: (false, false),
                        }),
                        value: Box::new(Expr::IntLit(String::from("0")))
                    });
                } else {
                    expr = Expr::For { 
                        for_this: Box::new(new_var.clone()),
                        in_this: Box::new(in_this.0[0].clone()),
                        iterator: String::new()
                    };
                    side_effects.push(Expr::Variable { info: Box::new(new_var), value: Box::new(Expr::None) });
                }
            },
            _ => (),
        }

        if self.in_defer {
            self.expr_stack.push(expr);
        } else {
            self.program_push(expr);
        }
        self.token_stack.clear();
        self.new_scope_vars(side_effects);
    }

    fn create_defer(&mut self, exprs: Vec<Expr>) {
        for expr in exprs {
            self.program_push(expr)
        }
    }

    fn handle_lcurl(&mut self) {
        let mut typ: Types = Types::None;
        let mut name = String::new();
        let mut keyword = Keyword::None;

        let mut is_inline = false;

        let mut brack_rc = 0;
        let mut in_bracks = false;
        let mut seen_colon = 0;
        let mut pointer_counter = 0;

        let mut params = Vec::new();

        let mut create_branch = false;
        let mut create_case = false;
        let mut create_fall = false;

        let mut create_loop = false;
        let mut loop_modifier = String::new();

        let mut create_for = false;

        let mut create_defer = false;

        let mut create_struct = false;
        let mut is_shared = false;
        let mut create_generic = false;
        let mut generic_subtype = Expr::None;

        let mut create_enum = false;

        for (i, token) in self.token_stack.clone().iter().enumerate() {
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
                        typ = Types::Void;

                        if pointer_counter > 0 {
                            let tmp_kw = self.create_keyword_pointer(Types::Void, pointer_counter).0;
                            typ = self.keyword_to_type(tmp_kw);
                        }
                    }
                },
                Token::Ident(ident) => {
                    if is_inline {
                        if let Token::Macro = self.token_stack[i-1] {
                            continue;
                        }
                    } else if is_shared {
                        if let Token::Macro = self.token_stack[i-1] {
                            continue;
                        }
                    }

                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        // TODO: LATER CHECK IF ERROR IS NUM TOO LARGE
                        let ident_num = ident.parse::<f64>();
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

                            if let Expr::StructDef { .. } | Expr::MacroStructDef { .. } | Expr::EnumDef { .. } = self.find_ident(ident.clone()) {
                                keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: Some(vec![]) };
                            }

                            match keyword {
                                Keyword::If | Keyword::OrIf | Keyword::Else => create_branch = true,
                                Keyword::Switch => create_branch = true,
                                Keyword::Case => create_case = true,
                                Keyword::Fall => create_fall = true,
                                Keyword::For => create_for = true,
                                Keyword::Loop => create_loop = true,
                                Keyword::Struct => create_struct = true,
                                Keyword::Enum => create_enum = true,
                                Keyword::Defer => create_defer = true,
                                _ => {
                                    if let Types::None = typ {
                                        typ = if create_generic {
                                            // TODO: check if ident is already declared
                                            Types::Generic(ident.to_owned()) // might cause errors idk lmao
                                        } else if self.in_struct_def && self.previous_func.is_empty() {
                                            Types::TypeDef { type_name: ident.to_owned(), generics: Some(vec![]) }
                                        } else {
                                            self.keyword_to_type(keyword.clone())
                                        };

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
                        if in_bracks {
                            params.push(token.clone());
                        }
                        in_bracks = true;
                    }

                },
                Token::Rbrack => {
                    brack_rc -= 1;
                    if brack_rc == 0 {
                        in_bracks = false;
                    } else {
                        params.push(token.clone());
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
                    } else if create_for {
                        loop_modifier = symbols.to_owned();
                    } else if create_branch && !in_bracks {
                        loop_modifier = symbols.to_owned();
                    } else if in_bracks {
                        params.push(token.clone());
                    } else if create_struct {
                        generic_subtype = Expr::IntLit(symbols.to_string());
                    } else if let Types::TypeDef { type_name, .. } = typ {
                        let pass_typs = self.parse_generics_to_pass(symbols, true);

                        typ = Types::TypeDef { type_name: type_name.clone(), generics: Some(pass_typs) };
                    } else {
                        self.comp_err(&format!("unexpected integer literal in block definition: {symbols}"));
                        exit(1);
                    }
                },
                Token::Lsquare => (),
                Token::Rsquare => (), // these might break stuff idk
                Token::Macro => {
                    if self.token_stack.len() < i + 1 {
                        self.comp_err(&format!("expected more tokens after macro"));
                        exit(1);
                    }

                    match &self.token_stack[i+1] {
                        Token::Ident(macro_ident) => {
                            let macro_res = self.macros_map.get(macro_ident);
                            let mac = match macro_res {
                                Some(m) => m.clone(),
                                None => {
                                    self.comp_err(&format!("expected macro, got {macro_ident}"));
                                    exit(1);
                                }
                            };

                            if let Macros::Inline = mac {
                                is_inline = true;
                            }

                            if let Macros::Shared = mac {
                                is_shared = true;
                            }
                        },
                        _ => (),
                    }
                },
                Token::Quote => (),
                Token::Str(word) => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err(&format!("unexpected token String token: `{word}`"));
                        exit(1);
                    }
                },
                Token::SingleQuote => (),
                Token::Char(charlit) => {
                    if in_bracks {
                        params.push(token.clone());
                    } else {
                        self.comp_err(&format!("unexpected token Char token: `{charlit}`"));
                        exit(1);
                    }
                },
                unexpected => {
                    self.comp_err(&format!("unexpected token: {unexpected:?}"));
                    exit(1);
                },
            }
        }

        if create_for {
            if (self.in_struct_def && !self.in_func) || self.in_enum_def {
                self.comp_err("can't use loops inside structs or enums");
                exit(1);
            }

            self.create_for(params, loop_modifier);
            return
        }

        if create_loop {
            if (self.in_struct_def && !self.in_func) || self.in_enum_def {
                self.comp_err("can't use loops inside structs or enums");
                exit(1);
            }

            self.create_loop(params, loop_modifier);
            return
        }

        if create_branch {
            if (self.in_struct_def && !self.in_func) || self.in_enum_def {
                self.comp_err("can't use branches inside structs or enums");
                exit(1);
            }

            self.create_branch(keyword, params, &loop_modifier);
            return
        }

        if create_case || create_fall {
            // TODO: check if inside switch as well, could do it inside generator
            if (self.in_struct_def && !self.in_func) || self.in_enum_def {
                self.comp_err("can't use cases inside structs or enums");
                exit(1);
            }

            self.create_case(&params, create_fall);
            return
        }

        if create_struct {
            if self.in_struct_def || self.in_enum_def {
                self.comp_err("can't create a struct inside an enum or struct");
                exit(1);
            }

            if seen_colon != 2 {
                self.comp_err(&format!("expected assigment operator `:`. did you mean `struct {name} :: {{`?"));
                exit(1);
            } else {
                self.create_struct_def(name.clone(), generic_subtype, is_shared);
                return
            }
        }

        if create_enum {
            if self.in_struct_def || self.in_enum_def {
                self.comp_err("can't create an enum inside an enum or struct");
                exit(1);
            }

            if seen_colon != 2 {
                self.comp_err(&format!("expected assigment operator `:`. did you mean `enum {name} :: {{`?"));
                exit(1);
            } else {
                self.create_enum_def(name.clone());
                return
            }
        }

        if create_defer {
            self.token_stack.clear();
            self.in_defer = true;
            self.defer_scope = self.current_scope;
            self.new_scope(Expr::None);
            return
        }

        if seen_colon == 2 {
            if self.in_struct_def {
                // struct hasn't been generated yet so even if we made it here, it would make a
                // function that has the struct inside which is not what we want

                if self.previous_func.is_empty() {
                    self.previous_func = self.current_func.clone();
                    // self.prev_scope(); not sure to keep this or not
                    self.create_struct();
                    self.in_struct_def = true;
                }

                let namespaced_name = format!("{}.{name}", self.previous_func);
                self.create_func(typ.clone(), params.clone(), namespaced_name, is_inline);
                return
            } else if self.in_enum_def {
                self.comp_err("can't create a function inside enums");
                exit(1);
            }

            self.create_func(typ.clone(), params.clone(), name.clone(), is_inline);
            return
        } else if seen_colon > 2 {
            self.comp_err("unexpected assignment operator `:`");
            exit(1);
        }

        self.new_scope(Expr::None);
        self.program_push(Expr::StartBlock);
    }

    fn find_global_variable(&self, ident: &String) -> Expr {
        for var in &self.global_vars {
            match var {
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
                            if &name.replace("->", ".") == ident {
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
                        Expr::StructName { name: acc_name, .. } => {
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
                        Expr::StructName { name: acc_name, .. } => {
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

    fn find_enum(&self, ident: &String) -> Expr {
        for enu in &self.enums {
            match enu {
                Expr::EnumDef { enum_name, .. } => {
                    match *enum_name.clone() {
                        Expr::EnumName(name) => {
                            if &name == ident {
                                return enu.clone()
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

    fn find_enum_fields(&self, ident: &String) -> Expr {
        for enum_field in &self.enums_fields {
            match enum_field {
                Expr::VariableName { name, .. } => {
                    if name == ident {
                        return enum_field.clone()
                    }
                },
                Expr::Variable { info, .. } => {
                    if let Expr::VariableName { name, .. } = *info.clone() {
                        if &name == ident {
                            return *info.clone();
                        }
                    }
                }
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

        found = self.find_global_variable(&ident);
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

        found = self.find_enum(&ident);
        if let Expr::None = found {}
        else {
            return found
        }

        found = self.find_enum_fields(&ident);
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
        let mut pointer_counter = 0;

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
                Token::Char(charlit) => {
                    if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                    } else if square_rc > 0 {
                        self.comp_err(&format!("expected integers inside [], found token {:?}", param));
                        exit(1);
                    } else {
                        expr_params.push(Expr::CharLit(charlit.clone()));
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
                    let ident_num = ident.parse::<f64>();
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
                                if pointer_counter > 0 {
                                    let (kw, _) = self.create_keyword_pointer(Types::TypeId, pointer_counter);
                                    let typ = self.keyword_to_type(kw);
                                    expr_params.push(Expr::VariableName { typ, name: ident.clone(), reassign: false, constant: false, field_data: (false, false) });
                                    pointer_counter = 0;
                                    continue;
                                }
                                let _ = self.keyword_to_type(keyword.clone());
                                expr_params.push(Expr::VariableName { typ: Types::TypeId, name: ident.clone(), reassign: false, constant: false, field_data: (false, false) })
                            },
                            None => {
                                self.comp_err(&format!("unknown identifier: {}", ident));
                                exit(1);
                            }
                        }
                    } else if let Expr::StructDef { ref struct_name, .. } = expr {
                        if pointer_counter > 0 {
                            let name = if let Expr::StructName { name: n, .. } = *struct_name.clone() { n } else { unreachable!() };
                            let (kw, _) = self.create_keyword_pointer(Types::TypeDef {
                                type_name: name.clone(),
                                generics: None,
                            }, pointer_counter);
                            pointer_counter = 0;
                            let typ = self.keyword_to_type(kw);
                            expr_params.push(Expr::VariableName { typ, name: ident.clone(), reassign: false, constant: false, field_data: (false, false) });
                        } else {
                            expr_params.push(expr);
                        }
                    } else if let Expr::Func { .. } = expr {
                        nested_func = expr;
                    } else if let Expr::MacroFunc { .. } = expr {
                        nested_func = expr;
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
                        intlit_buf.clear();
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
                Token::Multiply => {
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
                    if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                    } else if square_rc > 0 {
                        intlit_buf.push('&');
                    }
                },
                Token::Caret => {
                    pointer_counter += 1;
                    if nested_brack_rs > 0 {
                        nested_params.push(param.clone());
                    } else if square_rc > 0 {
                        intlit_buf.push('^');
                    }
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
                Token::SingleQuote => (),
                _ => {
                    self.comp_err(&format!("unexpected token: {:?}", param));
                    exit(1);
                } 
            }
        }

        match expr {
            Expr::Func { name, .. } | Expr::MacroFunc { name, .. } => {
                let san_name = name.replace(".", "__");
                return Expr::FuncCall { name: san_name, gave_params: expr_params }
            },
            _ => {
                self.comp_err(&format!("expected a function, got {:?}", expr));
                exit(1);
            },
        }
    }

    pub fn handle_import_macro(&mut self, mut path: String) -> Expr {
        if path.chars().nth(path.len()-1).unwrap() == 'h' {
            self.imports.push(path.to_string());
            let no_extension = path.split_at(path.len()-2);
            return Expr::Import(no_extension.0.to_string())
        }

        if !self.imports.is_empty() {
            for imp in &self.imports {
                if imp == &path {
                    return Expr::None
                }
            }

            self.imports.push(path.to_string());
        } else {
            self.imports.push(path.to_string());
        }

        if path.starts_with("base/") {
            path = format!("{CUR_PATH}/{path}");
        } else if path.starts_with("vendor/") {
            let mut split_path = path.splitn(3, '/');
            let which_vendor = if let (Some(first), Some(second)) = (split_path.next(), split_path.next()) {
                format!("{first}/{second}")
            } else {
                self.comp_err(&format!("unknown vendor target in import: {path}"));
                exit(1);
            };
            let str_to_vendor = format!("{CUR_PATH}/{which_vendor}");
            let path_to_vendor = Path::new(&str_to_vendor);
            let copy_options = CopyOptions::new();

            if !Path::new("./vendor").exists() {
                match fs::create_dir("./vendor") {
                    Ok(_) => (),
                    Err(e) => {
                        self.comp_err(&format!("unable to to create vendor folder in current path with error: {e:?}"));
                        exit(1);
                    }
                }
            }

            match copy_items(&vec![path_to_vendor], "./vendor", &copy_options) {
                Ok(_) => (),
                Err(e) => match e.kind {
                    fs_extra::error::ErrorKind::AlreadyExists => (),
                    _ => {
                        self.comp_err(&format!("unable to copy vendor {path} to current path with error: {e:?}"));
                        exit(1);
                    }
                }

            }
        }

        let file_res = fs::read_to_string(path.clone());
        let content = match file_res {
            Ok(content) => content,
            Err(err) => {
                self.comp_err(&format!("unable to read file {path} with error: {err:?}"));
                exit(1);
            },
        };

        let tokens = tokeniser(content);
        let mut parse = ExprWeights::new(tokens, &path);
        // REMEMBER TO DO ADD THIS IF YOU NEED INFO TO CARRY OVER FROM OTHER FILES
        parse.functions = self.functions.clone();
        parse.imports = self.imports.clone();
        parse.structures = self.structures.clone();
        parse.enums = self.enums.clone();
        parse.enums_fields = self.enums_fields.clone();
        parse.global_vars = self.global_vars.clone();
        let mut expressions = parse.parser();

        if self.imports.len() != parse.imports.len() {
            let mut new = parse.imports[self.imports.len()..].to_vec();
            self.imports.append(&mut new);
        }

        if self.functions.len() != parse.functions.len() {
            let mut new = parse.functions[self.functions.len()..].to_vec();
            self.functions.append(&mut new);
        }

        if self.structures.len() != parse.structures.len() {
            let mut new = parse.structures[self.structures.len()..].to_vec();
            self.structures.append(&mut new);
        }

        if self.enums.len() != parse.enums.len() {
            let mut new = parse.enums[self.enums.len()..].to_vec();
            self.enums.append(&mut new);
        }

        if self.enums_fields.len() != parse.enums_fields.len() {
            let mut new = parse.enums_fields[self.enums_fields.len()..].to_vec();
            self.enums_fields.append(&mut new);
        }

        if self.global_vars.len() != parse.global_vars.len() {
            let mut new = parse.global_vars[self.global_vars.len()..].to_vec();
            self.global_vars.append(&mut new);
        }

        self.program.append(&mut expressions);
        return Expr::None
    }

    fn handle_array_macro(&mut self, length: &String, tokens: Vec<Token>) -> Expr {
        if tokens.len() < 1 {
            self.comp_err(&format!("expected keyword and identifer (int x), got {tokens:?}"));
            exit(1);
        }

        let mut colon_counter = 0;
        let mut is_constant = false;
        for token in &tokens {
            match token {
                Token::Colon => {
                    colon_counter += 1;
                    if colon_counter == 2 {
                        is_constant = true;
                    }
                }
                _ => (),
            }
        }

        let keyword: Keyword;
        match &tokens[0] {
            Token::Ident(ident) => {
                let keyword_res = self.keyword_map.get(ident);
                match keyword_res {
                    Some(kw) => keyword = kw.clone(),
                    None => {
                        if let Expr::StructDef { .. } = self.find_structure(ident) {
                            keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: Some(vec![]) };
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
        self.propagate_struct_fields(name.clone(), String::from("array"), false, is_constant);
        Expr::VariableName {
            typ: Types::Arr {
                typ: Box::new(typ),
                length: length.to_owned(),
            },
            name,
            reassign: false,
            constant: is_constant,
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
                    return self.handle_import_macro(path.to_string());
                } else {
                    self.comp_err(&format!("expected \"path_to_file\", got {:?}", value[index+2]));
                    exit(1);
                }
            },
            _ => {
                self.comp_err(&format!("macro {mac:?} not reimplemented yet"));
                exit(1);
            }
        }
    }

    fn handle_left_assign(&mut self, var_info: Vec<Token>, is_constant: bool) -> Expr {
        let mut keyword = Keyword::None;
        let mut pointer_counter = 0;

        match &var_info[0] {
            Token::Ident(ident) => {
                let keyword_res = self.keyword_map.get(ident);
                match keyword_res {
                    Some(kw) => keyword = kw.clone(),
                    None => {
                        match self.find_ident(ident.to_string()) {
                            Expr::EnumDef { .. } => {
                                keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: None };
                            },
                            Expr::StructDef { .. } => {
                                keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: Some(vec![]) }
                            },
                            Expr::MacroStructDef { .. } => {
                                let pass_typs = if let Token::Int(typs) = &var_info[2] {
                                    self.parse_generics_to_pass(typs, false)
                                } else {
                                    self.comp_err(&format!("expected type for generic struct"));
                                    exit(1);
                                };
                                keyword = Keyword::TypeDef { type_name: ident.to_string(), generics: Some(pass_typs) }
                            },
                            _ => (),
                        };

                        if let Keyword::None = keyword {
                            let found_variable = self.find_ident(ident.to_string());
                            match found_variable {
                                Expr::None => {
                                    if self.in_enum_def {
                                        let new_name = format!("{}.{ident}", self.current_func);

                                        if !is_constant {
                                            self.comp_err(&format!("enum field {ident} must be constant if given value. did you mean `{ident} :: <value>`?"));
                                            exit(1);
                                        }
                                        return Expr::VariableName { typ: Types::None, name: new_name, reassign: false, constant: true, field_data: (false, false) }
                                    }
                                    self.comp_err(&format!("undeclared identifier: {ident}"));
                                    exit(1);
                                },
                                Expr::VariableName { typ, name, constant, field_data, .. } => {
                                    if constant {
                                        self.comp_err(&format!("var {name} is constant. can't be reassigned"));
                                        exit(1);
                                    } else if is_constant {
                                        self.comp_err(&format!("var {name} is mutable. constants need value during declaration"));
                                        exit(1);
                                    }

                                    if var_info.len() > 1 {
                                        if let Token::Caret = &var_info[1] {
                                            let mut deref = Expr::DerefPointer(Box::new(
                                                Expr::VariableName { 
                                                    typ,
                                                    name,
                                                    reassign: true,
                                                    constant: false,
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
                                                constant: false,
                                                field_data
                                            }
                                        } else {
                                            self.comp_err(&format!("expected integer to index array, got {:?}", &var_info[2]));
                                            exit(1);
                                        }
                                    }

                                    return Expr::VariableName { typ: typ.clone(), name: name.to_owned(), reassign: true, constant: false, field_data }
                                }
                                expr => {
                                    return expr
                                },
                            }
                        }
                    },
                }
            },
            Token::Lsquare => {
                // this is for arrays
                if var_info.len() < 5 {
                    self.comp_err(&format!("expected more tokens after macro"));
                    exit(1);
                }

                if let Token::Int(intlit) = &var_info[1] {
                    return self.handle_array_macro(intlit, var_info[3..].to_vec());
                } else {
                    self.comp_err(&format!("unexpected token: {:?}", var_info[0]));
                    exit(1);
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
                                        Expr::StructName { name, .. } => {
                                            let tmp = Types::TypeDef { type_name: name.clone(), generics: Some(vec![]) };
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
                                                                let typ = self.keyword_to_type(keyword.clone());
                                                                pass_typs.push(typ);
                                                            },
                                                            None => {
                                                                let found_var = self.find_variable(&name_buf);
                                                                if let Expr::VariableName { typ, name, .. } = found_var {
                                                                    if let Types::TypeId = typ {
                                                                        pass_typs.push(Types::TypeDef { type_name: name, generics: None });
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

                                            let tmp = Types::TypeDef { type_name: name.clone(), generics: Some(pass_typs) };
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
                                                    Expr::StructName { name, .. } => {
                                                        (keyword, pointer_counter) = self.create_keyword_pointer(Types::TypeDef {
                                                            type_name: name.clone(),
                                                            generics: Some(vec![]),
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
                    return Expr::VariableName { typ, name: varname.to_owned(), reassign: false, constant: false, field_data: (false, false) }
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
                    self.propagate_struct_fields(name.to_string(), user_def.to_string(), false, is_constant);
                },
                Keyword::Pointer(.., ref last_typ) => {
                    if let Types::TypeDef {type_name: user_def, ..} = last_typ {
                        self.propagate_struct_fields(name.to_string(), user_def.to_string(), true, is_constant);
                    }
                },
                _ => (),
            }

            let typ = self.keyword_to_type(keyword.clone());
            return Expr::VariableName { typ, name: name.to_owned(), reassign: false, constant: is_constant, field_data: (false, false)};
        } else if let Expr::VariableName { typ, name, constant, field_data, .. } = found_expr {
            if constant || is_constant {
                self.comp_err(&format!("var {name} is constant. can't be reassigned"));
                exit(1);
            } else if is_constant {
                self.comp_err(&format!("var {name} is mutable. constants need value during declaration"));
                exit(1);
            }
            match keyword {
                Keyword::None => {
                    return Expr::VariableName { typ, name, reassign: true, constant: false, field_data};
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
                    let ident_num = ident.parse::<f64>();
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
                        Expr::Func { typ, params, name, is_inline } => {
                            has_func = Expr::Func { typ, params, name, is_inline }
                        },
                        Expr::MacroFunc { typ, params, name } => {
                            has_func = Expr::MacroFunc { typ, params, name }
                        }
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
                Token::True => {
                    if in_func_params {
                        func_params.push(param.clone());
                        continue;
                    }
                    expr_params.push(Expr::True);
                },
                Token::False => {
                    if in_func_params {
                        func_params.push(param.clone());
                        continue;
                    }
                    expr_params.push(Expr::False);
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
                        func_params.clear();
                        in_func_params = false;
                    }
                },
                Token::Quote => (),
                Token::Str(word) => {
                    if in_func_params {
                        func_params.push(param.clone());
                        continue;
                    }
                    expr_params.push(Expr::StrLit { content: word.to_owned(), is_cstr: true });
                },
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
            Expr::VariableName { ref name, constant, .. } => {
                if constant {
                    self.comp_err(&format!("using constant \"{name}\" as address dequalifies it to a variable, did you mean for \"{name}\" to be a variable?"));
                    exit(1)
                }
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
        let mut colon_counter = 0;
        let mut found_macro = false;
        let mut found_amper = false;

        let mut returning = false;
        let mut defering = false;

        let mut create_generic = false;
        let mut is_constant = false;
        let mut array_lens = Vec::new();

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
                        } else if (self.in_struct_def && !self.in_func) || self.in_enum_def {
                            self.comp_err(&format!("can't call function {:?} inside struct or enum", buffer[0]));
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
                Token::Colon => {
                    colon_counter += 1;
                    if colon_counter == 2 {
                        is_constant = true;
                    }
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
                    // TODO: ARRAYS INDEXING WITH [i+1] WILL NOT WORK WITH THIS, edit: this might
                    // be wrong, double check later
                    if !is_right && !returning {
                        array_lens.push(intlit.clone());
                    } else {
                        buffer.push(self.check_intlit(intlit.to_string()));
                    }
                },
                Token::Char(charlit) => {
                    buffer.push(Expr::CharLit(charlit.clone()));
                },
                Token::Str(strlit) => {
                    buffer.push(Expr::StrLit {
                        content: strlit.to_string(),
                        is_cstr: false,
                    });
                },
                Token::Ident(ident) => {
                    if found_macro {
                        if (self.in_struct_def && !self.in_func) || self.in_defer {
                            let mac = self.handle_macros(ident, &i, &value);
                            self.expr_stack.push(mac);
                            return Expr::None;
                        }
                        return self.handle_macros(ident, &i, &value);
                    } else if found_amper {
                        buffer.push(self.create_address(ident));
                        found_amper = false;
                        continue;
                    }

                    // check if this is an expr without a :
                    if !is_right {
                        let keyword_res = self.keyword_map.get(ident);

                        if !array_lens.is_empty() {
                            if (self.in_struct_def && !self.in_func) || self.in_defer {
                                // TODO: make this work with multi dimensional arrays
                                let mac = self.handle_array_macro(&array_lens[0], value[i..].to_vec());
                                self.expr_stack.push(mac);
                                return Expr::None;
                            }
                            return self.handle_array_macro(&array_lens[0], value[i..].to_vec());
                        }

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
                                } else if let Keyword::Defer = k {
                                    defering = true;
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

                                if (self.in_struct_def && !self.in_func) || self.in_defer {
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
                                            let mut keyword = Keyword::Generic(name);

                                            if i + 1 == value.len() {
                                                self.comp_err(&format!("expected identifier after keyword {keyword:?}, got nothing"));
                                                exit(1);
                                            }

                                            if pointer_counter > 0 {
                                                (keyword, _) = self.create_keyword_pointer(self.keyword_to_type(keyword), pointer_counter);
                                            }

                                            if self.in_struct_def && !self.in_func {
                                                // i should've commented this earlier, don't
                                                // remember why this is needed lmao
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
                                        generics: None, // watch this, changed from Some(vec![])
                                    };

                                    if pointer_counter > 0 {
                                        (k, _) = self.create_keyword_pointer(self.keyword_to_type(k), pointer_counter);
                                    }

                                    if self.in_struct_def && !self.in_func {
                                        let expr = self.create_define_var(k, value[i+1].clone(), vec![]);
                                        self.expr_stack.push(expr);
                                        return Expr::None;
                                    }

                                    return self.create_define_var(k, value[i+1].clone(), vec![]);
                                } else if let Expr::MacroStructDef { .. } = found_ident {
                                    let mut k = Keyword::TypeDef {
                                        type_name: ident.clone(), 
                                        generics: Some(vec![])
                                    };

                                    if pointer_counter > 0 {
                                        (k, _) = self.create_keyword_pointer(self.keyword_to_type(k), pointer_counter);
                                    }

                                    let slice = value[i+1..value.len()-1].to_vec();
                                    let last = value.last().unwrap().clone();

                                    if self.in_struct_def && !self.in_func {
                                        let expr = self.create_define_var(k, last, slice);
                                        self.expr_stack.push(expr);
                                        return Expr::None;
                                    }

                                    return self.create_define_var(k, last, slice)
                                } else if let Expr::Func { .. } | Expr::MacroFunc { .. } = found_ident {
                                    if self.in_defer || defering {
                                        if defering {
                                            self.one_defer = true;
                                            self.defer_scope = self.current_scope;
                                        }
                                        let expr = self.create_func_call(&found_ident, value[i+1..].to_vec());
                                        self.expr_stack.push(expr);
                                        return Expr::None;
                                    }
                                    return self.create_func_call(&found_ident, value[i+1..].to_vec());
                                }  else if let Expr::EnumDef { .. } = found_ident {
                                    let mut k = Keyword::TypeDef {
                                        type_name: ident.clone(), 
                                        generics: None,
                                    };

                                    if pointer_counter > 0 {
                                        (k, _) = self.create_keyword_pointer(self.keyword_to_type(k), pointer_counter);
                                    }

                                    if self.in_struct_def && !self.in_func {
                                        let expr = self.create_define_var(k, value[i+1].clone(), vec![]);
                                        self.expr_stack.push(expr);
                                        return Expr::None;
                                    }

                                    return self.create_define_var(k, value[i+1].clone(), vec![]);
                                } else {
                                    // if in struct it reference it's own name
                                    if self.in_struct_def && ident == &self.current_func {
                                        let mut k = Keyword::TypeDef {
                                            type_name: format!("struct {}", self.current_func), 
                                            generics: Some(vec![])
                                        };
                                        if pointer_counter > 0 {
                                            k = self.create_keyword_pointer(self.keyword_to_type(k), pointer_counter).0;
                                        }

                                        let expr = self.create_define_var(k, value[i+1].clone(), vec![]);
                                        self.expr_stack.push(expr);
                                        return Expr::None
                                    } else if self.in_enum_def {
                                        let new_value = match &value[i] {
                                            Token::Ident(prev_name) => {
                                                Token::Ident(format!("{}.{prev_name}", self.current_func))
                                            },
                                            _ => unreachable!()
                                        };
                                        let expr = self.create_define_var(Keyword::None, new_value, vec![]);
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
                    let ident_num = ident.parse::<f64>();
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
                        Expr::VariableName { ref typ, ref name, reassign, constant: _, field_data } => {
                            buffer.push(Expr::VariableName { typ: typ.clone(), name: name.to_owned(), reassign, constant: is_constant, field_data })
                        },
                        Expr::None => {
                            let typ_kw_res = self.keyword_map.get(ident);
                            match typ_kw_res {
                                Some(typ_kw) => {
                                    let typ = self.keyword_to_type(typ_kw.clone());
                                    match typ {
                                        Types::None => {
                                            self.comp_err(&format!("unknown identifier: {}", ident));
                                            exit(1);
                                        },
                                        _ => {
                                            buffer.push(Expr::IntLit(format!("{ident}")));
                                        },
                                    }
                                },
                                None => {
                                    self.comp_err(&format!("unknown identifier: {}", ident));
                                    exit(1);
                                }
                            }
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
                Token::SingleQuote => (),
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

                            if self.in_struct_def || self.in_defer {
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

            self.comp_err(&format!("expected a token, got none"));
            exit(1);
        }

        if returning && buffer.len() <= 1 {
            if self.in_struct_def && !self.in_func {
                self.comp_err(&format!("can't use return inside struct"));
                exit(1);
            }

            return Expr::Return(Box::new(buffer[0].clone()))
        }
        
        if buffer.len() > 1 {
            match &buffer[0] {
                Expr::Address(varname) => {
                    if let Expr::VariableName { typ, name, constant, .. } = *varname.clone() {
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
                                constant,
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
                Expr::VariableName { typ, name, constant, field_data, .. } => {
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
                            constant: constant.to_owned(),
                            field_data: (field_data.0, field_data.1),
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

    fn propagate_struct_fields(&mut self, fname: String, user_def: String, is_ptr: bool, is_constant: bool) {
        match self.find_ident(user_def) {
            Expr::StructDef { struct_fields, .. } | Expr::MacroStructDef { struct_fields, .. } => {
                for field in struct_fields {
                    match field {
                        Expr::VariableName { typ, name, field_data, .. } => {
                            let new_name = if is_ptr {
                                format!("{fname}->{name}")
                            } else {
                                format!("{fname}.{name}")
                            };
                            if let Types::TypeDef { ref type_name, .. } = typ {
                                self.propagate_struct_fields(new_name.clone(), type_name.to_string(), field_data.1, is_constant);
                            } else if let Types::Arr { .. } = typ {
                                self.propagate_struct_fields(new_name.clone(), String::from("array"), field_data.1, is_constant);
                            }

                            let new_expr = Expr::Variable {
                                info: Box::new(Expr::VariableName {
                                    typ,
                                    name: new_name,
                                    reassign: false,
                                    constant: is_constant, // CAN BREAK, testing rn
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
            // Enums will be checked for propagation but they've already been handling
            Expr::EnumDef { .. } => (),
            unexpected => {
                self.comp_err(&format!("unexpected expression {unexpected:?} during field propagation"));
                exit(1);
            },
        }
    }

    fn create_define_var(&mut self, kw: Keyword, ident: Token, generics: Vec<Token>) -> Expr {
        let expr: Expr;
        let fname: String;
        let mut pass_typs: Vec<Types> = Vec::new();

        for gen in generics {
            match gen {
                Token::Lsquare => (),
                Token::Rsquare => (),
                Token::Int(typs) => {
                    pass_typs = self.parse_generics_to_pass(&typs, false);
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
                let ident_num = word.parse::<f64>();
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
                            *generics = Some(pass_typs);
                        }
                        expr = if self.in_enum_def {
                            Expr::VariableName { typ, name: word, reassign: false, constant: true, field_data: (false, false) }
                        } else {
                            Expr::VariableName { typ, name: word, reassign: false, constant: false, field_data: (false, false) }
                        };
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
            Keyword::Char | Keyword::I8 | Keyword::U8 | Keyword::U16 | Keyword::I16 | Keyword::U32 | Keyword::I32 |
            Keyword::F32 | Keyword::F64 | Keyword::Usize | Keyword::Bool | Keyword::UInt | Keyword::Int | Keyword::I64 | Keyword::U64 => (),
            Keyword::None => (),
            Keyword::Generic(_) => (),
            Keyword::Pointer(.., last) => {
                if self.in_struct_def && !self.in_func {
                    return expr;
                }
                if let Types::TypeDef { type_name: user_def, .. } = last {
                    self.propagate_struct_fields(fname, user_def.to_string(), true, false);
                }
            },
            Keyword::TypeDef { type_name: ref user_def, .. } => {
                if self.in_struct_def && !self.in_func {
                    return expr;
                }
                self.propagate_struct_fields(fname, user_def.to_string(), false, false);
            },
            _ => {
                self.comp_err(&format!("unexpected keyword: {kw:?}"));
                exit(1);
            },
        }

        expr
    }

    fn handle_type_mask(&mut self, type_name: String, masked_type: Expr) {
        match masked_type {
            Expr::StructDef { struct_fields, struct_name } => {
                let is_shared = match *struct_name {
                    Expr::StructName { is_shared, .. } => is_shared,
                    _ => false,
                };
                let expr = Expr::StructDef {
                    struct_name: Box::new(Expr::StructName { name: type_name, is_shared }),
                    struct_fields
                };
                self.structures.push(expr);
            },
            Expr::MacroStructDef { struct_name, struct_fields } => {
                let (generics, is_shared) = if let Expr::MacroStructName { generics, is_shared, .. } = *struct_name {
                    (generics, is_shared) 
                } else {(vec![], false)};
                let expr = Expr::MacroStructDef {
                    struct_name: Box::new(Expr::MacroStructName {
                        name: type_name,
                        generics,
                        is_shared
                    }),
                    struct_fields
                };
                self.structures.push(expr);
            },
            Expr::EnumDef { .. } => {
                self.comp_err(&format!("masking an enum is not supported, tried to mask `{masked_type:?}`"));
                exit(1);
            },
            _ => (),
        }
    }

    fn create_variable(&mut self, left: Vec<Token>, right: Vec<Token>, is_constant: bool) {
        let left_expr = self.handle_left_assign(left, is_constant);
        let right_expr = self.handle_right_assign(right, true);

        match left_expr {
            Expr::VariableName { ref typ, ref name, .. } => {
                if let Types::TypeId = typ {
                    if !is_constant {
                        self.comp_err(&format!("type masking must be constant. did you mean `typeid {name} :: <value>`?"));
                        exit(1);
                    }

                    if let Expr::StructDef { .. } = right_expr {
                        self.handle_type_mask(name.to_owned(), right_expr.clone());
                    } else if let Expr::IntLit(_) = right_expr {
                        self.structures.push(Expr::StructDef {
                            struct_name: Box::new(Expr::StructName { name: name.to_owned(), is_shared: false }),
                            struct_fields: vec![]
                        });
                    }

                    let new_name = name.replace(".", "__");

                    let expr = Expr::Variable {
                        info: Box::new(Expr::VariableName {
                            typ: typ.clone(),
                            name: new_name,
                            reassign: false,
                            constant: true,
                            field_data: (false, false)
                        }),
                        value: Box::new(right_expr)
                    };

                    self.token_stack.clear();

                    if self.in_defer {
                        self.expr_stack.push(expr.clone());
                    } else {
                        self.program_push(expr.clone());
                    }
                    return;
                }
            },
            Expr::DerefPointer(ref varname) => {
                match **varname {
                    Expr::VariableName { ref typ, ref name, .. } => {
                        if let Types::TypeId = typ {
                            if !is_constant {
                                self.comp_err(&format!("type masking must be constant. did you mean `typeid {name} :: <value>`?"));
                                exit(1);
                            }

                            if let Expr::StructDef { .. } = right_expr {
                                self.handle_type_mask(name.to_owned(), right_expr.clone());
                            } else if let Expr::IntLit(_) = right_expr {
                                self.structures.push(Expr::StructDef {
                                    struct_name: Box::new(Expr::StructName { name: name.to_owned(), is_shared: false }),
                                    struct_fields: vec![]
                                });
                            }

                            let new_name = name.replace(".", "__");

                            let expr = Expr::Variable {
                                info: Box::new(Expr::VariableName {
                                    typ: typ.clone(),
                                    name: new_name,
                                    reassign: false,
                                    constant: true,
                                    field_data: (false, false)
                                }),
                                value: Box::new(right_expr)
                            };

                            self.token_stack.clear();

                            if self.in_defer {
                                self.expr_stack.push(expr.clone());
                            } else {
                                self.program_push(expr.clone());
                            }
                            return;
                        }
                    },
                    _ => unreachable!()
                }
            },
            ref unexpected => {
                self.comp_err(&format!("unexpected {unexpected:?} on left side of variable"));
                exit(1);
            },
        }

        let expr = Expr::Variable { info: Box::new(left_expr.clone()), value: Box::new(right_expr.clone()) };
        if !self.in_func && !self.in_enum_def {
            self.global_vars.push(expr.clone());
        } else if self.in_enum_def {
            if let Expr::IntLit(_) = right_expr {
                self.expr_stack.push(expr);
                self.token_stack.clear();
                return;
            } else {
                self.comp_err(&format!("enum field must be an integer, found {right_expr:?}"));
                exit(1);
            }
        } else if let Some(vars) = self.func_to_vars.get_mut(&self.current_func) {
            vars[self.current_scope].push(expr.clone());
        }

        self.token_stack.clear();
        if self.in_defer {
            self.expr_stack.push(expr);
        } else {
            self.program_push(expr);
        }
    }

    fn handle_semicolon(&mut self) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut seen_colon = 0;
        let mut is_constant = false;

        for (_i, token) in self.token_stack.iter().enumerate() {
            match token {
                Token::Colon => {
                    seen_colon += 1;
                    if seen_colon == 2 {
                        is_constant = true;
                    }
                },
                _ => {
                    if seen_colon == 0 {
                        left.push(token.clone());
                    } else if seen_colon == 1 || seen_colon == 2 {
                        right.push(token.clone());
                    } else {
                        self.comp_err("unexpected assignment operator `:`");
                        exit(1);
                    }
                },
            }
        }

        if !right.is_empty() {
            if self.in_struct_def && !self.in_func {
                self.comp_err("can't initalise members inside a struct");
                exit(1);
            }
            self.create_variable(left, right, is_constant);
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

        let mut defer_rc = 0;
        let mut defer_paste_next_time = false;
        let mut stored_defers: Vec<DeferInfo> = Vec::new();

        while self.current_token < self.tokens.len() {
            if self.one_defer {
                self.one_defer = false;

                let defer_info = DeferInfo {
                    scope: self.defer_scope,
                    exprs: self.expr_stack.clone(),
                };

                self.expr_stack.clear();
                stored_defers.push(defer_info);
                defer_paste_next_time = true;
            }

            match self.tokens[self.current_token] {
                Token::Lcurl => {
                    curl_rc += 1;

                    if self.in_defer {
                        defer_rc += 1;
                    }
                    self.handle_lcurl();
                },
                Token::Rcurl => {
                    self.error_if_token_stack_not_empty();

                    curl_rc -= 1;
                    self.prev_scope();

                    if defer_paste_next_time && !self.in_defer {
                        // need to specifically check because you can be in scope 0 while inside
                        // and outside a function

                        'pasting: loop {
                            let defer_info = match stored_defers.pop() {
                                Some(info) => {
                                    info
                                },
                                None => {
                                    self.comp_err("failed to paste defered expression.");
                                    exit(1);
                                }
                            };

                            let mut repeat_checking = false;
                            if defer_info.scope == 0 && defer_info.scope == self.current_scope && !self.in_func {
                                self.create_defer(defer_info.exprs);
                                repeat_checking = true;
                            } else if defer_info.scope > 0 && defer_info.scope == self.current_scope+1 {
                                self.create_defer(defer_info.exprs);
                                repeat_checking = true;
                            } else {
                                stored_defers.push(defer_info);
                            }

                            if stored_defers.is_empty() {
                                defer_paste_next_time = false;
                                repeat_checking = false;
                            }

                            if !repeat_checking {
                                break 'pasting;
                            }
                        }
                    }

                    if self.in_struct_def && curl_rc == 0 {
                        if self.previous_func.is_empty() {
                            self.create_struct();
                        } else {
                            self.previous_func.clear();
                            self.in_struct_def = false;
                        }
                    } else if self.in_defer {
                        let mut include_rcurl = false;
                        if defer_rc != 0 {
                            defer_rc -= 1;
                            include_rcurl = true;
                        }

                        if self.defer_scope == self.current_scope {
                            self.in_defer = false;

                            if curl_rc == 0 {
                                self.create_defer(self.expr_stack.clone());
                                self.expr_stack.clear();
                                self.program_push(Expr::EndBlock);
                            } else if defer_rc == 0 {
                                let defer_info = DeferInfo {
                                    scope: self.defer_scope,
                                    exprs: self.expr_stack.clone(),
                                };
                                self.expr_stack.clear();
                                stored_defers.push(defer_info);
                                defer_paste_next_time = true;
                            }
                        } else if include_rcurl {
                            self.expr_stack.push(Expr::EndBlock);
                        }
                    } else if self.in_enum_def {
                        self.create_enum();
                    } else if !self.in_defer {
                        self.program_push(Expr::EndBlock);
                    }
                },
                Token::SemiColon => {
                    self.handle_semicolon();
                },
                Token::Newline => {
                    self.error_if_token_stack_not_empty();
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
