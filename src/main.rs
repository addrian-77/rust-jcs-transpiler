use std::{
    fs::File,
    io::{Read, Write},
};

// use rust_jcs_transpiler::parser_cs::find_everything;
use rust_jcs_transpiler::{builder_java::*, generator_java::*};
use tree_sitter::Parser;
use tree_sitter_c_sharp;

fn main() {
    let mut input = File::open("input.cs").expect("Input file not present!");
    let mut input_string = String::new();
    input
        .read_to_string(&mut input_string)
        .expect("Failed to read file");
    let code = input_string.as_str();
    // create a new parser
    let mut parser = Parser::new();
    // set the programming language
    let language = tree_sitter_c_sharp::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("Error loading C# parser");

    // parse the code, create a tree containing everything
    let tree = parser.parse(code, None).unwrap();

    // print_tree(tree.root_node(), 0);
    // find_everything(tree.root_node(), code, 0);

    // build the program using our parser
    let program = build_program(tree.root_node(), code);
    // println!("program? {:#?}", program);
    // build the program based on the ast
    let java_code = JavaGenerator::generate(&program);
    let mut output = File::create("output.java").expect("Failed to create output file");
    output
        .write_all(java_code.as_bytes())
        .expect("Failed to write to file");
}
