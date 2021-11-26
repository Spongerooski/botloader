import { Commands } from "./commands";
import { OpCreateMessageFields, OpEditMessageFields, Guild, Message, Role, GuildChannel, IntervalTimer } from "./models/index";
import { EventDataType, EventListenerFunction, EventType, ScriptEventMuxer } from "./events";
export declare class Script {
    scriptId: number;
    description: string;
    eventMuxer: ScriptEventMuxer;
    commandSystem: Commands.System;
    intervalTimers: IntervalTimerListener[];
    private runCalled;
    constructor(id: number);
    on<T extends EventType>(eventType: T, f: EventListenerFunction<EventDataType<T>>): void;
    registerCommand<T extends Commands.OptionsMap>(cmd: Commands.CommandDef<T>): void;
    registerIntervalTimer(name: string, interval: string | number, callback: () => any): void;
    run(): void;
    private onInterval;
    getGuild(): Guild;
    editGuild(): void;
    getMessage(): void;
    getMessages(): void;
    createMessage(channelId: string, fields: OpCreateMessageFields): Promise<Message>;
    editMessage(channelId: string, messageId: string, fields: OpEditMessageFields): Promise<Message>;
    deleteMessage(channelId: string, messageId: string): Promise<void>;
    bulkDeleteMessages(channelId: string, ...messageIds: string[]): Promise<void>;
    getRole(roleId: string): Promise<Role>;
    getRoles(): Promise<Role[]>;
    createRole(): void;
    editRole(): void;
    deleteRole(): void;
    getChannel(channelId: string): Promise<GuildChannel>;
    getChannels(): Promise<GuildChannel[]>;
    createChannel(): void;
    editChannel(): void;
    deleteChannel(): void;
    getInvite(): void;
    getInvites(): void;
    createInvite(): void;
    deleteInvite(): void;
}
interface IntervalTimerListener {
    timer: IntervalTimer;
    callback: () => any;
}
export {};
