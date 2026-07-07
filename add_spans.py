import re

with open('oz-parser/src/typechecker/inference.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if 'self.unify(' in line and ')?' in line:
        # Determine span variable based on context (infer_expr uses expr, infer_stmt uses stmt)
        span_var = "expr.span.clone()" if i < 660 else "stmt.span.clone()"
        # Replace `self.unify(a, b)?` with `self.unify(a, b).map_err(|e| e.with_span(expr.span.clone()))?`
        # Careful: don't double replace if we already did something.
        lines[i] = re.sub(
            r'(self\.unify\([^)]+\))\?',
            rf'\1.map_err(|e| e.with_span({span_var}))?',
            line
        )
    elif 'return Err(type_err!' in line or 'Err(type_err!' in line:
        span_var = "expr.span.clone()" if i < 660 else "stmt.span.clone()"
        # `Err(type_err!("..."))` -> `Err(type_err!("...").with_span(span))`
        lines[i] = re.sub(
            r'Err\(type_err!\(([^)]+)\)\)',
            rf'Err(type_err!(\1).with_span({span_var}))',
            line
        )

with open('oz-parser/src/typechecker/inference.rs', 'w', encoding='utf-8') as f:
    f.writelines(lines)
