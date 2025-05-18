mod code_generator;
mod executor;
mod lexer;
mod parser;

fn main() {
    let tokens = lexer::parse(&std::fs::read_to_string("main.abys").unwrap());
    let ast = parser::parse_program(tokens);
    let codes = code_generator::generate(ast);
    executor::execute(codes);
}
