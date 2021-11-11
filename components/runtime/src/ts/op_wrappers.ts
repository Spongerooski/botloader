import { Message, ScriptMeta } from "./commonmodels";

// This file contains op wrappers
// They are used internally and you should generally not need to use them in your own scripts.
// May be removed from the publid API at some point.

export namespace OpWrappers {
    export interface CreateMessageData {
        content: string,
        channelId: string,
    }

    export async function createMessage(args: CreateMessageData): Promise<Message> {
        return await Deno.core.opAsync(
            "op_jack_sendmessage",
            args
        ) as Message;
    }

    export interface InteractionFollowUpData {
        content: string,
        token: string,
    }

    export async function interactionFollowUp(args: InteractionFollowUpData): Promise<Message> {
        return await Deno.core.opAsync(
            "op_interaction_followup",
            args
        ) as Message;
    }

    export function scriptStarted(meta: ScriptMeta) {
        Deno.core.opSync(
            "op_botloader_script_start",
            meta
        );
    }
}