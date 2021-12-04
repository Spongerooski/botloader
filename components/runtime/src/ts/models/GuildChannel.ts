import type { CategoryChannel } from "./CategoryChannel";
import type { TextChannel } from "./TextChannel";
import type { NewsThread } from "./NewsThread";
import type { PublicThread } from "./PublicThread";
import type { VoiceChannel } from "./VoiceChannel";
import type { PrivateThread } from "./PrivateThread";

export type GuildChannel =
  | CategoryChannel
  | NewsThread
  | PrivateThread
  | PublicThread
  | TextChannel
  | VoiceChannel
  | VoiceChannel;
