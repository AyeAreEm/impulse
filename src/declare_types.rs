pub enum Lang {
    C,
    Cpp,
}

#[derive(Debug, Clone)]
pub enum Types {
    U8,
    I8,
    Char,

    U16,
    I16,

    U32,
    I32,

    U64,
    I64,

    Usize,
    Int,

    F32,
    F64,

    Bool,
    TypeId,
    Generic(String),

    Arr {
        typ: Box<Types>,
        length: String,
    },
    ArrIndex {
        arr_typ: Box<Types>,
        index_at: String,
    },

    Pointer(Box<Types>),
    Address,

    // Dynam(Box<Types>),
    Void,

    TypeDef {
        type_name: String,
        generics: Option<Vec<String>>,
    },
    None,
}

#[derive(Debug, Clone)]
pub enum Keyword {
    U8,
    I8,
    Char,

    U16,
    I16,

    U32,
    I32,

    U64,
    I64,

    Usize,
    Int,

    F32,
    F64,

    Bool,
    TypeId,
    Generic(String),

    Println,
    Print,
    ReadIn,

    Underscore,
    Return,
    Break,
    Continue,

    If,
    OrIf,
    Else,

    Loop,
    For,

    Or,
    And,
    
    Pointer(Types, Types),
    Address,

    Struct,
    Enum,
    TypeDef {
        type_name: String,
        generics: Option<Vec<String>>,
    },
    None,
}

#[derive(Debug, Clone)]
pub enum Macros {
    C,
    Import,
    Arr,
    Inline,
    Dynam,
    None,
}
