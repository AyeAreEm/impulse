use std::collections::HashMap;
use crate::{declare_types::Types, parser::*};
use lazy_static::lazy_static;

pub enum TCError<'a> {
    FuncNotExist,
    GenericNotExist(&'a String),
    WrongArgLength(usize, usize),
    MismatchExprType(&'a Expr, &'a Types),
    // MismatchExprExpr(&'a Expr, &'a Expr),
    MismatchTypeType(&'a Types, &'a Types),
    Custom(String),
}

pub struct ArgError<'a> {
    pub pos: usize,
    pub error: TCError<'a>,
}

lazy_static! {
    static ref STRING_TYPES: HashMap<&'static str, Types> = {
        let string_types = HashMap::from([
            ("u8", Types::U8),
            ("i8", Types::I8),
            ("char", Types::Char),

            ("u16", Types::U16),
            ("i16", Types::I16),

            ("u32", Types::U32),
            ("i32", Types::I32),

            ("uint", Types::UInt),
            ("int", Types::Int),

            ("u64", Types::U64),
            ("i64", Types::I64),

            ("usize", Types::Usize),

            ("f32", Types::F32),
            ("f64", Types::F64),

            ("bool", Types::Bool),

            ("any", Types::Any),
        ]);
        string_types
    };
}

fn create_pointer(typ: Types, pointer_counter: i32) -> Types {
    let mut ret = typ.clone();

    for _ in 0..pointer_counter {
        ret = Types::Pointer(Box::new(ret));
    }

    ret
}

fn unwrap_pointer(t: &Types) -> &Types {
    match t {
        Types::Pointer(pointer_to) => return unwrap_pointer(pointer_to),
        _ => return t
    }
}

pub fn string_to_type(type_name: &String) -> Types {
    let mut buffer = String::new();
    let mut pointer_counter = 0;

    for (i, ch) in type_name.chars().enumerate() {
        if i == type_name.len() - 1{
            buffer.push(ch);

            let typ = match STRING_TYPES.get(buffer.as_str()) {
                Some(t) => t.clone(),
                None => Types::TypeDef { type_name: type_name.clone(), generics: None },
            };

            return create_pointer(typ, pointer_counter);
        } else if ch == '^' {
            pointer_counter += 1;
        } else {
            buffer.push(ch);
        }
    }

    return Types::None
}

fn get_return_type<'a, 'b>(func_name: &'a String, funcs: &'b Vec<Expr>) -> Option<&'b Types> {
    if funcs.is_empty() {
        return None
    }

    for current_name in funcs {
        match current_name {
            Expr::Func { typ, name, .. } | Expr::MacroFunc { typ, name, .. } => {
                let san_name = name.replace('.', "__");
                if &san_name == func_name {
                    return Some(typ);
                }
            }
            _ => unreachable!(),
        }
    }

    None
}

fn get_args<'a, 'b>(func_name: &'a String, funcs: &'b Vec<Expr>) -> Option<&'b Vec<Expr>> {
    if funcs.is_empty() {
        return None;
    }

    for current_name in funcs {
        match current_name {
            Expr::Func { params, name, .. } | Expr::MacroFunc { params, name, .. } => {
                let san_name = name.replace('.', "__");
                if &san_name == func_name {
                    return Some(params);
                }
            }
            _ => unreachable!(),
        }
    }

    None
}

fn compare_type_and_type(t1: &Types, t2: &Types) -> bool {
    match (t1, t2) {
        (Types::Any, _) => return true,
        (
            Types::U8 | Types::I8 | Types::U16 | Types::I16 | Types::U32 | Types::I32 | Types::Usize |
            Types::U64 | Types::I64 | Types::Int | Types::F32 | Types::F64 | Types::Char | Types::UInt,
            Types::U8 | Types::I8 | Types::U16 | Types::I16 | Types::U32 | Types::I32 | Types::Usize |
            Types::U64 | Types::I64 | Types::Int | Types::F32 | Types::F64 | Types::Char | Types::UInt
         ) => return true,
        (Types::Pointer(t1_subtype), Types::Pointer(t2_subtype)) => {
            let t1_unwrapped = unwrap_pointer(&t1_subtype);
            let t2_unwrapped = unwrap_pointer(&t2_subtype);
            if t1_unwrapped == &Types::Void || t2_unwrapped == &Types::Void { return true; }
            // else if t1_unwrapped == &Types::Generic || t2_unwrapped == &Types::Generic { return true; }
            else if let Types::Generic(_) = t1_unwrapped {
                return true;
            } else if let Types::Generic(_) = t2_unwrapped {
                return true;
            }

            return t1_unwrapped == t2_unwrapped;
        },
        (Types::TypeDef { type_name: t1_name, generics: t1_generics }, Types::TypeDef { type_name: t2_name, generics: t2_generics }) => {
            if t1_name != t2_name { return false }

            match (t1_generics, t2_generics) {
                (None, None) => return true,
                (Some(t1_gen), Some(t2_gen)) => {
                    // TODO: actually compare the generics
                    return t1_gen.len() == t2_gen.len()
                }
                (None, Some(_)) => return false,
                (Some(_), None) => return false,
            }
        }
        // TODO: maybe change the generic to the correct type when checking
        (Types::Generic(_), _) => return true,
        (_, Types::Generic(_)) => return true,
        _ => {
            return t1 == t2
        },
    }
}

pub fn compare_type_and_expr(t: &Types, e: &Expr, funcs: &Vec<Expr>) -> (bool, Types) {
    match (t, e) {
        (Types::Let, Expr::IntLit(_)) => return (true, Types::Int), 
        (Types::Let, Expr::CharLit(_)) => return (true, Types::Char),
        (Types::Let, Expr::StrLit(_)) => return (true, Types::Pointer(Box::new(Types::Char))),
        (Types::Let, Expr::True | Expr::False) => return (true, Types::Bool),
        (Types::Let, Expr::VariableName { typ, .. }) => {
            return (true, typ.clone())
        },
        (Types::Let, Expr::FuncCall { name, gave_params }) => {
            let ret_type_op = get_return_type(name, funcs);

            let mut types = Vec::new();
            match ret_type_op {
                Some(ret_type) => {
                    for param in gave_params {
                        if let Expr::VariableName { typ, name, .. } = param {
                            // if let Types::Pointer(pointer_to) = typ {
                            //     let unwrapped = unwrap_pointer(pointer_to);
                            //     if let Types::TypeId = unwrapped {
                            //         types.push(string_to_type(name));
                            //     }
                            // }
                            if let Types::TypeId = typ {
                                types.push(string_to_type(name));
                            }
                        } else if let Expr::StructDef { struct_name, .. } = param {
                            if let Expr::StructName { name, .. } = *struct_name.clone() {
                                types.push(string_to_type(&name))
                            }
                        }
                    }

                    if let Types::TypeDef { generics, type_name } = ret_type {
                        if let Some(gens) = generics {
                            if types.len() != gens.len() {
                                return (false, Types::None)
                            }

                            return (true, Types::TypeDef { type_name: type_name.to_owned(), generics: Some(types) })
                        }
                    }
                    return (true, ret_type.clone())
                },
                None => return (false, Types::None),
            }
        },
        // TODO: double check, think this is fine as parser handles this?
        (Types::Any, _) => return (true, Types::None),
        (Types::TypeId, _) => return (true, Types::None),
        (Types::Char, Expr::CharLit(_)) => return (true, Types::None),
        (
            Types::U8 | Types::I8 | Types::U16 | Types::I16 | Types::U32 | Types::I32 | Types::Usize |
            Types::U64 | Types::I64 | Types::Int | Types::F32 | Types::F64 | Types::Char | Types::UInt |
            Types::Pointer(_),
            Expr::IntLit(_)
        ) => return (true, Types::None),
        // TODO: maybe change the generic to the correct type when checking
        (Types::Generic(_), _) => return (true, Types::None),
        (Types::ArrIndex { arr_typ, .. }, _) => {
            let typ = unwrap_pointer(arr_typ);
            let compare = compare_type_and_expr(typ, e, funcs);
            return (compare.0, compare.1);
        }
        (_, Expr::VariableName { typ, .. }) => {
            return (compare_type_and_type(t, typ), Types::None);
        },
        (_, Expr::FuncCall { name, .. }) => {
            let ret_type_op = get_return_type(name, funcs);
            match ret_type_op {
                Some(ret_type) => return (compare_type_and_type(t, ret_type), Types::None),
                None => return (false, Types::None),
            }
        },
        (Types::Pointer(pointer_to), Expr::Address(address_to)) => {
            let compare = compare_type_and_expr(&pointer_to, address_to, funcs);
            return (compare.0, compare.1);
        },
        (Types::Pointer(pointer_to), Expr::StrLit(_)) => {
            return (compare_type_and_type(&pointer_to, &Types::Char), Types::None)
        }
        (Types::Pointer(pointer_to), _) => {
            let compare = compare_type_and_expr(&pointer_to, e, funcs);
            return (compare.0, compare.1)
        },
        // if you are using a CEmbed, you should know what you're doing. either way, gcc will pick it up
        (_, Expr::CEmbed(_)) => return (true, Types::None),
        (Types::TypeDef { .. }, Expr::ArrayLit(_)) => return (true, Types::None),
        (Types::Bool, Expr::True | Expr::False) => return (true, Types::None),
        _ => return (false, Types::None),
    }
}

pub fn compare_exprs_and_args<'a>(exprs: &'a Vec<Expr>, func_name: &'a String, funcs: &'a Vec<Expr>) -> Result<(), ArgError<'a>> {
    let args = match get_args(func_name, funcs) {
        Some(a) => a,
        None => return Err(ArgError { pos: 0, error: TCError::FuncNotExist }),
    };

    if exprs.len() != args.len() && (func_name != "print" && func_name != "println") {
        return Err(ArgError { pos: 0, error: TCError::WrongArgLength(exprs.len(), args.len()) });
    }

    let mut skip_check = false;
    if func_name == "print" || func_name == "println" {
        match exprs[0] {
            Expr::StrLit(_) => skip_check = true,
            _ => return Err(ArgError { pos: 1, error: TCError::Custom(format!("expected StrLit as first argument, got {:?}", exprs[0])) }) 
        }
    }

    if skip_check { return Ok(()) }
    let mut typeid_name_to_type: HashMap<&String, &String> = HashMap::new();
    for (i, expr) in exprs.iter().enumerate() {
        match (&args[i], expr) {
            (Expr::VariableName { typ: arg_typ, name: arg_name, ..}, _) => {
                if let Expr::VariableName { typ: expr_typ, name: expr_name, .. } = expr {
                    if let Types::TypeId = expr_typ {
                        if expr_typ != arg_typ {
                            return Err(ArgError { pos: i + 1, error: TCError::MismatchTypeType(arg_typ, expr_typ) })
                        }

                        typeid_name_to_type.entry(arg_name).or_insert(expr_name);
                    }
                } else if let Types::Generic(generic) = arg_typ {
                    let value = match typeid_name_to_type.get(generic) {
                        Some(v) => v,
                        None => return Err(ArgError { pos: i + 1, error: TCError::GenericNotExist(generic) })
                    };

                    let arg_real_typ = string_to_type(value);
                    if !compare_type_and_expr(&arg_real_typ, expr, funcs).0 {
                        return Err(ArgError { pos: i + 1, error: TCError::Custom(format!("argument {} expected type {arg_real_typ:?}, got {expr:?}", i + 1)) });
                    }
                }

                if !compare_type_and_expr(&arg_typ, expr, funcs).0 {
                    return Err(ArgError { pos: i + 1, error: TCError::MismatchExprType(expr, arg_typ) });
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
