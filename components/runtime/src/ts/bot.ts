import { Message, MessageDelete, MessageUpdate } from './commonmodels';
import { console } from './core_util';

export namespace Bot {

    type EventType = "MESSAGE_CREATE" | "MESSAGE_UPDATE" | "MESSAGE_DELETE";
    type EventListenerFunction<T> = (a: T) => void;

    let eventListeners: EventListener[] = [];

    interface EventListener {
        f: (arg: any) => void;
        event: EventType;
    }

    type EventDataType<T extends EventType> =
        T extends "MESSAGE_CREATE" ? MessageCreate :
        T extends "MESSAGE_UPDATE" ? MessageUpdate :
        T extends "MESSAGE_DELETE" ? MessageDelete
        : never;

    export function on<T extends EventDataType<U>, U extends EventType>(eventType: U, f: EventListenerFunction<T>) {
        let listener: EventListener = {
            f: f,
            event: eventType,
        }

        eventListeners.push(listener);

        console.log(`added event listener for ${eventType}`)
    }

    function dispatchEvent(evt: DispatchEvent) {
        let name = evt.name;
        let data = evt.data;
        console.log("Got event in js!: " + name);
        // console.log("data: " + JSON.stringify(data));

        // onMessageCreate(new Message(data));
        for (var listener of eventListeners) {
            if (listener.event === evt.name) {
                listener.f(evt.data)
            }
        }
    }

    export function registerMeta(meta: ScriptMeta) {
        console.log("Registering meta in js: " + JSON.stringify(meta));
        Deno.core.opSync(
            "op_jack_register_meta",
            meta
        );
    }

    if ((typeof $jackGlobal) !== "undefined") {
        $jackGlobal.handleDispath = dispatchEvent;
    }

    export interface ScriptMeta {
        name: string,
        context: "Channel" | "Guild",

        author?: string,
        version?: string,
        description?: string,
    }

    interface DispatchEvent {
        name: string,
        data: any,
    }

    export type MessageCreate = Message;
}
