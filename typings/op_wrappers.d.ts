import { OpCreateChannelMessage, LogMessage, OpCreateFollowUpMessage, OpEditChannelMessage, Guild, Message, ScriptMeta, OpDeleteMessage, OpDeleteMessagesBulk, Role, GuildChannel, OpGetMessage, OpGetMessages, OpStorageBucketSetValue, OpStorageBucketIncr, OpStorageBucketList, OpStorageBucketEntry, OpStorageBucketEntryId, OpStorageBucketSortedList, OpStorageBucketSetIf } from "./models/index";
export declare namespace OpWrappers {
    function scriptStarted(meta: ScriptMeta): void;
    function consoleLog(args: LogMessage): void;
    function getGuild(): Guild;
    function getMessage(args: OpGetMessage): Promise<Message>;
    function getMessages(args: OpGetMessages): Promise<Message[]>;
    function createChannelMessage(args: OpCreateChannelMessage): Promise<Message>;
    function editChannelMessage(args: OpEditChannelMessage): Promise<Message>;
    function deleteChannelMessage(args: OpDeleteMessage): Promise<void>;
    function deleteChannelMessagesBulk(args: OpDeleteMessagesBulk): Promise<void>;
    function createInteractionFollowup(args: OpCreateFollowUpMessage): Promise<Message>;
    function getRole(roleId: string): Promise<Role>;
    function getRoles(): Promise<Role[]>;
    function getChannels(): Promise<GuildChannel[]>;
    function getChannel(channelId: string): Promise<GuildChannel>;
    function bucketStorageSet(opts: OpStorageBucketSetValue): Promise<OpStorageBucketEntry>;
    function bucketStorageSetIf(opts: OpStorageBucketSetIf): Promise<OpStorageBucketEntry | null>;
    function bucketStorageGet(opts: OpStorageBucketEntryId): Promise<OpStorageBucketEntry | null>;
    function bucketStorageDel(opts: OpStorageBucketEntryId): Promise<OpStorageBucketEntry | null>;
    function bucketStorageList(opts: OpStorageBucketList): Promise<OpStorageBucketEntry[]>;
    function bucketStorageIncr(opts: OpStorageBucketIncr): Promise<OpStorageBucketEntry>;
    function bucketStorageSortedList(opts: OpStorageBucketSortedList): Promise<OpStorageBucketEntry[]>;
}
