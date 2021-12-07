import type { User } from "./User";

export interface InteractionMember {
  user: User;
  deaf: boolean;
  joinedAt: number;
  mute: boolean;
  nick: string | null;
  premiumSince: number | null;
  roles: Array<string>;
  permissions: string;
}
