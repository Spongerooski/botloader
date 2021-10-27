import { Message, MessageDelete, MessageUpdate } from './commonmodels';

export type EventType = "MESSAGE_CREATE" | "MESSAGE_UPDATE" | "MESSAGE_DELETE";
export type EventListenerFunction<T> = (a: T) => void;

export type EventDataType<T extends EventType> =
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
    $jackGlobal.handleDispatch = InternalEventSystem.dispatchEvent;
}

interface DispatchEvent {
    name: string,
    data: any,
}