import { Commands } from "./commands";
import { EventDataType, EventListenerFunction, EventType, InternalEventSystem, ScriptEventMuxer } from "./events";
import { OpWrappers } from "./op_wrappers";

export class Script {

    description: string;

    eventMuxer = new ScriptEventMuxer();
    commandSystem = new Commands.System();

    private runCalled = false;

    constructor(description: string) {
        this.description = description;
    }

    on<T extends EventType>(eventType: T, f: EventListenerFunction<EventDataType<T>>) {
        this.eventMuxer.listeners.push({
            f: f,
            event: eventType,
        });
    }

    registerCommand<T extends Commands.OptionsMap>(cmd: Commands.CommandDef<T>) {
        this.commandSystem.commands.push(cmd as Commands.CommandDef<Commands.OptionsMap>);
    }

    run() {
        if (this.runCalled) {
            throw new Error("run already called");
        }

        this.runCalled = true;

        const [cmds, groups] = this.commandSystem.genOpBinding();

        OpWrappers.scriptStarted({
            description: this.description,
            commands: cmds,
            commandGroups: groups,
        });

        this.commandSystem.addEventListeners(this.eventMuxer);
        InternalEventSystem.registerEventMuxer(this.eventMuxer);
    }
}