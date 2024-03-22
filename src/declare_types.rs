pub enum Lang {
    C,
    JS,
}

#[derive(Debug, Clone)]
pub enum Types {
    Int,
    Str,
    Arr(Box<Types>),
    Void,
    None,
}

#[derive(Debug, Clone)]
pub enum Keyword {
    Int,
    Str,
    Print,
    Underscore,
    None,
}

#[derive(Debug, Clone)]
pub enum Macros {
    Import,
    Arr,
    None,
}
