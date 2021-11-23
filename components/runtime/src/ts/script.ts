import { Commands } from "./commands";
import { OpCreateMessageFields, OpEditMessageFields, Guild, Message, Role } from "./models/index";
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
    getGuild(): Guild {
        return OpWrappers.getGuild()
    }
    editGuild() { }

    // Message functions
    getMessage() { }
    getMessages() { }

    createMessage(channelId: string, fields: OpCreateMessageFields): Promise<Message> {
        return OpWrappers.createChannelMessage({
            channelId,
            fields,
        });
    }
    editMessage(channelId: string, messageId: string, fields: OpEditMessageFields): Promise<Message> {
        return OpWrappers.editChannelMessage({
            channelId,
            messageId,
            fields,
        });
    }

    deleteMessage(channelId: string, messageId: string) {
        return OpWrappers.deleteChannelMessage({
            channelId,
            messageId,
        })
    }

    bulkDeleteMessages(channelId: string, ...messageIds: string[]) {
        return OpWrappers.deleteChannelMessagesBulk({
            channelId,
            messageIds,
        })
    }

    // Role functions
    getRole(roleId: string): Promise<Role> {
        return OpWrappers.getRole(roleId);
    }
    getRoles(): Promise<Role[]> {
        return OpWrappers.getRoles();
    }

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