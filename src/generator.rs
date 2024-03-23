use std::{fs, process::{exit, Command}};
use rand::Rng;
use crate::parser::Expr;
use crate::declare_types::*;

pub struct Gen {
    imports: String,
    comp_imports:  String,
    code: String,
    out_file: String,
    lang: Lang,
}

fn rand_varname() -> String {
    let alphabet = String::from("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let mut varname = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..5 {
        let r = rng.gen_range(0..alphabet.len());
        varname.push(alphabet.chars().nth(r).unwrap());
    }

    varname
}

impl Gen {
    pub fn new(out_file: String, lang: Lang) -> Gen {
        return Gen {
            imports: String::new(),
            comp_imports: String::new(),
            code: String::new(),
            out_file,
            lang,
        }
    }

    fn import_dynam(&mut self) {
        if !self.imports.contains("#include \"libc/dynamic.h\"\n") {
            self.imports.push_str("#include \"libc/dynamic.h\"\n");
            self.comp_imports.push_str("./libc/dynamic.c ");
        }
    }

    pub fn generate(&mut self, expressions: Vec<Expr>) {
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
                                    func_call.push_str(&format!("string_from(\"{string}\")"));
                                    first_param = false;
                                } else {
                                    func_call.push_str(&format!(",string_from(\"{string}\")"));
                                }
                            },
                            _ => (),
                        }
                    }

                    func_call.push_str(");");
                    self.code.push_str(&func_call);
                },
                Expr::Dynam(value) => {
                    let mut dynam_var = String::new();
                    let mut dynam_name = String::new();

                    match value.0 {
                        Expr::VarName((typ, name)) => {
                            match typ {
                                Types::Dynam(dynam_typ) => {
                                    match *dynam_typ {
                                        _ => {
                                            self.import_dynam();
                                            dynam_name = name.clone();
                                            dynam_var.push_str(&format!("dynam {name}=dynam_new();"));
                                        },
                                    }
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }

                    for v in value.1 {
                        match v {
                            Expr::StrLit(string) => {
                                let varname = rand_varname();
                                dynam_var.push_str(&format!("string {varname}=string_from(\"{string}\");"));
                                dynam_var.push_str(&format!("dynam_push(&{dynam_name},&{varname});"));
                            },
                            Expr::IntLit(integer) => {
                                let varname = rand_varname();
                                dynam_var.push_str(&format!("int {varname}={integer};"));
                                dynam_var.push_str(&format!("dynam_push(&{dynam_name},&{varname});"));
                            },
                            Expr::VarName((_, name)) => {
                                dynam_var.push_str(&format!("dynam_push(&{dynam_name},&{name});"));
                            },
                            _ => (),
                        }
                    }

                    self.code.push_str(&dynam_var);
                },
                Expr::Arr(value) => {
                    let mut arr_var = String::new();
                    let mut first_elem = true;

                    match value.0 {
                        Expr::VarName((typ, name)) => {
                            match typ {
                                Types::Arr(arr_typ) => {
                                    match *arr_typ {
                                        Types::Str => {
                                            self.import_dynam();
                                            arr_var.push_str(&format!("string {name}[]={{"));
                                        },
                                        Types::Int => arr_var.push_str(&format!("int {name}[]={{")),
                                        _ => (),
                                    }
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }

                    for v in value.1 {
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
                    self.code.push_str(&arr_var);
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
                        Expr::IntLit(integer) => re_arr.push_str(&format!("={integer};")),
                        Expr::StrLit(string) => re_arr.push_str(&format!("=\"{string}\";")),
                        Expr::VarName(_) => (),
                        _ => (),
                    }

                    self.code.push_str(&re_arr);
                },
                Expr::Var(value) => {
                    let mut variable = String::new();

                    match value.0 {
                        Expr::VarName((typ, name)) => {
                            match typ {
                                Types::Str => {
                                    self.import_dynam();
                                    variable.push_str(&format!("string {name}=string_from("))
                                },
                                Types::Int => variable.push_str(&format!("int {name}=")),
                                _ => (),
                            }
                        },
                        _ => (),
                    }

                    match value.1 {
                        Expr::StrLit(value) => variable.push_str(&format!("\"{value}\");")),
                        Expr::IntLit(value) => variable.push_str(&format!("{value};")),
                        _ => (),
                    }

                    self.code.push_str(&variable);
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
                            re_var.push_str(&format!("{var_name} = string_from(\"{string}\");"));
                        },
                        Expr::IntLit(integer) => {
                            re_var.push_str(&format!("{var_name} = {integer};"));
                        },
                        Expr::VarName((_, name)) => {
                            re_var.push_str(&format!("{var_name} = {name};"));
                        },
                        _ => (),
                    }

                    self.code.push_str(&re_var);
                },
                Expr::Print(value) => {
                    if !self.imports.contains("#include <stdio.h>\n") {
                        self.imports.push_str("#include <stdio.h>\n");
                    }

                    let mut print_code = String::from(format!("printf(\""));
                    let mut param_buf = String::new();
                    let mut var_buf = String::new();
                    let mut arr_name = String::new();
                    let mut is_arr = false;
                    let mut is_string = false; // FIX THIS LATER. WE NEEDA DO ACTUAL CALLING ARRAYS
                                               // AND A WAY TO PASS THE CONTENT OF A STRING INSTEAD
                                               // OF THE WHOLE STRUCT BRUHHH

                    for v in *value {
                        match v {
                            Expr::StrLit(string) => {
                                param_buf.push_str(&format!("{string}"));
                            },
                            Expr::IntLit(integer) => {
                                if is_arr {
                                    if is_string {
                                        var_buf.push_str(&format!(", {arr_name}[{integer}].data"));
                                        is_arr = false;
                                    } else {
                                        var_buf.push_str(&format!(", {arr_name}[{integer}]"));
                                        is_arr = false;
                                    }
                                } else {
                                    param_buf.push_str(&format!("%d"));
                                    var_buf.push_str(&format!(", {integer}"));
                                }
                            },
                            Expr::VarName((typ, name)) => {
                                match typ {
                                    Types::Arr(arr_typ) => {
                                        arr_name = name;
                                        is_arr = true;
                                        match *arr_typ {
                                            Types::Int => {
                                                param_buf.push_str(&format!("%d"));
                                            },
                                            Types::Str => {
                                                is_string = true;
                                                param_buf.push_str(&format!("%s"));
                                            },
                                            _ => (),
                                        }
                                    },
                                    Types::Int => {
                                        param_buf.push_str(&format!("%d"));
                                        var_buf.push_str(&format!(", {name}"));
                                    },
                                    Types::Str => {
                                        param_buf.push_str(&format!("%s"));
                                        var_buf.push_str(&format!(", {name}.data"));
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
                    self.code.push_str(&print_code);
                }
                Expr::EndBlock => {
                    self.code.push_str("}");
                },
                Expr::Func(f) => {
                    let ty = &f.0;
                    let params = &f.1;
                    let name = &f.2;
                    let mut f_ty = String::new();
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
                        Types::None => {
                            println!("\x1b[91merror\x1b[0m: line {}, unexpected type", index + 1);
                            exit(1)
                        },
                    }

                    let func = format!("{f_ty} {f_name}({f_params}) {{");
                    self.code.push_str(&func);
                },
                Expr::Return(value) => {
                    let mut ret = String::new();
                    match *value {
                        Expr::StrLit(string) => {
                            ret.push_str(&format!("return {string};"));
                        },
                        Expr::IntLit(integer) => {
                            println!("{integer}");
                            ret.push_str(&format!("return {integer};"));
                        },
                        Expr::VarName((_, name)) => {
                            ret.push_str(&format!("return {name};"));
                        },
                        _ => (),
                    }
                    self.code.push_str(&ret);
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
