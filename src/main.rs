use std::io::Write;
use std::path::PathBuf;
use std::{env, fs, process::exit};

use crate::tokeniser::*;
use crate::parser::*;
use crate::generator::*;
use crate::declare_types::Lang;

mod tokeniser;
mod parser;
mod generator;
mod declare_types;
mod type_checker;

fn initalise(dir: &String) {
    let file_res = fs::File::create(format!("{dir}/c_flags.txt"));
    match file_res {
        Ok(mut file) => {
            let _ = file.write_all(b"-O -Wall -Wextra -Wfloat-equal");
        },
        Err(e) => {
            println!("{e:?}");
            println!("\x1b[91merror\x1b[0m: unable to create c_flags config file");
            exit(1);
        },
    }
}

fn get_imp_files(dir: &String) -> Vec<PathBuf> {
    let paths_res = fs::read_dir(dir);
    match paths_res {
        Ok(paths) => {
            let p = paths.filter_map(|res| res.ok())
            .map(|entry| entry.path())
            .filter_map(|path| {
                if path.extension().map_or(false, |ext| ext == "imp") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
            return p
        }
        Err(_) => return vec![PathBuf::from(dir)],
    }
}

fn find_entry_point(files: Vec<PathBuf>) -> Option<(PathBuf, String)> {
    for file in files {
        let content = match fs::read_to_string(file.clone()) {
            Ok(c) => c,
            Err(_) => {
                println!("\x1b[91merror\x1b[0m: unable to read file: {file:?}");
                exit(1)
            },
        };

        if content.contains("main :: (") {
            return Some((file, content));
        }
    }

    None
}

fn setup_step(dir: &String) -> (PathBuf, Vec<(Expr, String, u32)>) {
    let (filename, content) = match find_entry_point(get_imp_files(dir)) {
        Some(c) => c,
        None => {
            println!("\x1b[91merror\x1b[0m: unable to find main function.");
            exit(1)
        }
    };

    let tokens = tokeniser(content.clone());
    let mut parse = ExprWeights::new(tokens, filename.to_str().unwrap());

    if !content.contains("@import \"base/builtin.imp\";") {
        parse.handle_import_macro(String::from("base/builtin.imp"));
    }

    let expressions = parse.parser();

    return (filename, expressions)
}

fn build(dir: &String, keep_gen: bool, lang: Lang) {
    let current_dir = env::current_dir();
    let path = match current_dir {
        Ok(path) => path,
        Err(e) => {
            println!("\x1b[91merror\x1b[0m: failed to get current directory name with error: {e:?}");
            exit(1)
        }
    };

    let out_filename = if let Some(folder_name) = path.file_name() {
        folder_name.to_str()
    } else {
        println!("\x1b[91merror\x1b[0m: can't find current directory name.");
        exit(1)
    };

    let (filename, expressions) = setup_step(dir);
    // for expr in &expressions {
    //     println!("{:?}", expr.0);
    // }

    let mut gen = Gen::new(filename.to_str().unwrap(), out_filename.unwrap(), true, keep_gen, lang);
    gen.generate(expressions);
}

fn transpile(dir: &String, lang: Lang) {
    let (filename, expressions) = setup_step(dir);
    // for expr in &expressions {
    //     println!("{:?}", expr.0);
    // }

    let mut gen = Gen::new(filename.to_str().unwrap(), "output", false, true, lang);
    gen.generate(expressions);
}

fn usage() {
    println!("Usage:");
    println!("impulse <command> [arguments]");
    println!();
    println!("-----------------------------------------------------");
    println!();
    println!("COMMANDS:");
    // println!("| -h: help | impulse -h |");
    println!("| init <directory>: initalise new project | impulse init . |");
    // println!("| -r: run | impulse -r FILE.imp OUTPUT_NAME |");
    println!("| build [--keep] <directory>: build project | impulse build . |");
    println!();
    println!("-----------------------------------------------------");
    println!();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        usage();
        println!("\x1b[91merror\x1b[0m: invalid usage");
        exit(1)
    }

    match args[1].as_str() {
        "init" => initalise(&args[2]),
        "build" => {
            if &args[2] == "--keep" {
                if args.len() < 4 {
                    println!("\x1b[91merror\x1b[0m: expected path after --keep");
                    exit(1)
                }

                build(&args[3], true, Lang::C);
                return
            }
            build(&args[2], false, Lang::C);
        },
        "transpile" => {
            // TODO: tidy this up and maybe use a hashmap with the available languages
            if &args[2] == "c" {
                if args.len() < 4 {
                    println!("\x1b[91merror\x1b[0m: expected path after language option");
                    exit(1)
                }

                transpile(&args[3], Lang::C);
            } else if &args[2] == "cpp" {
                if args.len() < 4 {
                    println!("\x1b[91merror\x1b[0m: expected path after language option");
                    exit(1)
                }

                transpile(&args[3], Lang::Cpp);
            } else {
                transpile(&args[2], Lang::C);
            }
        }
        _ => usage(),
    }
}
