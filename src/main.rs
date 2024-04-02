use std::{env, fs, process::exit};

use crate::tokeniser::*;
use crate::parser::*;
use crate::generator::*;
use crate::declare_types::Lang;

mod tokeniser;
mod parser;
mod generator;
mod declare_types;

fn build(filename: &String, out_filename: &String, keep_c: bool) {
    let file_res = fs::read_to_string(filename);
    let content = match file_res {
        Ok(content) => content,
        Err(_) => {
            println!("\x1b[91merror\x1b[0m: unable to read file");
            exit(1)
        },
    };

    let tokens = tokeniser(content);
    // for token in &tokens {
    //     println!("{:?}", token);
    // }

    let mut parse = ExprWeights::new(tokens);
    let expressions = parse.parser();
    // for expr in &expressions {
    //     println!("{:?}", expr);
    // }

    let mut gen = Gen::new(out_filename.clone(), Lang::C);
    gen.generate(expressions);

    if !keep_c {
        match fs::remove_file("output.c") {
            Ok(_) => (),
            Err(_) => {
                println!("\x1b[91merror\x1b[0m: error handling code generation.");
                exit(1)
            },
        }
    }
}

fn usage_c() {
    println!("| -c: generate c file | impulse -b FILE.imp OUTPUT_NAME |");
}

fn usage_build() {
    println!("| -b: build | impulse -b FILE.imp OUTPUT_NAME |");
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
    println!("| -r: run | impulse -b FILE.imp OUTPUT_NAME |");
    usage_build();
    usage_c();
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
    if args.len() < 1 {
        usage();
        println!("\x1b[91merror\x1b[0m: invalid usage");
        exit(1)
    }

    if &args[1] == "-h" {
        usage();
    } else if &args[1] == "-b" {
        incorrect_usage(&args, usage_build);
        build(&args[2], &args[3], false);
    } else if &args[1] == "-c" {
        incorrect_usage(&args, usage_c);
        build(&args[2], &args[3], true);
    } else if &args[1] == "-js" {
        println!("js step")
    } else if &args[1] == "-r" {
        println!("run step")
    } else {
        usage();
        println!("\x1b[91merror\x1b[0m: unknown command, \x1b[93m{}\x1b[0m", &args[1]);
        exit(1)
    }
}
