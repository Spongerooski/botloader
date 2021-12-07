import type { Embed } from "../discord/Embed";
import type { MessageType } from "../discord/MessageType";
import type { Attachment } from "../discord/Attachment";
import type { User } from "../discord/User";
import type { Mention } from "../discord/Mention";

export interface MessageUpdate {
  attachments: Array<Attachment> | null;
  author: User | null;
  channelId: string;
  content: string | null;
  editedTimestamp: number | null;
  embeds: Array<Embed> | null;
  guildId: string | null;
  id: string;
  kind: MessageType | null;
  mentionEveryone: boolean | null;
  mentionRoles: Array<string> | null;
  mentions: Array<Mention> | null;
  pinned: boolean | null;
  timestamp: number | null;
  tts: boolean | null;
}
