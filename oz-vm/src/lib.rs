pub mod instruction;
pub mod compiler;
pub mod vm;

use instruction::Val;

pub fn run_bytecode(src: &str) -> Result<(Option<Val>, vm::VM), String> {
    use oz_lexer::Token;
    use logos::Logos;

    let lexer = Token::lexer(src);
    let mut tokens = Vec::new();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => return Err(format!("Lexer hatası: {:?}", span)),
        }
    }

    let ast = oz_parser::parse_tokens(tokens, src.len()).map_err(|e| format!("{:?}", e))?;
    let compiler = compiler::Compiler::new();
    let insts = compiler.compile_program(&ast)?;

    let mut vm = vm::VM::new(insts);
    vm.run()?;
    Ok((None, vm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use instruction::Val;

    #[test]
    fn test_kosul() {
        let src = include_str!("../../examples/ornek1_kosul.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("sayı"), Some(Val::Number(8.0)));
    }

    #[test]
    fn test_dongu() {
        let src = include_str!("../../examples/ornek2_dongu.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("sayaç"), Some(Val::Number(4.0)));
    }

    #[test]
    fn test_islev() {
        let src = include_str!("../../examples/ornek3_islev.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("sonuç"), Some(Val::Number(30.0)));
    }

    #[test]
    fn test_hesap() {
        let src = include_str!("../../examples/ornek4_hesap.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("toplam"), Some(Val::Number(19.0)));
        assert_eq!(vm.get_global("kalan"), Some(Val::Number(3.0)));
    }

    #[test]
    fn test_karma() {
        let src = include_str!("../../examples/ornek5_karma.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("limit"), Some(Val::Number(5.0)));
    }
}
