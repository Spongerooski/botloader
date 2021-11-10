import { Commands } from "./commands";
import { EventDataType, EventListenerFunction, EventType, ScriptEventMuxer } from "./events";
export declare class Script {
    scriptId: number;
    description: string;
    eventMuxer: ScriptEventMuxer;
    commandSystem: Commands.System;
    private runCalled;
    constructor(id: number);
    on<T extends EventType>(eventType: T, f: EventListenerFunction<EventDataType<T>>): void;
    registerCommand<T extends Commands.OptionsMap>(cmd: Commands.CommandDef<T>): void;
    run(): void;
}
