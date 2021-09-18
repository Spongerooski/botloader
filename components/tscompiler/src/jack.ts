import { console } from './core_util';

export namespace Jack {

    type EventType = "MESSAGE_CREATE"
    type EventListenerFunction<T> = (a: T) => void;

    let eventListeners: EventListener[] = [];

    interface EventListener {
        f: (arg: any) => void;
        event: EventType;
    }

    export function addEventListener<T>(typ: EventType, f: EventListenerFunction<T>) {
        let listener: EventListener = {
            f: f,
            event: typ,
        }

        eventListeners.push(listener);

        console.log(`added event listener for ${typ}`)
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

    export interface MessageCreate {
        id: String,
        channel_id: string,
        author: User,
        content: String,
    }

    export interface User {
        id: string,
        username: String,
        bot: boolean,
    }
}
