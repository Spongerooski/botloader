import { CreateChannelMessage, CreateFollowUpMessage, EditChannelMessage, Guild, Message, ScriptMeta } from "./commonmodels";

// This file contains op wrappers
// They are used internally and you should generally not need to use them in your own scripts.
// May be removed from the publid API at some point.

export namespace OpWrappers {

    export async function createChannelMessage(args: CreateChannelMessage): Promise<Message> {
        return await Deno.core.opAsync(
            "discord_create_message",
            args
        ) as Message;
    }

    export async function editChannelMessage(args: EditChannelMessage): Promise<Message> {
        return await Deno.core.opAsync(
            "discord_edit_message",
            args
        ) as Message;
    }

    export async function createInteractionFollowup(args: CreateFollowUpMessage): Promise<Message> {
        return await Deno.core.opAsync(
            "discord_create_followup_message",
            args
        ) as Message;
    }

    export function scriptStarted(meta: ScriptMeta) {
        Deno.core.opSync(
            "op_botloader_script_start",
            meta
        );
    }

    export function getGuild(): Guild {
        return Deno.core.opSync("discord_get_guild");
    }
}