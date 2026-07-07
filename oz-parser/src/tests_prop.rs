#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests_prop {
    use logos::Logos;
    use proptest::prelude::*;

    /// Rastgele aritmetik ifadeler üreten strateji
    fn arith_expr_strategy() -> impl Strategy<Value = String> {
        let operands = prop_oneof![
            (1i64..1000).prop_map(|n| n.to_string()),
            Just("doğru".to_string()),
            Just("yanlış".to_string()),
            Just("boş".to_string()),
            "\"[a-zçğışöü]{1,10}\"".prop_map(|s| s),
        ];

        let operators = prop_oneof![
            Just("+"),
            Just("-"),
            Just("*"),
            Just("/"),
            Just("%"),
            Just("=="),
            Just("!="),
            Just("<"),
            Just(">"),
        ];

        // basit: operand op operand;
        (operands.clone(), operators, operands)
            .prop_map(|(a, op, b)| format!("sonuc = {} {} {};\n", a, op, b))
    }

    // TİLK kodunun parse + tip çıkarımı aşamasında panic vermemesini doğrula
    proptest! {
        #[test]
        fn parse_asla_panic_vermez(kod in arith_expr_strategy()) {
            let lexer = oz_lexer::Token::lexer(&kod);
            let mut tokens = Vec::new();
            for (token_res, span) in lexer.spanned() {
                if let Ok(token) = token_res {
                    tokens.push((token, span));
                }
            }
            // Parse sonucu Ok veya Err olabilir ama panic olmamalı
            let _ = crate::parse_tokens(tokens, kod.len());
        }
    }

    // Rastgele oluşturulan ifadelerin tip çıkarımında panic vermemesini doğrula
    proptest! {
        #[test]
        fn typechecker_asla_panic_vermez(kod in arith_expr_strategy()) {
            use crate::typechecker::inference::TypeChecker;
            use crate::typechecker::types::TypeEnv;

            let lexer = oz_lexer::Token::lexer(&kod);
            let mut tokens = Vec::new();
            for (token_res, span) in lexer.spanned() {
                if let Ok(token) = token_res {
                    tokens.push((token, span));
                }
            }

            if let Ok(ast) = crate::parse_tokens(tokens, kod.len()) {
                let mut tc = TypeChecker::new();
                let mut env = TypeEnv::new();
                for stmt in &ast {
                    // Tip çıkarımı hata dönebilir ama panic olmamalı
                    let _ = tc.infer_stmt(stmt, &mut env, &None);
                }
            }
        }
    }
}
