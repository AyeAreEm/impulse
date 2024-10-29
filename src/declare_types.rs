pub enum Lang {
    C,
    Cpp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    UInt,
    Int,

    Usize,

    F32,
    F64,

    Bool,

    TypeId,
    Any,
    Let,

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
        generics: Option<Vec<Types>>,
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

    UInt,
    Int,

    F32,
    F64,

    Bool,

    TypeId,
    Any,
    Let,

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

    Switch,
    Case,
    Fall,

    Loop,
    For,

    Or,
    And,
    
    Pointer(Types, Types),
    Address,

    Arr {
        typ: Box<Types>,
        length: String,
    },

    Struct,
    Enum,
    TypeDef {
        type_name: String,
        generics: Option<Vec<Types>>,
    },

    Defer,
    None,
}

#[derive(Debug, Clone)]
pub enum Macros {
    C,
    Mut,
    Import,
    Inline,
    Shared,
    Default,
    Garbage,
    None,
}
