import { Message, ScriptMeta } from "./commonmodels";
export declare namespace OpWrappers {
    interface CreateMessageData {
        content: string;
        channelId: string;
    }
    function createMessage(args: CreateMessageData): Promise<Message>;
    function scriptStarted(meta: ScriptMeta): void;
}
