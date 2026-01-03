use crate::{ast::*, parser_cs::*};
use tree_sitter::Node;

pub fn build_program(root: Node, source: &str) -> Program {
    let mut classes = Vec::new();
    find_classes(root, source, &mut classes);
    Program { classes }
}
