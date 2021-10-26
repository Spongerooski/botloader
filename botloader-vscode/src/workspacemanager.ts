import * as vscode from 'vscode';
import { ApiClient } from './apiclient';
import { GuildScriptWorkspace } from './guildspace';

export class WorkspaceManager implements vscode.Disposable {
    openGuildWorkspaces: GuildScriptWorkspace[] = [];
    apiClient: ApiClient;

    otherDisposables: vscode.Disposable[] = [];

    constructor(apiClient: ApiClient) {
        this.apiClient = apiClient;

        this.otherDisposables.push(vscode.workspace.onDidChangeWorkspaceFolders(this.workspaceFoldersChange.bind(this)));

        for (let folder of vscode.workspace.workspaceFolders || []) {
            this.checkFolder(folder.uri);
        }
    }

    async workspaceFoldersChange(evt: vscode.WorkspaceFoldersChangeEvent) {
        for (let added of evt.added) {
            await this.checkFolder(added.uri);
        }

        for (let removed of evt.removed) {
            const index = this.openGuildWorkspaces.findIndex(e => e.folder === removed.uri);
            if (index !== -1) {
                const elem = this.openGuildWorkspaces[index];
                elem.dispose();
                this.openGuildWorkspaces.splice(index, 1);
            }
        }
    }


    async checkFolder(folder: vscode.Uri) {
        try {
            // should throw an error if it dosen't exist
            await vscode.workspace.fs.stat(vscode.Uri.joinPath(folder, "/.botloader/index.json"));
            if (!this.openGuildWorkspaces.some(elem => elem.folder === folder)) {
                this.openGuildWorkspaces.push(new GuildScriptWorkspace(folder, this.apiClient));
            }
        } catch { }
    }

    dispose() {
        for (let dis of this.otherDisposables) {
            dis.dispose();
        }

        for (let guild of this.openGuildWorkspaces) {
            guild.dispose();
        }
    }
}