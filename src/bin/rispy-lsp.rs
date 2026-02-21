//! rispy-lsp: Language Server Protocol implementation for the Rispy Scheme dialect.
//!
//! Run this binary and configure your editor to use it as an LSP server for `.scm` or `.rsp` files.
//! It communicates over stdin/stdout using the LSP JSON-RPC protocol.
//!
//! Features:
//!   - Diagnostics (parse errors reported as you type)
//!   - Completion (built-in functions, special forms, user-defined symbols)
//!   - Hover documentation for built-ins and special forms
//!   - Document symbols (list all `define`d names in a file)
//!   - Go-to-definition within a single file

#![feature(box_patterns)]
#![feature(map_try_insert)]
#![feature(iterator_try_reduce)]
#![feature(if_let_guard)]
#![feature(assert_matches)]

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use rispy::compile::BUILTIN_FNS;
use rispy::expr::Expr;
use rispy::parse::{parse, ParseInput, SrcLoc};

// ---------------------------------------------------------------------------
// Documentation strings for built-in functions
// ---------------------------------------------------------------------------

fn builtin_docs(name: &str) -> Option<&'static str> {
    Some(match name {
        "+" => "(+ num...) → num\nAdd zero or more numbers together.",
        "-" => "(- a b) → num\nSubtract b from a.",
        "*" => "(* a b) → num\nMultiply a and b.",
        "/" => "(/ a b) → num\nDivide a by b.",
        "%" => "(% a b) → num\nRemainder of a divided by b.",
        "^" => "(^ a b) → num\nRaise a to the power b.",
        "=" => "(= a b) → bool\nStructural equality.",
        "<" => "(< a b) → bool\nTrue if a is less than b.",
        ">" => "(> a b) → bool\nTrue if a is greater than b.",
        "not" => "(not bool) → bool\nLogical negation.",
        "cons" => "(cons head tail) → pair\nConstruct a new pair.",
        "car" => "(car pair) → value\nReturn the head of a pair.",
        "cdr" => "(cdr pair) → value\nReturn the tail of a pair.",
        "nil?" => "(nil? x) → bool\nTrue if x is the empty list.",
        "pair?" => "(pair? x) → bool\nTrue if x is a pair (list).",
        "number?" => "(number? x) → bool\nTrue if x is a number.",
        "boolean?" => "(boolean? x) → bool\nTrue if x is a boolean.",
        "string?" => "(string? x) → bool\nTrue if x is a string.",
        "symbol?" => "(symbol? x) → bool\nTrue if x is a symbol.",
        "function?" => "(function? x) → bool\nTrue if x is a lambda.",
        "abs" => "(abs num) → num\nAbsolute value.",
        "str-append" => "(str-append a b) → string\nConcatenate two strings.",
        "to-string" => "(to-string x) → string\nConvert any value to its string representation.",
        "error" => "(error msg) → !\nSignal a runtime error with the given message.",
        _ => return None,
    })
}

fn special_form_docs(name: &str) -> Option<&'static str> {
    Some(match name {
        "lambda" => "(lambda (params...) body...)\nCreate an anonymous function.",
        "define" => "(define name value) or (define (name params...) body...)\nBind a name to a value or define a function.",
        "if" => "(if predicate consequent alternate)\nConditional expression.",
        "and" => "(and a b) → bool\nShort-circuit logical AND.",
        "or" => "(or a b) → bool\nShort-circuit logical OR.",
        "quote" => "(quote expr) or 'expr\nReturn expr unevaluated.",
        "apply" => "(apply fn args-list)\nApply fn to the elements of args-list.",
        "display" => "(display value)\nPrint value to output.",
        "defmacro" => "(defmacro (name params...) body...)\nDefine a compile-time macro.",
        _ => return None,
    })
}

// All known special forms
const SPECIAL_FORMS: &[&str] = &[
    "lambda", "define", "if", "and", "or", "quote", "apply", "display", "defmacro",
];

// ---------------------------------------------------------------------------
// Helper: walk an AST and collect (name, srcloc) for all top-level `define`s
// ---------------------------------------------------------------------------

fn collect_defines(exprs: &[Expr]) -> Vec<(String, SrcLoc)> {
    let mut out = Vec::new();
    for expr in exprs {
        if let Expr::Pair(
            box Expr::Keyword(kw, _),
            box rest,
            _,
        ) = expr
        {
            if kw == "define" {
                match rest {
                    // (define name value)
                    Expr::Pair(box Expr::Keyword(name, Some(loc)), _, _) => {
                        out.push((name.clone(), loc.clone()));
                    }
                    // (define (name params...) body...)
                    Expr::Pair(
                        box Expr::Pair(box Expr::Keyword(name, Some(loc)), _, _),
                        _,
                        _,
                    ) => {
                        out.push((name.clone(), loc.clone()));
                    }
                    _ => {}
                }
            } else if kw == "defmacro" {
                // (defmacro (name params...) body...)
                if let Expr::Pair(
                    box Expr::Pair(box Expr::Keyword(name, Some(loc)), _, _),
                    _,
                    _,
                ) = rest
                {
                    out.push((name.clone(), loc.clone()));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Helper: convert a 1-based SrcLoc to a zero-based LSP Position / Range
// ---------------------------------------------------------------------------

fn srcloc_to_position(loc: &SrcLoc) -> Position {
    Position {
        line: loc.line.saturating_sub(1),
        character: (loc.column.saturating_sub(1)) as u32,
    }
}

fn srcloc_to_range(loc: &SrcLoc) -> Range {
    let pos = srcloc_to_position(loc);
    Range { start: pos, end: pos }
}

// ---------------------------------------------------------------------------
// Per-document state
// ---------------------------------------------------------------------------

#[derive(Default)]
struct DocumentState {
    /// Raw source text
    text: String,
    /// Successfully parsed AST (empty if parse failed)
    ast: Vec<Expr>,
    /// All define names with their source locations
    defines: Vec<(String, SrcLoc)>,
}

impl DocumentState {
    fn update(&mut self, text: String) {
        self.text = text.clone();
        match parse(&ParseInput {
            source: &text,
            file_name: Some("buffer"),
        }) {
            Ok(ast) => {
                self.defines = collect_defines(&ast);
                self.ast = ast;
            }
            Err(_) => {
                self.ast = vec![];
                self.defines = vec![];
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Backend
// ---------------------------------------------------------------------------

struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Publish diagnostics for a document by re-parsing its text.
    async fn publish_diagnostics(&self, uri: Url, text: &str) {
        let diagnostics = match parse(&ParseInput {
            source: text,
            file_name: Some("buffer"),
        }) {
            Ok(_) => vec![],
            Err(err) => {
                // The error message from nom doesn't have a clean position, so we
                // report it at 0:0 with the raw message.
                vec![Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: err,
                    source: Some("rispy".to_string()),
                    ..Default::default()
                }]
            }
        };
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

// ---------------------------------------------------------------------------
// LSP trait implementation
// ---------------------------------------------------------------------------

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Receive the full document text on every change
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                // Completion
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["(".to_string()]),
                    ..Default::default()
                }),
                // Hover
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                // Document symbols (outline)
                document_symbol_provider: Some(OneOf::Left(true)),
                // Go-to-definition
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "rispy-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "rispy-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    // --- Document lifecycle -------------------------------------------------

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();

        let mut docs = self.documents.lock().await;
        let state = docs.entry(uri.clone()).or_default();
        state.update(text.clone());
        drop(docs);

        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        // We requested FULL sync, so there's always exactly one change with the whole text
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text.clone();

            let mut docs = self.documents.lock().await;
            let state = docs.entry(uri.clone()).or_default();
            state.update(text.clone());
            drop(docs);

            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.lock().await.remove(&uri);
        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    // --- Completion ---------------------------------------------------------

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> LspResult<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let docs = self.documents.lock().await;
        let user_defines: Vec<(String, SrcLoc)> = docs
            .get(uri)
            .map(|s| s.defines.clone())
            .unwrap_or_default();
        drop(docs);

        let mut items: Vec<CompletionItem> = Vec::new();

        // Special forms
        for form in SPECIAL_FORMS {
            items.push(CompletionItem {
                label: form.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("special form".to_string()),
                documentation: special_form_docs(form).map(|doc| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc.to_string(),
                    })
                }),
                ..Default::default()
            });
        }

        // Built-in functions
        for name in BUILTIN_FNS.keys() {
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("built-in".to_string()),
                documentation: builtin_docs(name).map(|doc| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc.to_string(),
                    })
                }),
                ..Default::default()
            });
        }

        // User-defined symbols from the current file
        for (name, _) in &user_defines {
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some("defined in file".to_string()),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    // --- Hover --------------------------------------------------------------

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        // Find the word under cursor from the raw document text
        let docs = self.documents.lock().await;
        let text = match docs.get(uri) {
            Some(s) => s.text.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let word = word_at_position(&text, pos);
        if word.is_empty() {
            return Ok(None);
        }

        let doc = builtin_docs(&word)
            .or_else(|| special_form_docs(&word));

        Ok(doc.map(|d| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: d.to_string(),
            }),
            range: None,
        }))
    }

    // --- Document Symbols ---------------------------------------------------

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;
        let docs = self.documents.lock().await;
        let defines = match docs.get(uri) {
            Some(s) => s.defines.clone(),
            None => return Ok(None),
        };
        drop(docs);

        #[allow(deprecated)]
        let symbols: Vec<SymbolInformation> = defines
            .iter()
            .map(|(name, loc)| {
                let range = srcloc_to_range(loc);
                SymbolInformation {
                    name: name.clone(),
                    kind: SymbolKind::FUNCTION,
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    tags: None,
                    container_name: None,
                }
            })
            .collect();

        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }

    // --- Go-to-definition ---------------------------------------------------

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.documents.lock().await;
        let state = match docs.get(uri) {
            Some(s) => (s.text.clone(), s.defines.clone()),
            None => return Ok(None),
        };
        drop(docs);

        let (text, defines) = state;
        let word = word_at_position(&text, pos);
        if word.is_empty() {
            return Ok(None);
        }

        // Find the definition location
        for (name, loc) in &defines {
            if name == &word {
                let range = srcloc_to_range(loc);
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                })));
            }
        }

        Ok(None)
    }
}

// ---------------------------------------------------------------------------
// Utility: extract the identifier word at a given (line, char) position
// ---------------------------------------------------------------------------

fn word_at_position(text: &str, pos: Position) -> String {
    let line_idx = pos.line as usize;
    let char_idx = pos.character as usize;

    let line = match text.lines().nth(line_idx) {
        Some(l) => l,
        None => return String::new(),
    };

    // Characters that can appear in a Rispy identifier
    let is_ident = |c: char| !matches!(c, '(' | ')' | ' ' | '\t' | '\n' | '\r' | ';' | '"');

    let bytes = line.as_bytes();
    if char_idx >= bytes.len() {
        return String::new();
    }

    // Walk left to find the start of the identifier
    let mut start = char_idx;
    while start > 0 && is_ident(bytes[start - 1] as char) {
        start -= 1;
    }

    // Walk right to find the end
    let mut end = char_idx;
    while end < bytes.len() && is_ident(bytes[end] as char) {
        end += 1;
    }

    line[start..end].to_string()
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
