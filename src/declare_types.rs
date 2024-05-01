pub enum Lang {
    C,
    JS,
}

#[derive(Debug, Clone)]
pub enum Types {
    I32,
    U8,
    I8,
    // Str,
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

    TypeDef(String),
    None,
}

#[derive(Debug, Clone)]
pub enum Keyword {
    I32,
    U8,
    I8,
    Str,

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

    Or,
    And,
    
    Pointer(Types, Types),
    Address,

    Struct,
    TypeDef(String),
    None,
}

#[derive(Debug, Clone)]
pub enum Macros {
    C,
    Import,
    Arr,
    Dynam,
    None,
}
