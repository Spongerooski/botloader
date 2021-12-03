import type { PublicThread } from "./PublicThread";
import type { TextChannel } from "./TextChannel";
import type { CategoryChannel } from "./CategoryChannel";
import type { VoiceChannel } from "./VoiceChannel";
import type { NewsThread } from "./NewsThread";
import type { PrivateThread } from "./PrivateThread";

export type GuildChannel =
  | CategoryChannel
  | NewsThread
  | PrivateThread
  | PublicThread
  | TextChannel
  | VoiceChannel
  | VoiceChannel;
