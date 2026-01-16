#[cfg(test)]
mod tests {
    use rust_jcs_transpiler::{builder_java::build_program, generator_java::JavaGenerator};
    use tree_sitter::Parser;

    #[test]
    fn test_code_sample_1() {
        let input_code = r#"class Program {
    public static void Main() {
        Console.WriteLine("Hello World!");
    }
}
"#;

        let expected_code = r#"class Program {
    public static void main(String[] args) {
        System.out.println("Hello World!");
    }
}
"#;

        assert_eq!(expected_code, generate_code(input_code));
    }

    #[test]
    fn test_code_sample_2() {
        let input_code = r#"class Program {
    public static void Main() {
        int x = int.Parse(Console.ReadLine());
    }
}
"#;

        let expected_code = r#"import java.util.Scanner;
class Program {
    public static void main(String[] args) {
        Scanner scanner = new Scanner(System.in);
        int x = scanner.nextInt();
    }
}
"#;

        assert_eq!(expected_code, generate_code(input_code));
    }

    #[test]
    fn test_code_sample_3() {
        let input_code = r#"class Program {
    public static void Main() {
        int x = int.Parse(Console.ReadLine()) + 5;
        Console.WriteLine("Result: " + x);
    }
}
"#;

        let expected_code = r#"import java.util.Scanner;
class Program {
    public static void main(String[] args) {
        Scanner scanner = new Scanner(System.in);
        int x = scanner.nextInt() + 5;
        System.out.println("Result: " + x);
    }
}
"#;

        assert_eq!(expected_code, generate_code(input_code));
    }

    #[test]
    fn test_code_sample_4() {
        let input_code = r#"class Program {
    public static void Main() {
        double db = double.Parse(Console.ReadLine());
    }
}
"#;

        let expected_code = r#"import java.util.Scanner;
class Program {
    public static void main(String[] args) {
        Scanner scanner = new Scanner(System.in);
        double db = scanner.nextDouble();
    }
}
"#;

        assert_eq!(expected_code, generate_code(input_code));
    }

    #[test]
    fn test_code_sample_5() {
        let input_code = r#"class Program {
    public static void Main() {
        bool flag = bool.Parse(Console.ReadLine());
    }
}
"#;

        let expected_code = r#"import java.util.Scanner;
class Program {
    public static void main(String[] args) {
        Scanner scanner = new Scanner(System.in);
        boolean flag = scanner.nextBoolean();
    }
}
"#;

        assert_eq!(expected_code, generate_code(input_code));
    }
    /// This function works just like the main function
    fn generate_code(input_code: &str) -> String {
        let mut parser = Parser::new();
        let language = tree_sitter_c_sharp::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("Error loading C# parser");

        let tree = parser.parse(input_code, None).unwrap();

        let program = build_program(tree.root_node(), input_code);

        let java_code = JavaGenerator::generate(&program);

        java_code
    }
}
