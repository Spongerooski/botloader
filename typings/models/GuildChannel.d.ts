import type { NewsThread } from "./NewsThread";
import type { PrivateThread } from "./PrivateThread";
import type { TextChannel } from "./TextChannel";
import type { VoiceChannel } from "./VoiceChannel";
import type { PublicThread } from "./PublicThread";
import type { CategoryChannel } from "./CategoryChannel";
export declare type GuildChannel = CategoryChannel | NewsThread | PrivateThread | PublicThread | TextChannel | VoiceChannel | VoiceChannel;
