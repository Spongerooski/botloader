import type { StickerType } from "./StickerType";
import type { User } from "./User";
import type { StickerFormatType } from "./StickerFormatType";

export interface Sticker {
  available: boolean;
  description: string | null;
  formatType: StickerFormatType;
  guildId: string | null;
  id: string;
  name: string;
  packId: string | null;
  sortValue: number | null;
  tags: string;
  user: User | null;
  kind: StickerType;
}
