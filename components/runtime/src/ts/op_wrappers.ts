import { Message } from "./commonmodels";

export interface CreateMessageData {
    content: string,
    channelId: string,
}

export async function CreateMessage(args: CreateMessageData): Promise<Message> {
    return await Deno.core.opAsync(
        "op_jack_sendmessage",
        args
    ) as Message;
}