export interface PartialMember {
    deaf: boolean;
    joinedAt: bigint | null;
    mute: boolean;
    nick: string | null;
    premiumSince: bigint | null;
    roles: Array<string>;
}
