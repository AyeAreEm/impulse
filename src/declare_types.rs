pub enum Lang {
    C,
    JS,
}

#[derive(Debug, Clone)]
pub enum Types {
    I32,
    Str,
    Arr {
        typ: Box<Types>,
        length: String,
    },
    ArrIndex {
        arr_typ: Box<Types>,
        index_at: String,
    },

    Dynam(Box<Types>),
    Void,

    TypeDef(String),
    None,
}

#[derive(Debug, Clone)]
pub enum Keyword {
    I32,
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
