# rust-jcs-transpiler

A transpiler written in rust, which translates C# code into Java code.

## How to use
- Place your **C#** code inside **input.cs**
- Run the program using 
    ```
    cargo run
    ```
- The resulting code can be found inside **output.java**

## Features
- Parses **C#** classes and methods
- Detects C# **Main()** and converts it into a Java **main(String[] args)** function
- Supports **variable declaration**
- Supports **assignments** and **expressions**, including
    - Binary expressions
    - Unary expressions
- Parses **if-else** statements
- Parses **for loops** and **while-loops**
- Handles **function calls**
- Handles **user output** and **input**
    - `Console.WriteLine` -> `System.out.println`
    - `Console.ReadLine` -> `Scanner` type input
    - `int.Parse`, `double.Parse`, `bool.Parse` -> `scanner.nextInt()`, etc
- Supports **nested function calls**, such as `int.Parse(Console.ReadLine())`
- Automatically declares a **Java Scanner** when user input is needed

## Limitations
- Does not support **switch**, **try-catch** and other advanced features
- Does not support arrays
- Only handles predefined types (int, bool, float, double, string)
- Invalid or unsupported C# syntax may cause the program to panic
