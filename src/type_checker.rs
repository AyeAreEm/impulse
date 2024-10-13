use crate::{declare_types::Types, parser::*};

pub enum TCError<'a> {
    FuncNotExist,
    WrongArgLength(usize, usize),
    MismatchExprType(&'a Expr, &'a Types),
    // MismatchExprExpr(&'a Expr, &'a Expr),
    // MismatchTypeType(&'a Types, &'a Types),
    Custom(String),
}

pub struct ArgError<'a> {
    pub pos: usize,
    pub error: TCError<'a>,
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
                    println!("{name}: {typ:?}");
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

fn unwrap_pointer(t: &Types) -> &Types {
    match t {
        Types::Pointer(pointer_to) => return unwrap_pointer(pointer_to),
        _ => return t
    }
}

fn compare_type_and_type(t1: &Types, t2: &Types) -> bool {
    match (t1, t2) {
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

        // TODO: maybe change the generic to the correct type when checking
        (Types::Generic(_), _) => return true,
        (_, Types::Generic(_)) => return true,
        _ => {
            return t1 == t2
        },
    }
}

pub fn compare_type_and_expr(t: &Types, e: &Expr, funcs: &Vec<Expr>) -> bool {
    match (t, e) {
        // TODO: double check, think this is fine as parser handles this?
        (Types::TypeId, _) => return true,
        (Types::Char, Expr::CharLit(_)) => return true,
        (
            Types::U8 | Types::I8 | Types::U16 | Types::I16 | Types::U32 | Types::I32 | Types::Usize |
            Types::U64 | Types::I64 | Types::Int | Types::F32 | Types::F64 | Types::Char | Types::UInt |
            Types::Pointer(_),
            Expr::IntLit(_)
        ) => return true,
        // TODO: maybe change the generic to the correct type when checking
        (Types::Generic(_), _) => return true,
        (Types::ArrIndex { arr_typ, .. }, _) => {
            let typ = unwrap_pointer(arr_typ);
            return compare_type_and_expr(typ, e, funcs);
        }
        (_, Expr::VariableName { typ, .. }) => {
            return compare_type_and_type(t, typ);
        },
        (_, Expr::FuncCall { name, .. }) => {
            let ret_type_op = get_return_type(name, funcs);
            match ret_type_op {
                Some(ret_type) => return compare_type_and_type(t, ret_type),
                None => return false,
            }
        },
        (Types::Pointer(pointer_to), Expr::StrLit(_)) => {
            return compare_type_and_type(&pointer_to, &Types::Char)
        }
        (Types::Pointer(pointer_to), _) => return compare_type_and_expr(pointer_to, e, funcs),
        // if you are using a CEmbed, you should know what you're doing. either way, gcc will pick it up
        (_, Expr::CEmbed(_)) => true,
        (Types::Bool, Expr::True | Expr::False) => true,
        _ => return false,
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
    for (i, expr) in exprs.iter().enumerate() {
        match (&args[i], expr) {
            (Expr::VariableName { typ, ..}, _) => {
                if !compare_type_and_expr(&typ, expr, funcs) {
                    return Err(ArgError { pos: i + 1, error: TCError::MismatchExprType(expr, typ) });
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
