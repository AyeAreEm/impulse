use std::io::Write;
use std::{env, fs, process::exit};

use fs_extra::copy_items;
use fs_extra::dir;

use crate::tokeniser::*;
use crate::parser::*;
use crate::generator::*;
use crate::declare_types::Lang;

mod tokeniser;
mod parser;
mod generator;
mod declare_types;

fn initalise(dir: &String) {
    let path_to_base = format!("{}/base", env::current_dir().unwrap().as_os_str().to_str().unwrap().to_string());

    let options = dir::CopyOptions::new();

    match copy_items(&vec![path_to_base], dir, &options) {
        Ok(_) => (),
        Err(e) => match e.kind {
            fs_extra::error::ErrorKind::AlreadyExists => (),
            _ => {
                println!("{e:?}");
                println!("\x1b[91merror\x1b[0m: unable to copy base standard library during initalising");
                exit(1);
            }
        }
    }

    let file_res = fs::File::create(format!("{dir}/c_flags.txt"));
    match file_res {
        Ok(mut file) => {
            let _ = file.write_all(b"-O2");
        },
        Err(e) => {
            println!("{e:?}");
            println!("\x1b[91merror\x1b[0m: unable to create c_flags config file");
            exit(1);
        },
    }
}

fn build(filename: &String, out_filename: &String, compile: bool, keep_gen: bool, lang: Lang) {
    let file_res = fs::read_to_string(filename);
    let content = match file_res {
        Ok(content) => content,
        Err(_) => {
            println!("\x1b[91merror\x1b[0m: unable to read file: {filename}");
            exit(1)
        },
    };

    let tokens = tokeniser(content.clone());
    let mut parse = ExprWeights::new(tokens, filename);

    if !content.contains("@import \"base/builtin.imp\";") {
        parse.handle_import_macro(&String::from("base/builtin.imp"));
    }

    let expressions = parse.parser();
    for expr in &expressions {
        println!("{:?}", expr.0);
    }

    let mut gen = Gen::new(filename.to_string(), out_filename.clone(), compile, keep_gen, lang);
    gen.generate(expressions);
}

fn usage_c() {
    println!("| -c: generate c file | impulse -c FILE.imp OUTPUT_NAME |");
}

fn usage_cpp() {
    println!("| -cpp: generate c++ file | impulse -cpp FILE.imp OUTPUT_NAME |");
}

fn usage_build() {
    println!("| -b [-c / -cpp]: generate c or c++ file then build | impulse -b FILE.imp OUTPUT_NAME |");
}

fn usage_init() {
    println!("| -init [__path__]: initalise new impulse project | impulse -init . |");
}

fn usage() {
    println!("USAGE:");
    println!("impulse <COMMAND> [file.imp] <OUTPUT_NAME>");
    println!("cargo run -- <COMMAND> [file.imp] <OUTPUT_NAME>");
    println!();
    println!("-----------------------------------------------------");
    println!();
    println!("COMMANDS:");
    println!("| -h: help | impulse -h |");
    usage_init();
    println!("| -r: run | impulse -r FILE.imp OUTPUT_NAME |");
    usage_build();
    usage_c();
    usage_cpp();
    println!();
    println!("-----------------------------------------------------");
    println!();
}

fn incorrect_usage(args: &Vec<String>, usage_type: fn()) {
    if args.len() < 4 {
        println!("USAGE: ");
        usage_type();
        println!();
        println!("-----------------------------------------------------");
        println!();
        exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        println!("\x1b[91merror\x1b[0m: invalid usage");
        exit(1)
    }

    match args[1].as_str() {
        "-init" => {
            // incorrect_usage(&args, usage_init);
            initalise(&args[2]);
        },
        "-b" => {
            if args[2] == "-c" {
                incorrect_usage(&args, usage_build);
                build(&args[3], &args[4], true, true, Lang::C);
            } else if args[2] == "-cpp" {
                incorrect_usage(&args, usage_build);
                build(&args[3], &args[4], true, true, Lang::Cpp);
            } else {
                incorrect_usage(&args, usage_build);
                build(&args[2], &args[3], true, false, Lang::C);
            }
        },
        "-c" => {
            incorrect_usage(&args, usage_c);
            build(&args[2], &args[3], false, true, Lang::C);
        },
        "-cpp" => {
            incorrect_usage(&args, usage_cpp);
            build(&args[2], &args[3], false, true, Lang::Cpp);
        },
        "-r" => {
            println!("run step")
        },
        "-h"  => usage(),
        _ => {
            usage();
            println!("\x1b[91merror\x1b[0m: unknown command, \x1b[93m{}\x1b[0m", &args[1]);
            exit(1)
        }
    }
}
