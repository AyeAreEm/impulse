pub enum Lang {
    C,
    JS,
}

#[derive(Debug, Clone)]
pub enum Types {
    Int,
    Str,
    Arr(Box<Types>),
    Dynam(Box<Types>),
    Void,
    None,
}

#[derive(Debug, Clone)]
pub enum Keyword {
    Int,
    Str,

    Println,
    Print,
    ReadIn,

    Underscore,
    Return,

    If,
    OrIf,
    Else,

    Loop,

    Or,
    And,

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
