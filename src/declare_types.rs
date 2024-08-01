pub enum Lang {
    C,
    Cpp,
}

#[derive(Debug, Clone)]
pub enum Types {
    U32,
    I32,

    U8,
    I8,
    Char,

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
    U32,
    I32,

    U8,
    I8,
    Char,

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
