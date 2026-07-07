use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oz_vm::{compiler::Compiler, vm::VM};
use oz_parser::ast::Statement;

fn bench_vm_math_loop(c: &mut Criterion) {
    let source_code = r#"
        toplam = 0;
        i, 1'den 1000'e dek artarak {
            toplam = toplam + 1;
        }
    "#;

    // We do parsing and compiling outside of the benchmark loop to only measure VM execution speed
    let lexer = oz_lexer::Token::lexer(source_code);
    let mut tokens = Vec::new();
    for (token_res, span) in lexer.spanned() {
        if let Ok(t) = token_res {
            tokens.push((t, span));
        }
    }
    
    let ast = oz_parser::parse_tokens(tokens, source_code.len()).unwrap();
    let mut compiler = Compiler::new();
    let instructions = compiler.compile(&ast);

    c.bench_function("vm_math_loop_1000", |b| {
        b.iter(|| {
            let mut vm = VM::new(instructions.clone());
            let _ = black_box(vm.run());
        })
    });
}

criterion_group!(benches, bench_vm_math_loop);
criterion_main!(benches);
