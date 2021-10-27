import { Message } from "./commonmodels";
export interface CreateMessageData {
    content: string;
    channelId: string;
}
export declare function CreateMessage(args: CreateMessageData): Promise<Message>;
