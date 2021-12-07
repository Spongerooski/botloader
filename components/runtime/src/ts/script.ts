import { Commands } from "./commands";
import { Ops, Events, Discord } from "./models";
import { EventDataType, EventListenerFunction, EventType, InternalEventSystem, ScriptEventMuxer } from "./events";
import { OpWrappers } from "./op_wrappers";
import { Storage } from "./storage";

export class Script {

    scriptId: number;
    description: string;

    eventMuxer = new ScriptEventMuxer();
    commandSystem = new Commands.System();
    intervalTimers: IntervalTimerListener[] = [];

    storageBuckets: Storage.Bucket<unknown>[] = [];

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

    registerIntervalTimer(name: string, interval: string | number, callback: () => any) {
        let timerType;
        if (typeof interval === "number") {
            timerType = { minutes: interval };
        } else {
            timerType = { cron: interval };
        }

        this.intervalTimers.push({
            callback,
            timer: {
                name: name,
                interval: timerType,
            }
        });
    }

    registerStorageBucket<T extends Storage.Bucket<U>, U>(bucket: T): T {
        this.storageBuckets.push(bucket);
        return bucket;
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
            intervalTimers: this.intervalTimers.map(inner => inner.timer),
        });

        this.commandSystem.addEventListeners(this.eventMuxer);
        InternalEventSystem.registerEventMuxer(this.eventMuxer);
        this.eventMuxer.listeners.push({
            f: this.onInterval.bind(this),
            event: "BOTLOADER_INTERVAL_TIMER_FIRED",
        })
    }

    private onInterval(evt: Events.IntervalTimerEvent) {
        const timer = this.intervalTimers.find(timer => timer.timer.name === evt.name && this.scriptId === evt.scriptId);
        if (timer) {
            timer.callback();
        }
    }

    // Guild functions
    getGuild(): Discord.Guild {
        return OpWrappers.getGuild()
    }
    // editGuild() { }

    // Message functions
    getMessage(channelId: string, messageId: string): Promise<Discord.Message> {
        return OpWrappers.getMessage({
            channelId,
            messageId,
        })
    }

    getMessages(channelId: string, options?: GetMessagesOptions): Promise<Discord.Message[]> {
        return OpWrappers.getMessages({
            channelId,
            after: options?.after,
            before: options?.before,
            limit: options?.limit,
        })
    }

    createMessage(channelId: string, fields: Ops.OpCreateMessageFields): Promise<Discord.Message> {
        return OpWrappers.createChannelMessage({
            channelId,
            fields,
        });
    }
    editMessage(channelId: string, messageId: string, fields: Ops.OpEditMessageFields): Promise<Discord.Message> {
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
    getRole(roleId: string): Promise<Discord.Role> {
        return OpWrappers.getRole(roleId);
    }
    getRoles(): Promise<Discord.Role[]> {
        return OpWrappers.getRoles();
    }

    // createRole() { }
    // editRole() { }
    // deleteRole() { }

    // Channel functions
    getChannel(channelId: string): Promise<Discord.GuildChannel> {
        return OpWrappers.getChannel(channelId);
    }
    getChannels(): Promise<Discord.GuildChannel[]> {
        return OpWrappers.getChannels();
    }

    // createChannel() { }
    // editChannel() { }
    // deleteChannel() { }

    // Invite functions
    // getInvite() { }
    // getInvites() { }
    // createInvite() { }
    // deleteInvite() { }

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

interface IntervalTimerListener {
    timer: Ops.IntervalTimer,
    callback: () => any,
}

interface GetMessagesOptions {
    limit?: number,
    after?: string,
    before?: string,
}