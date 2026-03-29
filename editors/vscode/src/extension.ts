import * as path from 'node:path';
import * as os from 'node:os';
import { workspace, type ExtensionContext } from 'vscode';
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

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'rust' },
      { scheme: 'file', language: 'python' },
      { scheme: 'file', language: 'javascript' },
      { scheme: 'file', language: 'typescript' },
      { scheme: 'file', language: 'java' },
      { scheme: 'file', language: 'csharp' },
      { scheme: 'file', language: 'gdscript' },
    ],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/.swtrc'),
    },
  };

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
