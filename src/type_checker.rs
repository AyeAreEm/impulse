use crate::{declare_types::Types, parser::*};

fn get_return_type<'a, 'b>(func_name: &'a String, funcs: &'b Vec<Expr>) -> Option<&'b Types> {
    if funcs.is_empty() {
        return None
    }

    for current_name in funcs {
        match current_name {
            Expr::Func { typ, name, .. } | Expr::MacroFunc { typ, name, .. } => {
                let san_name = name.replace('.', "__");
                if &san_name == func_name {
                    println!("{func_name}: {typ:?}");
                    return Some(typ);
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

pub fn compare_type_and_expr(t: &Types, e: &Expr, funcs: &Vec<Expr>) -> bool {
    match (t, e) {
        (
            Types::U8 | Types::I8 | Types::U16 | Types::I16 | Types::U32 | Types::I32 | Types::Usize |
            Types::U64 | Types::I64 | Types::Int | Types::F32 | Types::F64 | Types::Char | Types::UInt,
            Expr::IntLit(_)
        ) => {
            true
        },
        // TODO: maybe change the generic to the correct type when checking
        (Types::Generic(_), _) => return true,
        (Types::ArrIndex { arr_typ, .. }, Expr::VariableName { typ, .. }) => {
            if let Types::Pointer(pointer_to) = *arr_typ.clone() {
                let pointer_to_type = unwrap_pointer(&pointer_to);
                let type_unwrapped = unwrap_pointer(typ);
                return pointer_to_type == type_unwrapped
            }

            return **arr_typ == *typ
        },
        (Types::Pointer(pointer_to), Expr::VariableName { typ, .. }) => {
            let pointer_to_type = unwrap_pointer(pointer_to);
            let type_unwrapped = unwrap_pointer(typ);
            return pointer_to_type == type_unwrapped
        },
        (_, Expr::VariableName { typ, .. }) => {
            return t == typ 
        },
        (_, Expr::FuncCall { name, .. }) => {
            let ret_type_op = get_return_type(name, funcs);
            match ret_type_op {
                Some(ret_type) => if ret_type == t {
                    return true
                } else {
                    return false
                },
                None => return false,
            }
        },
        // if you are using a CEmbed, you should know what you're doing. either way, gcc will pick it up
        (_, Expr::CEmbed(_)) => true,
        (Types::Bool, Expr::True | Expr::False) => true,
        _ => return false,
    }
}
