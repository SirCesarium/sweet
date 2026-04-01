//! Backend implementation for the Sweet Language Server.

pub mod diag;

use dashmap::DashMap;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use swt::Config;
use swt::analyzer::analyze_content;
use swt::analyzer::ignore::get_disabled_rules;
use swt::languages::{Language, LanguageRegistry};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
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
    pub workspace_roots: Arc<RwLock<Vec<PathBuf>>>,
    pub pending_validations: Arc<DashMap<Url, JoinHandle<()>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_roots: Arc::new(RwLock::new(Vec::new())),
            pending_validations: Arc::new(DashMap::new()),
        }
    }

    pub async fn validate_document(&self, uri: Url, content: String) {
        if let Some(report) = self.perform_analysis(uri.clone(), content).await {
            let config = Config::load(&uri.to_file_path().unwrap_or_default()).unwrap_or_default();
            let diagnostics = diag::generate(&report, &config);
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn perform_analysis(&self, uri: Url, content: String) -> Option<swt::FileReport> {
        let path = uri.to_file_path().ok()?;

        if !Config::is_supported_file(&path) {
            return None;
        }

        let canonical_path = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());

        {
            let roots_lock = self.workspace_roots.read().await;
            if !roots_lock.is_empty()
                && !roots_lock
                    .iter()
                    .any(|root| canonical_path.starts_with(root))
            {
                return None;
            }
        }

        let config = Config::load(&canonical_path).unwrap_or_default();
        if config.is_excluded(&canonical_path) {
            return None;
        }

        let extension = canonical_path.extension()?.to_str()?;
        let thresholds = config.get_thresholds(extension);
        let disabled_rules = get_disabled_rules(content.as_bytes());

        Some(analyze_content(
            content.as_bytes(),
            extension,
            &thresholds,
            &canonical_path,
            &config,
            &disabled_rules,
            true,
        ))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut roots = Vec::new();

        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                if let Ok(path) = folder.uri.to_file_path() {
                    let canonical = fs::canonicalize(&path).unwrap_or(path);
                    roots.push(canonical);
                }
            }
        }

        if roots.is_empty()
            && let Some(root_uri) = params.root_uri
            && let Ok(path) = root_uri.to_file_path()
        {
            let canonical = fs::canonicalize(&path).unwrap_or(path);
            roots.push(canonical);
        }

        *self.workspace_roots.write().await = roots;

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
        self.validate_document(params.text_document.uri, params.text_document.text)
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
        let workspace_roots = self.workspace_roots.clone();
        let pending_validations = self.pending_validations.clone();
        let uri_clone = uri.clone();
        let content = change.text;

        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(300)).await;

            let path = uri_clone.to_file_path().unwrap_or_default();
            if !Config::is_supported_file(&path) {
                return;
            }

            let canonical_path = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());

            {
                let roots_lock = workspace_roots.read().await;
                if !roots_lock.is_empty()
                    && !roots_lock
                        .iter()
                        .any(|root| canonical_path.starts_with(root))
                {
                    return;
                }
            }

            let config = Config::load(&canonical_path).unwrap_or_default();
            if config.is_excluded(&canonical_path) {
                return;
            }

            let extension = canonical_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            let thresholds = config.get_thresholds(extension);
            let disabled_rules = get_disabled_rules(content.as_bytes());
            let report = analyze_content(
                content.as_bytes(),
                extension,
                &thresholds,
                &canonical_path,
                &config,
                &disabled_rules,
                true,
            );

            let diagnostics = diag::generate(&report, &config);
            let () = client
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
