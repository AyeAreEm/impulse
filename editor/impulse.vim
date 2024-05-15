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

syntax match impulseSymbols /\v[;:=<>!&^()[\]{}*\/]/
syntax keyword impulseKeywords break continue return
syntax keyword impulseBranches if orif else
syntax keyword impulseLoops loop
syntax keyword impulseDataStruct struct
syntax keyword impulseTypeNames int i32 i8 u8 _ char bool
syntax keyword impulseTrueFalse true false

syntax match impulseFuncCallName "\<\w\+\>\ze\s*("
syntax match impulseMacros "@\(import\|c\|array\)"

syntax region impulseComment start="#.*" end="$"
syntax match impulseString /"\v[^"]*"/
syntax match impulseNumber "\<\d\+\>"
syntax match impulseEscapes /\\[nr\"']/

highlight link impulseKeywords Keyword
highlight link impulseBranches Conditional
highlight link impulseLoops Repeat
highlight link impulseDataStruct Structure
highlight link impulseMacros Include
highlight link impulseComment Comment
highlight link impulseString String
highlight link impulseNumber Number
highlight link impulseTypeNames Type
highlight link impulseEscapes SpecialChar
highlight link impulseFuncCallName Function
highlight link impulseTrueFalse Function
highlight link impulseSymbols Operator

let b:current_syntax = "impulse"
