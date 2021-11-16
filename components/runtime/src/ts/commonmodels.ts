export interface CommandInteraction {
    channelId: string,
    id: string,
    member: PartialMember,
    token: string,
    name: string,
    parentName?: string,
    parentParentName?: string,
    options: CommandInteractionOption[],
}

export interface CommandInteractionOption {
    name: string,
    value: CommandInteractionOptionValue,
}

export type CommandInteractionOptionValue = {
    kind: "String",
    value: string,
} | {
    kind: "Integer",
    value: number,
} | {
    kind: "Boolean",
    value: boolean,
} | {
    kind: "User",
    value: string,
} | {
    kind: "Channel",
    value: string,
} | {
    kind: "Role",
    value: string,
} | {
    kind: "Mentionable",
    value: string,
} | {
    kind: "Number",
    value: number,
};

export interface CreateChannelMessage {
    channelId: string,
    fields: CreateMessageFields,
}

export interface EditChannelMessage {
    channelId: string,
    messageId: string,
    fields: EditMessageFields,
}

export interface CreateFollowUpMessage {
    interactionToken: string,
    fields: CreateMessageFields,
}

export interface CreateMessageFields {
    content: string,
    embeds?: Embed[],
    allowedMentions?: AllowedMentions,
}

export interface EditMessageFields {
    content?: string,
    embeds?: Embed[],
    allowedMentions?: AllowedMentions,
}

export interface AllowedMentions {
    parse: ParseTypes[],
    users: string[],
    roles: string[],
    repliedUser: boolean,
}

export type ParseTypes = "Everyone" | "Roles" | "Users";

export interface Embed {
    author?: EmbedAuthor,
    color?: number,
    description?: string,
    fields: EmbedField[],
    footer?: EmbedFooter,
    image?: EmbedImage,
    kind: string,
    provider?: EmbedProvider,
    thumbnail?: EmbedThumbnail,
    timestamp?: number,
    title?: string,
    url?: string,
    video?: EmbedVideo,
}

export interface EmbedAuthor {
    iconUrl?: string,
    name?: string,
    proxyIconUrl?: string,
    url?: string,
}

export interface EmbedField {
    inline: boolean,
    name: string,
    value: string,
}

export interface EmbedFooter {
    iconUrl?: string,
    proxyIconUrl?: string,
    text: string,
}

export interface EmbedImage {
    height?: number,
    proxyUrl?: string,
    url?: string,
    width?: number,
}

export interface EmbedProvider {
    name?: string,
    url?: string,
}

export interface EmbedThumbnail {
    height?: number,
    proxyUrl?: string,
    url?: string,
    width?: number,
}

export interface EmbedVideo {
    height?: number,
    proxyUrl?: string,
    url?: string,
    width?: number,
}

export interface Guild {
    afkChannelId?: string,
    afkTimeout: number,
    applicationId?: string,
    banner?: string,
    defaultMessageNotifications: DefaultMessageNotificationLevel,
    description?: string,
    discoverySplash?: string,
    explicitContentFilter: ExplicitContentFilter,
    features: string[],
    icon?: string,
    id: string,
    joinedAt?: number,
    large: boolean,
    maxMembers?: number,
    maxPresences?: number,
    memberCount?: number,
    mfaLevel: MfaLevel,
    name: string,
    nsfwLevel: NSFWLevel,
    ownerId: string,
    preferredLocale: string,
    premiumSubscriptionCount?: number,
    premiumTier: PremiumTier,
    rulesChannelId?: string,
    splash?: string,
    systemChannelId?: string,
    unavailable: boolean,
    vanityUrlCode?: string,
    verificationLevel: VerificationLevel,
    widgetChannelId?: string,
    widgetEnabled?: boolean,
}

export type DefaultMessageNotificationLevel = "All" | "Mentions";

export type ExplicitContentFilter = "None" | "MembersWithoutRole" | "AllMembers";

export type MfaLevel = "None" | "Elevated";

export type NSFWLevel = "Default" | "Explicit" | "Safe" | "AgeRestricted";

export type PremiumTier = "None" | "Tier1" | "Tier2" | "Tier3";

export type VerificationLevel = "None" | "Low" | "Medium" | "High" | "VeryHigh";

export interface Message {
    activity?: MessageActivity,
    application?: MessageApplication,
    attachments: Attachment[],
    author: User,
    channelId: string,
    content: string,
    editedTimestamp?: number,
    embeds: Embed[],
    flags?: number,
    guildId?: string,
    id: string,
    kind: MessageType,
    member?: PartialMember,
    mentionChannels: ChannelMention[],
    mentionEveryone: boolean,
    mentionRoles: string[],
    mentions: Mention[],
    pinned: boolean,
    reactions: MessageReaction[],
    reference?: MessageReference,
    referencedMessage?: Message,
    timestamp: number,
    tts: boolean,
    webhookId?: string,
}

export interface MessageActivity {
    kind: MessageActivityType,
    partyId?: string,
}

export type MessageActivityType = "Join" | "Spectate" | "Listen" | "JoinRequest";

export interface MessageApplication {
    coverImage?: string,
    description: string,
    icon?: string,
    id: string,
    name: string,
}

export interface Attachment {
    contentType?: string,
    filename: string,
    height?: number,
    id: string,
    proxyUrl: string,
    size: number,
    url: string,
    width?: number,
}

export type MessageType = "Regular" | "RecipientAdd" | "RecipientRemove" | "Call" | "ChannelNameChange" | "ChannelIconChange" | "ChannelMessagePinned" | "GuildMemberJoin" | "UserPremiumSub" | "UserPremiumSubTier1" | "UserPremiumSubTier2" | "UserPremiumSubTier3" | "ChannelFollowAdd" | "GuildDiscoveryDisqualified" | "GuildDiscoveryRequalified" | "GuildDiscoveryGracePeriodInitialWarning" | "GuildDiscoveryGracePeriodFinalWarning" | "Reply" | "GuildInviteReminder" | "ApplicationCommand" | "ThreadCreated" | "ThreadStarterMessage" | "ContextMenuCommand";

export interface PartialMember {
    deaf: boolean,
    joinedAt?: number,
    mute: boolean,
    nick?: string,
    premiumSince?: number,
    roles: string[],
}

export interface ChannelMention {
    guildId: string,
    id: string,
    kind: ChannelType,
    name: string,
}

export type ChannelType = "GuildText" | "Private" | "GuildVoice" | "Group" | "GuildCategory" | "GuildNews" | "GuildStore" | "GuildStageVoice" | "GuildNewsThread" | "GuildPublicThread" | "GuildPrivateThread";

export interface Mention {
    avatar?: string,
    bot: boolean,
    discriminator: number,
    id: string,
    member?: PartialMember,
    username: string,
    publicFlags: number,
}

export interface MessageReaction {
    count: number,
    emoji: ReactionType,
    me: boolean,
}

export type ReactionType = {
    kind: "Custom",
    animated: boolean,
    id: string,
    name?: string,
} | {
    kind: "Unicode",
    name: string,
};

export interface MessageReference {
    channelId?: string,
    guildId?: string,
    messageId?: string,
    failIfNotExists?: boolean,
}

export type StickerType = "Standard" | "Guild";

export interface Sticker {
    available: boolean,
    description?: string,
    formatType: StickerFormatType,
    guildId?: string,
    id: string,
    name: string,
    packId?: string,
    sortValue?: number,
    tags: string,
    user?: User,
    kind: StickerType,
}

export type StickerFormatType = "Png" | "Apng" | "Lottie";

export interface MessageDelete {
    channelId: string,
    guildId?: string,
    id: string,
}

export interface MessageUpdate {
    attachments?: Attachment[],
    author?: User,
    channelId: string,
    content?: string,
    editedTimestamp?: number,
    embeds?: Embed[],
    guildId?: string,
    id: string,
    kind?: MessageType,
    mentionEveryone?: boolean,
    mentionRoles?: string[],
    mentions?: Mention[],
    pinned?: boolean,
    timestamp?: number,
    tts?: boolean,
}

export interface ScriptMeta {
    description: string,
    scriptId: number,
    commands: Command[],
    commandGroups: CommandGroup[],
}

export interface CommandGroup {
    name: string,
    description: string,
    subGroups: CommandSubGroup[],
}

export interface CommandSubGroup {
    name: string,
    description: string,
}

export interface Command {
    name: string,
    description: string,
    options: CommandOption[],
    group?: string,
    subGroup?: string,
}

export type CommandOptionType = "String" | "Integer" | "Boolean" | "User" | "Channel" | "Role" | "Mentionable" | "Number";

export interface CommandOption {
    name: string,
    description: string,
    kind: CommandOptionType,
    required: boolean,
}

export interface User {
    avatar?: string,
    bot: boolean,
    discriminator: number,
    email?: string,
    id: string,
    locale?: string,
    mfaEnabled?: boolean,
    username: string,
    premiumType?: PremiumType,
    publicFlags?: number,
    system?: boolean,
    verified?: boolean,
}

export type PremiumType = "None" | "NitroClassic" | "Nitro";

