import { createHash } from 'crypto';
import * as vscode from 'vscode';
import { ApiClient } from './apiclient';
import { relative } from 'path';

export class GuildScriptWorkspace implements vscode.Disposable {
    folder: vscode.Uri;
    apiClient: ApiClient;

    disposables: vscode.Disposable[] = [];

    scm: vscode.SourceControl;
    changedFilesGroup: vscode.SourceControlResourceGroup;

    constructor(folder: vscode.Uri, apiClient: ApiClient) {
        console.log("botloader: adding script workspace folder:", folder);
        this.folder = folder;
        this.apiClient = apiClient;

        this.scm = vscode.scm.createSourceControl("botloader", "BotLoader", this.folder);
        this.changedFilesGroup = this.scm.createResourceGroup("changed", "Changed scripts");

        const watcherPattern = new vscode.RelativePattern(this.folder, "*.ts");
        console.log("Watcing: ", watcherPattern);
        const watcher = vscode.workspace.createFileSystemWatcher(watcherPattern);
        console.log(watcher);
        this.disposables.push(watcher);

        watcher.onDidChange(this.onFileDidhange.bind(this));
        watcher.onDidCreate(this.onFileDidCreate.bind(this));
        watcher.onDidDelete(this.onFileDidDelete.bind(this));

        this.initialScan();
    }

    dispose() {
        for (let dis of this.disposables) {
            dis.dispose();
        }
    }

    async initialScan() {
        let filesWorking = await vscode.workspace.fs.readDirectory(this.folder);
        filesWorking = filesWorking.filter(f => f[0].endsWith(".ts"));


        let filesIndex = await vscode.workspace.fs.readDirectory(vscode.Uri.joinPath(this.folder, "/.botloader/scripts"));
        filesIndex = filesIndex.filter(f => f[0].endsWith(".ts"));

        let deletedFiles = filesIndex.filter(iff => !filesWorking.some(wf => wf[0] === iff[0]));
        for (let file of filesWorking) {
            await this.checkWorkingFile(file[0]);
        }

        for (let file of deletedFiles) {
            this.setFileDeleted(vscode.Uri.joinPath(this.folder, "/" + file[0]));
        }
    }

    async checkWorkingFile(name: string) {
        console.log("Checking ", name);

        let uri = vscode.Uri.joinPath(this.folder, "/" + name);
        // let stat = await vscode.workspace.fs.stat(uri);

        let uriIndex = vscode.Uri.joinPath(this.folder, "/.botloader/scripts/" + name);

        try {
            await vscode.workspace.fs.stat(uriIndex);
        } catch {
            this.setFileCreated(uri);
            return;
        }


        let hashWorking = await this.hashFile(uri);
        let hashIndex = await this.hashFile(uriIndex);

        if (hashIndex !== hashWorking) {
            console.log(name, "Changed");
            this.setFileModified(uri);
        }
    }

    modifiedStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Modified", iconPath: new vscode.ThemeIcon("pulse") };
    createdStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Created", iconPath: new vscode.ThemeIcon("new-file") };
    deletedStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Deleted", strikeThrough: true, iconPath: new vscode.ThemeIcon("trash") };

    setFileModified(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.modifiedStateDeco });
    }

    setFileCreated(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.createdStateDeco });
    }

    setFileDeleted(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.deletedStateDeco });
    }

    setFileResourceState(state: vscode.SourceControlResourceState) {
        const current = this.changedFilesGroup.resourceStates.findIndex(c => {
            return c.resourceUri.toString() === state.resourceUri.toString();
        });
        if (current !== -1) {
            // let lastHalf = this.changedFilesGroup.resourceStates.splice(current);
            // lastHalf[current] = state;
            // this.changedFilesGroup.resourceStates = [this.changedFilesGroup.resourceStates] 
            let newArr = [...this.changedFilesGroup.resourceStates];
            newArr[current] = state;
            this.changedFilesGroup.resourceStates = newArr;
            return;
        }

        this.changedFilesGroup.resourceStates = [...this.changedFilesGroup.resourceStates, state];
    }

    async onFileDidhange(uri: vscode.Uri) {
        const relativePath = relative(this.folder.path, uri.path);
        this.checkWorkingFile(relativePath);
    }


    async onFileDidCreate(uri: vscode.Uri) {
        const relativePath = relative(this.folder.path, uri.path);
        this.checkWorkingFile(relativePath);
    }


    async onFileDidDelete(uri: vscode.Uri) {
        const relativePath = relative(this.folder.path, uri.path);
        let indexPath = vscode.Uri.joinPath(this.folder, "/.botloader/scripts/" + relativePath);
        try {
            await vscode.workspace.fs.stat(indexPath);
            console.log("Deleted existing file");
            this.setFileDeleted(uri);
        } catch {
            return;
        }
    }



    async hashFile(file: vscode.Uri) {
        const contents = await vscode.workspace.fs.readFile(file);

        let hash = createHash("sha256");
        hash.update(contents);
        return hash.digest('hex');
    }
}