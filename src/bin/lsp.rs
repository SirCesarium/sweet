#![deny(clippy::pedantic)]

use swt::Config;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, Position, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl Backend {
    async fn validate_document(&self, uri: Url, content: &str) {
        let Ok(path) = uri.to_file_path() else {
            return;
        };

        if !Config::is_supported_file(&path) {
            return;
        }

        let config = Config::load(&path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let thresholds = config.get_thresholds(extension);

        let report =
            swt::analyzer::analyze_content(content, extension, &thresholds, &path, &config);

        let mut diagnostics = Vec::new();

        for issue in report.issues {
            diagnostics.push(Diagnostic {
                range: Range::new(Position::new(0, 0), Position::new(0, 80)),
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("🍬 Sweet: {issue}"),
                source: Some("sweet".to_string()),
                ..Default::default()
            });
        }

        for duplicate in report.duplicates {
            #[allow(clippy::cast_possible_truncation)]
            let start_line = (duplicate.line as u32).saturating_sub(1);
            diagnostics.push(Diagnostic {
                range: Range::new(Position::new(start_line, 0), Position::new(start_line, 80)),
                severity: Some(DiagnosticSeverity::HINT),
                message: format!(
                    "🍬 Sweet: Code duplication detected! (repeated in {} other places)",
                    duplicate.occurrences.len()
                ),
                source: Some("sweet".to_string()),
                ..Default::default()
            });
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Sweet LSP server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.validate_document(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.validate_document(params.text_document.uri, &change.text)
                .await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::LspService;

    #[tokio::test]
    async fn test_initialization() {
        let (service, _) = LspService::new(|client| Backend { client });
        let params = InitializeParams::default();
        let result = service.inner().initialize(params).await.unwrap();
        assert!(result.capabilities.text_document_sync.is_some());
    }

    #[tokio::test]
    async fn test_unsupported_file() {
        let (service, _) = LspService::new(|client| Backend { client });
        let uri = Url::parse("file:///test.txt").unwrap();
        // Should not panic or return error, just skip
        service.inner().validate_document(uri, "test").await;
    }
}
