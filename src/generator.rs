use std::{collections::HashMap, fs, process::{exit, Command}};
use crate::declare_types::*;
use crate::parser::*;
// use rand::Rng;

pub struct Gen {
    imports: String,
    comp_imports:  String,
    code: String,
    out_file: String,
    libc_map: HashMap<String, bool>,
    indent: i32,
    lang: Lang,
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

fn comp_err(error_msg: &str) {
    println!("\x1b[91merror\x1b[0m: {error_msg}");
}

impl Gen {
    pub fn new(out_file: String, lang: Lang) -> Gen {
        let libc_map = HashMap::from([
            ("stdio".to_string(), true),
            ("stdlib".to_string(), true),
            ("stdbool".to_string(), true),
            ("string".to_string(), true),
        ]);

        return Gen {
            imports: String::new(),
            comp_imports: String::new(),
            code: String::new(),
            out_file,
            libc_map,
            indent: 0,
            lang,
        }
    }

    fn add_spaces(&mut self, indent: i32) {
        let spaces = indent * 4;
        if spaces > 0 {
            for _ in 0..spaces {
                self.code.push(' ');
            }
        }
    }

    fn handle_typ(&self, typ: Types) -> (String, String) {
        match typ {
            Types::I32 => (String::from("int"), String::new()),
            Types::U8 => (String::from("unsigned char"), String::new()),
            Types::I8 => (String::from("signed char"), String::new()),
            Types::Char => (String::from("char"), String::new()),
            Types::Bool => (String::from("bool"), String::new()),
            Types::TypeDef(user_def) => return (format!("{user_def}"), String::new()),
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
            unimpl => {
                comp_err(&format!("{unimpl:?} is not implemented yet"));
                exit(1);
            }
        }
    }

    fn handle_varname(&self, varname: Expr) -> String {
        match varname {
            Expr::VariableName { ref typ, reassign, .. } => {
                let new_name = self.handle_deref_struct(varname.clone());
                if let Types::ArrIndex { index_at, .. } = typ {
                    return format!("{new_name}[{index_at}]")
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
                comp_err(&format!("unexpected expression: {unexpected:?}"));
                exit(1);
            },
        }
    }

    fn handle_funccall(&self, funccall: Expr) -> String {
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
                        Expr::VariableName { name, .. } => {
                            if i == 0 {
                                funccall_code.push_str(name)
                            } else {
                                funccall_code.push_str(&format!(", {}", name))
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
                            comp_err(&format!("expression {unimpl:?} not implemented yet"));
                            exit(1);
                        }
                    }
                }

                funccall_code.push(')');
                return funccall_code
            },
            unexpected => {
                comp_err(&format!("unexpected expression: {unexpected:?}"));
                exit(1);
            },
        }

    }

    fn handle_arraylit(&self, arrlit: Vec<Expr>) -> String {
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
                    match typ {
                        Types::Pointer(_) => {
                            let new_name = name.replace(".", "->");
                            return new_name
                        },
                        _ => {
                            return name
                        }
                    }
                } else {
                    return name
                }
            },
            unexpected => {
                comp_err(&format!("can't deference {unexpected:?}"));
                exit(1);
            }
        }
    }

    fn handle_value(&self, value: Expr) -> String {
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
                comp_err(&format!("expression {unimpl:?} not implemented yet"));
                exit(1);
            }
        }

    }

    fn handle_boolean_condition(&self, conditions: &Vec<Expr>) -> String {
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
                _ => (),
            }
        }

        boolean_condition_code
    }

    fn handle_branch(&self, branch_typ: &String, conditions: Vec<Expr>) -> String {
        let mut branch_code = String::new();
        branch_code.push_str(&format!("{branch_typ}("));

        let condition_str = self.handle_boolean_condition(&conditions);
        branch_code.push_str(&condition_str);

        branch_code.push_str(") {\n");
        branch_code
    }

    fn handle_loop(&self, conditions: Vec<Expr>, modifier: Expr) -> String {
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

        loop_code.push_str(") {\n");
        loop_code
    }

    pub fn generate(&mut self, expressions: Vec<Expr>) {
        for (_index, expr) in expressions.into_iter().enumerate() {
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
                    self.add_spaces(self.indent);
                    self.code.push_str(&format!("{embed}\n"))
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
                }
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
                Expr::VariableName { typ, name, reassign, field_data } => {
                    self.add_spaces(self.indent);

                    let varname = self.handle_varname(Expr::VariableName { typ, name, reassign, field_data });
                    self.code.push_str(&format!("{varname} = {{0}};\n"));
                },
                Expr::Variable { info, value } => {
                    self.add_spaces(self.indent);

                    let varname = self.handle_varname(*info);
                    let var_val = self.handle_value(*value);

                    self.code.push_str(&format!("{varname} = {var_val};\n"));
                },
                Expr::FuncCall { name, gave_params } => {
                    self.add_spaces(self.indent);

                    let call = self.handle_funccall(Expr::FuncCall { name, gave_params });
                    self.code.push_str(&format!("{call};\n"));
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
                    self.code.push_str(&format!("return {val};\n"))
                },
                Expr::EndBlock => {
                    self.indent -= 1;
                    self.add_spaces(self.indent);
                    self.code.push_str("}\n");
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
                    comp_err(&format!("{unimpl:?} is not implemented yet"));
                    exit(1);
                }
            }
        }

        let c_code = format!("{}{}", self.imports, self.code);
        match fs::write("./output.c", c_code) {
            Ok(_) => (),
            Err(err) => {
                comp_err(&format!("{err}"));
                exit(1);
            }
        }

        let com = format!("gcc output.c {}-o {}", self.comp_imports, self.out_file);
        println!("{com}");
        Command::new("cmd")
            .args(["/C", &com])
            .output()
            .unwrap();
    }
}
