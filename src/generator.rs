use std::{collections::HashMap, fs, process::{exit, Command}};
use crate::declare_types::*;
use crate::parser::*;
use rand::Rng;

pub struct Gen {
    imports: String,
    comp_imports:  String,

    indent: i32,
    code: String,

    in_file: String,
    line_num: u32,
    out_file: String,
    compile: bool,
    keep_gen: bool,
    lang: Lang,

    libc_map: HashMap<String, bool>,
    defs_location: Vec<usize>,

    generated_structs: Vec<String>,

    in_macro_func: bool,
    curl_rc: i32,
}

fn rand_varname() -> String {
    let alphabet = String::from("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let mut varname = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..6 {
        let r = rng.gen_range(0..alphabet.len());
        varname.push(alphabet.chars().nth(r).unwrap());
    }

    varname
}

impl Gen {
    pub fn new(in_file: String, out_file: String, compile: bool, keep_gen: bool, lang: Lang) -> Gen {
        let libc_map = HashMap::from([
            ("stdio".to_string(), true),
            ("stdlib".to_string(), true),
            ("stdbool".to_string(), true),
            ("stddef".to_string(), true),
            ("stdint".to_string(), true),
            ("string".to_string(), true),
            ("math".to_string(), true),
            ("stdarg".to_string(), true),
            ("assert".to_string(), true),
            ("errno".to_string(), true),
            ("ctype".to_string(), true),
            ("time".to_string(), true),
            ("unistd".to_string(), true),
        ]);

        return Gen {
            imports: String::new(),
            comp_imports: String::new(),
            code: String::new(),
            in_file,
            out_file,
            compile,
            keep_gen,
            lang,
            line_num: 0,
            libc_map,
            defs_location: Vec::new(),
            indent: 0,
            generated_structs: Vec::new(),
            in_macro_func: false,
            curl_rc: 0,
        }
    }

    fn comp_err(&self, error_msg: &str) {
        println!("\x1b[91merror\x1b[0m: {}:{}", self.in_file, self.line_num);
        println!("\x1b[91merror\x1b[0m: {error_msg}");
    }

    fn add_spaces(&mut self, indent: i32) {
        let spaces = indent * 4;
        if spaces > 0 {
            for _ in 0..spaces {
                self.code.push(' ');
            }
        }
    }

    fn update_struct_definitions(&mut self, mut line_num: usize, offset: usize) {
        for i in self.defs_location.iter_mut() {
            if i > &mut line_num {
                *i += offset;
            }
        }
    }

    fn generate_new_struct(&mut self, fullname: String, og_name: &String) {
        for gen_struct in &self.generated_structs {
            if gen_struct == &format!("{og_name}_{fullname}") {
                return;
            }
        }

        let parts: Vec<&str> = fullname.split("_").collect();

        let mut gen_code = format!("{og_name}(");
        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                gen_code.push_str(part);
            } else {
                gen_code.push_str(&format!(", {part}"));
            }
        }
        gen_code.push_str(");\n");

        let index_op = self.defs_location.last();
        let index = match index_op {
            Some(i) => *i,
            None => {
                self.comp_err("failed to generate generic during compilation");
                exit(1);
            }
        };

        self.code.insert_str(index, &gen_code);
        self.update_struct_definitions(index, gen_code.len());
        self.generated_structs.push(format!("{og_name}_{fullname}"));
    }

    fn handle_typ(&mut self, typ: Types) -> (String, String) {
        match typ {
            Types::U8 => (String::from("u8"), String::new()),
            Types::I8 => (String::from("i8"), String::new()),
            Types::Char => (String::from("char"), String::new()),
            Types::U16 => (String::from("u16"), String::new()),
            Types::I16 => (String::from("i16"), String::new()),
            Types::U32 => (String::from("u32"), String::new()),
            Types::I32 => (String::from("i32"), String::new()),
            Types::U64 => (String::from("u64"), String::new()),
            Types::I64 => (String::from("i64"), String::new()),
            Types::Int => (String::from("int"), String::new()),
            Types::UInt => (String::from("uint"), String::new()),
            Types::F32 => (String::from("f32"), String::new()),
            Types::F64 => (String::from("f64"), String::new()),
            Types::Usize => {
                if !self.imports.contains("#include <stddef.h>\n") {
                    self.imports.push_str("#include <stddef.h>\n");
                }
                (String::from("size_t"), String::new())
            },
            Types::Bool => {
                if !self.imports.contains("#include <stdbool.h>\n") {
                    self.imports.push_str("#include <stdbool.h>\n");
                }
                (String::from("bool"), String::new())
            },
            Types::TypeDef { type_name: user_def, generics: generics_op } => {
                let replaced_def = user_def.replace(".", "__");
                let mut typ = format!("{replaced_def}");

                match generics_op {
                    Some(generics) => {
                        for (i, generic) in generics.iter().enumerate() {
                            if self.in_macro_func {
                                if i == 0 {
                                    typ.push_str(&format!("_##{generic}"));
                                } else {
                                    typ.push_str(&format!("##{generic}"));
                                }
                            } else {
                                if i == 0 {
                                    typ.push_str(&format!("_{generic}"));
                                } else {
                                    typ.push_str(&format!("{generic}"));
                                }
                            }
                        }
                    },
                    None => (),
                }
                return (typ, String::new())
            },
            Types::Void => return (String::from("void"), String::new()),
            Types::Arr { typ: arr_typ, length } => {
                let newstr_typ = self.handle_typ(*arr_typ);
                // MIGHT NOT WORK WITH MULTI-DIMENSIONAL ARRAYS
                return (format!("{}", newstr_typ.0), format!("[{length}]"))
            },
            Types::ArrIndex { arr_typ: _, index_at } => {
                return (String::new(), index_at)
            },
            Types::Pointer(ptotyp) => {
                let sub_typ = self.handle_typ(*ptotyp).0;
                return (format!("{sub_typ}*"), String::new())
            },
            Types::Generic(typeid) => {
                if !self.in_macro_func {
                    self.comp_err("failed to handle generic at compile time.");
                    exit(1);
                }

                (format!("{typeid}"), String::new())
            },
            Types::None => (String::new(), String::new()),
            unimpl => {
                self.comp_err(&format!("{unimpl:?} is not implemented yet"));
                exit(1);
            }
        }
    }

    fn handle_varname(&mut self, varname: Expr) -> String {
        match varname {
            Expr::VariableName { ref typ, reassign, constant, .. } => {
                let mut vardec = if constant && !reassign {
                    String::from("const ")
                } else {
                    String::new()
                };

                let new_name = self.handle_sanitise_varname(varname.clone());
                if let Types::ArrIndex { index_at, .. } = typ {
                    vardec.push_str(&format!("{new_name}[{index_at}]"));
                    return vardec
                } else if let Types::TypeDef { type_name, generics: generics_op } = typ {
                    match generics_op {
                        Some(generics) => {
                            if !generics.is_empty() {
                                let mut fullname = String::new();
                                for (i, generic) in generics.iter().enumerate() {
                                    if i == 0 {
                                        fullname.push_str(&format!("{generic}"));
                                    } else {
                                        fullname.push_str(&format!("_{generic}"));
                                    }
                                }

                                if !self.in_macro_func {
                                    self.generate_new_struct(fullname, type_name);
                                }
                            }
                        },
                        None => (),
                    }
                }

                if reassign == false {
                    let str_typ = self.handle_typ(typ.clone());
                    if !str_typ.1.is_empty() {
                        if let Types::Arr { .. } = typ {
                            self.generate_new_struct(str_typ.0.clone(), &String::from("array"));
                            vardec.push_str(&format!("array_{} {new_name} = {{.data = ({}{})A", str_typ.0, str_typ.0, str_typ.1));
                            return vardec
                        }
                    }

                    vardec.push_str(&format!("{} {new_name}{}", str_typ.0, str_typ.1));
                    return vardec
                } else {
                    vardec.push_str(&format!("{new_name}"));
                    return vardec
                }
            },
            Expr::DerefPointer(value) => {
                let derefed = self.handle_varname(*value);
                return format!("*{derefed}")
            },
            Expr::CEmbed(mut value) => {
                if value.chars().last().unwrap() == ';' {
                    value.pop();
                }
                return value;
            }
            unexpected => {
                self.comp_err(&format!("unexpected expression: {unexpected:?}"));
                exit(1);
            },
        }
    }

    pub fn handle_funccall(&mut self, funccall: Expr) -> String {
        fn handle_local_pointer(this: &mut Gen, name: String, typ: Types) -> (String, bool) {
            if let Types::Pointer(subtyp) = typ {
                let handled = handle_local_pointer(this, name, *subtyp);
                return (format!("{}*", handled.0), handled.1);
            } else if let Types::TypeId = typ {
                return (name, true);
            } else if let Types::TypeDef { ref type_name, .. } = typ {
                if type_name == &name {
                    return (name, true);
                } else {
                    return (name, false);
                }
            } else {
                return (name, false);
            }
        }

        match funccall {
            Expr::FuncCall { name, gave_params } => {
                let mut funccall_code = String::new();
                let mut add_newline = false;

                if name == String::from("print")  {
                    funccall_code.push_str("printf(");
                } else if name == String::from("println") {
                    funccall_code.push_str("printf(");
                    add_newline = true;
                } else if name == String::from("string__format") {
                    funccall_code.push_str("__IMPULSE__STRING__FORMAT__(");
                } else {
                    funccall_code.push_str(&format!("{name}("));
                }

                if gave_params.is_empty() {
                    funccall_code.push(')');
                    return funccall_code
                } 

                for (i, param) in gave_params.iter().enumerate() {
                    match param {
                        Expr::IntLit(intlit) => {
                            if i == 0 {
                                funccall_code.push_str(&intlit);
                            } else {
                                funccall_code.push_str(&format!(", {intlit}"))
                            }
                        },
                        Expr::VariableName { typ, .. } => {
                            let mut sanitised_name = self.handle_sanitise_varname(param.clone());
                            if let Types::Pointer(_) = typ {
                                let handled_varname = handle_local_pointer(self, sanitised_name.clone(), typ.clone());
                                sanitised_name = if handled_varname.1 {
                                    handled_varname.0
                                } else {
                                    sanitised_name
                                };
                            }
                            if i == 0 {
                                funccall_code.push_str(&sanitised_name)
                            } else {
                                funccall_code.push_str(&format!(", {}", sanitised_name))
                            }
                        },
                        Expr::FuncCall { .. } => {
                            if i == 0 {
                                funccall_code.push_str(&self.handle_funccall(param.clone()))
                            } else {
                                funccall_code.push_str(&format!(", {}", self.handle_funccall(param.clone())))
                            }
                        },
                        Expr::Address(_) => {
                            if i == 0 {
                                funccall_code.push_str(&self.handle_value(param.clone()));
                            } else {
                                funccall_code.push_str(&format!(", {}", self.handle_value(param.clone())));
                            }
                        },
                        Expr::StrLit { .. } => {
                            let mut string = self.handle_value(param.clone());
                            if i == 0 {
                                if add_newline {
                                    string.insert_str(string.len()-1, "\\n");
                                    funccall_code.push_str(&string);
                                } else {
                                    funccall_code.push_str(&string);
                                }
                            } else {
                                funccall_code.push_str(&format!(", {}", string));
                            }
                        },
                        Expr::CharLit(_) => {
                            if i == 0 {
                                funccall_code.push_str(&self.handle_value(param.clone()));
                            } else {
                                funccall_code.push_str(&format!(", {}", self.handle_value(param.clone())));
                            }
                        },
                        Expr::True => {
                            if i == 0 {
                                funccall_code.push_str("true");
                            } else {
                                funccall_code.push_str(", true");
                            }
                        },
                        Expr::False => {
                            if i == 0 {
                                funccall_code.push_str("false");
                            } else {
                                funccall_code.push_str(", false");
                            }
                        },
                        Expr::StructDef { struct_name, .. } => {
                            if let Expr::StructName(name) = *struct_name.clone() {
                                if i == 0 {
                                    funccall_code.push_str(&format!("{name}"));
                                } else {
                                    funccall_code.push_str(&format!(", {name}"));
                                }
                            }
                        },
                        Expr::EnumDef { enum_name, .. } => {
                            if let Expr::EnumName(name) = *enum_name.clone() {
                                if i == 0 {
                                    funccall_code.push_str(&format!("{name}"));
                                } else {
                                    funccall_code.push_str(&format!(", {name}"));
                                }
                            }
                        },
                        unimpl => {
                            self.comp_err(&format!("expression {unimpl:?} not implemented yet"));
                            exit(1);
                        }
                    }
                }

                funccall_code.push(')');
                return funccall_code
            },
            unexpected => {
                self.comp_err(&format!("unexpected expression: {unexpected:?}"));
                exit(1);
            },
        }

    }

    fn handle_arraylit(&mut self, arrlit: Vec<Expr>, length: String) -> String {
        let mut arrlit_code = String::new();
        arrlit_code.push_str("{");

        for (i, elem) in arrlit.iter().enumerate() {
            let literal = self.handle_value(elem.clone());

            if i == 0 {
                arrlit_code.push_str(&literal);
            } else {
                arrlit_code.push_str(&format!(", {literal}"));
            }
        }

        if length.is_empty() {
            arrlit_code.push_str(&format!("}}, .len = {}}}", arrlit.len()));
        } else {
            arrlit_code.push_str(&format!("}}, .len = {}}}", length));
        }
        arrlit_code
    }

    fn handle_sanitise_varname(&mut self, value: Expr) -> String {
        match value {
            Expr::VariableName { name, field_data, .. } => {
                // we do this cuz it should be a enum at this point
                let mut sanitised = name.clone();
                if !field_data.0 && name.contains(".") {
                    sanitised = name.replace(".", "_")
                }

                return sanitised 
            },
            unexpected => {
                self.comp_err(&format!("can't deference {unexpected:?}"));
                exit(1);
            }
        }
    }

    fn handle_value(&mut self, value: Expr) -> String {
        match value {
            Expr::IntLit(intlit) => intlit,
            Expr::CharLit(charlit) => format!("'{charlit}'"),
            Expr::StrLit { content, .. } => format!("\"{content}\""),
            Expr::True => String::from("true"),
            Expr::False => String::from("false"),
            Expr::VariableName { ref typ, .. } => {
                let new_name = self.handle_sanitise_varname(value.clone());
                if let Types::ArrIndex { index_at, .. } = typ {
                    return format!("{new_name}[{index_at}]")
                } else {
                    return new_name
                }
            },
            Expr::FuncCall { .. } => return self.handle_funccall(value.clone()),
            Expr::Address(atoval) =>{
                let sub_val = self.handle_value(*atoval);
                return format!("&{sub_val}")
            },
            Expr::DerefPointer(dptoval) => {
                let sub_val = self.handle_value(*dptoval);
                return format!("*{sub_val}")
            },
            Expr::None => String::new(),
            Expr::ArrayLit(arraylit) => {
                let mut values = String::from("{");
                for (i, elem) in arraylit.iter().enumerate() {
                    let val = self.handle_value(elem.to_owned());
                    if i == 0 {
                        values.push_str(&val);
                        continue;
                    }

                    values.push_str(&format!(", {val}"));
                }

                values.push('}');
                return values
            },
            Expr::StructDef { struct_name, .. } => {
                match *struct_name {
                    Expr::StructName(structname) => {
                        return structname.replace(".", "__")
                    },
                    _ => unreachable!(),
                }
            },
            unimpl => {
                self.comp_err(&format!("expression {unimpl:?} not implemented yet"));
                exit(1);
            }
        }

    }

    fn handle_boolean_condition(&mut self, conditions: &Vec<Expr>) -> String {
        let mut boolean_condition_code = String::new();
        let mut had_angled = false;
        for condition in conditions {
            match condition {
                Expr::FuncCall { .. } => {
                    let func_call_code = self.handle_funccall(condition.clone());
                    boolean_condition_code.push_str(&format!("{func_call_code}"));
                },
                Expr::VariableName { typ, .. } => {
                    let new_name = self.handle_sanitise_varname(condition.clone());

                    if let Types::ArrIndex { index_at, .. } = typ {
                        boolean_condition_code.push_str(&format!("{new_name}[{index_at}]"));
                    } else {
                        boolean_condition_code.push_str(&format!("{new_name}"))
                    }
                },
                Expr::Equal => {
                    if had_angled {
                        boolean_condition_code.push_str("=");
                        had_angled = false;
                    } else {
                        boolean_condition_code.push_str("==");
                    }
                },
                Expr::SmallerThan => {
                    boolean_condition_code.push_str("<");
                    had_angled = true;
                },
                Expr::BiggerThan => {
                    boolean_condition_code.push_str(">");
                    had_angled = true;
                },
                Expr::Exclaim => {
                    boolean_condition_code.push_str("!");
                    had_angled = true;
                },
                Expr::IntLit(intlit) => {
                    boolean_condition_code.push_str(&format!("{intlit}"))
                },
                Expr::CharLit(charlit) => {
                    boolean_condition_code.push_str(&format!("'{charlit}'"));
                },
                // Expr::StrLit(string) => boolean_condition_code.push_str(&format!("{string}")),
                Expr::Or => boolean_condition_code.push_str("||"),
                Expr::And => boolean_condition_code.push_str("&&"),
                Expr::True => {
                    if !self.imports.contains("#include <stdbool.h>\n") {
                        self.imports.push_str("#include <stdbool.h>\n");
                    }
                    boolean_condition_code.push_str("true")
                },
                Expr::False => {
                    if !self.imports.contains("#include <stdbool.h>\n") {
                        self.imports.push_str("#include <stdbool.h>\n");
                    }
                    boolean_condition_code.push_str("false")
                },
                unexpected => {
                    self.comp_err(&format!("unexpected expression {unexpected:?} in branch / boolean condition"));
                    exit(1);
                },
            }
        }

        boolean_condition_code
    }

    fn handle_branch(&mut self, branch_typ: &String, conditions: Vec<Expr>) -> String {
        let mut branch_code = String::new();
        branch_code.push_str(&format!("{branch_typ}("));

        let condition_str = self.handle_boolean_condition(&conditions);
        branch_code.push_str(&condition_str);

        self.curl_rc += 1;
        if self.in_macro_func {
            branch_code.push_str(") {\\\n");
        } else {
            branch_code.push_str(") {\n");
        }
        branch_code
    }

    fn handle_loop(&mut self, conditions: Vec<Expr>, modifier: Expr) -> String {
        let mut loop_code = String::new();
        let mut varname = String::new();
        loop_code.push_str("for (");

        match &conditions[0] {
            Expr::VariableName { typ, name, reassign, .. } => {
                varname = name.to_owned();
                if reassign == &true {
                    loop_code.push_str(&format!("{} {name} = 0; ", self.handle_typ(typ.clone()).0));
                } else {
                    loop_code.push_str(";");
                }
            },
            Expr::Exclaim | Expr::True | Expr::False => loop_code.push_str(";"),
            _ => (),
        }

        let condition_str = self.handle_boolean_condition(&conditions);
        loop_code.push_str(&format!("{condition_str};"));

        match modifier {
            Expr::IntLit(modif) => {
                match modif.as_str() {
                    "+" => loop_code.push_str(&format!(" {varname}++")),
                    "-" => loop_code.push_str(&format!(" {varname}--")),
                    "_" => (),
                    _ => (),
                }
            }
            _ => (),
        }

        self.curl_rc += 1;
        loop_code.push_str(") {\n");
        loop_code
    }

    fn handle_for(&mut self,  in_this: Box<Expr>, iterator: String) -> (String, String, String) {
        let mut for_code = String::new();
        let arr_name;
        let length: String = match *in_this {
            Expr::VariableName { typ, name, .. } => {
                if let Types::Arr { .. } = typ {
                    arr_name = name.clone();
                    format!("{name}.len")
                } else if let Types::TypeDef { type_name, .. } = typ {
                    if type_name == String::from("dyn") || type_name == String::from("string") || type_name == String::from("str") || type_name == String::from("array")  {
                        arr_name = name.clone();
                        format!("{name}.len")
                    } else {
                        self.comp_err(&format!("unable to handle {name} in for loop as it's of type {type_name:?}"));
                        exit(1);
                    }
                } else {
                    self.comp_err(&format!("unable to handle {name} in for loop as it's of type {typ:?}"));
                    exit(1);
                }
            },
            unexpected => {
                self.comp_err(&format!("{unexpected:?} is not implemented yet"));
                exit(1);
            }
        };

        if iterator.is_empty() {
            let randname = rand_varname();
            for_code.push_str(&format!("for (size_t {randname} = 0; {randname} < {length}; {randname}++) {{\n"));
            (for_code, arr_name, randname)
        } else {
            for_code.push_str(&format!("for (size_t {iterator} = 0; {iterator} < {length}; {iterator}++) {{\n"));
            (for_code, arr_name, iterator)
        }
    }

    fn generate_c(&mut self, expressions: Vec<(Expr, String, u32)>) {
        let mut struct_generics = Vec::new();
        self.imports.push_str("#include <stddef.h>\n");
        self.imports.push_str("#include <stdint.h>\n");
        self.imports.push_str("#include <stdbool.h>\n");
        self.code.push_str("typedef uint8_t u8;\n");
        self.code.push_str("typedef int8_t i8;\n");
        self.code.push_str("typedef uint16_t u16;\n");
        self.code.push_str("typedef int16_t i16;\n");
        self.code.push_str("typedef uint32_t u32;\n");
        self.code.push_str("typedef int32_t i32;\n");
        self.code.push_str("typedef uint64_t u64;\n");
        self.code.push_str("typedef int64_t i64;\n");
        self.code.push_str("typedef float f32;\n");
        self.code.push_str("typedef double f64;\n");
        self.code.push_str("typedef size_t usize;\n");
        self.code.push_str("typedef unsigned int uint;\n");
        self.code.push_str("#define $inline static inline __attribute__((always_inline))\n");

        let mut first_case = false;
        let mut fall_case = false;

        for (_index, info) in expressions.into_iter().enumerate() {
            let expr = info.0;
            self.in_file = info.1;
            self.line_num = info.2;

            match expr {
                Expr::Import(loc) => {
                    if !self.imports.contains(&format!("#include <{loc}.h>\n")) && !self.imports.contains(&format!("#include \"{loc}.h\"\n")) {
                        let libc_res = self.libc_map.get(&loc);
                        match libc_res {
                            Some(_) => {
                                self.imports.push_str(&format!("#include <{loc}.h>\n"));
                            },
                            None => {
                                self.imports.push_str(&format!("#include \"{loc}.h\"\n"));
                                self.comp_imports.push_str(&format!("{loc}.c "))
                            },
                        }
                    }
                },
                Expr::CEmbed(embed) => {
                    let mut clean_embed = String::new();
                    let mut added_space = false;

                    for (i, ch) in embed.chars().enumerate() {
                        if ch == '\n' {
                            if self.in_macro_func {
                                clean_embed.push_str("\\\n");
                            } else {
                                clean_embed.push(ch);
                            }

                            if !added_space {self.add_spaces(self.indent); added_space = true}
                            self.code.push_str(&clean_embed);
                            clean_embed.clear();
                        } else if i == embed.len() - 1 {
                            if self.in_macro_func {
                                clean_embed.push_str(&format!("{ch}\\"));
                            } else {
                                clean_embed.push(ch);
                            }

                            if !added_space {self.add_spaces(self.indent); added_space = true}
                            self.code.push_str(&clean_embed);
                            clean_embed.clear();
                        } else {
                            clean_embed.push(ch);
                        }
                    }

                    self.code.push('\n')
                },
                Expr::EnumDef { enum_name, enum_fields } => {
                    let mut def_code = String::new();
                    match *enum_name {
                        Expr::EnumName(name) => {
                            def_code.push_str(&format!("typedef enum {name} {{\n"));
                        },
                        _ => unreachable!(),
                    }

                    let mut fields = String::new();
                    for field in enum_fields {
                        match field {
                            Expr::Variable { info, value } => {
                                let mut varname = self.handle_varname(*info);
                                varname = varname.strip_prefix("const  ").unwrap().to_owned();
                                let varvalue = self.handle_value(*value);
                                fields.push_str(&format!("    {varname} = {varvalue},\n"));
                            },
                            Expr::VariableName { .. } => {
                                let varname = self.handle_varname(field).replace(".", "_");
                                fields.push_str(&format!("    {varname},\n"));
                            },
                            _ => (),
                        }
                    }
                    self.code.push_str(&format!("{def_code}{fields}"));
                    self.defs_location.push(self.code.len());
                },
                Expr::StructDef { struct_name, struct_fields } => {
                    let mut def_code = String::new();
                    match *struct_name {
                        Expr::StructName(name) => {
                            def_code.push_str(&format!("typedef struct {name} {{\n"));
                        },
                        _ => (),
                    }

                    let mut fields = String::new();
                    for field in struct_fields {
                        let mut varname = self.handle_varname(field.clone());
                        if varname.chars().last().unwrap() == 'A' {
                            match field {
                                Expr::VariableName { typ, .. } => {
                                    if let Types::Arr { .. } = typ {
                                        let varname_eq_index = varname.find("=");
                                        match varname_eq_index {
                                            Some(index) => {
                                                varname.truncate(index-1);
                                            },
                                            None => (),
                                        }
                                    }
                                },
                                _ => (),
                            }
                        }
                        fields.push_str(&format!("    {varname};\n"));
                    }

                    // don't need to put \n at the end, end struct covers that
                    self.code.push_str(&format!("{def_code}{fields}"));
                },
                Expr::EndStruct(name) => {
                    self.code.push_str(&format!("}}{name};\n"));
                    self.defs_location.push(self.code.len());
                },
                Expr::MacroStructDef { struct_name, struct_fields } => {
                    self.in_macro_func = true;

                    let mut def_code = String::new();
                    match *struct_name {
                        Expr::MacroStructName { name, generics } => {
                            def_code.push_str(&format!("#define {name}("));
                            for (i, generic) in generics.iter().enumerate() {
                                match generic {
                                    Expr::Variable { info, .. } => {
                                        match *info.clone() {
                                            Expr::VariableName { name, .. } => {
                                                struct_generics.push(name.clone());
                                                if i == 0 {
                                                    def_code.push_str(&format!("{name}"));
                                                } else {
                                                    def_code.push_str(&format!(", {name}"));
                                                }
                                            },
                                            _ => (),
                                        }
                                    },
                                    _ => (),
                                }
                            }
                            def_code.push_str(&format!(")\\\n"));
                            def_code.push_str("typedef struct {\\\n");
                        },
                        _ => (),
                    }

                    let mut fields = String::new();
                    for field in struct_fields {
                        let varname = self.handle_varname(field);
                        fields.push_str(&format!("    {varname};\\\n"));
                    }

                    // don't need to put \n at the end, end struct covers that
                    self.code.push_str(&format!("{def_code}{fields}"));
                },
                Expr::MacroEndStruct(name) => {
                    self.code.push_str(&format!("}} {name}"));
                    for (i, generic) in struct_generics.iter().enumerate() {
                        if i == 0 {
                            self.code.push_str(&format!("_##{generic}"));
                        } else {
                            self.code.push_str(&format!("##{generic}"));
                        }
                    }

                    self.code.push_str(";\n");
                    self.defs_location.push(self.code.len());
                    struct_generics.clear();
                    self.in_macro_func = false;
                },
                Expr::Func { typ, params, name, is_inline } => {
                    self.indent += 1;
                    let mut func_code = String::new();
                    if is_inline && name == String::from("main") {
                        self.comp_err("can't make main function inline");
                        exit(1);
                    }

                    if is_inline {
                        func_code.push_str("$inline ");
                    }

                    if name == String::from("main") {
                        func_code.push_str("int");
                    } else {
                        // fix this to allow array returns
                        func_code.push_str(&self.handle_typ(typ).0);
                    }

                    func_code.push_str(&format!(" {name}("));
                    for (i, param) in params.iter().enumerate() {
                        let mut varname = self.handle_varname(param.clone());

                        if varname.chars().last().unwrap() == 'A' {
                            match param {
                                Expr::VariableName { typ, .. } => {
                                    if let Types::Arr { .. } = typ {
                                        let varname_eq_index = varname.find("=");
                                        match varname_eq_index {
                                            Some(index) => {
                                                varname.truncate(index-1);
                                            },
                                            None => (),
                                        }
                                    }
                                },
                                _ => (),
                            }
                        }

                        if i == 0 {
                            func_code.push_str(&varname);
                        } else {
                            let comma_separated = format!(", {}", varname);
                            func_code.push_str(&comma_separated);
                        }
                    }
                    func_code.push_str(") {\n");
                    self.defs_location.push(self.code.len());
                    self.code.push_str(&func_code);
                },
                Expr::MacroFunc { params, name, .. } => {
                    self.indent += 1;
                    self.curl_rc += 1;
                    self.in_macro_func = true;

                    let mut func_code = String::from("#define ");
                    func_code.push_str(&format!("{name}("));

                    for (i, param) in params.iter().enumerate() {
                        if i == 0 {
                            func_code.push_str(&self.handle_value(param.clone()));
                        } else {
                            let comma_separated = format!(", {}", self.handle_value(param.clone()));
                            func_code.push_str(&comma_separated);
                        }
                    }
                    func_code.push_str(") ({\\\n");
                    self.defs_location.push(self.code.len());
                    self.code.push_str(&func_code);
                },
                Expr::VariableName { typ, name, reassign, constant, field_data } => {
                    self.add_spaces(self.indent);

                    let mut varname = self.handle_varname(Expr::VariableName { typ: typ.clone(), name, reassign, constant, field_data });
                    let last_varname = varname.chars().last().unwrap();
                    if last_varname == 'A' {
                        if let Types::Arr { length, .. } = typ {
                            varname.pop();
                            if self.in_macro_func {
                                self.code.push_str(&format!("{varname}{{}}, .len = {length}}};\\\n"));
                            } else {
                                self.code.push_str(&format!("{varname}{{}}, .len = {length}}};\n"));
                            }
                        }
                        continue;
                    }
                    if self.in_macro_func {
                        self.code.push_str(&format!("{varname} = {{0}};\\\n"));
                    } else {
                        self.code.push_str(&format!("{varname} = {{0}};\n"));
                    }
                },
                Expr::Variable { info, value } => {
                    self.add_spaces(self.indent);

                    match *info {
                        Expr::VariableName { ref typ, ref name, .. } => {
                            if let Types::TypeId = typ {
                                let typeid_type = self.handle_value(*value.clone());
                                self.code.push_str(&format!("typedef {typeid_type} {name};\n"));
                                continue;
                            }
                        },
                        _ => (),
                    }

                    let mut varname = self.handle_varname(*info.clone());
                    if varname.chars().last().unwrap() == 'A' {
                        match (*value.clone(), *info) {
                            (Expr::ArrayLit(arrlit), Expr::VariableName { typ, .. }) => {
                                if let Types::Arr { length, .. } = typ {
                                    varname.pop();
                                    let var_val = self.handle_arraylit(arrlit, length);
                                    if self.in_macro_func {
                                        self.code.push_str(&format!("{varname}{var_val};\\\n"));
                                    } else {
                                        self.code.push_str(&format!("{varname}{var_val};\n"));
                                    }
                                    continue;
                                }
                            }
                            (_, Expr::VariableName { typ, .. })=> {
                                if let Types::Arr { .. } = typ {
                                    self.comp_err("unable to handle array macro");
                                    exit(1);
                                }
                            },
                            _ => (),
                        }
                    }
                    let var_val = self.handle_value(*value);
                    
                    if self.in_macro_func {
                        self.code.push_str(&format!("{varname} = {var_val};\\\n"));
                    } else {
                        self.code.push_str(&format!("{varname} = {var_val};\n"));
                    }
                },
                Expr::FuncCall { name, gave_params } => {
                    self.add_spaces(self.indent);

                    let call = self.handle_funccall(Expr::FuncCall { name, gave_params });
                    if self.in_macro_func {
                        self.code.push_str(&format!("{call};\\\n"));
                    } else {
                        self.code.push_str(&format!("{call};\n"));
                    }
                },
                Expr::Switch(conditions) => {
                    self.add_spaces(self.indent);
                    self.indent += 1;

                    first_case = true;

                    let switch_code = self.handle_branch(&String::from("switch "), conditions);
                    self.code.push_str(&switch_code);
                },
                Expr::Case(conditions) => {
                    self.add_spaces(self.indent);

                    // conditions[0] might be unsafe but oh wells, we'll see when it comes to
                    // that point
                    let case_code = if let Expr::None = conditions[0] {
                        String::from("default: {\n")
                    } else {
                        let condition_str = self.handle_boolean_condition(&conditions);
                        format!("case {condition_str}: {{\n")
                    };

                    if first_case || fall_case {
                        first_case = false;
                        fall_case = false;
                    } else {
                        self.code.push_str("break;\n");
                        self.add_spaces(self.indent);
                    }

                    self.code.push_str(&case_code);
                    self.indent += 1;
                },
                Expr::Fall(conditions) => {
                    self.add_spaces(self.indent);

                    let case_code = if let Expr::None = conditions[0] {
                        String::from("default: {\n")
                    } else {
                        let condition_str = self.handle_boolean_condition(&conditions);
                        format!("case {condition_str}: {{\n")
                    };

                    if first_case || fall_case {
                        first_case = false;
                    } else {
                        self.code.push_str("break;\n");
                        self.add_spaces(self.indent);
                    }

                    fall_case = true; // maybe remove this since it wasn't made false
                    self.code.push_str(&case_code);
                    self.indent += 1;
                },
                Expr::If(conditions, capture) => {
                    self.add_spaces(self.indent);
                    self.indent += 1;

                    match (&conditions[0], *capture) {
                        (Expr::VariableName { typ, name, .. }, Expr::Variable { info, value }) => {
                            if let Types::TypeDef { type_name, .. } = typ {
                                if type_name == &String::from("option") {
                                    let if_code = if self.in_macro_func {
                                        format!("if (!{name}.none) {{\\\n")
                                    } else {
                                        format!("if (!{name}.none) {{\n")
                                    };
                                    self.code.push_str(&if_code);
                                    self.add_spaces(self.indent);

                                    let varname = self.handle_varname(*info);
                                    let varvalue = self.handle_value(*value);
                                    
                                    let capture_code = format!("{varname} = {varvalue};\n");
                                    self.code.push_str(&capture_code);
                                    continue;
                                }
                            }
                        },
                        _ => (),
                    }
                    
                    let if_code = self.handle_branch(&String::from("if "), conditions);
                    // don't need to add \n, handle_branch does it
                    self.code.push_str(&if_code);

                },
                Expr::OrIf(conditions, capture) => {
                    self.add_spaces(self.indent);
                    self.indent += 1;

                    match (&conditions[0], *capture) {
                        (Expr::VariableName { typ, name, .. }, Expr::Variable { info, value }) => {
                            if let Types::TypeDef { type_name, .. } = typ {
                                if type_name == &String::from("option") {
                                    let if_code = if self.in_macro_func {
                                        format!("if (!{name}.none) {{\\\n")
                                    } else {
                                        format!("if (!{name}.none) {{\n")
                                    };
                                    self.code.push_str(&if_code);
                                    self.add_spaces(self.indent);

                                    let varname = self.handle_varname(*info);
                                    let varvalue = self.handle_value(*value);
                                    
                                    let capture_code = format!("{varname} = {varvalue};\n");
                                    self.code.push_str(&capture_code);
                                    continue;
                                }
                            }
                        },
                        _ => (),
                    }
                    
                    let orif_code = self.handle_branch(&String::from("else if "), conditions);
                    // don't need to add \n, handle_branch does it
                    self.code.push_str(&orif_code);
                },
                Expr::Else => {
                    self.add_spaces(self.indent);
                    self.indent += 1;
                    self.curl_rc += 1;

                    if self.in_macro_func {
                        self.code.push_str("else {\\\n");
                    } else {
                        self.code.push_str("else {\n");
                    }
                },
                Expr::Loop { condition, modifier } => {
                    self.add_spaces(self.indent);
                    self.indent += 1;

                    let loop_code = self.handle_loop(condition, *modifier);
                    self.code.push_str(&loop_code);
                },
                Expr::For { for_this, in_this, iterator } => {
                    self.add_spaces(self.indent);
                    self.indent += 1;

                    let for_code = self.handle_for(in_this, iterator);
                    self.code.push_str(&for_code.0);
                    self.add_spaces(self.indent);

                    let for_this_extract = match *for_this {
                        Expr::VariableName { typ, name, .. } => {
                            if let Types::None = typ {
                                (format!("typeof({}.data[0])", for_code.1), name)
                            } else {
                                (self.handle_typ(typ).0, name)
                            }
                        },
                        unexpected => {
                            self.comp_err(&format!("{unexpected:?} is not implemented yet for for loops"));
                            exit(1);
                        },
                    };

                    let var = format!("{} {} = {}.data[{}];\n", for_this_extract.0, for_this_extract.1, for_code.1, for_code.2);
                    self.code.push_str(&var);
                },
                Expr::Return(value) => {
                    self.add_spaces(self.indent);
                    
                    let val = self.handle_value(*value);
                    if self.in_macro_func {
                        self.code.push_str(&format!("{val};\\\n"))
                    } else {
                        self.code.push_str(&format!("return {val};\n"))
                    }
                },
                Expr::StartBlock => {
                    self.add_spaces(self.indent);
                    self.indent += 1;
                    self.code.push_str("{\n");
                },
                Expr::EndBlock => {
                    self.indent -= 1;
                    self.add_spaces(self.indent);

                    if self.curl_rc > 0 {
                        self.curl_rc -= 1;

                        // WATCH THIS CAREFULLY, MIGHT BREAK
                        if self.curl_rc == 0 && self.in_macro_func {
                            self.in_macro_func = false;
                            self.code.push_str("})\n");
                        } else if self.in_macro_func {
                            self.code.push_str("}\\\n");
                        } else {
                            self.code.push_str("}\n");
                        }
                    } else {
                        self.code.push_str("}\n");
                    }
                },
                Expr::Break => {
                    self.add_spaces(self.indent);
                    self.code.push_str("break;\n");
                },
                Expr::Continue => {
                    self.add_spaces(self.indent);
                    self.code.push_str("continue;\n");
                },
                unimpl => {
                    self.comp_err(&format!("{unimpl:?} is not implemented yet"));
                    exit(1);
                }
            }
        }
    }

    fn generate_cpp(&mut self, _expressions: Vec<(Expr, String, u32)>) {
        println!("lmao not ready");
        exit(1);
    }

    pub fn generate(&mut self, expressions: Vec<(Expr, String, u32)>) {
        let c_flags_res = fs::read_to_string("c_flags.txt");
        let c_flags = match c_flags_res {
            Ok(file) => file,
            Err(_) => String::new(),
        };

        match self.lang {
            Lang::C => {
                self.generate_c(expressions);

                let c_code = format!("{}{}", self.imports, self.code);
                if self.compile {
                    match fs::write("./output.c", c_code) {
                        Ok(_) => (),
                        Err(err) => {
                            self.comp_err(&format!("{err}"));
                            exit(1);
                        }
                    }

                    let com = format!("gcc output.c {}-o {} {}", self.comp_imports, self.out_file, c_flags);
                    println!("{com}");
                    if cfg!(target_os = "windows") {
                        Command::new("cmd")
                            .args(["/C", &com])
                            .output()
                            .unwrap();
                    } else {
                        Command::new("sh")
                            .args(["-c", &com])
                            .output()
                            .unwrap();
                    }
                } else {
                    match fs::write(&format!("./{}.c", self.out_file), c_code) {
                        Ok(_) => (),
                        Err(err) => {
                            self.comp_err(&format!("{err}"));
                            exit(1);
                        }
                    }

                    let com = format!("gcc {}.c {}-o {} {}", self.out_file, self.comp_imports, self.out_file, c_flags);
                    println!("{com}");
                }

                // this can only happen after build so there's no problem if the file isn't output.c 
                if !self.keep_gen {
                    match fs::remove_file("output.c") {
                        Ok(_) => (),
                        Err(_) => {
                            self.comp_err("error handling code generation");
                            exit(1)
                        },
                    }
                }
            },
            Lang::Cpp => self.generate_cpp(expressions),
        }
    }
}
