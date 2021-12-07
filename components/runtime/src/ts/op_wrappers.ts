import {
    OpCreateChannelMessage, LogMessage, OpCreateFollowUpMessage, OpEditChannelMessage,
    Guild, Message, ScriptMeta, OpDeleteMessage, OpDeleteMessagesBulk, Role, GuildChannel,
    OpGetMessage, OpGetMessages,

    OpStorageBucketSetValue, OpStorageBucketIncr, OpStorageBucketList, OpStorageBucketEntry, OpStorageBucketEntryId, OpStorageBucketSortedList, OpStorageBucketSetIf,
} from "./models/index";

// This file contains op wrappers
// They are used internally and you should generally not need to use them in your own scripts.
// May be removed from the publid API at some point.

export namespace OpWrappers {
    export function scriptStarted(meta: ScriptMeta) {
        Deno.core.opSync(
            "op_botloader_script_start",
            meta
        );
    }

    export function consoleLog(args: LogMessage) {
        Deno.core.opSync(
            "op_botloader_log",
            args
        );
    }

    export function getGuild(): Guild {
        return Deno.core.opSync("discord_get_guild");
    }

    export async function getMessage(args: OpGetMessage): Promise<Message> {
        return await Deno.core.opAsync(
            "discord_get_message",
            args
        ) as Message;
    }

    export async function getMessages(args: OpGetMessages): Promise<Message[]> {
        return await Deno.core.opAsync(
            "discord_get_messages",
            args
        ) as Message[];
    }

    export async function createChannelMessage(args: OpCreateChannelMessage): Promise<Message> {
        return await Deno.core.opAsync(
            "discord_create_message",
            args
        ) as Message;
    }

    export async function editChannelMessage(args: OpEditChannelMessage): Promise<Message> {
        return await Deno.core.opAsync(
            "discord_edit_message",
            args
        ) as Message;
    }

    export async function deleteChannelMessage(args: OpDeleteMessage): Promise<void> {
        await Deno.core.opAsync(
            "discord_delete_message",
            args
        );
    }
    export async function deleteChannelMessagesBulk(args: OpDeleteMessagesBulk): Promise<void> {
        await Deno.core.opAsync(
            "discord_bulk_delete_messages",
            args
        );
    }

    export async function createInteractionFollowup(args: OpCreateFollowUpMessage): Promise<Message> {
        return await Deno.core.opAsync(
            "discord_create_followup_message",
            args
        ) as Message;
    }

    export async function getRole(roleId: string): Promise<Role> {
        return await Deno.core.opSync(
            "discord_get_role",
            roleId
        ) as Role;
    }

    export async function getRoles(): Promise<Role[]> {
        return await Deno.core.opSync(
            "discord_get_roles",
        ) as Role[];
    }

    export async function getChannels(): Promise<GuildChannel[]> {
        return await Deno.core.opAsync(
            "discord_get_channels",
        );
    }

    export async function getChannel(channelId: string): Promise<GuildChannel> {
        return await Deno.core.opAsync(
            "discord_get_channel",
            channelId,
        );
    }

    // Storage
    export async function bucketStorageSet(opts: OpStorageBucketSetValue): Promise<OpStorageBucketEntry> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_set", opts);
    }

    export async function bucketStorageSetIf(opts: OpStorageBucketSetIf): Promise<OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_set_if", opts);
    }

    export async function bucketStorageGet(opts: OpStorageBucketEntryId): Promise<OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_get", opts);
    }

    export async function bucketStorageDel(opts: OpStorageBucketEntryId): Promise<OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_del", opts);
    }

    export async function bucketStorageList(opts: OpStorageBucketList): Promise<OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_list", opts);
    }

    export async function bucketStorageIncr(opts: OpStorageBucketIncr): Promise<OpStorageBucketEntry> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_incr", opts);
    }

    export async function bucketStorageSortedList(opts: OpStorageBucketSortedList): Promise<OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_sorted_list", opts);
    }

}