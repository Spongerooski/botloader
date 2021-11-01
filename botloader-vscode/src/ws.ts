/* eslint-disable @typescript-eslint/naming-convention */
import { setTimeout } from "timers";
import { User } from "./apiclient";
import { WebSocket, CloseEvent, MessageEvent } from 'ws';

export class BotloaderWS {
    ws?: WebSocket;
    baseUrl: string;
    token?: string;
    auth = false;
    onLogMessage: (msg: WsLogItem) => void;

    subQueue: string[] = [];

    constructor(baseUrl: string, onLogMessage: (msg: WsLogItem) => void, token?: string) {
        this.token = token;
        this.baseUrl = baseUrl;
        this.onLogMessage = onLogMessage;
        if (token) {
            this.open();
        }
    }

    setToken(token: string) {
        this.logToOutput("updated token");
        this.token = token;
        if (this.ws) {
            this.ws.onclose = () => { };
            this.ws.close();
        }
        this.open();
    }

    open() {
        let url = this.baseUrl + "/api/ws";
        this.logToOutput("opening ws to " + url);
        this.ws = new WebSocket(url);
        this.ws.onopen = this.wsOnOpen.bind(this);
        this.ws.onclose = this.wsOnClose.bind(this);
        this.ws.onmessage = this.wsOnMessage.bind(this);
    }

    send(cmd: WsCommand) {
        if (this.ws) {
            this.ws.send(JSON.stringify(cmd));
        }
    }

    subscribeGuild(guildId: string) {
        if (!this.auth) {
            this.logToOutput("not authorized yet, pushing to queue... " + guildId);
            this.subQueue.push(guildId);
            return;
        }

        this.logToOutput("subscribing to " + guildId);
        this.send({
            t: "SubscribeLogs",
            d: guildId,
        });
    }

    sendAuth() {
        this.logToOutput("authorizing ws...");
        this.send({
            t: "Authorize",
            d: this.token!,
        });
    }

    wsOnOpen() {
        if (this.token) {
            this.sendAuth();
        }
    }

    wsOnMessage(msg: MessageEvent) {
        let decoded: WsEvent = JSON.parse(msg.data.toString());
        switch (decoded.t) {
            case "AuthSuccess":
                this.auth = true;
                this.logToOutput("successfully authorized");

                for (let g of this.subQueue) {
                    this.subscribeGuild(g);
                }
                this.subQueue = [];

                break;
            case "ScriptLogMessage":
                this.handleScriptLogMessage(decoded);
                break;
            case "SubscriptionsUpdated":
                this.logToOutput("sbuscriptions updated successfully: " + decoded.d);
                break;
        }
    }

    wsOnClose(ev: CloseEvent) {
        this.logToOutput("ws closed :( " + ev.reason);

        let that = this;
        setTimeout(() => {
            that.open();
        }, 1000);
    }

    handleScriptLogMessage(msg: WsEventScriptLogMessage) {
        console.log("Script log message yoo");
        this.onLogMessage(msg.d);
    }

    logToOutput(msg: string) {
        this.onLogMessage({
            kind: "WS",
            message: msg,
        });
    }
}

type WsEventType = "AuthSuccess" | "SubscriptionsUpdated" | "ScriptLogMessage";

type WsEvent = WsEventAuthorized | WsEventSubscriptionsUpdated | WsEventScriptLogMessage;

interface WsEventAuthorized {
    t: "AuthSuccess",
    d: User,
}

interface WsEventSubscriptionsUpdated {
    t: "SubscriptionsUpdated",
    d: string[],
}


interface WsEventScriptLogMessage {
    t: "ScriptLogMessage",
    d: WsScriptLogItem,
}

export type WsLogItem = SimpleWsLogItem | WsScriptLogItem;

export interface SimpleWsLogItem {
    kind: "WS" | "Info",
    message: string,
}

export interface WsScriptLogItem {
    guild_id: string,
    filename: string,
    linenumber: number,
    column: number,
    message: string,
    kind?: "ScriptError" | "ScriptInfo",
}


type WsCommandType = "Authorize" | "SubscribeLogs" | "UnSubscribeLogs";

type WsCommand = WsCommandAuthorize | WsCommandSubscribe | WsCommandSubscribe;

interface WsCommandAuthorize {
    t: "Authorize",
    d: string,
}

interface WsCommandSubscribe {
    t: "SubscribeLogs"
    d: string,
}

interface WsCommandUnSubscribe {
    t: "UnSubscribeLogs"
    d: string,
}


// int*
// interface Auth