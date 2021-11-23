import { CommandInteraction, Message, MessageDelete, MessageUpdate } from './models/index';

export type EventType = "BOTLOADER_COMMAND_INTERACTION_CREATE" | "MESSAGE_CREATE" | "MESSAGE_UPDATE" | "MESSAGE_DELETE";
export type EventListenerFunction<T> = (a: T) => void;

export type EventDataType<T extends EventType> =
    T extends "BOTLOADER_COMMAND_INTERACTION_CREATE" ? CommandInteraction :
    T extends "MESSAGE_CREATE" ? Message :
    T extends "MESSAGE_UPDATE" ? MessageUpdate :
    T extends "MESSAGE_DELETE" ? MessageDelete
    : never;

export class ScriptEventMuxer {

    listeners: EventListener[] = [];

    handleEvent(evt: DispatchEvent) {
        for (let listener of this.listeners) {
            if (listener.event === evt.name) {
                listener.f(evt.data)
            }
        }
    }
}

interface EventListener {
    f: (arg: any) => void;
    event: EventType;
}

export namespace InternalEventSystem {

    const eventMuxers: ScriptEventMuxer[] = [];

    export function registerEventMuxer(muxer: ScriptEventMuxer) {
        eventMuxers.push(muxer)
    }

    export function dispatchEvent(evt: DispatchEvent) {
        for (let muxer of eventMuxers) {
            muxer.handleEvent(evt);
        }
    }
}

if ((typeof $jackGlobal) !== "undefined") {
    $jackGlobal.runEventLoop(InternalEventSystem.dispatchEvent)
}

interface DispatchEvent {
    name: string,
    data: any,
}