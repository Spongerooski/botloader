import type { ThreadMember } from "./ThreadMember";
import type { AutoArchiveDuration } from "./AutoArchiveDuration";
import type { ThreadMetadata } from "./ThreadMetadata";
export interface NewsThread {
    default_auto_archive_duration: AutoArchiveDuration | null;
    guild_id: string;
    id: string;
    kind: "GuildNewsThread";
    last_message_id: string | null;
    member: ThreadMember | null;
    member_count: number;
    message_count: number;
    name: string;
    owner_id: string | null;
    parent_id: string | null;
    rate_limit_per_user: bigint | null;
    thread_metadata: ThreadMetadata;
}
