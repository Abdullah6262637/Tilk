use crate::ast::{Expr, InterpolatedPart, Literal, Spanned, Statement};
use crate::parse_expr_source;

pub fn desugar_ast(stmts: &mut Vec<Spanned<Statement>>) -> Result<(), String> {
    for stmt in stmts {
        desugar_stmt(stmt)?;
    }
    Ok(())
}

fn desugar_stmt(stmt: &mut Spanned<Statement>) -> Result<(), String> {
    match &mut stmt.node {
        Statement::VarDecl(_, expr) => desugar_expr(expr)?,
        Statement::Assignment(_, expr) => desugar_expr(expr)?,
        Statement::IndexAssignment(arr, idx, val) => {
            desugar_expr(arr)?;
            desugar_expr(idx)?;
            desugar_expr(val)?;
        }
        Statement::If(cond, then_body, else_body) => {
            desugar_expr(cond)?;
            for s in then_body {
                desugar_stmt(s)?;
            }
            if let Some(else_b) = else_body {
                for s in else_b {
                    desugar_stmt(s)?;
                }
            }
        }
        Statement::While(cond, body) => {
            desugar_expr(cond)?;
            for s in body {
                desugar_stmt(s)?;
            }
        }
        Statement::For {
            start, end, body, ..
        } => {
            desugar_expr(start)?;
            desugar_expr(end)?;
            for s in body {
                desugar_stmt(s)?;
            }
        }
        Statement::ForEach { iterable, body, .. } => {
            desugar_expr(iterable)?;
            for s in body {
                desugar_stmt(s)?;
            }
        }
        Statement::FnDecl { body, .. } => {
            for s in body {
                desugar_stmt(s)?;
            }
        }
        Statement::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                desugar_expr(expr)?;
            }
        }
        Statement::Expr(expr) => desugar_expr(expr)?,
        Statement::Tamamlaninca(task, body) => {
            desugar_expr(task)?;
            for s in body {
                desugar_stmt(s)?;
            }
        }
    }
    Ok(())
}

fn desugar_expr(expr: &mut Spanned<Expr>) -> Result<(), String> {
    match &mut expr.node {
        Expr::Binary(lhs, _, rhs) => {
            desugar_expr(lhs)?;
            desugar_expr(rhs)?;
        }
        Expr::Unary(_, operand) => desugar_expr(operand)?,
        Expr::Call(_, _, args) => {
            for arg in args {
                desugar_expr(arg)?;
            }
        }
        Expr::Array(items) => {
            for item in items {
                desugar_expr(item)?;
            }
        }
        Expr::Map(pairs) => {
            for (k, v) in pairs {
                desugar_expr(k)?;
                desugar_expr(v)?;
            }
        }
        Expr::Index(arr, idx) => {
            desugar_expr(arr)?;
            desugar_expr(idx)?;
        }
        Expr::HataIse(base, body) => {
            desugar_expr(base)?;
            for s in body {
                desugar_stmt(s)?;
            }
        }
        Expr::Literal(Literal::String(s)) => {
            if s.contains('{') {
                let parts = parse_interpolation_content(s, expr.span.start + 1)?;
                if !parts.is_empty()
                    && (parts.len() > 1 || matches!(parts.first(), Some(InterpolatedPart::Expr(_))))
                {
                    expr.node = Expr::InterpolatedString(parts);
                }
            }
        }
        Expr::Literal(_) | Expr::Identifier(_, _) => {}
        Expr::InterpolatedString(_) => {}
        Expr::Lambda { body, .. } => {
            for stmt in body {
                desugar_stmt(stmt)?;
            }
        }
    }
    Ok(())
}

fn parse_interpolation_content(
    s: &str,
    start_offset: usize,
) -> Result<Vec<InterpolatedPart>, String> {
    let mut parts = Vec::new();
    let mut chars = s.char_indices().peekable();
    let mut current_text = String::new();

    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            if let Some((_, next_c)) = chars.next() {
                match next_c {
                    '{' => current_text.push('{'),
                    '}' => current_text.push('}'),
                    'n' => current_text.push('\n'),
                    't' => current_text.push('\t'),
                    '"' => current_text.push('"'),
                    '\\' => current_text.push('\\'),
                    _ => {
                        current_text.push('\\');
                        current_text.push(next_c);
                    }
                }
            } else {
                current_text.push('\\');
            }
        } else if c == '{' {
            if !current_text.is_empty() {
                parts.push(InterpolatedPart::Text(current_text.clone()));
                current_text.clear();
            }

            let mut expr_str = String::new();
            let mut brace_depth = 1;
            let expr_start = i + 1;

            for (_, ec) in chars.by_ref() {
                if ec == '{' {
                    brace_depth += 1;
                    expr_str.push(ec);
                } else if ec == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        break;
                    } else {
                        expr_str.push(ec);
                    }
                } else {
                    expr_str.push(ec);
                }
            }

            if brace_depth != 0 {
                return Err(format!(
                    "Hata: Süslü parantez kapatılmamış (karakter no: {})",
                    start_offset + i
                ));
            }
            if expr_str.trim().is_empty() {
                return Err(format!(
                    "Hata: Boş interpolasyon {{}} kullanılamaz (karakter no: {})",
                    start_offset + i
                ));
            }

            let inner_offset = start_offset + expr_start;
            let parsed_expr = parse_expr_source(&expr_str, inner_offset)?;
            parts.push(InterpolatedPart::Expr(Box::new(parsed_expr)));
        } else {
            current_text.push(c);
        }
    }
    if !current_text.is_empty() {
        parts.push(InterpolatedPart::Text(current_text));
    }
    Ok(parts)
}
