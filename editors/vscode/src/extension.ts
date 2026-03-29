import * as path from 'node:path';
import * as os from 'node:os';
import {
  workspace,
  languages,
  CompletionItem,
  CompletionItemKind,
  type ExtensionContext,
} from 'vscode';
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const platform = os.platform();
  let binaryName = 'sweet-lsp-linux';

  if (platform === 'win32') {
    binaryName = 'sweet-lsp-win.exe';
  } else if (platform === 'darwin') {
    binaryName = 'sweet-lsp-macos';
  }

  const serverModule = path.join(context.extensionPath, 'bin', binaryName);

  const serverOptions: ServerOptions = {
    run: {
      command: serverModule,
      transport: TransportKind.stdio,
    },
    debug: {
      command: serverModule,
      transport: TransportKind.stdio,
    },
  };

  const supportedLanguages = [
    'rust',
    'python',
    'javascript',
    'typescript',
    'java',
    'csharp',
    'gdscript',
  ];

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      ...supportedLanguages.map((lang) => ({ scheme: 'file', language: lang })),
      { scheme: 'file', pattern: '**/*.gd' },
      { scheme: 'file', pattern: '**/*.rs' },
      { scheme: 'file', pattern: '**/*.py' },
      { scheme: 'file', pattern: '**/*.js' },
      { scheme: 'file', pattern: '**/*.ts' },
      { scheme: 'file', pattern: '**/*.java' },
      { scheme: 'file', pattern: '**/*.cs' },
    ],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/.swtrc'),
    },
  };

  // Autocomplete for @swt-disable
  const provider = languages.registerCompletionItemProvider(
    supportedLanguages.map((lang) => ({ scheme: 'file', language: lang })),
    {
      provideCompletionItems(document, position) {
        const line = document.lineAt(position).text;
        const linePrefix = line.substring(0, position.character);

        // If typing the directive itself
        if (linePrefix.endsWith('@swt-')) {
          const item = new CompletionItem(
            'swt-disable',
            CompletionItemKind.Keyword
          );
          item.documentation = 'Disable specific health checks for this file';
          item.insertText = 'swt-disable ';
          return [item];
        }

        // If typing rules after @swt-disable
        if (linePrefix.includes('@swt-disable')) {
          return [
            new CompletionItem('max-lines', CompletionItemKind.Enum),
            new CompletionItem('max-depth', CompletionItemKind.Enum),
            new CompletionItem('max-imports', CompletionItemKind.Enum),
            new CompletionItem('max-repetition', CompletionItemKind.Enum),
          ];
        }

        return undefined;
      },
    },
    '-',
    ' ' // trigger characters
  );

  context.subscriptions.push(provider);

  client = new LanguageClient(
    'sweet',
    'Sweet LSP Server',
    serverOptions,
    clientOptions
  );

  // Start the client. This will also launch the server
  client.start().catch((err) => {
    console.error(`[Sweet] Failed to start LSP client: ${err}`);
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
