import { OpCreateChannelMessage, OpCreateFollowUpMessage, OpEditChannelMessage, Guild, Message, ScriptMeta, OpDeleteMessage, OpDeleteMessagesBulk, Role, GuildChannel } from "./models/index";
export declare namespace OpWrappers {
    function scriptStarted(meta: ScriptMeta): void;
    function getGuild(): Guild;
    function createChannelMessage(args: OpCreateChannelMessage): Promise<Message>;
    function editChannelMessage(args: OpEditChannelMessage): Promise<Message>;
    function deleteChannelMessage(args: OpDeleteMessage): Promise<void>;
    function deleteChannelMessagesBulk(args: OpDeleteMessagesBulk): Promise<void>;
    function createInteractionFollowup(args: OpCreateFollowUpMessage): Promise<Message>;
    function getRole(roleId: string): Promise<Role>;
    function getRoles(): Promise<Role[]>;
    function getChannels(): Promise<GuildChannel[]>;
    function getChannel(channelId: string): Promise<GuildChannel>;
}
