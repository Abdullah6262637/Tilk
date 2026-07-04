#![allow(clippy::single_char_add_str)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::len_zero)]
use oz_parser::ast::{BinaryOp, Expr, Literal, Spanned, Statement, StepDir, UnaryOp};

use std::fs;

pub struct CCodegen {
    code: String,
}

impl CCodegen {
    pub fn new() -> Self {
        CCodegen {
            code: String::new(),
        }
    }

    pub fn transpile(mut self, stmts: &[Spanned<Statement>]) -> Result<String, String> {
        // C runtime header
        self.code.push_str(r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <math.h>
#include <time.h>
#include <stdarg.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

typedef enum {
    VAL_BOS,
    VAL_NUMBER,
    VAL_STRING,
    VAL_BOOLEAN,
    VAL_ARRAY,
    VAL_MAP,
    VAL_HATA
} TilkType;

struct TilkVal;

typedef struct {
    struct TilkVal* data;
    size_t len;
    size_t capacity;
} TilkArray;

typedef struct {
    char** keys;
    struct TilkVal* values;
    size_t len;
    size_t capacity;
} TilkMap;

typedef struct TilkVal {
    TilkType type;
    union {
        double number;
        char* string;
        bool boolean;
        TilkArray array;
        TilkMap map;
        char* error;
    } val;
} TilkVal;

TilkVal make_bos() {
    TilkVal v;
    v.type = VAL_BOS;
    return v;
}

TilkVal make_number(double n) {
    TilkVal v;
    v.type = VAL_NUMBER;
    v.val.number = n;
    return v;
}

TilkVal make_string(const char* s) {
    TilkVal v;
    v.type = VAL_STRING;
    v.val.string = strdup(s);
    return v;
}

TilkVal make_boolean(bool b) {
    TilkVal v;
    v.type = VAL_BOOLEAN;
    v.val.boolean = b;
    return v;
}

TilkVal make_hata(const char* msg) {
    TilkVal v;
    v.type = VAL_HATA;
    v.val.error = strdup(msg);
    return v;
}

void print_val_raw(TilkVal v) {
    switch (v.type) {
        case VAL_BOS: printf("boş"); break;
        case VAL_NUMBER: printf("%g", v.val.number); break;
        case VAL_STRING: printf("%s", v.val.string); break;
        case VAL_BOOLEAN: printf("%s", v.val.boolean ? "doğru" : "yanlış"); break;
        case VAL_ARRAY:
            printf("[");
            for (size_t i = 0; i < v.val.array.len; i++) {
                if (i > 0) printf(", ");
                print_val_raw(v.val.array.data[i]);
            }
            printf("]");
            break;
        case VAL_MAP:
            printf("{");
            for (size_t i = 0; i < v.val.map.len; i++) {
                if (i > 0) printf(", ");
                printf("\"%s\": ", v.val.map.keys[i]);
                print_val_raw(v.val.map.values[i]);
            }
            printf("}");
            break;
        case VAL_HATA: printf("Hata: %s", v.val.error); break;
    }
}

TilkVal yazdir(TilkVal v) {
    print_val_raw(v);
    printf("\n");
    return make_bos();
}
TilkVal yazd_r(TilkVal v) { return yazdir(v); }

TilkVal boyut(TilkVal v) {
    if (v.type == VAL_ARRAY) return make_number(v.val.array.len);
    if (v.type == VAL_MAP) return make_number(v.val.map.len);
    if (v.type == VAL_STRING) return make_number(strlen(v.val.string));
    return make_number(0);
}

TilkVal ekle(TilkVal arr, TilkVal item) {
    if (arr.type == VAL_ARRAY) {
        TilkArray* a = &arr.val.array;
        if (a->len >= a->capacity) {
            a->capacity = a->capacity == 0 ? 4 : a->capacity * 2;
            a->data = realloc(a->data, a->capacity * sizeof(TilkVal));
        }
        a->data[a->len++] = item;
    }
    return make_bos();
}

TilkVal hata_firlat(TilkVal v) {
    if (v.type == VAL_STRING) return make_hata(v.val.string);
    return make_hata("Hata");
}
TilkVal hata_f_rlat(TilkVal v) { return hata_firlat(v); }

TilkVal kok(TilkVal v) {
    if (v.type == VAL_NUMBER) {
        if (v.val.number < 0) return make_hata("Negatif sayının karekökü alınamaz");
        return make_number(sqrt(v.val.number));
    }
    return make_hata("Sayısal değer bekleniyordu");
}
TilkVal karekok(TilkVal v) { return kok(v); }

TilkVal us(TilkVal base, TilkVal exp) {
    if (base.type == VAL_NUMBER && exp.type == VAL_NUMBER) {
        return make_number(pow(base.val.number, exp.val.number));
    }
    return make_hata("Sayısal değer bekleniyordu");
}
TilkVal ust(TilkVal base, TilkVal exp) { return us(base, exp); }

TilkVal mutlak(TilkVal v) {
    if (v.type == VAL_NUMBER) return make_number(fabs(v.val.number));
    return make_hata("Sayısal değer bekleniyordu");
}

TilkVal simdi() {
    return make_number((double)time(NULL));
}
TilkVal simdi_zaman() { return simdi(); }

TilkVal uyku(TilkVal ms) {
    if (ms.type == VAL_NUMBER) {
        #ifdef _WIN32
        Sleep((DWORD)ms.val.number);
        #else
        usleep((useconds_t)(ms.val.number * 1000));
        #endif
    }
    return make_bos();
}

TilkVal dosya_oku(TilkVal path) {
    if (path.type != VAL_STRING) return make_hata("Dosya yolu metin olmalıdır");
    FILE* f = fopen(path.val.string, "r");
    if (!f) return make_hata("Dosya okunamadı");
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char* buf = malloc(len + 1);
    fread(buf, 1, len, f);
    buf[len] = '\0';
    fclose(f);
    TilkVal v = make_string(buf);
    free(buf);
    return v;
}

TilkVal dosya_yaz(TilkVal path, TilkVal content) {
    if (path.type != VAL_STRING || content.type != VAL_STRING) return make_hata("Dosya yolu ve içerik metin olmalıdır");
    FILE* f = fopen(path.val.string, "w");
    if (!f) return make_hata("Dosya yazılamadı");
    fputs(content.val.string, f);
    fclose(f);
    return make_boolean(true);
}

TilkVal dosya_sil(TilkVal path) {
    if (path.type != VAL_STRING) return make_hata("Dosya yolu metin olmalıdır");
    if (remove(path.val.string) == 0) return make_boolean(true);
    return make_hata("Dosya silinemedi");
}

TilkVal add_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(a.val.number + b.val.number);
    if (a.type == VAL_STRING && b.type == VAL_STRING) {
        char* buf = malloc(strlen(a.val.string) + strlen(b.val.string) + 1);
        strcpy(buf, a.val.string);
        strcat(buf, b.val.string);
        TilkVal v = make_string(buf);
        free(buf);
        return v;
    }
    return make_hata("Uyumsuz tipler");
}
TilkVal sub_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(a.val.number - b.val.number);
    return make_hata("Uyumsuz tipler");
}
TilkVal mul_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(a.val.number * b.val.number);
    return make_hata("Uyumsuz tipler");
}
TilkVal div_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) {
        if (b.val.number == 0) return make_hata("Sıfıra bölme hatası");
        return make_number(a.val.number / b.val.number);
    }
    return make_hata("Uyumsuz tipler");
}
TilkVal mod_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(fmod(a.val.number, b.val.number));
    return make_hata("Uyumsuz tipler");
}
TilkVal eq_val(TilkVal a, TilkVal b) {
    if (a.type != b.type) return make_boolean(false);
    switch (a.type) {
        case VAL_BOS: return make_boolean(true);
        case VAL_NUMBER: return make_boolean(a.val.number == b.val.number);
        case VAL_STRING: return make_boolean(strcmp(a.val.string, b.val.string) == 0);
        case VAL_BOOLEAN: return make_boolean(a.val.boolean == b.val.boolean);
        default: return make_boolean(false);
    }
}
TilkVal ne_val(TilkVal a, TilkVal b) { return make_boolean(!eq_val(a, b).val.boolean); }
TilkVal lt_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number < b.val.number);
    return make_boolean(false);
}
TilkVal gt_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number > b.val.number);
    return make_boolean(false);
}
TilkVal le_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number <= b.val.number);
    return make_boolean(false);
}
TilkVal ge_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number >= b.val.number);
    return make_boolean(false);
}
TilkVal and_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_BOOLEAN && b.type == VAL_BOOLEAN) return make_boolean(a.val.boolean && b.val.boolean);
    return make_boolean(false);
}
TilkVal or_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_BOOLEAN && b.type == VAL_BOOLEAN) return make_boolean(a.val.boolean || b.val.boolean);
    return make_boolean(false);
}

TilkVal index_val(TilkVal target, TilkVal idx) {
    if (target.type == VAL_ARRAY && idx.type == VAL_NUMBER) {
        size_t i = (size_t)idx.val.number;
        if (i < target.val.array.len) return target.val.array.data[i];
        return make_hata("Dizi sınırları dışında erişim");
    }
    if (target.type == VAL_MAP && idx.type == VAL_STRING) {
        TilkMap* m = &target.val.map;
        for (size_t i = 0; i < m->len; i++) {
            if (strcmp(m->keys[i], idx.val.string) == 0) return m->values[i];
        }
        return make_bos();
    }
    return make_hata("İndeksleme hatası");
}

TilkVal index_assign(TilkVal target, TilkVal idx, TilkVal val) {
    if (target.type == VAL_ARRAY && idx.type == VAL_NUMBER) {
        size_t i = (size_t)idx.val.number;
        if (i < target.val.array.len) {
            target.val.array.data[i] = val;
            return make_bos();
        }
        return make_hata("Dizi sınırları dışında yazma");
    }
    if (target.type == VAL_MAP && idx.type == VAL_STRING) {
        TilkMap* m = &target.val.map;
        for (size_t i = 0; i < m->len; i++) {
            if (strcmp(m->keys[i], idx.val.string) == 0) {
                m->values[i] = val;
                return make_bos();
            }
        }
        if (m->len >= m->capacity) {
            m->capacity = m->capacity == 0 ? 4 : m->capacity * 2;
            m->keys = realloc(m->keys, m->capacity * sizeof(char*));
            m->values = realloc(m->values, m->capacity * sizeof(TilkVal));
        }
        m->keys[m->len] = strdup(idx.val.string);
        m->values[m->len++] = val;
        return make_bos();
    }
    return make_hata("İndeksleme hatası");
}

TilkVal create_array(size_t len, ...) {
    TilkVal v;
    v.type = VAL_ARRAY;
    v.val.array.data = len == 0 ? NULL : malloc(len * sizeof(TilkVal));
    v.val.array.len = len;
    v.val.array.capacity = len;
    va_list args;
    va_start(args, len);
    for (size_t i = 0; i < len; i++) {
        v.val.array.data[i] = va_arg(args, TilkVal);
    }
    va_end(args);
    return v;
}

TilkVal create_map(size_t pair_count, ...) {
    TilkVal v;
    v.type = VAL_MAP;
    v.val.map.keys = pair_count == 0 ? NULL : malloc(pair_count * sizeof(char*));
    v.val.map.values = pair_count == 0 ? NULL : malloc(pair_count * sizeof(TilkVal));
    v.val.map.len = pair_count;
    v.val.map.capacity = pair_count;
    va_list args;
    va_start(args, pair_count);
    for (size_t i = 0; i < pair_count; i++) {
        TilkVal k = va_arg(args, TilkVal);
        TilkVal val = va_arg(args, TilkVal);
        v.val.map.keys[i] = strdup(k.val.string);
        v.val.map.values[i] = val;
    }
    va_end(args);
    return v;
}

TilkVal neg_val(TilkVal v) {
    if (v.type == VAL_NUMBER) return make_number(-v.val.number);
    return make_hata("Negatif islem sadece sayilarla yapilabilir");
}

TilkVal not_val(TilkVal v) {
    if (v.type == VAL_BOOLEAN) return make_boolean(!v.val.boolean);
    return make_hata("Mantiksal degil islemi sadece mantiksal degerlerle yapilabilir");
}
"#);

        // Collect all function declarations
        let fn_decls = collect_function_decls(stmts);

        // Append forward declarations
        self.code.push_str("\n// Forward Declarations\n");
        for decl in &fn_decls {
            if let Statement::FnDecl { name, params, .. } = decl {
                self.code
                    .push_str(&format!("TilkVal {}(", sanitize_identifier(name)));
                for i in 0..params.len() {
                    if i > 0 {
                        self.code.push_str(", ");
                    }
                    self.code.push_str("TilkVal");
                }
                self.code.push_str(");\n");
            }
        }

        // Generate C function bodies for all collected declarations
        for decl in &fn_decls {
            if let Statement::FnDecl { name, params, body } = decl {
                let mut fn_code = format!("\nTilkVal {}(", sanitize_identifier(name));
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        fn_code.push_str(", ");
                    }
                    fn_code.push_str(&format!("TilkVal {}", sanitize_identifier(p)));
                }
                fn_code.push_str(") {\n");
                for s in body {
                    fn_code.push_str(&self.compile_stmt(s)?);
                }
                fn_code.push_str("    return make_bos();\n}\n");
                self.code.push_str(&fn_code);
            }
        }

        // Generate global main statements
        let mut main_body = String::new();
        for stmt in stmts {
            if let Statement::FnDecl { .. } = &stmt.node {
                continue;
            }
            main_body.push_str(&self.compile_stmt(stmt)?);
        }

        // main function
        self.code.push_str("\nint main() {\n");
        self.code.push_str(&main_body);
        self.code.push_str("    return 0;\n}\n");

        Ok(self.code)
    }

    fn compile_stmt(&self, stmt: &Spanned<Statement>) -> Result<String, String> {
        let mut out = String::new();
        match &stmt.node {
            Statement::VarDecl(name, expr) => {
                let expr_str = self.compile_expr(expr)?;
                out.push_str(&format!(
                    "    TilkVal {} = {};\n",
                    sanitize_identifier(name),
                    expr_str
                ));
            }
            Statement::Assignment(name, expr) => {
                let expr_str = self.compile_expr(expr)?;
                out.push_str(&format!(
                    "    {} = {};\n",
                    sanitize_identifier(name),
                    expr_str
                ));
            }
            Statement::IndexAssignment(target, idx, val) => {
                let target_str = self.compile_expr(target)?;
                let idx_str = self.compile_expr(idx)?;
                let val_str = self.compile_expr(val)?;
                out.push_str(&format!(
                    "    index_assign({}, {}, {});\n",
                    target_str, idx_str, val_str
                ));
            }
            Statement::If(cond, then_branch, else_branch) => {
                let cond_str = self.compile_expr(cond)?;
                out.push_str(&format!("    if ({}.val.boolean) {{\n", cond_str));
                for s in then_branch {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }");
                if let Some(eb) = else_branch {
                    out.push_str(" else {\n");
                    for s in eb {
                        out.push_str(&self.compile_stmt(s)?);
                    }
                    out.push_str("    }");
                }
                out.push_str("\n");
            }
            Statement::While(cond, body) => {
                let cond_str = self.compile_expr(cond)?;
                out.push_str(&format!("    while ({}.val.boolean) {{\n", cond_str));
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }\n");
            }
            Statement::For {
                var,
                start,
                end,
                step_dir,
                body,
            } => {
                let start_str = self.compile_expr(start)?;
                let end_str = self.compile_expr(end)?;
                let s_var = sanitize_identifier(var);
                let start_var = format!("{}_start", s_var);
                let end_var = format!("{}_end", s_var);
                out.push_str(&format!("    TilkVal {} = {};\n", start_var, start_str));
                out.push_str(&format!("    TilkVal {} = {};\n", end_var, end_str));

                out.push_str(&format!(
                    "    for (double {} = {}.val.number; ",
                    s_var, start_var
                ));
                match step_dir {
                    StepDir::Artarak => {
                        out.push_str(&format!("{} <= {}.val.number; {}++", s_var, end_var, s_var))
                    }
                    StepDir::Azalarak => {
                        out.push_str(&format!("{} >= {}.val.number; {}--", s_var, end_var, s_var))
                    }
                }
                out.push_str(") {\n");
                out.push_str(&format!(
                    "        TilkVal {}_val = make_number({});\n",
                    s_var, s_var
                ));
                out.push_str(&format!("        TilkVal {} = {}_val;\n", s_var, s_var));
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }\n");
            }
            Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let expr_str = self.compile_expr(expr)?;
                    out.push_str(&format!("    return {};\n", expr_str));
                } else {
                    out.push_str("    return make_bos();\n");
                }
            }
            Statement::Expr(expr) => {
                let expr_str = self.compile_expr(expr)?;
                out.push_str(&format!("    {};\n", expr_str));
            }
            Statement::Tamamlaninca(gorev, body) => {
                let gorev_str = self.compile_expr(gorev)?;
                out.push_str("    {\n");
                out.push_str(&format!("        TilkVal sonuc = {};\n", gorev_str));
                out.push_str("        TilkVal sonuc_val = sonuc;\n");
                out.push_str("        TilkVal sonuc = sonuc_val;\n");
                out.push_str("        TilkVal sonuç = sonuc_val;\n");
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }\n");
            }
            _ => {}
        }
        Ok(out)
    }

    fn compile_expr(&self, expr: &Spanned<Expr>) -> Result<String, String> {
        match &expr.node {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => Ok(format!("make_number({})", n)),
                Literal::String(s) => Ok(format!("make_string(\"{}\")", escape_string(s))),
                Literal::Boolean(b) => Ok(format!("make_boolean({})", b)),
                Literal::Bos => Ok("make_bos()".to_string()),
            },
            Expr::Identifier(name) => Ok(sanitize_identifier(name)),
            Expr::Unary(op, operand) => {
                let op_str = self.compile_expr(operand)?;
                let helper = match op {
                    UnaryOp::Neg => "neg_val",
                    UnaryOp::Not => "not_val",
                };
                Ok(format!("{}({})", helper, op_str))
            }
            Expr::Binary(lhs, op, rhs) => {
                let lhs_str = self.compile_expr(lhs)?;
                let rhs_str = self.compile_expr(rhs)?;
                let helper = match op {
                    BinaryOp::Add => "add_val",
                    BinaryOp::Sub => "sub_val",
                    BinaryOp::Mul => "mul_val",
                    BinaryOp::Div => "div_val",
                    BinaryOp::Mod => "mod_val",
                    BinaryOp::Eq => "eq_val",
                    BinaryOp::Ne => "ne_val",
                    BinaryOp::Lt => "lt_val",
                    BinaryOp::Gt => "gt_val",
                    BinaryOp::Le => "le_val",
                    BinaryOp::Ge => "ge_val",
                    BinaryOp::And => "and_val",
                    BinaryOp::Or => "or_val",
                };
                Ok(format!("{}({}, {})", helper, lhs_str, rhs_str))
            }
            Expr::Call(name, args) => {
                if name == "dahil_et" {
                    if let Some(Expr::Literal(Literal::String(path))) =
                        args.first().map(|s| &s.node)
                    {
                        let content = fs::read_to_string(path)
                            .map_err(|e| format!("Modül yüklenemedi ({}): {}", path, e))?;

                        use logos::Logos;
                        use oz_lexer::Token;
                        let lexer = Token::lexer(&content);
                        let mut tokens = Vec::new();
                        for (token_res, _) in lexer.spanned() {
                            if let Ok(token) = token_res {
                                tokens.push((token, 0..0));
                            }
                        }

                        let ast = oz_parser::parse_tokens(tokens, content.len())
                            .map_err(|e| format!("Ayrıştırma hatası: {:?}", e))?;

                        let mut inline_code = String::new();
                        for stmt in &ast {
                            inline_code.push_str(&self.compile_stmt(stmt)?);
                        }
                        return Ok(format!("({{\n{} make_bos();\n}})", inline_code));
                    }
                }

                if name == "arkaplanda_çalıştır" || name == "arkaplanda_calistir" {
                    if args.len() >= 1 {
                        let inner_call = self.compile_expr(&args[0])?;
                        return Ok(inner_call);
                    }
                }

                let mut args_str = String::new();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        args_str.push_str(", ");
                    }
                    args_str.push_str(&self.compile_expr(arg)?);
                }
                Ok(format!("{}({})", sanitize_identifier(name), args_str))
            }
            Expr::Array(elements) => {
                let mut args_str = String::new();
                for el in elements {
                    args_str.push_str(", ");
                    args_str.push_str(&self.compile_expr(el)?);
                }
                Ok(format!("create_array({}{})", elements.len(), args_str))
            }
            Expr::Map(elements) => {
                let mut args_str = String::new();
                for (k, v) in elements {
                    args_str.push_str(", ");
                    args_str.push_str(&self.compile_expr(k)?);
                    args_str.push_str(", ");
                    args_str.push_str(&self.compile_expr(v)?);
                }
                Ok(format!("create_map({}{})", elements.len(), args_str))
            }
            Expr::Index(array, idx) => {
                let array_str = self.compile_expr(array)?;
                let idx_str = self.compile_expr(idx)?;
                Ok(format!("index_val({}, {})", array_str, idx_str))
            }
            Expr::HataIse(base, body) => {
                let base_str = self.compile_expr(base)?;
                let mut body_code = String::new();
                for stmt in body {
                    body_code.push_str(&self.compile_stmt(stmt)?);
                }
                Ok(format!(
                    r#"({{
                    TilkVal base = {};
                    if (base.type == VAL_HATA) {{
                        TilkVal hata_mesajı = make_string(base.val.error);
                        TilkVal hata_mesaji = hata_mesajı;
                        {}
                    }}
                    base;
                }})"#,
                    base_str, body_code
                ))
            }
        }
    }
}

fn collect_function_decls(stmts: &[Spanned<Statement>]) -> Vec<Statement> {
    let mut decls = Vec::new();
    for stmt in stmts {
        match &stmt.node {
            Statement::FnDecl { .. } => {
                decls.push(stmt.node.clone());
            }
            Statement::If(_, then_branch, else_branch) => {
                decls.extend(collect_function_decls(then_branch));
                if let Some(eb) = else_branch {
                    decls.extend(collect_function_decls(eb));
                }
            }
            Statement::While(_, body) => {
                decls.extend(collect_function_decls(body));
            }
            Statement::For { body, .. } => {
                decls.extend(collect_function_decls(body));
            }
            Statement::Tamamlaninca(_, body) => {
                decls.extend(collect_function_decls(body));
            }
            Statement::Expr(spanned_expr) => {
                if let Expr::Call(name, args) = &spanned_expr.node {
                    if name == "dahil_et" {
                        if let Some(spanned_arg) = args.first() {
                            if let Expr::Literal(oz_parser::ast::Literal::String(path)) =
                                &spanned_arg.node
                            {
                                if let Ok(content) = std::fs::read_to_string(path) {
                                    use logos::Logos;
                                    use oz_lexer::Token;
                                    let lexer = Token::lexer(&content);
                                    let mut tokens = Vec::new();
                                    for (token_res, _) in lexer.spanned() {
                                        if let Ok(token) = token_res {
                                            tokens.push((token, 0..0));
                                        }
                                    }
                                    if let Ok(ast) = oz_parser::parse_tokens(tokens, content.len())
                                    {
                                        decls.extend(collect_function_decls(&ast));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    decls
}

fn escape_string(s: &str) -> String {
    s.replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
}

fn sanitize_identifier(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' | 'İ' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;
    use oz_lexer::Token;

    #[test]
    fn test_codegen_basic() {
        let src = r#"
            sayı = 42;
            işlev topla(a, b) {
                döndür a + b;
            }
            sonuç = topla(sayı, 8);
        "#;
        let lexer = Token::lexer(src);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            if let Ok(token) = token_res {
                tokens.push((token, span));
            }
        }
        let ast = oz_parser::parse_tokens(tokens, src.len()).unwrap();
        let codegen = CCodegen::new();
        let c_code = codegen.transpile(&ast).unwrap();
        assert!(c_code.contains("TilkVal sayi = make_number(42);"));
        assert!(c_code.contains("TilkVal topla(TilkVal a, TilkVal b)"));
        assert!(c_code.contains("sonuc = topla(sayi, make_number(8))"));
    }
}
