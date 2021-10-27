import { Message, MessageDelete, MessageUpdate } from './commonmodels';
import { Timers } from './timers';
export declare namespace Bot {
    type EventType = "MESSAGE_CREATE" | "MESSAGE_UPDATE" | "MESSAGE_DELETE";
    type EventListenerFunction<T> = (a: T) => void;
    type EventDataType<T extends EventType> = T extends "MESSAGE_CREATE" ? MessageCreate : T extends "MESSAGE_UPDATE" ? MessageUpdate : T extends "MESSAGE_DELETE" ? MessageDelete : never;
    export function on<U extends EventType>(eventType: U, f: EventListenerFunction<EventDataType<U>>): void;
    export function registerMeta(meta: ScriptMeta): void;
    export interface ScriptMeta {
        name: string;
        author?: string;
        version?: string;
        description?: string;
        timers?: (Timers.IntervalTimerCron | Timers.IntervalTimerSeconds)[];
    }
    export type MessageCreate = Message;
    export {};
}
