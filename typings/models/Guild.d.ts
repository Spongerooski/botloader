import type { MfaLevel } from "./MfaLevel";
import type { PremiumTier } from "./PremiumTier";
import type { VerificationLevel } from "./VerificationLevel";
import type { NsfwLevel } from "./NsfwLevel";
import type { ExplicitContentFilter } from "./ExplicitContentFilter";
import type { DefaultMessageNotificationLevel } from "./DefaultMessageNotificationLevel";
export interface Guild {
    afkChannelId: string | null;
    afkTimeout: number;
    applicationId: string | null;
    banner: string | null;
    defaultMessageNotifications: DefaultMessageNotificationLevel;
    description: string | null;
    discoverySplash: string | null;
    explicitContentFilter: ExplicitContentFilter;
    features: Array<string>;
    icon: string | null;
    id: string;
    joinedAt: number | null;
    large: boolean;
    maxMembers: number | null;
    maxPresences: number | null;
    memberCount: number | null;
    mfaLevel: MfaLevel;
    name: string;
    nsfwLevel: NsfwLevel;
    ownerId: string;
    preferredLocale: string;
    premiumSubscriptionCount: number | null;
    premiumTier: PremiumTier;
    rulesChannelId: string | null;
    splash: string | null;
    systemChannelId: string | null;
    unavailable: boolean;
    vanityUrlCode: string | null;
    verificationLevel: VerificationLevel;
    widgetChannelId: string | null;
    widgetEnabled: boolean | null;
}
