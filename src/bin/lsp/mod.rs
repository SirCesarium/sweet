//! Backend implementation for the Sweet Language Server.

pub mod diag;

use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use swt::Config;
use swt::analyzer::analyze_content;
use swt::analyzer::ignore::get_disabled_rules;
use swt::languages::{Language, LanguageRegistry};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
    Position, Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextEdit, Url, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub workspace_root: Arc<RwLock<Option<PathBuf>>>,
    pub pending_validations: Arc<DashMap<Url, JoinHandle<()>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_root: Arc::new(RwLock::new(None)),
            pending_validations: Arc::new(DashMap::new()),
        }
    }

    pub async fn validate_document(&self, uri: Url, content: &str) {
        let Ok(path) = uri.to_file_path() else { return };

        if !Config::is_supported_file(&path) {
            return;
        }

        if let Some(ref root) = *self.workspace_root.read().await
            && !path.starts_with(root)
        {
            return;
        }

        let config = Config::load(&path).unwrap_or_default();

        if config.is_excluded(&path) {
            return;
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let thresholds = config.get_thresholds(extension);
        let disabled_rules = get_disabled_rules(content);
        let report = analyze_content(
            content,
            extension,
            &thresholds,
            &path,
            &config,
            &disabled_rules,
            true,
        );

        let diagnostics = diag::generate(&report, &config);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root_uri) = params.root_uri
            && let Ok(root_path) = root_uri.to_file_path()
        {
            *self.workspace_root.write().await = Some(root_path);
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
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
        let uri = params.text_document.uri;
        let Some(change) = params.content_changes.into_iter().next() else {
            return;
        };

        if let Some((_, handle)) = self.pending_validations.remove(&uri) {
            handle.abort();
        }

        let client = self.client.clone();
        let workspace_root = self.workspace_root.clone();
        let pending_validations = self.pending_validations.clone();
        let uri_clone = uri.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;

            let Ok(path) = uri_clone.to_file_path() else {
                return;
            };

            if let Some(ref root) = *workspace_root.read().await
                && !path.starts_with(root)
            {
                return;
            }

            if !Config::is_supported_file(&path) {
                return;
            }

            let config = Config::load(&path).unwrap_or_default();
            if config.is_excluded(&path) {
                return;
            }

            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            let thresholds = config.get_thresholds(extension);
            let disabled_rules = get_disabled_rules(&change.text);
            let report = analyze_content(
                &change.text,
                extension,
                &thresholds,
                &path,
                &config,
                &disabled_rules,
                true,
            );

            let diagnostics = diag::generate(&report, &config);
            let _ = client
                .publish_diagnostics(uri_clone.clone(), diagnostics, None)
                .await;

            pending_validations.remove(&uri_clone);
        });

        self.pending_validations.insert(uri, handle);
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let mut actions = Vec::new();
        let Ok(path) = params.text_document.uri.to_file_path() else {
            return Ok(None);
        };
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let registry = LanguageRegistry::get();
        let comment = registry
            .get_by_extension(extension)
            .and_then(Language::line_comment)
            .unwrap_or("//");

        for diagnostic in params.context.diagnostics {
            if let Some(rule) = diagnostic.data.as_ref().and_then(|v| v.as_str()) {
                if rule == "unknown" {
                    continue;
                }
                let title = format!("🍬 Disable rule '{rule}' for this file");
                let edit = TextEdit::new(
                    Range::new(Position::new(0, 0), Position::new(0, 0)),
                    format!("{comment} @swt-disable {rule}\n"),
                );
                let mut changes = HashMap::new();
                changes.insert(params.text_document.uri.clone(), vec![edit]);
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title,
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    diagnostics: Some(vec![diagnostic]),
                    ..Default::default()
                }));
            }
        }
        Ok(Some(actions))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
