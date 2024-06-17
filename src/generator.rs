use std::{collections::HashMap, fs, process::{exit, Command}};
use crate::declare_types::*;
use crate::parser::*;
// use rand::Rng;

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
    definition_map: HashMap<String, usize>,

    generated_structs: Vec<String>,

    in_macro_func: bool,
    curl_rc: i32,
}

// fn rand_varname() -> String {
//     let alphabet = String::from("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
//     let mut varname = String::new();
//     let mut rng = rand::thread_rng();
//
//     for _ in 0..6 {
//         let r = rng.gen_range(0..alphabet.len());
//         varname.push(alphabet.chars().nth(r).unwrap());
//     }
//
//     varname
// }

impl Gen {
    pub fn new(in_file: String, out_file: String, compile: bool, keep_gen: bool, lang: Lang) -> Gen {
        let libc_map = HashMap::from([
            ("stdio".to_string(), true),
            ("stdlib".to_string(), true),
            ("stdbool".to_string(), true),
            ("stddef".to_string(), true),
            ("string".to_string(), true),
        ]);

        let definition_map = HashMap::new();

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
            definition_map,
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

    fn type_to_name<'a>(&'a mut self, typ: &'a str) -> &str {
        match typ {
            // "i32" => "int",
            // "u8" => "u8",
            // "i8" => "signed char",
            "usize" => {
                if !self.imports.contains("#include <stddef.h>\n") {
                    self.imports.push_str("#include <stddef.h>\n");
                }
                return "size_t"
            },
            "bool" => {
                if !self.imports.contains("#include <stdbool.h>\n") {
                    self.imports.push_str("#include <stdbool.h>\n");
                }
                return "bool"
            },
            same => same,
        }
    }

    fn update_definitions(&mut self, mut line_num: usize) {
        for (_, value) in self.definition_map.iter_mut() {
            if value > &mut line_num {
                let mut pos = 0;
                let mut count = 0;
                for i in value.clone()..self.code.len() {
                    match self.code.chars().nth(i) {
                        Some(ch) => {
                            if ch == '\n' {
                                pos = i;
                                count += 1;

                                if count == 2 {
                                    break;
                                }
                            }
                        },
                        None => {
                            self.comp_err("failed to generate generics");
                            exit(1);
                        }
                    }
                }
                *value = pos + 1;
            }
        }
    }

    fn generate_new_struct(&mut self, fullname: String) {
        let mut found = false;
        for gen_struct in &self.generated_structs {
            if gen_struct == &fullname {
                found = true;
            }
        }

        let mut parts: Vec<&str> = fullname.split("_").collect();
        let name = parts.remove(0);

        let mut gen_code = format!("{name}(");
        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                gen_code.push_str(part);
            } else {
                gen_code.push_str(&format!(", {part}"));
            }
        }
        gen_code.push_str(");\n");

        if found {
            return;
        }

        let index_op = self.definition_map.get(name);
        let index = match index_op {
            Some(i) => i.to_owned(),
            None => {
                self.comp_err("failed to handle generating struct at compile time.");
                exit(1);
            },
        };

        self.code.insert_str(index, &gen_code);
        self.update_definitions(index);
        self.generated_structs.push(fullname);
    }

    fn handle_typ(&mut self, typ: Types) -> (String, String) {
        match typ {
            Types::I32 => (String::from("i32"), String::new()),
            Types::U8 => (String::from("u8"), String::new()),
            Types::I8 => (String::from("i8"), String::new()),
            Types::Char => (String::from("char"), String::new()),
            Types::Int => (String::from("int"), String::new()),
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
            Types::TypeDef { type_name: user_def, generics } => {
                let mut typ = format!("{user_def}");

                for generic in generics {
                    if self.in_macro_func {
                        typ.push_str(&format!("_##{generic}"));
                    } else {
                        typ.push_str(&format!("_{generic}"));
                    }
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
            }
            unimpl => {
                self.comp_err(&format!("{unimpl:?} is not implemented yet"));
                exit(1);
            }
        }
    }

    fn handle_varname(&mut self, varname: Expr) -> String {
        match varname {
            Expr::VariableName { ref typ, reassign, .. } => {
                let new_name = self.handle_deref_struct(varname.clone());
                if let Types::ArrIndex { index_at, .. } = typ {
                    return format!("{new_name}[{index_at}]")
                }

                if let Types::TypeDef { type_name, generics } = typ {
                    if !generics.is_empty() {
                        let mut fullname = format!("{type_name}");
                        for generic in generics {
                            fullname.push_str(&format!("_{generic}"));
                        }

                        if !self.in_macro_func {
                            self.generate_new_struct(fullname);
                        }
                    }
                }

                if reassign == false {
                    let str_typ = self.handle_typ(typ.clone());
                    return format!("{} {new_name}{}", str_typ.0, str_typ.1)
                } else {
                    return format!("{new_name}")
                }
            },
            Expr::DerefPointer(value) => {
                let derefed = self.handle_varname(*value);
                return format!("*{derefed}")
            },
            unexpected => {
                self.comp_err(&format!("unexpected expression: {unexpected:?}"));
                exit(1);
            },
        }
    }

    fn handle_funccall(&mut self, funccall: Expr) -> String {
        match funccall {
            Expr::FuncCall { name, gave_params } => {
                let mut funccall_code = String::new();
                funccall_code.push_str(&format!("{name}("));

                if gave_params.is_empty() {
                    funccall_code.push(')');
                    return funccall_code
                } 

                for (i, param) in gave_params.iter().enumerate() {
                    match param {
                        Expr::IntLit(intlit) => {
                            if i == 0 {
                                funccall_code.push_str(&intlit)
                            } else {
                                funccall_code.push_str(&format!(", {intlit}"))
                            }
                        },
                        Expr::VariableName { name, typ, .. } => {
                            let mut clean_name = name.clone();
                            if let Types::TypeId = typ {
                                clean_name = self.type_to_name(&clean_name).to_string();
                            }

                            if i == 0 {
                                funccall_code.push_str(&clean_name)
                            } else {
                                funccall_code.push_str(&format!(", {}", clean_name))
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

    fn handle_arraylit(&mut self, arrlit: Vec<Expr>) -> String {
        let mut arrlit_code = String::new();
        arrlit_code.push('{');

        for (i, elem) in arrlit.iter().enumerate() {
            let literal = self.handle_value(elem.clone());

            if i == 0 {
                arrlit_code.push_str(&literal);
            } else {
                arrlit_code.push_str(&format!(", {literal}"));
            }
        }

        arrlit_code.push('}');
        arrlit_code
    }

    fn handle_deref_struct(&self, value: Expr) -> String {
        match value {
            Expr::VariableName { typ, name, field_data, .. } => {
                if let Types::ArrIndex { ref arr_typ, .. } = typ {
                    if field_data.0 && field_data.1 {
                        if let Types::Pointer(_) = **arr_typ {
                            let new_name = name.replace(".", "->");
                            return new_name
                        } else {
                            return name
                        }
                    } else {
                        return name
                    }
                }

                if field_data.0 && field_data.1 {
                    let new_name = name.replace(".", "->");
                    return new_name
                } else {
                    return name
                }
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
            Expr::StrLit { content, .. } => format!("\"{content}\""),
            Expr::True => String::from("true"),
            Expr::False => String::from("false"),
            Expr::VariableName { ref typ, .. } => {
                let new_name = self.handle_deref_struct(value.clone());
                if let Types::ArrIndex { index_at, .. } = typ {
                    return format!("{new_name}[{index_at}]")
                } else {
                    return new_name
                }
            },
            Expr::FuncCall { .. } => return self.handle_funccall(value.clone()),
            Expr::ArrayLit(arrlit) => return self.handle_arraylit(arrlit),
            Expr::Address(atoval) =>{
                let sub_val = self.handle_value(*atoval);
                return format!("&{sub_val}")
            },
            Expr::DerefPointer(dptoval) => {
                let sub_val = self.handle_value(*dptoval);
                return format!("*{sub_val}")
            },
            Expr::None => String::new(),
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
                Expr::VariableName { typ, .. } => {
                    let new_name = self.handle_deref_struct(condition.clone());

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
                _ => (),
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
                    loop_code.push_str(&format!(";"));
                }
            },
            Expr::Exclaim => loop_code.push_str(&format!(";")),
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

    fn generate_c(&mut self, expressions: Vec<(Expr, String, u32)>) {
        let mut struct_generics = Vec::new();
        self.code.push_str("#include <stdint.h>\n");
        self.code.push_str("typedef uint8_t u8;\n");
        self.code.push_str("typedef int8_t i8;\n");
        self.code.push_str("typedef int32_t i32;\n");

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

                    for (i, ch) in embed.chars().enumerate() {
                        if ch == '\n' && self.in_macro_func {
                            clean_embed.push_str("\\\n");
                        } else if i == embed.len() - 1 && self.in_macro_func {
                            clean_embed.push_str(&format!("{ch}\\"));
                        } else {
                            clean_embed.push(ch);
                        }
                    }

                    self.code.push_str(&format!("{clean_embed}\n"))
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
                        let varname = self.handle_varname(field);
                        fields.push_str(&format!("    {varname};\n"));
                    }

                    // don't need to put \n at the end, end struct covers that
                    self.code.push_str(&format!("{def_code}{fields}"));
                },
                Expr::EndStruct(name) => {
                    self.code.push_str(&format!("}}{name};\n"));
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
                    let mut useable_name = format!("{name}");
                    for generic in &struct_generics {
                        self.code.push_str(&format!("_##{generic}"));
                        useable_name.push_str(&format!("_{generic}"));
                    }

                    self.code.push_str(";\n");
                    self.definition_map.entry(name).or_insert(self.code.len());
                    struct_generics.clear();
                    self.in_macro_func = false;
                },
                Expr::Func { typ, params, name } => {
                    self.indent += 1;
                    let mut func_code = String::new();
                    if name == String::from("main") {
                        func_code.push_str("int");
                    } else {
                        // fix this to allow array returns
                        func_code.push_str(&self.handle_typ(typ).0);
                    }

                    func_code.push_str(&format!(" {name}("));
                    for (i, param) in params.iter().enumerate() {
                        if i == 0 {
                            func_code.push_str(&self.handle_varname(param.clone()));
                        } else {
                            let comma_separated = format!(", {}", self.handle_varname(param.clone()));
                            func_code.push_str(&comma_separated);
                        }
                    }
                    func_code.push_str(") {\n");
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
                    self.code.push_str(&func_code);
                },
                Expr::VariableName { typ, name, reassign, field_data } => {
                    self.add_spaces(self.indent);

                    let varname = self.handle_varname(Expr::VariableName { typ, name, reassign, field_data });
                    if self.in_macro_func {
                        self.code.push_str(&format!("{varname} = {{0}};\\\n"));
                    } else {
                        self.code.push_str(&format!("{varname} = {{0}};\n"));
                    }
                },
                Expr::Variable { info, value } => {
                    self.add_spaces(self.indent);

                    let varname = self.handle_varname(*info);
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
                Expr::If(conditions) => {
                    self.add_spaces(self.indent);
                    self.indent += 1;
                    
                    let if_code = self.handle_branch(&String::from("if "), conditions);
                    // don't need to add \n, handle_branch does it
                    self.code.push_str(&if_code);
                },
                Expr::OrIf(conditions) => {
                    self.add_spaces(self.indent);
                    self.indent += 1;
                    
                    let orif_code = self.handle_branch(&String::from("else if "), conditions);
                    // don't need to add \n, handle_branch does it
                    self.code.push_str(&orif_code);
                },
                Expr::Else => {
                    self.add_spaces(self.indent);
                    self.indent += 1;
                    self.code.push_str("else {\n");
                },
                Expr::Loop { condition, modifier } => {
                    self.add_spaces(self.indent);
                    self.indent += 1;

                    let loop_code = self.handle_loop(condition, *modifier);
                    self.code.push_str(&loop_code);
                }
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

                        if self.curl_rc == 0 && self.in_macro_func {
                            self.in_macro_func = false;
                            self.code.push_str("})\n");
                        } else {
                            self.code.push_str("}\\\n");
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

                    let com = format!("gcc output.c {}-o {}", self.comp_imports, self.out_file);
                    println!("{com}");
                    Command::new("cmd")
                        .args(["/C", &com])
                        .output()
                        .unwrap();
                } else {
                    match fs::write(&format!("./{}.c", self.out_file), c_code) {
                        Ok(_) => (),
                        Err(err) => {
                            self.comp_err(&format!("{err}"));
                            exit(1);
                        }
                    }

                    let com = format!("gcc {}.c {}-o {}", self.out_file, self.comp_imports, self.out_file);
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
