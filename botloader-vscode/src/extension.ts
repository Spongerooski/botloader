// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import { ApiClient, isErrorResponse, Script, UserGuild } from './apiclient';

import { tmpdir } from 'os';
import { mkdtemp } from 'fs/promises';
import { join } from 'path';
import { WorkspaceManager } from './workspacemanager';
import { BotloaderWS, WsLogItem, WsScriptLogItem } from './ws';
import { CHANGED_FILES_SCM_GROUP } from './guildspace';

const API_HOST_BASE = "127.0.0.1:7447";
const API_BASE_URL = "http://" + API_HOST_BASE;
const WS_BASE_URL = "ws://" + API_HOST_BASE;
// this method is called when your extension is activated 
// your extension is activated the very first time the command is executed
export async function activate(context: vscode.ExtensionContext) {

	let outputChannel = vscode.window.createOutputChannel("botloader");
	context.subscriptions.push(outputChannel);

	let token = await context.secrets.get("botloader-api-key");
	let ws = new BotloaderWS(WS_BASE_URL, handleLogMessage, token);
	let apiClient = new ApiClient(API_BASE_URL, token);

	let manager = new WorkspaceManager(apiClient, ws);
	context.subscriptions.push(manager);

	context.subscriptions.push(vscode.commands.registerCommand('botloader-vscode.setup-workspace', async () => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user
		let resp = await apiClient.getCurrentUserGuilds();
		if (isErrorResponse(resp)) {
			vscode.window.showErrorMessage("Invalid token:" + JSON.stringify(resp));
			return;
		}

		const filtered = resp.guilds.filter(elem => elem.connected && hasAdmin(elem.guild));
		let selection = await vscode.window.showQuickPick(filtered.map(elem => `${elem.guild.name} (${elem.guild.id})`), {
			canPickMany: false,
			title: "Select server"
		});

		let selectedServer = filtered.find(elem => `${elem.guild.name} (${elem.guild.id})` === selection);
		if (!selectedServer) {
			vscode.window.showErrorMessage("Unknown server");
		}

		vscode.window.showInformationMessage(`selected as ${selection}`);

		await setupWorkspace(selectedServer!.guild);


	}), vscode.commands.registerCommand('botloader-vscode.set-accesstoken', async () => {

		let key = await vscode.window.showInputBox({
			password: true,
			title: "API key",
		});

		let newClient = new ApiClient(API_BASE_URL, key);
		let resp = await newClient.getCurrentUser();

		if (isErrorResponse(resp)) {
			vscode.window.showErrorMessage("Invalid token:" + JSON.stringify(resp));
		} else {
			vscode.window.showInformationMessage(`Logged in as ${resp.username}#${resp.discriminator}`);
			apiClient.token = newClient.token;
			ws.setToken(newClient.token!);
			await context.secrets.store("botloader-api-key", key as string);
		}
	}), vscode.commands.registerCommand('botloader-vscode.push', async (f: any) => {
		if ((f as { resourceUri: vscode.Uri }).resourceUri !== undefined) {
			let { resourceUri } = f as { resourceUri: vscode.Uri };
			await manager.pushUri(resourceUri);
		} else if ((f as vscode.SourceControlResourceGroup).id === CHANGED_FILES_SCM_GROUP) {
			let fScm = f as vscode.SourceControlResourceGroup;
			await manager.pushScmGroup(fScm);
		}
	}));

	async function setupWorkspace(guild: UserGuild) {
		let tmpDir = await mkdtemp(join(tmpdir(), "botloader"));
		let dirUri = vscode.Uri.parse("file:/" + tmpDir);


		let scripts = await apiClient.getAllScripts(guild.id);
		if (isErrorResponse(scripts)) {
			throw new Error("failed fetching scripts: " + JSON.stringify(scripts));
		}

		await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(dirUri, "/.botloader"));
		await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(dirUri, "/.botloader/scripts"));

		let textEncoder = new TextEncoder();
		for (let script of scripts) {
			await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(dirUri, `/${script.name}.ts`), textEncoder.encode(script.original_source));
			await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(dirUri, `/.botloader/scripts/${script.name}.ts.bloader`), textEncoder.encode(script.original_source));
		}

		await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(dirUri, `/.botloader/index.json`), textEncoder.encode(JSON.stringify({
			guild: guild,
			openScripts: scripts.map(script => { return { id: script.id, name: script.name }; }),
		})));

		await vscode.workspace.fs.copy(vscode.Uri.joinPath(context.extensionUri, "/out/typings/lib.deno_core.d.ts"), vscode.Uri.joinPath(dirUri, "/.botloader/lib.global.d.ts"));

		await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(dirUri, `/tsconfig.json`), textEncoder.encode(JSON.stringify(generateTsConfig(context.extensionPath), undefined, 4)));

		vscode.workspace.updateWorkspaceFolders(0, 0, {
			uri: dirUri,
			name: guild.name,
		});
	}

	function handleLogMessage(msg: WsLogItem) {
		let tag = msg.kind || "unknown";
		let content = ": " + msg.message;
		if (!msg.kind || msg.kind === "ScriptError" || msg.kind === "ScriptInfo") {
			msg = msg as WsScriptLogItem;
			content = `[${msg.filename}:${msg.linenumber}:${msg.column}]${content}`;
		}

		let full = `(${tag})${content}`;
		outputChannel.appendLine(full);
		outputChannel.show();
	}
}

function generateTsConfig(extensionPath: string) {
	return {
		"include": [
			"*.ts",
			".botloader/*.d.ts"
		],
		"compilerOptions": {
			"module": "ES2020",
			"noImplicitAny": true,
			"removeComments": true,
			"preserveConstEnums": true,
			"sourceMap": false,
			"target": "ES2020",
			"alwaysStrict": true,
			"strict": true,
			"strictNullChecks": true,
			"baseUrl": "./",
			"paths": {
				"botloader": [
					extensionPath + "/out/typings/index"
				]
			}
		}
	};
}

// this method is called when your extension is deactivated
export function deactivate() { }

const permAdmin = BigInt("0x0000000008");
const permManageServer = BigInt("0x0000000020");

function hasAdmin(g: UserGuild): boolean {
	if (g.owner) {
		return true;
	}


	const perms = BigInt(g.permissions);
	if ((perms & permAdmin) === permAdmin) {
		return true;
	}

	if ((perms & permManageServer) === permManageServer) {
		return true;
	}

	return false;
}

interface BotloaderJson {
	guild: UserGuild,
	openScripts: number[],
}

