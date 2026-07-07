import re

with open('oz-parser/src/typechecker/inference.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if i >= 482:
        lines[i] = line.replace('expr.span.clone()', 'stmt.span.clone()')

with open('oz-parser/src/typechecker/inference.rs', 'w', encoding='utf-8') as f:
    f.writelines(lines)
