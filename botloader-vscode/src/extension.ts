// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import { ApiClient, isErrorResponse, UserGuild } from 'botloader-common';

import { tmpdir } from 'os';
import { mkdtemp } from 'fs/promises';
import { join } from 'path';
import { WorkspaceManager } from './workspacemanager';
import { BotloaderWS, LogItem } from './ws';
import { BotloaderSourceControl, CHANGED_FILES_SCM_GROUP } from './guildspace';
import { createFetcher } from './util';


// this method is called when your extension is activated 
// your extension is activated the very first time the command is executed
export async function activate(context: vscode.ExtensionContext) {

	let outputChannel = vscode.window.createOutputChannel("Botloader");
	context.subscriptions.push(outputChannel);

	let token = await context.secrets.get("botloader-api-key");

	const config = vscode.workspace.getConfiguration("botloader");
	const apiBase: string = config.get("apiHost")!;
	const apiHttps: boolean = config.get("apiHttpsEnabled")!;

	const httpApiBase = apiHttps ? "https://" + apiBase : "http://" + apiBase;
	const wsApiBase = apiHttps ? "wss://" + apiBase : "ws://" + apiBase;

	let ws = new BotloaderWS(wsApiBase, handleLogMessage, token);
	let apiClient = new ApiClient(createFetcher(), httpApiBase, token);

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

		let newClient = new ApiClient(createFetcher(), httpApiBase, key);
		let resp = await newClient.getCurrentUser();

		if (isErrorResponse(resp)) {
			vscode.window.showErrorMessage("Invalid token:" + JSON.stringify(resp));
		} else {
			vscode.window.showInformationMessage(`Logged in as ${resp.username}#${resp.discriminator}`);
			apiClient.token = newClient.token;
			ws.setToken(newClient.token!);
			await context.secrets.store("botloader-api-key", key as string);
		}
	}), vscode.commands.registerCommand('botloader-vscode.push', async (arg: any) => {
		if (isScmGroup(arg)) {
			await manager.pushScmGroup(arg);
		} else if (containsResourceUri(arg)) {
			await manager.pushUri(arg.resourceUri);
		}
	}), vscode.commands.registerCommand('botloader-vscode.sync', async (arg: any) => {
		if (arg === undefined) {
			manager.syncOne();
		} else if (isScmProvider(arg)) {
			manager.syncScm(arg);
		}
	}));

	async function setupWorkspace(guild: UserGuild) {
		let tmpDir = await mkdtemp(join(tmpdir(), "botloader"));
		let dirUri = vscode.Uri.parse("file:/" + tmpDir);

		await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(dirUri, "/.botloader"));
		await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(dirUri, "/.botloader/scripts"));

		let textEncoder = new TextEncoder();
		await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(dirUri, `/.botloader/index.json`), textEncoder.encode(JSON.stringify({
			guild: guild,
			openScripts: [],
		})));

		// await vscode.workspace.fs.copy(vscode.Uri.joinPath(context.extensionUri, "/out/typings/lib.deno_core.d.ts"), vscode.Uri.joinPath(dirUri, "/.botloader/lib.global.d.ts"));
		await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(dirUri, `/tsconfig.json`), textEncoder.encode(JSON.stringify(generateTsConfig(context.extensionPath), undefined, 4)));

		vscode.workspace.updateWorkspaceFolders(0, 0, {
			uri: dirUri,
			name: guild.name,
		});
	}

	function handleLogMessage(item: LogItem) {
		let tag = item.level;
		if (item.guild_id) {
			tag += " " + item.guild_id;
		}
		if (item.script_context) {
			tag += ` ${item.script_context.filename}.ts`;
			if (item.script_context.line_col) {
				const [line, col] = item.script_context.line_col;
				tag += `:${line}:${col}`;
			}
		}

		let full = `[${tag}] ${item.message}`;
		outputChannel.appendLine(full);
		// TODO: remove this?
		outputChannel.show(true);
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

function isScmGroup(arg: any): arg is vscode.SourceControlResourceGroup {
	if ((arg as vscode.SourceControlResourceGroup).id === CHANGED_FILES_SCM_GROUP) {
		return true;
	}

	return false;
}


interface ResourceUriContainer {
	resourceUri: vscode.Uri
}

function containsResourceUri(arg: any): arg is ResourceUriContainer {
	if ((arg as ResourceUriContainer).resourceUri !== undefined) {
		return true;
	}

	return false;
}

function isScmProvider(arg: any): arg is BotloaderSourceControl {
	let cast = arg as BotloaderSourceControl;
	if (cast.isBotloaderSourceControl !== undefined && cast.isBotloaderSourceControl) {
		return true;
	}

	return false;
}
