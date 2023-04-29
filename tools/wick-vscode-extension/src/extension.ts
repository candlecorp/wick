// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	const schemaUrl: string | undefined = vscode.workspace.getConfiguration('wick.yaml').get('schemaUrl');
	if (schemaUrl) {
		const yamlConfig = vscode.workspace.getConfiguration('yaml');
		const existingSchemas = yamlConfig.get<object>('schemaStore.uri') || {};

		yamlConfig.update('schemaStore.uri', {
			...existingSchemas,
			[schemaUrl]: ['*.wick']
		}, vscode.ConfigurationTarget.Global);
	}
}

// This method is called when your extension is deactivated
export function deactivate() { }
