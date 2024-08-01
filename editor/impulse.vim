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
syntax keyword impulseAndOr and or
syntax keyword impulseKeywords break continue return
syntax keyword impulseBranches if orif else
syntax keyword impulseLoops loop for
syntax keyword impulseTypeDefs struct enum
syntax keyword impulseTypeNames int i32 u32 i8 u8 f32 f64 _ char bool usize
syntax keyword impulseTrueFalse true false
syntax keyword impulseTypeid typeid

syntax match impulseFuncCallName "\<\w\+\>\ze\s*("
syntax match impulseMacros "@\(import\|c\|array\|inline\)"
syntax match impulseIdent '\w\+\ze\.\w*('

syntax region impulseComment start="#.*" end="$"
syntax match impulseString /"\v[^"]*"/
syntax match impulseNumber "\<\d\+\>"
syntax match impulseEscapes /\\[nr\"']/

highlight link impulseKeywords Keyword
highlight link impulseBranches Conditional
highlight link impulseLoops Repeat
highlight link impulseTypeDefs Include 
highlight link impulseMacros Include
highlight link impulseComment Comment
highlight link impulseString String
highlight link impulseNumber Number
highlight link impulseTypeNames Type
highlight link impulseEscapes SpecialChar
highlight link impulseFuncCallName Function
highlight link impulseTrueFalse Function
highlight link impulseSymbols Operator
highlight link impulseAndOr Operator
highlight link impulseTypeid Define
highlight link impulseIdent Statement

let b:current_syntax = "impulse"
