import type { User } from "./User";

export interface Member {
  deaf: boolean;
  guild_id: string;
  joined_at: number | null;
  mute: boolean;
  nick: string | null;
  pending: boolean;
  premium_since: number | null;
  roles: Array<string>;
  user: User;
}
