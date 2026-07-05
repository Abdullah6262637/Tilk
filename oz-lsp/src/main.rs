use logos::Logos;
use std::collections::HashMap;
use std::sync::Mutex;
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "Tilk dosyası açıldı")
            .await;
        self.on_change(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.on_change(params.text_document.uri, change.text).await;
        }
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "Tilk dosyası kaydedildi")
            .await;
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.lock().unwrap();
        let content = match docs.get(&uri) {
            Some(c) => c,
            None => return Ok(None),
        };

        let offset = position_to_offset(content, position);
        let word = match get_word_at_offset(content, offset) {
            Some(w) => w,
            None => return Ok(None),
        };

        // Run parser and typechecker to get the word's type
        let lexer = oz_lexer::Token::lexer(content);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            if let Ok(token) = token_res {
                tokens.push((token, span));
            }
        }

        if let Ok(ast) = oz_parser::parse_tokens(tokens, content.len()) {
            let mut checker = oz_parser::typechecker::TypeChecker::new();
            let mut env = oz_parser::typechecker::create_default_type_env(&mut checker);
            for stmt in &ast {
                let _ = checker.infer_stmt(stmt, &mut env, &None);
            }
            if let Some(ty) = checker.recorded_types.get(word) {
                let hover_text = format!("**{}** : `{:?}`", word, ty);
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(hover_text)),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;

        let mut completions = Vec::new();

        // 1. Suggest builtins
        let builtins = vec![
            "yazdır",
            "boyut",
            "ekle",
            "kök",
            "karekok",
            "üs",
            "ust",
            "mutlak",
            "şimdi",
            "simdi",
            "uyku",
            "dosya_oku",
            "dosya_yaz",
            "dosya_sil",
            "hata_fırlat",
            "hata_firlat",
        ];
        for b in builtins {
            completions.push(CompletionItem {
                label: b.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Yerleşik Fonksiyon".to_string()),
                ..Default::default()
            });
        }

        // 2. Suggest locally defined variables/functions if AST parses
        let docs = self.documents.lock().unwrap();
        if let Some(content) = docs.get(&uri) {
            let lexer = oz_lexer::Token::lexer(content);
            let mut tokens = Vec::new();
            for (token_res, span) in lexer.spanned() {
                if let Ok(token) = token_res {
                    tokens.push((token, span));
                }
            }
            if let Ok(ast) = oz_parser::parse_tokens(tokens, content.len()) {
                let mut checker = oz_parser::typechecker::TypeChecker::new();
                let mut env = oz_parser::typechecker::create_default_type_env(&mut checker);
                for stmt in &ast {
                    let _ = checker.infer_stmt(stmt, &mut env, &None);
                }
                for (name, ty) in &checker.recorded_types {
                    if !name.starts_with('_') {
                        let kind = match ty {
                            oz_parser::typechecker::Type::Function { .. } => {
                                CompletionItemKind::FUNCTION
                            }
                            _ => CompletionItemKind::VARIABLE,
                        };
                        completions.push(CompletionItem {
                            label: name.clone(),
                            kind: Some(kind),
                            detail: Some(format!("{:?}", ty)),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Deduplicate completion suggestions
        completions.sort_by(|a, b| a.label.cmp(&b.label));
        completions.dedup_by(|a, b| a.label == b.label);

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().unwrap();
        let content = match docs.get(&uri) {
            Some(c) => c,
            None => return Ok(None),
        };

        let lexer = oz_lexer::Token::lexer(content);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            if let Ok(token) = token_res {
                tokens.push((token, span));
            }
        }

        let mut symbols = Vec::new();
        if let Ok(ast) = oz_parser::parse_tokens(tokens, content.len()) {
            for stmt in &ast {
                match &stmt.node {
                    oz_parser::ast::Statement::FnDecl { name, .. } => {
                        let range = find_symbol_range(content, name);
                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: name.clone(),
                            detail: Some("işlev".to_string()),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        });
                    }
                    oz_parser::ast::Statement::VarDecl(name, _) => {
                        let range = find_symbol_range(content, name);
                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: name.clone(),
                            detail: Some("değişken".to_string()),
                            kind: SymbolKind::VARIABLE,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.lock().unwrap();
        let content = match docs.get(&uri) {
            Some(c) => c,
            None => return Ok(None),
        };

        let offset = position_to_offset(content, position);
        let word = match get_word_at_offset(content, offset) {
            Some(w) => w,
            None => return Ok(None),
        };

        let lexer = oz_lexer::Token::lexer(content);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            if let Ok(token) = token_res {
                tokens.push((token, span));
            }
        }

        if let Ok(ast) = oz_parser::parse_tokens(tokens, content.len()) {
            let mut def_span = None;
            for stmt in &ast {
                find_definition_in_stmt(stmt, word, &mut def_span);
                if def_span.is_some() {
                    break;
                }
            }
            if let Some(span) = def_span {
                let range = span_to_range(content, span);
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri,
                    range,
                })));
            }
        }

        Ok(None)
    }
}

fn find_definition_in_stmt(stmt: &oz_parser::ast::Spanned<oz_parser::ast::Statement>, target_name: &str, def_span: &mut Option<std::ops::Range<usize>>) {
    use oz_parser::ast::Statement;
    match &stmt.node {
        Statement::VarDecl(name, _) => {
            if name == target_name {
                *def_span = Some(stmt.span.clone());
            }
        }
        Statement::FnDecl { name, body, .. } => {
            if name == target_name {
                *def_span = Some(stmt.span.clone());
            } else {
                for s in body {
                    find_definition_in_stmt(s, target_name, def_span);
                }
            }
        }
        Statement::If(_, then_block, else_block) => {
            for s in then_block {
                find_definition_in_stmt(s, target_name, def_span);
            }
            if let Some(eb) = else_block {
                for s in eb {
                    find_definition_in_stmt(s, target_name, def_span);
                }
            }
        }
        Statement::While(_, body) | Statement::For { body, .. } | Statement::ForEach { body, .. } => {
            for s in body {
                find_definition_in_stmt(s, target_name, def_span);
            }
        }
        Statement::Assignment(name, _) => {
            // Note: In an exact IDE this would trace back to the actual VarDecl,
            // but for simplicity we'll just check if there's no other decl.
        }
        _ => {}
    }
}

fn span_to_range(src: &str, span: std::ops::Range<usize>) -> tower_lsp::lsp_types::Range {
    tower_lsp::lsp_types::Range {
        start: offset_to_position(src, span.start),
        end: offset_to_position(src, span.end),
    }
}

impl Backend {
    async fn on_change(&self, uri: Url, text: String) {
        self.documents
            .lock()
            .unwrap()
            .insert(uri.clone(), text.clone());
        let diagnostics = parse_and_diagnose(&text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

fn offset_to_position(src: &str, offset: usize) -> Position {
    let mut line = 0;
    let mut character = 0;
    let mut cur_offset = 0;
    for c in src.chars() {
        if cur_offset >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += c.len_utf16() as u32;
        }
        cur_offset += c.len_utf8();
    }
    Position { line, character }
}

fn position_to_offset(src: &str, position: Position) -> usize {
    let mut line = 0;
    let mut character = 0;
    let mut offset = 0;
    for c in src.chars() {
        if line == position.line && character >= position.character {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += c.len_utf16() as u32;
        }
        offset += c.len_utf8();
        if line > position.line {
            break;
        }
    }
    offset
}

fn get_word_at_offset(src: &str, offset: usize) -> Option<&str> {
    if offset >= src.len() {
        return None;
    }
    let mut start = offset;
    while start > 0 {
        let prev_char = src[..start].chars().next_back()?;
        if prev_char.is_alphanumeric() || prev_char == '_' || "çğışöüÇĞİŞÖÜ".contains(prev_char)
        {
            start -= prev_char.len_utf8();
        } else {
            break;
        }
    }
    let mut end = offset;
    while end < src.len() {
        let next_char = src[end..].chars().next()?;
        if next_char.is_alphanumeric() || next_char == '_' || "çğışöüÇĞİŞÖÜ".contains(next_char)
        {
            end += next_char.len_utf8();
        } else {
            break;
        }
    }
    if start < end {
        Some(&src[start..end])
    } else {
        None
    }
}

fn find_error_range(src: &str, error_msg: &str) -> tower_lsp::lsp_types::Range {
    if let Some(start_idx) = error_msg.find('\'') {
        if let Some(end_idx) = error_msg[start_idx + 1..].find('\'') {
            let var_name = &error_msg[start_idx + 1..start_idx + 1 + end_idx];
            if let Some(offset) = src.find(var_name) {
                let start_pos = offset_to_position(src, offset);
                let end_pos = offset_to_position(src, offset + var_name.len());
                return tower_lsp::lsp_types::Range {
                    start: start_pos,
                    end: end_pos,
                };
            }
        }
    }
    let start_pos = Position {
        line: 0,
        character: 0,
    };
    let end_pos = Position {
        line: 0,
        character: 10,
    };
    tower_lsp::lsp_types::Range {
        start: start_pos,
        end: end_pos,
    }
}

fn find_symbol_range(src: &str, name: &str) -> tower_lsp::lsp_types::Range {
    if let Some(offset) = src.find(name) {
        let start = offset_to_position(src, offset);
        let end = offset_to_position(src, offset + name.len());
        tower_lsp::lsp_types::Range { start, end }
    } else {
        tower_lsp::lsp_types::Range::default()
    }
}

fn parse_and_diagnose(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lexer = oz_lexer::Token::lexer(content);
    let mut tokens = Vec::new();
    let mut lexer_errors = Vec::new();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => {
                lexer_errors.push((span, "Sözcüksel analiz hatası: Tanımlanamayan karakter"));
            }
        }
    }

    if !lexer_errors.is_empty() {
        for (span, msg) in lexer_errors {
            let start = offset_to_position(content, span.start);
            let end = offset_to_position(content, span.end);
            diagnostics.push(Diagnostic {
                range: tower_lsp::lsp_types::Range { start, end },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("Tilk-Lexer".to_string()),
                message: msg.to_string(),
                related_information: None,
                tags: None,
                data: None,
            });
        }
        return diagnostics;
    }

    match oz_parser::parse_tokens(tokens, content.len()) {
        Ok(ast) => {
            let mut checker = oz_parser::typechecker::TypeChecker::new();
            let mut env = oz_parser::typechecker::create_default_type_env(&mut checker);
            for stmt in &ast {
                if let Err(type_err) = checker.infer_stmt(stmt, &mut env, &None) {
                    let range = find_error_range(content, &type_err);
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("Tilk-TypeChecker".to_string()),
                        message: type_err,
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
        }
        Err(parser_errors) => {
            for err in parser_errors {
                let span = err.span();
                let start = offset_to_position(content, span.start);
                let end = offset_to_position(content, span.end);
                diagnostics.push(Diagnostic {
                    range: tower_lsp::lsp_types::Range { start, end },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("Tilk-Parser".to_string()),
                    message: format!("{:?}", err),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }
    }
    diagnostics
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Mutex::new(HashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
