#![allow(clippy::new_without_default)]
#![allow(clippy::len_zero)]
pub mod compiler;

pub mod instruction;
pub mod vm;

use instruction::Val;

pub fn run_bytecode(src: &str) -> Result<(Option<Val>, vm::VM), String> {
    use logos::Logos;
    use oz_lexer::Token;

    let lexer = Token::lexer(src);
    let mut tokens = Vec::new();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => return Err(format!("Lexer hatası: {:?}", span)),
        }
    }

    let ast = oz_parser::parse_tokens(tokens, src.len()).map_err(|e| format!("{:?}", e))?;
    oz_parser::typechecker::check_program(&ast)?;
    let compiler = compiler::Compiler::new();
    let insts = compiler.compile_program(&ast)?;

    let mut vm = vm::VM::new(insts);
    vm.run()?;
    Ok((None, vm))
}

#[cfg(test)]
mod tests;
