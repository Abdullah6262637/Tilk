use chumsky::prelude::*;
use oz_lexer::Token;
use std::ops::Range;

pub mod ast;
use ast::*;

fn num_parser() -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Number(s) => s
            .parse::<f64>()
            .map(|n| Expr::Literal(Literal::Number(n)))
            .map_err(|_| Simple::custom(span, "Geçersiz sayı")),
        _ => Err(Simple::custom(span, "Sayı bekleniyordu")),
    })
}

fn string_parser() -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::String(s) => Ok(Expr::Literal(Literal::String(s))),
        _ => Err(Simple::custom(span, "Metin bekleniyordu")),
    })
}

fn ident_parser() -> impl Parser<Token, String, Error = Simple<Token>> + Clone {
    filter_map(|span, tok| match tok {
        Token::Identifier(s) => Ok(s),
        _ => Err(Simple::custom(span, "Tanımlayıcı bekleniyordu")),
    })
}

fn suffix_parser(values: &'static [&'static str]) -> impl Parser<Token, String, Error = Simple<Token>> + Clone {
    ident_parser().try_map(move |s, span| {
        if values.contains(&s.as_str()) {
            Ok(s)
        } else {
            Err(Simple::custom(span, "Beklenen ek bulunamadı"))
        }
    })
}

fn expr_parser() -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let literal = num_parser()
            .or(string_parser())
            .or(just(Token::Dogru).to(Expr::Literal(Literal::Boolean(true))))
            .or(just(Token::Yanlis).to(Expr::Literal(Literal::Boolean(false))))
            .or(just(Token::Bos).to(Expr::Literal(Literal::Bos)));

        let call_or_var = ident_parser()
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not(),
            )
            .map(|(name, args)| {
                if let Some(args) = args {
                    Expr::Call(name, args)
                } else {
                    Expr::Identifier(name)
                }
            });

        let atom = literal
            .or(call_or_var)
            .or(expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)));

        // Factor: multiplicative operators
        let op_mul = just(Token::Mul)
            .to(BinaryOp::Mul)
            .or(just(Token::Div).to(BinaryOp::Div))
            .or(just(Token::Mod).to(BinaryOp::Mod));
        let factor = atom.then(op_mul.then(expr.clone()).repeated()).foldl(
            |lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)),
        );

        // Term: additive operators
        let op_add = just(Token::Plus)
            .to(BinaryOp::Add)
            .or(just(Token::Minus).to(BinaryOp::Sub));
        let term = factor.then(op_add.then(expr.clone()).repeated()).foldl(
            |lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)),
        );

        // Comparison
        let op_comp = just(Token::Eq)
            .to(BinaryOp::Eq)
            .or(just(Token::Ne).to(BinaryOp::Ne))
            .or(just(Token::Le).to(BinaryOp::Le))
            .or(just(Token::Ge).to(BinaryOp::Ge))
            .or(just(Token::Lt).to(BinaryOp::Lt))
            .or(just(Token::Gt).to(BinaryOp::Gt));
        let comparison = term.then(op_comp.then(expr.clone()).repeated()).foldl(
            |lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)),
        );

        // Logical
        let op_logical = just(Token::And)
            .to(BinaryOp::And)
            .or(just(Token::Or).to(BinaryOp::Or));
        comparison.then(op_logical.then(expr.clone()).repeated()).foldl(
            |lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)),
        )
    })
}

fn statement_parser() -> impl Parser<Token, Statement, Error = Simple<Token>> + Clone {
    recursive(|stmt| {
        let block = stmt
            .clone()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        // x = 5;
        let assign_or_decl = ident_parser()
            .then_ignore(just(Token::Assign))
            .then(expr_parser())
            .then_ignore(just(Token::Semicolon))
            .map(|(name, value)| Statement::VarDecl(name, value));

        // koşul ise { ... } değilse { ... }
        let if_stmt = expr_parser()
            .then_ignore(suffix_parser(&["ise", "se"]))
            .then(block.clone())
            .then(just(Token::Degilse).ignore_then(block.clone()).or_not())
            .map(|((cond, then_block), else_block)| Statement::If(cond, then_block, else_block));

        // koşul iken { ... }
        let while_stmt = expr_parser()
            .then_ignore(suffix_parser(&["iken"]))
            .then(block.clone())
            .map(|(cond, body)| Statement::While(cond, body));

        // i, 1'den 10'a dek artarak { ... }
        let for_stmt = ident_parser()
            .then_ignore(just(Token::Comma))
            .then(expr_parser())
            .then_ignore(suffix_parser(&["dan", "den", "tan", "ten"]))
            .then(expr_parser())
            .then_ignore(suffix_parser(&["a", "e", "ya", "ye"]))
            .then_ignore(suffix_parser(&["dek"]))
            .then(
                suffix_parser(&["artarak", "azalarak"])
                    .or_not()
                    .map(|d| match d.as_deref() {
                        Some("azalarak") => StepDir::Azalarak,
                        _ => StepDir::Artarak,
                    }),
            )
            .then(block.clone())
            .map(|((((var, start), end), step_dir), body)| Statement::For {
                var,
                start,
                end,
                step_dir,
                body,
            });

        // işlev topla(a, b) { ... }
        let fn_decl = just(Token::Islev)
            .ignore_then(ident_parser())
            .then(
                ident_parser()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then(block.clone())
            .map(|((name, params), body)| Statement::FnDecl { name, params, body });

        // döndür x;
        let return_stmt = just(Token::Dondur)
            .ignore_then(expr_parser().or_not())
            .then_ignore(just(Token::Semicolon))
            .map(Statement::Return);

        // Expression statements (e.g. function calls)
        let expr_stmt = expr_parser()
            .then_ignore(just(Token::Semicolon))
            .map(Statement::Expr);

        assign_or_decl
            .or(if_stmt)
            .or(while_stmt)
            .or(for_stmt)
            .or(fn_decl)
            .or(return_stmt)
            .or(expr_stmt)
    })
}

pub fn parse_tokens(
    tokens: Vec<(Token, Range<usize>)>,
    len: usize,
) -> Result<Vec<Statement>, Vec<Simple<Token>>> {
    let parser = statement_parser().repeated().then_ignore(end());
    let stream = chumsky::Stream::from_iter(
        len..len,
        tokens.into_iter(),
    );
    parser.parse(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oz_lexer::Token;
    use logos::Logos;

    fn parse_helper(src: &str) -> Result<Vec<Statement>, String> {
        let lexer = Token::lexer(src);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            match token_res {
                Ok(token) => tokens.push((token, span)),
                Err(_) => return Err(format!("Lexer hatası: {:?}", span)),
            }
        }
        parse_tokens(tokens, src.len()).map_err(|e| format!("{:?}", e))
    }

    #[test]
    fn test_ornek1_kosul() {
        let src = include_str!("../../examples/ornek1_kosul.oz");
        let res = parse_helper(src);
        assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
    }

    #[test]
    fn test_ornek2_dongu() {
        let src = include_str!("../../examples/ornek2_dongu.oz");
        let res = parse_helper(src);
        assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
    }

    #[test]
    fn test_ornek3_islev() {
        let src = include_str!("../../examples/ornek3_islev.oz");
        let res = parse_helper(src);
        assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
    }

    #[test]
    fn test_ornek4_hesap() {
        let src = include_str!("../../examples/ornek4_hesap.oz");
        let res = parse_helper(src);
        assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
    }

    #[test]
    fn test_ornek5_karma() {
        let src = include_str!("../../examples/ornek5_karma.oz");
        let res = parse_helper(src);
        assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
    }
}
