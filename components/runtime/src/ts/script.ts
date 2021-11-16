import { Commands } from "./commands";
import { CreateChannelMessage, EditChannelMessage } from "./commonmodels";
import { EventDataType, EventListenerFunction, EventType, InternalEventSystem, ScriptEventMuxer } from "./events";
import { OpWrappers } from "./op_wrappers";

export class Script {

    scriptId: number;
    description: string;

    eventMuxer = new ScriptEventMuxer();
    commandSystem = new Commands.System();

    private runCalled = false;

    constructor(id: number) {
        this.description = `script id ${id}`;
        this.scriptId = id;
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
            scriptId: this.scriptId,
        });

        this.commandSystem.addEventListeners(this.eventMuxer);
        InternalEventSystem.registerEventMuxer(this.eventMuxer);
    }

    // Guild functions
    getGuild() {
        return OpWrappers.getGuild()
    }
    editGuild() { }

    // Message functions
    getMessage() { }
    getMessages() { }

    createMessage(args: CreateChannelMessage) {
        return OpWrappers.createChannelMessage(args);
    }
    editMessage(args: EditChannelMessage) {
        return OpWrappers.editChannelMessage(args);
    }
    deleteMessage() { }
    bulkDeleteMessages() { }

    // Role functions
    getRole() { }
    getRoles() { }
    createRole() { }
    editRole() { }
    deleteRole() { }

    // Channel functions
    getChannel() { }
    getChannels() { }
    createChannel() { }
    editChannel() { }
    deleteChannel() { }

    // Invite functions
    getInvite() { }
    getInvites() { }
    createInvite() { }
    deleteInvite() { }

    // // Emoji functions
    // getEmoji() { }
    // getEmojis() { }
    // createEmoji() { }
    // editEmoji() { }
    // deleteEmoji() { }


    // // Sticker functions
    // getSticker() { }
    // getStickers() { }
    // createSticker() { }
    // editSticker() { }
    // deleteSticker() { }
}