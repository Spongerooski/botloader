import { CommandInteraction, Message, MessageDelete, MessageUpdate } from './commonmodels';
export declare type EventType = "BOTLOADER_COMMAND_INTERACTION_CREATE" | "MESSAGE_CREATE" | "MESSAGE_UPDATE" | "MESSAGE_DELETE";
export declare type EventListenerFunction<T> = (a: T) => void;
export declare type EventDataType<T extends EventType> = T extends "BOTLOADER_COMMAND_INTERACTION_CREATE" ? CommandInteraction : T extends "MESSAGE_CREATE" ? Message : T extends "MESSAGE_UPDATE" ? MessageUpdate : T extends "MESSAGE_DELETE" ? MessageDelete : never;
export declare class ScriptEventMuxer {
    listeners: EventListener[];
    handleEvent(evt: DispatchEvent): void;
}
interface EventListener {
    f: (arg: any) => void;
    event: EventType;
}
export declare namespace InternalEventSystem {
    function registerEventMuxer(muxer: ScriptEventMuxer): void;
    function dispatchEvent(evt: DispatchEvent): void;
}
interface DispatchEvent {
    name: string;
    data: any;
}
export {};
