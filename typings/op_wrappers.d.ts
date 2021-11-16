import { CreateChannelMessage, CreateFollowUpMessage, EditChannelMessage, Guild, Message, ScriptMeta } from "./commonmodels";
export declare namespace OpWrappers {
    function createChannelMessage(args: CreateChannelMessage): Promise<Message>;
    function editChannelMessage(args: EditChannelMessage): Promise<Message>;
    function createInteractionFollowup(args: CreateFollowUpMessage): Promise<Message>;
    function scriptStarted(meta: ScriptMeta): void;
    function getGuild(): Guild;
}
