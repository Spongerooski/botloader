import { Commands } from "./commands";
import { Ops, Events, Discord } from "./models";
import { InternalEventSystem, EventMuxer, EventTypes } from "./events";
import { OpWrappers } from "./op_wrappers";
import { Storage } from "./storage";

/**
 * The script class is the main way you interact with botloader and discord.
 */
export class Script {

    get scriptId() {
        return this._scriptId;
    }

    get description() {
        return this._description;
    }

    private _scriptId: number;
    private _description: string;

    private events = new EventMuxer();
    private commandSystem = new Commands.System();
    private intervalTimers: IntervalTimerListener[] = [];
    private storageBuckets: Storage.Bucket<unknown>[] = [];

    private runCalled = false;

    /**
     * @internal
     */
    constructor(id: number) {
        this._description = `script id ${id}`;
        this._scriptId = id;
    }



    on(eventType: "MESSAGE_DELETE", cb: (evt: EventTypes["MESSAGE_DELETE"]) => void): void;
    on(eventType: "MESSAGE_UPDATE", cb: (evt: EventTypes["MESSAGE_UPDATE"]) => void): void;
    on(eventType: "MESSAGE_CREATE", cb: (evt: EventTypes["MESSAGE_CREATE"]) => void): void;
    on<T extends keyof EventTypes>(eventType: T, cb: (evt: EventTypes[T]) => void): void {
        this.events.on(eventType, cb);
    }

    /**
     * Register a command to this guild.
     * 
     * @param cmd The command to register
     * 
     * @example ```ts
     * * script.registerCommand({
     *     name: "sub",
     *     description: "subtracts 2 numbers",
     *     group: mathGroup,
     *     options: {
     *         "a": { description: "a", kind: "Integer", required: true },
     *         "b": { description: "b", kind: "Integer", required: true },
     *     },
     *     callback: async (ctx, args) => {
     *         const result = args.a - args.b;
     *         await ctx.sendResponse(`Result: ${result}`)
     *     }
     * });
     * ```
     */
    registerCommand<T extends Commands.OptionsMap>(cmd: Commands.CommandDef<T>) {
        this.commandSystem.commands.push(cmd as Commands.CommandDef<Commands.OptionsMap>);
    }

    /**
     * 
     * @param name The name of the timer
     * @param interval The interval, either in minutes for running the callback at every x minutes, or a cron style timer. 
     * 
     * https://crontab.guru/ is a neat helper for making cron intervals 
     * 
     * @param callback Callback to run at every interval
     * 
     * @example ```ts
     *  script.registerIntervalTimer("gaming", "*\/5 * * * *", () => {
     *     // do stuff here
     * });
     * ```
     */
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

    /**
     * Register a storage bucket to the script
     * 
     * Note that the same storage bucket can be registered in multiple scripts, and you can use this to share data betweem scripts.
     *
     * @param bucket The bucket itself
     * @returns The registered bucket
     * 
     * @example ```ts
     * interface Data{
     *     key: string,
     * }
     * script.registerStorageBucket(new Storage.JsonBucket<Data>("fun-data"));
     * ```
     */
    registerStorageBucket<T extends Storage.Bucket<U>, U>(bucket: T): T {
        this.storageBuckets.push(bucket);
        return bucket;
    }

    /**
     * @internal
     */
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

        this.commandSystem.addEventListeners(this.events);
        InternalEventSystem.registerEventMuxer(this.events);

        this.events.on("BOTLOADER_INTERVAL_TIMER_FIRED", this.onInterval.bind(this));
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

    async getMember(id: string): Promise<Discord.Member | undefined> {
        return (await OpWrappers.getMembers([id]))[0] || undefined;
    }

    async getMembers(ids: string[]): Promise<(Discord.Member | null)[]> {
        return await OpWrappers.getMembers(ids);
    }
}

interface IntervalTimerListener {
    timer: Ops.IntervalTimer,
    callback: () => any,
}

export interface GetMessagesOptions {
    /**
     * Limit max results, max 100, default 50
     */
    limit?: number,

    /**
     * Return messages made after this message id
     */
    after?: string,
    /**
     * Return messages made before this message id
     */
    before?: string,
}