use rust_jcs_transpiler::{builder_java::*, generator_java::*};
use tree_sitter::Parser;
use tree_sitter_c_sharp;

fn main() {
    let code = r#"
    class Test {
        int Add(int a, int b) {
            return a + b;
        }
    }"#;

    let mut parser = Parser::new();
    let language = tree_sitter_c_sharp::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("Error loading C# parser");

    let tree = parser.parse(code, None).unwrap();
    // print_tree(tree.root_node(), 0);
    let program = build_program(tree.root_node(), code);
    // println!("program? {:#?}", program);
    // find_everything(tree.root_node(), code);
    let java_code = JavaGenerator::generate(&program);
    println!("{java_code}");
}
