export interface PartialMember {
  deaf: boolean;
  joinedAt: number | null;
  mute: boolean;
  nick: string | null;
  premiumSince: number | null;
  roles: Array<string>;
}
