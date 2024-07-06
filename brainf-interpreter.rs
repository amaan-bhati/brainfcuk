use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

fn main() {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();

    let mut output_name = String::from("bf.out");
    let mut c_output_name = String::from("bf.out.c");
    let mut verbose = false;
    let mut keep_file = false;
    let mut no_compile = false;

    // Parse command line arguments
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => output_name = args.next().unwrap(),
            "-k" => keep_file = true,
            "-c" => c_output_name = args.next().unwrap(),
            "-n" => {
                no_compile = true;
                keep_file = true;
            }
            "-v" => verbose = true,
            "-h" | "--help" => {
                usage(&program_name);
                return;
            }
            _ => eprintln!("Invalid argument: {}", arg),
        }
    }

    // Check for input file
    let input_file = match args.next() {
        Some(filename) => filename,
        None => {
            eprintln!("Error: No input file specified");
            usage(&program_name);
            return;
        }
    };

    // Open input file
    let mut infile = match OpenOptions::new().read(true).open(&input_file) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening input file: {}", err);
            return;
        }
    };

    // Open output file
    let mut outfile = match OpenOptions::new().write(true).create(true).open(&c_output_name) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening output file: {}", err);
            return;
        }
    };

    // Generate C source code
    generate(&mut outfile, &mut infile, verbose);

    // Compile (if not disabled)
    if !no_compile {
        compile(&output_name, &c_output_name, verbose);
    }

    // Cleanup (if not keeping the file)
    if !keep_file {
        if verbose {
            println!("Deleting intermediate file {}...", c_output_name);
        }
        match std::fs::remove_file(&c_output_name) {
            Ok(_) => println!("Ok"),
            Err(err) => eprintln!("Error deleting file: {}", err),
        }
    }
}

fn generate<R: Read, W: Write>(outfile: &mut W, infile: &mut R, verbose: bool) {
    let mut buffer = [0; 1];

    fprintf!(outfile, "/*\n * This source was automatically generated with\n * obfc - The \"brainfuck compiler\"\n * \n */\n");
    fprintf!(outfile, "#include <stdio.h>\n");
    fprintf!(outfile, "int main() {\nchar a[30000]; char *ptr = a;\n");

    while infile.read(&mut buffer).unwrap() > 0 {
        let c = buffer[0] as char;
        match c {
            '>' => fprintf!(outfile, "ptr++;\n"),
            '<' => fprintf!(outfile, "ptr--;\n"),
            '+' => fprintf!(outfile, "++*ptr;\n"),
            '-' => fprintf!(outfile, "--*ptr;\n"),
            '[' => fprintf!(outfile, "while(*ptr){\n"),
            ']' => fprintf!(outfile, "}\n"),
            '.' => fprintf!(outfile, "putchar(*ptr);\n"),
            ',' => fprintf!(outfile, "*ptr=getchar();\n"),
            _ => {} // Ignore unknown characters
        }
    }

    fprintf!(outfile, "return 0;\n}\n");

    if verbose {
        println!("Generated C source code");
    }
}

fn compile(output_name: &str, c_output_name: &str, verbose: bool) {
    let mut command = format!("gcc -o {} {}", output_name, c
