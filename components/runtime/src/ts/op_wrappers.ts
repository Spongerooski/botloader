import {
    Ops,
    Discord,
} from "./models";

// This file contains op wrappers
// They are used internally and you should generally not need to use them in your own scripts.
// May be removed from the publid API at some point.

export namespace OpWrappers {
    export function scriptStarted(meta: Ops.ScriptMeta) {
        Deno.core.opSync(
            "op_botloader_script_start",
            meta
        );
    }

    export function consoleLog(args: Ops.LogMessage) {
        Deno.core.opSync(
            "op_botloader_log",
            args
        );
    }

    export function getGuild(): Discord.Guild {
        return Deno.core.opSync("discord_get_guild");
    }

    export async function getMessage(args: Ops.OpGetMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_get_message",
            args
        );
    }

    export async function getMessages(args: Ops.OpGetMessages): Promise<Discord.Message[]> {
        return await Deno.core.opAsync(
            "discord_get_messages",
            args
        );
    }

    export async function createChannelMessage(args: Ops.OpCreateChannelMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_create_message",
            args
        );
    }

    export async function editChannelMessage(args: Ops.OpEditChannelMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_edit_message",
            args
        );
    }

    export async function deleteChannelMessage(args: Ops.OpDeleteMessage): Promise<void> {
        await Deno.core.opAsync(
            "discord_delete_message",
            args
        );
    }
    export async function deleteChannelMessagesBulk(args: Ops.OpDeleteMessagesBulk): Promise<void> {
        await Deno.core.opAsync(
            "discord_bulk_delete_messages",
            args
        );
    }

    export async function createInteractionFollowup(args: Ops.OpCreateFollowUpMessage): Promise<Discord.Message> {
        return await Deno.core.opAsync(
            "discord_create_followup_message",
            args
        );
    }

    export async function getRole(roleId: string): Promise<Discord.Role> {
        return await Deno.core.opSync(
            "discord_get_role",
            roleId
        );
    }

    export async function getRoles(): Promise<Discord.Role[]> {
        return await Deno.core.opSync(
            "discord_get_roles",
        );
    }

    export async function getChannels(): Promise<Discord.GuildChannel[]> {
        return await Deno.core.opAsync(
            "discord_get_channels",
        );
    }

    export async function getChannel(channelId: string): Promise<Discord.GuildChannel> {
        return await Deno.core.opAsync(
            "discord_get_channel",
            channelId,
        );
    }

    // Storage
    export async function bucketStorageSet(opts: Ops.OpStorageBucketSetValue): Promise<Ops.OpStorageBucketEntry> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_set", opts);
    }

    export async function bucketStorageSetIf(opts: Ops.OpStorageBucketSetIf): Promise<Ops.OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_set_if", opts);
    }

    export async function bucketStorageGet(opts: Ops.OpStorageBucketEntryId): Promise<Ops.OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_get", opts);
    }

    export async function bucketStorageDel(opts: Ops.OpStorageBucketEntryId): Promise<Ops.OpStorageBucketEntry | null> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_del", opts);
    }

    export async function bucketStorageList(opts: Ops.OpStorageBucketList): Promise<Ops.OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_list", opts);
    }

    export async function bucketStorageIncr(opts: Ops.OpStorageBucketIncr): Promise<Ops.OpStorageBucketEntry> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_incr", opts);
    }

    export async function bucketStorageSortedList(opts: Ops.OpStorageBucketSortedList): Promise<Ops.OpStorageBucketEntry[]> {
        return await Deno.core.opAsync("op_botloader_bucket_storage_sorted_list", opts);
    }

}