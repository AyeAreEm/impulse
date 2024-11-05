" Language: Impulse
" Instructions:
" Move this file into .vim/syntax/impulse.vim or
" .config/nvim/syntax/impulse.vim
"
" in .vimrc, put
" autocmd BufRead,BufNewFile *.imp set filetype=impulse

if exists("b:current_syntax")
    finish
endif

syntax match impulseSymbols /\v[|$+%-;:=<>!&^()[\]{}*\/]/
syntax keyword impulseAndOr and or xor lshift rshift
syntax keyword impulseKeywords break continue return
syntax keyword impulseBranches if orif else switch case fall
syntax keyword impulseLoops loop for
syntax keyword impulseTypeDefs struct enum
syntax keyword impulseTypeNames int uint i32 u32 i8 u8 i16 u16 i64 u64 f32 f64 _ char bool usize str string
syntax keyword impulseTrueFalse true false
syntax keyword impulseTypeid typeid any let
syntax keyword impulseDefer defer
syntax keyword impulseBuiltin println print

syntax match impulseFuncCallName "\<\w\+\>\ze\s*("
syntax match impulseMacros "@\(import\|c\|inline\|shared\|default\|garbage\|mut\)"
syntax match impulseIdent '\w\+\ze\.\w*('
syntax match impulseFuncDef "\v\w+\ze\s*::\s*\("
syntax match impulseNamespaceFuncDef "\v\w+\ze\s*\.\s*\w+\s*::\s*\("

syntax region impulseComment start="#.*" end="$"
syntax match impulseString /"\v[^"]*"/ contains=impulseEscapes
syntax match impulseChar "'[^'\\]\{1,1}'" contains=impulseEscapes
syntax match impulseNumber "\<\d\+\>\(\w\)\@!"
syntax match impulseHex /\<0x[0-9A-Fa-f]\+\>/
syntax match impulseBinary /\<0b[0-1]\+\>/
syntax match impulseEscapes /\\[nr\"']/

highlight link impulseKeywords Keyword
highlight link impulseBranches Conditional
highlight link impulseLoops Repeat
highlight link impulseTypeDefs Include

highlight link impulseMacros Include
highlight link impulseComment Comment
highlight link impulseString String
highlight link impulseChar String
highlight link impulseNumber Number
highlight link impulseHex Number
highlight link impulseBinary Number
highlight link impulseTypeNames Type
highlight link impulseEscapes SpecialChar
highlight link impulseFuncCallName Function
highlight link impulseFuncDef Function
highlight link impulseTrueFalse Function
highlight link impulseSymbols Operator
highlight link impulseAndOr Operator
highlight link impulseTypeid Define
highlight link impulseIdent Statement
highlight link impulseNamespaceFuncDef Statement
highlight link impulseDefer Error
highlight link impulseBuiltin Error

let b:current_syntax = "impulse"
