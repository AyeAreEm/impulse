use std::{collections::HashMap, fs, process::{exit, Command}};
use rand::Rng;
use crate::parser::Expr;
use crate::declare_types::*;

pub struct Gen {
    imports: String,
    comp_imports:  String,
    code: String,
    out_file: String,
    libc_map: HashMap<String, bool>,
    indent: i32,
    lang: Lang,
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

fn sanitise_intlit(intlit: String) -> String {
    let mut sanitised = String::new();
    let mut start_index = false;

    for c in intlit.chars() {
        if c == '|' {
            if !start_index {
                sanitised.push('[');
            } else {
                sanitised.push(']');
            }

            start_index = !start_index;
        } else {
            sanitised.push(c);
        }
    } 

    sanitised
}

impl Gen {
    pub fn new(out_file: String, lang: Lang) -> Gen {
        let libc_map = HashMap::from([
            ("stdio".to_string(), true),
            ("stdlib".to_string(), true),
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

    fn import_dynam(&mut self) {
        if !self.imports.contains("#include \"libc/dynamic.h\"\n") {
            self.imports.push_str("#include \"libc/dynamic.h\"\n");
            self.comp_imports.push_str("./libc/dynamic.c ");
        }
    }

    fn import_io(&mut self) {
        if !self.imports.contains("#include \"libc/io.h\"\n") {
            self.imports.push_str("#include \"libc/io.h\"\n");
            self.comp_imports.push_str("./libc/io.c ");
        }
    }

    fn make_arr_var(&mut self, typ: Types, name: String, elems: Expr) -> String {
        let mut arr_var = String::new();
        let mut first_elem = true;

        match typ {
            Types::Int => arr_var.push_str(&format!("int {name}[]={{")),
            Types::Str => {
                self.import_dynam();
                arr_var.push_str(&format!("string {name}[]={{"));
            },
            _ => (),
        }

        if let Expr::Array(elem) = elems {
            for v in *elem {
                match v {
                    Expr::StrLit(string) => {
                        if first_elem {
                            arr_var.push_str(&format!("string_from(\"{string}\")"));
                            first_elem = !first_elem;
                        } else {
                            arr_var.push_str(&format!(",string_from(\"{string}\")"));
                        }
                    },
                    Expr::IntLit(integer) => {
                        let lit = sanitise_intlit(integer.clone());
                        if first_elem {
                            arr_var.push_str(&format!("{lit}"));
                            first_elem = !first_elem;
                        } else {
                            arr_var.push_str(&format!(",{lit}"));
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
        }
        arr_var.push_str("};");
        arr_var
    }

    fn make_dynam_var(&mut self, name: String, elems: Expr) -> String {
        let mut dynam_var = String::new();

        self.import_dynam();
        dynam_var.push_str(&format!("dynam {name}=dynam_new();"));

        if let Expr::Dynamic(elem) = elems {
            for v in *elem {
                match v {
                    Expr::StrLit(string) => {
                        let varname = rand_varname();
                        dynam_var.push_str(&format!("string {varname}=string_from(\"{string}\");"));
                        dynam_var.push_str(&format!("dynam_push(&{name},&{varname});"));
                    },
                    Expr::IntLit(integer) => {
                        let varname = rand_varname();
                        let lit = sanitise_intlit(integer.clone());
                        dynam_var.push_str(&format!("int {varname}={lit};"));
                        dynam_var.push_str(&format!("dynam_push(&{name},&{varname});"));
                    },
                    Expr::VarName((_, varname)) => {
                        dynam_var.push_str(&format!("dynam_push(&{name},&{varname});"));
                    },
                    _ => (),
                }
            }
        }

        dynam_var
    }

    fn make_ifor(&mut self, mut stmnt: String, condition: Expr) {
        self.indent += 1;

        match condition {
            Expr::Condition(cond) => {
                let mut had_angled = false;
                for con in *cond {
                    match con {
                        Expr::VarName((_, name)) => stmnt.push_str(&format!("{name}")),
                        Expr::Equal => {
                            if had_angled {
                                stmnt.push_str("=");
                                had_angled = false;
                            } else {
                                stmnt.push_str("==");
                            }
                        },
                        Expr::SmallerThan => {
                            stmnt.push_str("<");
                            had_angled = true;
                        },
                        Expr::BiggerThan => {
                            stmnt.push_str(">");
                            had_angled = true;
                        },
                        Expr::Exclaim => {
                          stmnt.push_str("!");
                          had_angled = true;
                        },
                        Expr::IntLit(integer) => {
                            let lit = sanitise_intlit(integer.clone());
                            stmnt.push_str(&format!("{lit}"))
                        },
                        Expr::StrLit(string) => stmnt.push_str(&format!("{string}")),
                        Expr::Or => stmnt.push_str("||"),
                        Expr::And => stmnt.push_str("&&"),
                        _ => (),
                    }
                }
            },
            _ => (),
        }

        stmnt.push_str(") {\n");
        self.code.push_str(&stmnt);
    }

    fn handle_print(&mut self, value: Vec<Expr>, is_line: bool) {
        self.add_spaces(self.indent);

        if !self.imports.contains("#include <stdio.h>\n") {
            self.imports.push_str("#include <stdio.h>\n");
        }

        let mut print_code = String::from(format!("printf(\""));
        let mut param_buf = String::new();
        let mut var_buf = String::new();
       // FIND A WAY TO PASS THE CONTENT OF A STRING INSTEAD OF THE WHOLE STRUCT

        for v in value {
            match v {
                Expr::StrLit(string) => param_buf.push_str(&format!("{string}")),
                Expr::IntLit(integer) => {
                    let lit = sanitise_intlit(integer.clone());
                    param_buf.push_str(&format!("%d"));
                    var_buf.push_str(&format!(",{lit}"));
                },
                Expr::ArrIndex(arr_index) => {
                    let mut is_string = false;
                    let mut arr_name = String::new();

                    match arr_index.0 {
                        Expr::VarName((typ, name)) => {
                            arr_name = name;
                            match typ {
                                Types::Arr(arr_typ) => {
                                    match *arr_typ {
                                        Types::Int => {
                                            param_buf.push_str(&format!("%d"));
                                        },
                                        Types::Str => {
                                            is_string = true;
                                            param_buf.push_str(&format!("%s"));
                                        }
                                        _ => (),
                                    }
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }

                    match arr_index.1 {
                        Expr::IntLit(index) => {
                            if is_string {
                                var_buf.push_str(&format!(",{arr_name}[{index}].data"));
                            } else {
                                var_buf.push_str(&format!(",{arr_name}[{index}]"));
                            }
                        },
                        _ => (),
                    }
                },
                Expr::Var(var_info) => {
                    match var_info.0 {
                        Expr::VarName((typ, name)) => {
                            match typ {
                                Types::Int => {
                                    param_buf.push_str(&format!("%d"));
                                    var_buf.push_str(&format!(",{name}"));
                                },
                                Types::Str => {
                                    param_buf.push_str(&format!("%s"));
                                    var_buf.push_str(&format!(",{name}.data"));
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        print_code.push_str(&param_buf);
        if is_line {
            print_code.push_str("\\n");
        }
        print_code.push('"');
        print_code.push_str(&var_buf);
        print_code.push_str(");\n");
        self.code.push_str(&print_code);
    }

    pub fn generate(&mut self, expressions: Vec<Expr>) {
        let mut defined_struct_name = String::new();
        let mut varname_buf = String::new();

        for (index, expr) in expressions.into_iter().enumerate() {
            match expr {
                Expr::CImport(loc) => {
                    if !self.imports.contains(&format!("#include <{loc}.h>\n")) {
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
                Expr::FuncCall(f) => {
                    let mut func_call = String::new();
                    let mut first_param = true;

                    match f.0 {
                        Expr::FuncName(name) => func_call.push_str(&format!("{name}(")),
                        _ => (),
                    }

                    for param in f.1 {
                        match param {
                            Expr::Var(var_info) => {
                                match var_info.0 {
                                    Expr::VarName((_, name)) => {
                                        if first_param {
                                            func_call.push_str(&format!("{name}"));
                                            first_param = false;
                                        } else {
                                            func_call.push_str(&format!(",{name}"));
                                        }
                                    },
                                    _ => (),
                                }
                            },
                            Expr::StrLit(string) => {
                                if first_param {
                                    func_call.push_str(&format!("string_from(\"{string}\")"));
                                    first_param = false;
                                } else {
                                    func_call.push_str(&format!(",string_from(\"{string}\")"));
                                }
                            },
                            _ => (),
                        }
                    }

                    func_call.push_str(");\n");
                    self.code.push_str(&func_call);
                },
                Expr::ReArr(value) => {
                    let mut re_arr = String::new();
                    let pos = value.1;

                    match value.0 {
                        Expr::VarName((_, name)) => {
                            re_arr.push_str(&format!("{name}["));
                        },
                        _ => (),
                    }

                    re_arr.push_str(&format!("{pos}]"));

                    match value.2 {
                        Expr::IntLit(integer) => {
                            let lit = sanitise_intlit(integer.clone());
                            re_arr.push_str(&format!("={lit};\n"));
                        },
                        Expr::StrLit(string) => re_arr.push_str(&format!("=string_from(\"{string}\");\n")),
                        Expr::VarName(_) => (),
                        _ => (),
                    }

                    self.code.push_str(&re_arr);
                },
                Expr::Var(value) => {
                    self.add_spaces(self.indent);
                    let mut variable = String::new();

                    let mut is_arr = false;
                    let mut is_dynam = false;
                    let mut is_struct = false;

                    let mut coll_typ = Types::None;
                    let mut coll_name = String::new();

                    match value.0 {
                        Expr::VarName((typ, name)) => {
                            match typ {
                                Types::Str => {
                                    self.import_dynam();
                                    variable.push_str(&format!("string {name}="))
                                },
                                Types::Int => variable.push_str(&format!("int {name}=")),
                                Types::Arr(arr_typ) => {
                                    is_arr = true;
                                    coll_typ = *arr_typ;
                                    coll_name = name;
                                },
                                Types::Dynam(_) => {
                                    is_dynam = true;
                                    coll_name = name;
                                },
                                Types::UserDef(typedef) => {
                                    is_struct = true;
                                    variable.push_str(&format!("{typedef} {name};\n"));
                                    varname_buf = name.clone();
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }

                    if is_arr {
                        let arr_var = self.make_arr_var(coll_typ, coll_name, value.1);
                        self.code.push_str(&arr_var);
                        continue;
                    } else if is_dynam {
                        let dynam_var = self.make_dynam_var(coll_name, value.1);
                        self.code.push_str(&dynam_var);
                        continue;
                    } else if is_struct {

                    } else {
                        match value.1 {
                            Expr::StrLit(value) => variable.push_str(&format!("string_from(\"{value}\");\n")),
                            Expr::IntLit(value) => {
                                let lit = sanitise_intlit(value.clone());
                                variable.push_str(&format!("{lit};\n"));
                            },
                            Expr::ReadIn => {
                                self.import_io();
                                variable.push_str(&format!("readin();\n"));
                            },
                            Expr::VarName((_, value)) => variable.push_str(&format!("{value};\n")),
                            Expr::FuncCall(func_call) => {
                                match func_call.0 {
                                    Expr::FuncName(func_name) => variable.push_str(&format!("{func_name}(")),
                                    _ => (),
                                }

                                let mut first_param = true;
                                for param in func_call.1 {
                                    match param {
                                        Expr::Var(var_info) => {
                                            match var_info.0 {
                                                Expr::VarName((_, name)) => {
                                                    if first_param {
                                                        variable.push_str(&format!("{name}"));
                                                        first_param = false;
                                                    } else {
                                                        variable.push_str(&format!(",{name}"));
                                                    }
                                                }
                                                _ => (),
                                            }
                                        },
                                        Expr::StrLit(string) => {
                                            if first_param {
                                                variable.push_str(&format!("string_from(\"{string}\")"));
                                                first_param = false;
                                            } else {
                                                variable.push_str(&format!(",string_from(\"{string}\")"));
                                            }
                                        },
                                        _ => (),
                                    }
                                }

                                variable.push_str(");\n");
                            },
                            Expr::ArrIndex(value) => {
                                // value.0 = Varname -> (typ, name)
                                // value.1 = IntLit -> index

                                let mut arr_name = String::new();
                                match value.0 {
                                    Expr::VarName((_, name)) => arr_name = name,
                                    _ => (),
                                }

                                match value.1 {
                                    Expr::IntLit(num) => {
                                        let lit = sanitise_intlit(num.clone());
                                        variable.push_str(&format!("{arr_name}[{lit}];\n"));
                                    },
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }

                    self.code.push_str(&variable);
                },
                Expr::ReVar(value) => {
                    self.add_spaces(self.indent);

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
                            re_var.push_str(&format!("{var_name} = string_from(\"{string}\");\n"));
                        },
                        Expr::IntLit(integer) => {
                            let lit = sanitise_intlit(integer.clone());
                            re_var.push_str(&format!("{var_name} = {lit};\n"));
                        },
                        Expr::VarName((_, name)) => {
                            re_var.push_str(&format!("{var_name} = {name};\n"));
                        },
                        _ => (),
                    }

                    self.code.push_str(&re_var);
                },
                Expr::Println(value) => {
                    self.handle_print(*value, true);
                },
                Expr::Print(value) => {
                    self.handle_print(*value, false);
                },
                Expr::EndBlock => {
                    self.indent -= 1;
                    self.add_spaces(self.indent);
                    self.code.push_str("}\n");
                },
                Expr::Func(f) => {
                    let ty = &f.0;
                    let params = &f.1;
                    let name = &f.2;
                    let mut f_ty = String::new();
                    let mut f_params = String::new();
                    let mut f_params_touched = false;
                    let mut f_name = String::new();

                    self.indent += 1;

                    match params {
                        Expr::FuncParams(ps) => {
                            for p in *ps.clone() {
                                let mut this_typ = String::new();
                                let mut this_name = String::new();

                                match p {
                                    Expr::Var(var_info) => {
                                        match var_info.0 {
                                            Expr::VarName((typ, name)) => {
                                                this_name = name;
                                                match typ {
                                                    Types::Str => {
                                                        self.import_dynam();
                                                        this_typ = String::from("string");
                                                    },
                                                    Types::Int => this_typ = String::from("int"),
                                                    Types::Void => this_typ = String::from("void"),
                                                    Types::Arr(arr_typ) => {
                                                        match *arr_typ {
                                                            Types::Str => {
                                                                self.import_dynam();
                                                                this_typ = String::from("string");
                                                                this_name.push_str("[]");
                                                            },
                                                            Types::Int => {
                                                                this_typ = String::from("int");
                                                                this_name.push_str("[]");
                                                            },
                                                            _ => (),
                                                        }
                                                    },
                                                    Types::Dynam(dynam_typ) => {
                                                        match *dynam_typ {
                                                            Types::Str => {
                                                                self.import_dynam();
                                                                this_typ = String::from("string");
                                                                this_name.push_str("[]");
                                                            },
                                                            Types::Int => {
                                                                this_typ = String::from("int");
                                                                this_name.push_str("[]");
                                                            },
                                                            _ => (),
                                                        }
                                                    },
                                                    Types::UserDef(userdef_typ) => {
                                                        this_typ = userdef_typ;
                                                    },
                                                    Types::None => (),
                                                }
                                            },
                                            _ => (),
                                        }
                                    }
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
                        Types::Str => {
                            self.import_dynam();
                            f_ty = String::from("string");
                        },
                        Types::Arr(arr_typ) => {
                            match **arr_typ {
                                Types::Str => {
                                    self.import_dynam();
                                    f_ty = String::from("[]string");
                                },
                                Types::Int => f_ty = String::from("[]int"),
                                _ => (),
                            }
                        },
                        Types::Dynam(dynam_typ) => {
                            match **dynam_typ {
                                Types::Str => {
                                    self.import_dynam();
                                    f_ty = String::from("[]string");
                                },
                                Types::Int => f_ty = String::from("[]int"),
                                _ => (),
                            }
                        },
                        Types::UserDef(userdef_typ) => f_ty = userdef_typ.to_owned(),
                        Types::None => {
                            println!("\x1b[91merror\x1b[0m: line {}, unexpected type", index + 1);
                            exit(1)
                        },
                    }

                    let func = format!("{f_ty} {f_name}({f_params}) {{\n");
                    self.code.push_str(&func);
                },
                Expr::Return(value) => {
                    let mut ret = String::new();
                    match *value {
                        Expr::StrLit(string) => {
                            ret.push_str(&format!("return {string};\n"));
                        },
                        Expr::IntLit(integer) => {
                            let lit = sanitise_intlit(integer.clone());
                            ret.push_str(&format!("return {lit};\n"));
                        },
                        Expr::VarName((_, name)) => {
                            ret.push_str(&format!("return {name};\n"));
                        },
                        _ => (),
                    }
                    self.code.push_str(&ret);
                },
                Expr::Loop(loop_tup) => {
                    let mut conditions = String::new();
                    let mut is_inited = false;

                    self.indent += 1;
                    match loop_tup.0 {
                        Expr::Condition(expr_arr) => {
                            for expr in *expr_arr {
                                match expr {
                                    Expr::Var(var_info) => {
                                        match var_info.0 {
                                            Expr::VarName((_, name)) => {
                                                conditions.push_str(&format!("int {name}=0;{name}"));
                                                is_inited = true;
                                            },
                                            _ => (),
                                        }
                                    },
                                    Expr::VarName((_, name)) => {
                                        if is_inited {
                                            conditions.push_str(&format!("{name}"))
                                        } else {
                                            conditions.push_str(&format!(";{name}"))
                                        }
                                    },
                                    Expr::Equal => conditions.push_str("=="),
                                    Expr::SmallerThan => conditions.push_str("<"),
                                    Expr::BiggerThan => conditions.push_str(">"),
                                    Expr::Exclaim => conditions.push_str("!"),
                                    Expr::IntLit(integer) => {
                                        let lit = sanitise_intlit(integer.clone());
                                        conditions.push_str(&format!("{lit};"))
                                    },
                                    Expr::StrLit(string) => conditions.push_str(&format!("{string}")),
                                    Expr::Or => conditions.push_str("||"),
                                    Expr::And => conditions.push_str("&&"),
                                    _ => (),
                                }
                            }
                        },
                        _ => (),
                    }

                    match loop_tup.1 {
                        Expr::LoopMod(modif) => conditions.push_str(&format!("{modif}")),
                        _ => (),
                    }

                    let loop_code = format!("for ({conditions}) {{\n");
                    self.code.push_str(&loop_code);
                },
                Expr::StructDef(struct_name) => {
                    self.indent += 1;
                    defined_struct_name = struct_name;
                    self.code.push_str(&format!("typedef struct {defined_struct_name} {{\n"));
                },
                Expr::StructField(field_name) => {
                    self.add_spaces(self.indent);
                    match *field_name {
                        Expr::VarName((typ, name)) => {
                            match typ {
                                Types::Int => {
                                    self.code.push_str(&format!("int {name};\n"));
                                },
                                Types::Str => {
                                    self.import_dynam();
                                    self.code.push_str(&format!("string {name};\n"));
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                },
                Expr::EndStruct => {
                    self.indent -= 1;
                    self.code.push_str(&format!("}}{defined_struct_name};\n"));
                },
                Expr::StructVarField(field) => {
                    self.add_spaces(self.indent);
                    let mut var_field = String::new();
                    var_field.push_str(&varname_buf);

                    match *field {
                        Expr::Var(var_info) => {
                            match var_info.0 {
                                Expr::VarName((_, name)) => var_field.push_str(&format!(".{name}=")),
                                _ => (),
                            }

                            match var_info.1 {
                                Expr::IntLit(integer) => var_field.push_str(&format!("{integer};\n")),
                                Expr::StrLit(string) => {
                                    self.import_dynam();
                                    var_field.push_str(&format!("string_from(\"{string}\");\n"));
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                    self.code.push_str(&var_field);
                },
                Expr::EndStructVar => {
                    varname_buf.clear();
                },
                Expr::If(condition) => {
                    self.add_spaces(self.indent);
                    self.make_ifor(String::from("if ("), *condition);
                },
                Expr::OrIf(condition) => {
                    self.add_spaces(self.indent);
                    self.make_ifor(String::from("else if ("), *condition);
                },
                Expr::Else => {
                    self.add_spaces(self.indent);
                    self.code.push_str("else {");
                },
                Expr::CEmbed(embed) => {
                    self.add_spaces(self.indent);
                    self.code.push_str(&format!("{embed}\n"));
                },
                _ => (),
            }
        }

        let c_code = format!("{}{}", self.imports, self.code);
        match fs::write("./output.c", c_code) {
            Ok(_) => (),
            Err(err) => {
                println!("{err}");
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
