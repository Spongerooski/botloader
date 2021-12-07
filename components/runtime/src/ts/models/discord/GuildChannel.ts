import type { PublicThread } from "./PublicThread";
import type { VoiceChannel } from "./VoiceChannel";
import type { PrivateThread } from "./PrivateThread";
import type { CategoryChannel } from "./CategoryChannel";
import type { NewsThread } from "./NewsThread";
import type { TextChannel } from "./TextChannel";

export type GuildChannel =
  | CategoryChannel
  | NewsThread
  | PrivateThread
  | PublicThread
  | TextChannel
  | VoiceChannel
  | VoiceChannel;
