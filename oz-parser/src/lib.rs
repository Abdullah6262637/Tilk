#![allow(clippy::result_large_err)]

use chumsky::prelude::*;

use oz_lexer::Token;
use std::ops::Range;

pub mod ast;
pub mod typechecker;
use ast::*;

fn num_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    filter_map(|span: Range<usize>, tok| match tok {
        Token::Number(s) => s
            .parse::<f64>()
            .map(|n| Spanned::new(Expr::Literal(Literal::Number(n)), span.clone()))
            .map_err(|_| Simple::custom(span, "Geçersiz sayı")),
        _ => Err(Simple::custom(span, "Sayı bekleniyordu")),
    })
}

fn string_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    filter_map(|span: Range<usize>, tok| match tok {
        Token::String(s) => Ok(Spanned::new(Expr::Literal(Literal::String(s)), span)),
        _ => Err(Simple::custom(span, "Metin bekleniyordu")),
    })
}

fn ident_parser() -> impl Parser<Token, String, Error = Simple<Token>> + Clone {
    filter_map(|span: Range<usize>, tok| match tok {
        Token::Identifier(s) => Ok(s),
        _ => Err(Simple::custom(span, "Tanımlayıcı bekleniyordu")),
    })
}

fn suffix_parser(
    values: &'static [&'static str],
) -> impl Parser<Token, String, Error = Simple<Token>> + Clone {
    ident_parser().try_map(move |s, span| {
        if values.contains(&s.as_str()) {
            Ok(s)
        } else {
            Err(Simple::custom(span, "Beklenen ek bulunamadı"))
        }
    })
}

fn expr_parser(
    stmt: impl Parser<Token, Spanned<Statement>, Error = Simple<Token>> + Clone + 'static,
) -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let array_literal = just(Token::LBracket)
            .ignore_then(expr.clone().separated_by(just(Token::Comma)))
            .then_ignore(just(Token::RBracket))
            .map(Expr::Array)
            .map_with_span(Spanned::new);

        let map_literal = just(Token::LBrace)
            .ignore_then(
                expr.clone()
                    .then_ignore(just(Token::Colon))
                    .then(expr.clone())
                    .separated_by(just(Token::Comma)),
            )
            .then_ignore(just(Token::RBrace))
            .map(Expr::Map)
            .map_with_span(Spanned::new);

        let literal = num_parser()
            .or(string_parser())
            .or(just(Token::Dogru)
                .map_with_span(|_, span| Spanned::new(Expr::Literal(Literal::Boolean(true)), span)))
            .or(just(Token::Yanlis).map_with_span(|_, span| {
                Spanned::new(Expr::Literal(Literal::Boolean(false)), span)
            }))
            .or(just(Token::Bos)
                .map_with_span(|_, span| Spanned::new(Expr::Literal(Literal::Bos), span)))
            .or(array_literal)
            .or(map_literal);

        let qualified_path = ident_parser()
            .then_ignore(just(Token::DoubleColon))
            .repeated();

        let call_or_var = qualified_path
            .then(ident_parser())
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not(),
            )
            .map_with_span(|((path, name), args), span| {
                let module_prefix = if path.is_empty() { None } else { Some(path) };
                let node = if let Some(args) = args {
                    Expr::Call(module_prefix, name, args)
                } else {
                    Expr::Identifier(module_prefix, name)
                };
                Spanned::new(node, span)
            });

        let atom = literal.or(call_or_var).or(expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            // LParen'dan RParen'a kadar olan span'i koruyalım
            .map_with_span(|inner, span| Spanned::new(inner.node, span)));

        let index_suffix = just(Token::LBracket)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::RBracket))
            .or(
                suffix_parser(&["in", "ın", "un", "ün", "nin", "nın", "nun", "nün"])
                    .ignore_then(expr.clone())
                    .then_ignore(suffix_parser(&[
                        "inci", "ıncı", "uncu", "üncü", "nci", "ncı", "ncu", "ncü",
                    ]))
                    .then_ignore(suffix_parser(&["elemanı", "elemani", "değeri", "degeri"])),
            )
            .or(suffix_parser(&[
                "in", "ın", "un", "ün", "nin", "nın", "nun", "nün", "de", "da", "te", "ta", "yi",
                "yı", "yu", "yü",
            ])
            .ignore_then(ident_parser().map_with_span(|name, span| {
                Spanned::new(Expr::Literal(Literal::String(name)), span)
            })));

        let indexed_atom = atom.then(index_suffix.repeated()).foldl(|array, index| {
            let span = array.span.start..index.span.end;
            Spanned::new(Expr::Index(Box::new(array), Box::new(index)), span)
        });

        let unary = just(Token::Minus)
            .to(UnaryOp::Neg)
            .or(just(Token::Degil).to(UnaryOp::Not))
            .map_with_span(|op, span| (op, span))
            .repeated()
            .then(indexed_atom)
            .foldr(|(op, op_span), expr| {
                let span = op_span.start..expr.span.end;
                Spanned::new(Expr::Unary(op, Box::new(expr)), span)
            });

        // Factor: multiplicative operators
        let op_mul = just(Token::Mul)
            .to(BinaryOp::Mul)
            .or(just(Token::Div).to(BinaryOp::Div))
            .or(just(Token::Mod).to(BinaryOp::Mod));
        let factor = unary
            .clone()
            .then(op_mul.then(unary).repeated())
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned::new(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span)
            });

        // Term: additive operators
        let op_add = just(Token::Plus)
            .to(BinaryOp::Add)
            .or(just(Token::Minus).to(BinaryOp::Sub));
        let term = factor
            .clone()
            .then(op_add.then(factor).repeated())
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned::new(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span)
            });

        // Comparison
        let op_comp = just(Token::Eq)
            .to(BinaryOp::Eq)
            .or(just(Token::Ne).to(BinaryOp::Ne))
            .or(just(Token::Le).to(BinaryOp::Le))
            .or(just(Token::Ge).to(BinaryOp::Ge))
            .or(just(Token::Lt).to(BinaryOp::Lt))
            .or(just(Token::Gt).to(BinaryOp::Gt));
        let comparison =
            term.clone()
                .then(op_comp.then(term).repeated())
                .foldl(|lhs, (op, rhs)| {
                    let span = lhs.span.start..rhs.span.end;
                    Spanned::new(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span)
                });

        // Logical
        let op_logical = just(Token::And)
            .to(BinaryOp::And)
            .or(just(Token::Or).to(BinaryOp::Or));
        let base_expr = comparison
            .clone()
            .then(op_logical.then(comparison).repeated())
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned::new(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span)
            });

        let block = stmt
            .clone()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        base_expr
            .then(just(Token::HataIse).ignore_then(block).or_not())
            .map_with_span(|(base, body_opt), span| {
                if let Some(body) = body_opt {
                    Spanned::new(Expr::HataIse(Box::new(base), body), span)
                } else {
                    base
                }
            })
    })
}

fn statement_parser() -> impl Parser<Token, Spanned<Statement>, Error = Simple<Token>> + Clone {
    recursive(|stmt| {
        let expr = expr_parser(stmt.clone());
        let block = stmt
            .clone()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        // x = 5; or dizi[0] = 5;
        let assign_stmt = expr
            .clone()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .try_map(|(lhs, rhs), span| match lhs.node {
                Expr::Identifier(prefix, name) => {
                    if prefix.is_some() {
                        Err(Simple::custom(
                            span,
                            "Modül önekli tanımlayıcılara doğrudan atama yapılamaz",
                        ))
                    } else {
                        Ok(Spanned::new(Statement::VarDecl(name, rhs), span))
                    }
                }
                Expr::Index(array, index) => Ok(Spanned::new(
                    Statement::IndexAssignment(*array, *index, rhs),
                    span,
                )),
                _ => Err(Simple::custom(span, "Geçersiz atama hedefi (LHS)")),
            });

        // koşul ise { ... } değilse { ... }
        let if_stmt = expr
            .clone()
            .then_ignore(suffix_parser(&[
                "ise", "se", "olunca", "olünce", "ince", "ınca", "unca", "ünce",
            ]))
            .then(block.clone())
            .then(just(Token::Degilse).ignore_then(block.clone()).or_not())
            .map(|((cond, then_block), else_block)| Statement::If(cond, then_block, else_block))
            .map_with_span(Spanned::new);

        // koşul iken { ... }
        let while_stmt = expr
            .clone()
            .then_ignore(suffix_parser(&[
                "iken",
                "oldukça",
                "oldukcça",
                "dıkça",
                "dikçe",
                "dukça",
                "dükçe",
            ]))
            .then(block.clone())
            .map(|(cond, body)| Statement::While(cond, body))
            .map_with_span(Spanned::new);

        // i, 1'den 10'a dek artarak { ... }
        let for_stmt =
            ident_parser()
                .then_ignore(just(Token::Comma))
                .then(expr.clone())
                .then_ignore(suffix_parser(&["dan", "den", "tan", "ten"]))
                .then(expr.clone())
                .then_ignore(suffix_parser(&["a", "e", "ya", "ye"]))
                .then_ignore(suffix_parser(&["dek"]))
                .then(suffix_parser(&["artarak", "azalarak"]).or_not().map(
                    |d| match d.as_deref() {
                        Some("azalarak") => StepDir::Azalarak,
                        _ => StepDir::Artarak,
                    },
                ))
                .then(block.clone())
                .map(|((((var, start), end), step_dir), body)| Statement::For {
                    var,
                    start,
                    end,
                    step_dir,
                    body,
                })
                .map_with_span(Spanned::new);

        // her eleman dizi içinde { ... }
        let for_each_stmt = just(Token::Her)
            .ignore_then(ident_parser())
            .then(expr.clone())
            .then_ignore(just(Token::Icinde))
            .then(block.clone())
            .map(|((var, iterable), body)| Statement::ForEach {
                var,
                iterable,
                body,
            })
            .map_with_span(Spanned::new);

        // işlev topla(a, b) { ... }
        // işlev topla<T>(a: T, b: T): T { ... }
        let type_annot = just(Token::Colon)
            .ignore_then(ident_parser().then(just(Token::QuestionMark).or_not()))
            .map(|(t, opt_q)| {
                if opt_q.is_some() {
                    format!("{}?", t)
                } else {
                    t
                }
            });

        let generics_parser = just(Token::Lt)
            .ignore_then(ident_parser().separated_by(just(Token::Comma)))
            .then_ignore(just(Token::Gt))
            .or_not()
            .map(|opt| opt.unwrap_or_default());

        let param_parser = ident_parser().then(type_annot.clone().or_not());

        let fn_decl = just(Token::Islev)
            .ignore_then(ident_parser())
            .then(generics_parser)
            .then(
                param_parser
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen)),
            )
            .then(type_annot.or_not())
            .then(block.clone())
            .map(
                |((((name, generics), params), return_type), body)| Statement::FnDecl {
                    name,
                    generics,
                    params,
                    return_type,
                    body,
                },
            )
            .map_with_span(Spanned::new);

        // döndür x;
        let return_stmt = just(Token::Dondur)
            .ignore_then(expr.clone().or_not())
            .then_ignore(just(Token::Semicolon))
            .map(Statement::Return)
            .map_with_span(Spanned::new);

        // Expression statements (e.g. function calls)
        let expr_stmt = expr
            .clone()
            .then_ignore(just(Token::Semicolon))
            .map(Statement::Expr)
            .map_with_span(Spanned::new);

        // görev tamamlanınca { ... }
        let tamamlaninca_stmt = expr
            .clone()
            .then_ignore(just(Token::Tamamlaninca))
            .then(block.clone())
            .map(|(gorev, body)| Statement::Tamamlaninca(gorev, body))
            .map_with_span(Spanned::new);

        assign_stmt
            .or(if_stmt)
            .or(while_stmt)
            .or(for_stmt)
            .or(for_each_stmt)
            .or(fn_decl)
            .or(return_stmt)
            .or(tamamlaninca_stmt)
            .or(expr_stmt)
    })
}

pub fn parse_tokens(
    tokens: Vec<(Token, Range<usize>)>,
    len: usize,
) -> Result<Vec<Spanned<Statement>>, Vec<Simple<Token>>> {
    let parser = statement_parser().repeated().then_ignore(end());
    let stream = chumsky::Stream::from_iter(len..len, tokens.into_iter());
    parser.parse(stream)
}

#[cfg(test)]
mod tests;
