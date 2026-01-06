use rust_jcs_transpiler::{builder_java::*, generator_java::*, parser_cs::find_everything};
use tree_sitter::Parser;
use tree_sitter_c_sharp;

fn main() {
    let code = r#"
    using System;

    class Program
    {
        public static void Main(string[] args, int a, int b)
        {
            bool init_b = true;
            bool fc;
            int c = 2;
            int e = c + 3;
            int f = 2 + 3 + c + e;
            float d = 2.3f;
            double dbvar = 5.33d;
            string s = "hello?";
            int arr[3] = {1, 2, 3};
            bool fas = !init_b && init_b;

            if (init_b == true) {
                c = c + c;
            }

            if (!init_b == true) {
                c = c + c;
            }

            if (e > c)
            {
                for (int i = 0; i < 5; i++)
                {
                    while (e > 0)
                    {
                        e--;
                    }
                }
            }

            e = c;
            e = e + c;

            Console.WriteLine("a+b" + a);
            Console.WriteLine("Hello World!");    
        }
    }"#;

    let mut parser = Parser::new();
    let language = tree_sitter_c_sharp::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("Error loading C# parser");

    let tree = parser.parse(code, None).unwrap();
    // print_tree(tree.root_node(), 0);
    find_everything(tree.root_node(), code, 0);
    let program = build_program(tree.root_node(), code);
    // println!("program? {:#?}", program);
    let java_code = JavaGenerator::generate(&program);
    println!("{java_code}");
}
