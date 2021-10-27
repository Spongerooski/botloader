import { createHash } from 'crypto';
import * as vscode from 'vscode';
import { ApiClient } from './apiclient';
import { relative } from 'path';

export class GuildScriptWorkspace implements vscode.Disposable, vscode.FileDecorationProvider {
    folder: vscode.Uri;
    apiClient: ApiClient;

    disposables: vscode.Disposable[] = [];

    scm: vscode.SourceControl;
    changedFilesGroup: vscode.SourceControlResourceGroup;

    onChangeFileDecosEmitter: vscode.EventEmitter<vscode.Uri>;

    constructor(folder: vscode.Uri, apiClient: ApiClient) {
        console.log("botloader: adding script workspace folder:", folder);
        this.folder = folder;
        this.apiClient = apiClient;

        this.scm = vscode.scm.createSourceControl("botloader", "BotLoader", this.folder);
        this.scm.inputBox.visible = false;
        this.changedFilesGroup = this.scm.createResourceGroup("changed", "Changed scripts");

        const watcherPattern = new vscode.RelativePattern(this.folder, "*.ts");
        console.log("Watcing: ", watcherPattern);
        const watcher = vscode.workspace.createFileSystemWatcher(watcherPattern);
        console.log(watcher);

        this.onChangeFileDecosEmitter = new vscode.EventEmitter();

        this.disposables.push(watcher);
        this.disposables.push(vscode.window.registerFileDecorationProvider(this));

        this.scm.quickDiffProvider = {
            provideOriginalResource: ((uri: vscode.Uri, cancel: vscode.CancellationToken) => {
                return this.provideOriginalResource(uri, cancel);
            }).bind(this),
        };

        watcher.onDidChange(this.onFileDidhange.bind(this));
        watcher.onDidCreate(this.onFileDidCreate.bind(this));
        watcher.onDidDelete(this.onFileDidDelete.bind(this));

        this.initialScan();
    }

    get onDidChangeFileDecorations(): vscode.Event<vscode.Uri | vscode.Uri[] | undefined> | undefined {
        return this.onChangeFileDecosEmitter.event;
    }

    provideFileDecoration(uri: vscode.Uri, token: vscode.CancellationToken): vscode.ProviderResult<vscode.FileDecoration> {
        console.log("Checking file deco yooo", uri);

        const wsFolder = vscode.workspace.getWorkspaceFolder(uri);
        if (wsFolder?.uri.toString() !== this.folder.toString()) {
            return undefined;
        }

        const uriString = uri.toString();
        const state = this.changedFilesGroup.resourceStates.find(s => s.resourceUri.toString() === uriString);
        if (state) {
            let changeState = (state as ResourceState).state;
            return new vscode.FileDecoration(stateBadges[changeState], stateLabels[changeState], this.stateColor(changeState));
        } else {
            return null;
        }
    }

    async provideOriginalResource(uri: vscode.Uri, cancel: vscode.CancellationToken) {

        const relative = vscode.workspace.asRelativePath(uri, false);
        console.log("Checking file diff yooo", uri, relative);
        const indexPath = vscode.Uri.joinPath(this.folder, ".botloader/scripts/", relative + ".bloader");
        return indexPath;
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
        const filesIndexNames = filesIndex.filter(f => f[0].endsWith(".ts.bloader")).map(f => f[0].slice(0, f[0].length - 8));

        let deletedFiles = filesIndexNames.filter(iff => !filesWorking.some(wf => wf[0] === iff));
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

        let uriIndex = vscode.Uri.joinPath(this.folder, "/.botloader/scripts/" + name + ".bloader");

        try {
            await vscode.workspace.fs.stat(uriIndex);
        } catch {
            this.setFileCreated(uri);
            return;
        }


        let hashWorking = await this.hashFile(uri);
        let hashIndex = await this.hashFile(uriIndex);

        if (hashIndex !== hashWorking) {
            // modified
            console.log(name, "Changed");
            this.setFileModified(uri);
        } else {
            // unchanged
            console.log(name + " is unmodified");
            this.removeFileResourceState(uri);
        }
    }

    modifiedStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Modified", iconPath: new vscode.ThemeIcon("pulse", new vscode.ThemeColor(themeColorModifiedUri)) };
    createdStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Created", iconPath: new vscode.ThemeIcon("new-file", new vscode.ThemeColor(themeColorAddedUri)) };
    deletedStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Deleted", strikeThrough: true, iconPath: new vscode.ThemeIcon("trash", new vscode.ThemeColor(themeColorDeletedUri)) };

    setFileModified(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.modifiedStateDeco, state: ChangeState.modified });
    }

    setFileCreated(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.createdStateDeco, state: ChangeState.created });
    }

    setFileDeleted(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.deletedStateDeco, state: ChangeState.deleted });
    }

    setFileResourceState(state: ResourceState) {
        const current = this.changedFilesGroup.resourceStates.findIndex(c => {
            return c.resourceUri.toString() === state.resourceUri.toString();
        });
        if (current !== -1) {
            // we have to create a new array to presumably trigger the setter for the ui to update? 
            // idk, the docs are pretty shit and literally just tell you to look at a 2k+ line file with no comments and a bunch of 
            // internal concepts you have no idea about as an "example", honestly fuck off  
            let newArr = [...this.changedFilesGroup.resourceStates];
            newArr[current] = state;
            this.changedFilesGroup.resourceStates = newArr;

            this.onChangeFileDecosEmitter.fire(state.resourceUri);
            return;
        }

        this.changedFilesGroup.resourceStates = [...this.changedFilesGroup.resourceStates, state];
        this.onChangeFileDecosEmitter.fire(state.resourceUri);
    }


    removeFileResourceState(uri: vscode.Uri) {
        const index = this.changedFilesGroup.resourceStates.findIndex(u => u.resourceUri.toString() === uri.toString());
        const newArr = [...this.changedFilesGroup.resourceStates];
        newArr.splice(index, 1);
        this.changedFilesGroup.resourceStates = newArr;
        this.onChangeFileDecosEmitter.fire(uri);
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
        let indexPath = vscode.Uri.joinPath(this.folder, "/.botloader/scripts/" + relativePath + ".bloader");
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




    themeColorAdded = new vscode.ThemeColor(themeColorAddedUri);
    themeColorModified = new vscode.ThemeColor(themeColorModifiedUri);
    themeColorDeleted = new vscode.ThemeColor(themeColorDeletedUri);

    stateColor(state: ChangeState) {
        switch (state) {
            case ChangeState.created:
                return this.themeColorAdded;
            case ChangeState.deleted:
                return this.themeColorDeleted;
            case ChangeState.modified:
                return this.themeColorModified;
        }
    }
}

enum ChangeState {
    created,
    modified,
    deleted,
};

// why don't i just put them in the initializer you ask? 
// because if we reorder the enum then it will be fucked
let stateBadges: string[] = [];
stateBadges[ChangeState.created] = "U";
stateBadges[ChangeState.modified] = "M";
stateBadges[ChangeState.deleted] = "D";

let stateLabels: string[] = [];
stateLabels[ChangeState.created] = "Untracked";
stateLabels[ChangeState.modified] = "Modied";
stateLabels[ChangeState.deleted] = "Deleted";

interface ResourceState extends vscode.SourceControlResourceState {
    state: ChangeState,
}

const themeColorAddedUri = "botloaderDecoration.untrackedResourceForeground";
const themeColorModifiedUri = "botloaderDecoration.modifiedResourceForeground";
const themeColorDeletedUri = "botloaderDecoration.deletedResourceForeground";