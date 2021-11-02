use super::embed::Embed;
use super::user::User;
use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::message::sticker::{StickerId, StickerPackId},
    datetime::Timestamp,
    id::{
        ApplicationId, AttachmentId, ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId,
        WebhookId,
    },
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub activity: Option<MessageActivity>,
    pub application: Option<MessageApplication>,
    pub attachments: Vec<Attachment>,
    pub author: User,
    pub channel_id: ChannelId,
    pub content: String,
    pub edited_timestamp: Option<u64>,
    pub embeds: Vec<Embed>,
    pub flags: Option<u64>,
    pub guild_id: Option<GuildId>,
    pub id: MessageId,
    pub kind: MessageType,
    pub member: Option<PartialMember>,
    pub mention_channels: Vec<ChannelMention>,
    pub mention_everyone: bool,
    pub mention_roles: Vec<RoleId>,
    pub mentions: Vec<Mention>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub reference: Option<MessageReference>,
    pub referenced_message: Option<Box<Message>>,
    pub timestamp: u64,
    pub tts: bool,
    pub webhook_id: Option<WebhookId>,
}

impl From<twilight_model::channel::Message> for Message {
    fn from(v: twilight_model::channel::Message) -> Self {
        Self {
            activity: v.activity.map(From::from),
            application: v.application.map(From::from),
            attachments: v.attachments.into_iter().map(From::from).collect(),
            author: v.author.into(),
            channel_id: v.channel_id,
            content: v.content,
            edited_timestamp: v.edited_timestamp.map(|ts| ts.as_secs()),
            embeds: v.embeds.into_iter().map(From::from).collect(),
            flags: v.flags.map(|f| f.bits()),
            guild_id: v.guild_id,
            id: v.id,
            kind: v.kind.into(),
            member: v.member.map(From::from),
            mention_channels: v.mention_channels.into_iter().map(From::from).collect(),
            mention_everyone: v.mention_everyone,
            mention_roles: v.mention_roles,
            mentions: v.mentions.into_iter().map(From::from).collect(),
            pinned: v.pinned,
            reactions: v.reactions.into_iter().map(From::from).collect(),
            reference: v.reference.map(From::from),
            referenced_message: v.referenced_message.map(|e| Box::new((*e).into())),
            timestamp: v.timestamp.as_secs(),
            tts: v.tts,
            webhook_id: v.webhook_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageActivity {
    pub kind: MessageActivityType,
    pub party_id: Option<String>,
}

impl From<twilight_model::channel::message::MessageActivity> for MessageActivity {
    fn from(v: twilight_model::channel::message::MessageActivity) -> Self {
        Self {
            kind: v.kind.into(),
            party_id: v.party_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MessageActivityType {
    Join,
    Spectate,
    Listen,
    JoinRequest,
}

impl From<twilight_model::channel::message::MessageActivityType> for MessageActivityType {
    fn from(v: twilight_model::channel::message::MessageActivityType) -> Self {
        match v {
            twilight_model::channel::message::MessageActivityType::Join => Self::Join,
            twilight_model::channel::message::MessageActivityType::Spectate => Self::Spectate,
            twilight_model::channel::message::MessageActivityType::Listen => Self::Listen,
            twilight_model::channel::message::MessageActivityType::JoinRequest => Self::JoinRequest,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageApplication {
    pub cover_image: Option<String>,
    pub description: String,
    pub icon: Option<String>,
    pub id: ApplicationId,
    pub name: String,
}

impl From<twilight_model::channel::message::MessageApplication> for MessageApplication {
    fn from(v: twilight_model::channel::message::MessageApplication) -> Self {
        Self {
            cover_image: v.cover_image,
            description: v.description,
            icon: v.icon,
            id: v.id,
            name: v.name,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub content_type: Option<String>,
    pub filename: String,
    pub height: Option<u64>,
    pub id: AttachmentId,
    pub proxy_url: String,
    pub size: u64,
    pub url: String,
    pub width: Option<u64>,
}

impl From<twilight_model::channel::Attachment> for Attachment {
    fn from(v: twilight_model::channel::Attachment) -> Self {
        Self {
            content_type: v.content_type,
            filename: v.filename,
            height: v.height,
            id: v.id,
            proxy_url: v.proxy_url,
            size: v.size,
            url: v.url,
            width: v.width,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageType {
    Regular,
    RecipientAdd,
    RecipientRemove,
    Call,
    ChannelNameChange,
    ChannelIconChange,
    ChannelMessagePinned,
    GuildMemberJoin,
    UserPremiumSub,
    UserPremiumSubTier1,
    UserPremiumSubTier2,
    UserPremiumSubTier3,
    ChannelFollowAdd,
    GuildDiscoveryDisqualified,
    GuildDiscoveryRequalified,
    GuildDiscoveryGracePeriodInitialWarning,
    GuildDiscoveryGracePeriodFinalWarning,
    Reply,
    GuildInviteReminder,
    ApplicationCommand,
    ThreadCreated,
    ThreadStarterMessage,
    ContextMenuCommand,
}

impl From<twilight_model::channel::message::MessageType> for MessageType {
    fn from(v: twilight_model::channel::message::MessageType) -> Self {
        use twilight_model::channel::message::MessageType as TwilightMessageType;

        match v {
            TwilightMessageType::Regular => Self::Regular,
            TwilightMessageType::RecipientAdd => Self::RecipientAdd,
            TwilightMessageType::RecipientRemove => Self::RecipientRemove,
            TwilightMessageType::Call => Self::Call,
            TwilightMessageType::ChannelNameChange => Self::ChannelNameChange,
            TwilightMessageType::ChannelIconChange => Self::ChannelIconChange,
            TwilightMessageType::ChannelMessagePinned => Self::ChannelMessagePinned,
            TwilightMessageType::GuildMemberJoin => Self::GuildMemberJoin,
            TwilightMessageType::UserPremiumSub => Self::UserPremiumSub,
            TwilightMessageType::UserPremiumSubTier1 => Self::UserPremiumSubTier1,
            TwilightMessageType::UserPremiumSubTier2 => Self::UserPremiumSubTier2,
            TwilightMessageType::UserPremiumSubTier3 => Self::UserPremiumSubTier3,
            TwilightMessageType::ChannelFollowAdd => Self::ChannelFollowAdd,
            TwilightMessageType::GuildDiscoveryDisqualified => Self::GuildDiscoveryDisqualified,
            TwilightMessageType::GuildDiscoveryRequalified => Self::GuildDiscoveryRequalified,
            TwilightMessageType::GuildDiscoveryGracePeriodInitialWarning => {
                Self::GuildDiscoveryGracePeriodInitialWarning
            }
            TwilightMessageType::GuildDiscoveryGracePeriodFinalWarning => {
                Self::GuildDiscoveryGracePeriodFinalWarning
            }
            TwilightMessageType::Reply => Self::Reply,
            TwilightMessageType::GuildInviteReminder => Self::GuildInviteReminder,
            TwilightMessageType::ApplicationCommand => Self::ApplicationCommand,
            TwilightMessageType::ThreadCreated => Self::ThreadCreated,
            TwilightMessageType::ThreadStarterMessage => Self::ThreadStarterMessage,
            TwilightMessageType::ContextMenuCommand => Self::ContextMenuCommand,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialMember {
    pub deaf: bool,
    pub joined_at: Option<u64>,
    pub mute: bool,
    pub nick: Option<String>,
    pub premium_since: Option<u64>,
    pub roles: Vec<RoleId>,
}

impl From<twilight_model::guild::PartialMember> for PartialMember {
    fn from(v: twilight_model::guild::PartialMember) -> Self {
        Self {
            deaf: v.deaf,
            joined_at: v.joined_at.map(|ts| ts.as_secs()),
            mute: v.mute,
            nick: v.nick,
            premium_since: v.premium_since.map(|ts| ts.as_secs()),
            roles: v.roles,
        }
    }
}
impl From<PartialMember> for twilight_model::guild::PartialMember {
    fn from(v: PartialMember) -> Self {
        Self {
            deaf: v.deaf,
            joined_at: v.joined_at.map(Timestamp::from_secs).flatten(),
            mute: v.mute,
            nick: v.nick,
            premium_since: v.premium_since.map(Timestamp::from_secs).flatten(),
            roles: v.roles,
            permissions: None, // TODO
            user: None,        // TODO
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMention {
    pub guild_id: GuildId,
    pub id: ChannelId,
    pub kind: ChannelType,
    pub name: String,
}

impl From<twilight_model::channel::ChannelMention> for ChannelMention {
    fn from(v: twilight_model::channel::ChannelMention) -> Self {
        Self {
            guild_id: v.guild_id,
            id: v.id,
            kind: v.kind.into(),
            name: v.name,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ChannelType {
    GuildText,
    Private,
    GuildVoice,
    Group,
    GuildCategory,
    GuildNews,
    GuildStore,
    GuildStageVoice,
    GuildNewsThread,
    GuildPublicThread,
    GuildPrivateThread,
}

impl From<twilight_model::channel::ChannelType> for ChannelType {
    fn from(v: twilight_model::channel::ChannelType) -> Self {
        match v {
            twilight_model::channel::ChannelType::GuildText => Self::GuildText,
            twilight_model::channel::ChannelType::Private => Self::Private,
            twilight_model::channel::ChannelType::GuildVoice => Self::GuildVoice,
            twilight_model::channel::ChannelType::Group => Self::Group,
            twilight_model::channel::ChannelType::GuildCategory => Self::GuildCategory,
            twilight_model::channel::ChannelType::GuildNews => Self::GuildNews,
            twilight_model::channel::ChannelType::GuildStore => Self::GuildStore,
            twilight_model::channel::ChannelType::GuildStageVoice => Self::GuildStageVoice,
            twilight_model::channel::ChannelType::GuildNewsThread => Self::GuildNewsThread,
            twilight_model::channel::ChannelType::GuildPublicThread => Self::GuildPublicThread,
            twilight_model::channel::ChannelType::GuildPrivateThread => Self::GuildPrivateThread,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    /// Hash of the user's avatar, if any.
    pub avatar: Option<String>,
    /// Whether the user is a bot.
    pub bot: bool,
    /// Discriminator used to differentiate people with the same username.
    ///
    /// # serde
    ///
    /// The discriminator field can be deserialized from either a string or an
    /// integer. The field will always serialize into a string due to that being
    /// the type Discord's API uses.
    pub discriminator: u16,
    /// Unique ID of the user.
    pub id: UserId,
    /// Member object for the user in the guild, if available.
    pub member: Option<PartialMember>,
    /// Username of the user.
    pub username: String,
    /// Public flags on the user's account.
    pub public_flags: u64,
}

impl From<twilight_model::channel::message::Mention> for Mention {
    fn from(v: twilight_model::channel::message::Mention) -> Self {
        Self {
            avatar: v.avatar,
            bot: v.bot,
            discriminator: v.discriminator,
            id: v.id,
            member: v.member.map(From::from),
            username: v.name,
            public_flags: v.public_flags.bits(),
        }
    }
}

impl From<Mention> for twilight_model::channel::message::Mention {
    fn from(v: Mention) -> Self {
        Self {
            avatar: v.avatar,
            bot: v.bot,
            discriminator: v.discriminator,
            id: v.id,
            member: v.member.map(From::from),
            name: v.username,
            // TODO: remove unwrap
            public_flags: twilight_model::user::UserFlags::from_bits(v.public_flags).unwrap(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageReaction {
    pub count: u64,
    pub emoji: ReactionType,
    pub me: bool,
}

impl From<twilight_model::channel::message::MessageReaction> for MessageReaction {
    fn from(v: twilight_model::channel::message::MessageReaction) -> Self {
        Self {
            count: v.count,
            emoji: v.emoji.into(),
            me: v.me,
        }
    }
}

impl From<MessageReaction> for twilight_model::channel::message::MessageReaction {
    fn from(v: MessageReaction) -> Self {
        Self {
            count: v.count,
            emoji: v.emoji.into(),
            me: v.me,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum ReactionType {
    Custom {
        #[serde(default)]
        animated: bool,
        // Even though it says that the id can be nil in the docs,
        // it is a bit misleading as that should only happen when
        // the reaction is a unicode emoji and then it is caught by
        // the other variant.
        id: EmojiId,
        // Name is nil if the emoji data is no longer avaiable, for
        // example if the emoji have been deleted off the guild.
        name: Option<String>,
    },
    Unicode {
        name: String,
    },
}

impl From<twilight_model::channel::ReactionType> for ReactionType {
    fn from(v: twilight_model::channel::ReactionType) -> Self {
        match v {
            twilight_model::channel::ReactionType::Custom { animated, name, id } => {
                Self::Custom { animated, name, id }
            }
            twilight_model::channel::ReactionType::Unicode { name } => Self::Unicode { name },
        }
    }
}
impl From<ReactionType> for twilight_model::channel::ReactionType {
    fn from(v: ReactionType) -> Self {
        match v {
            ReactionType::Custom { animated, name, id } => Self::Custom { animated, name, id },
            ReactionType::Unicode { name } => Self::Unicode { name },
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageReference {
    pub channel_id: Option<ChannelId>,
    pub guild_id: Option<GuildId>,
    pub message_id: Option<MessageId>,
    pub fail_if_not_exists: Option<bool>,
}

impl From<twilight_model::channel::message::MessageReference> for MessageReference {
    fn from(v: twilight_model::channel::message::MessageReference) -> Self {
        Self {
            channel_id: v.channel_id,
            guild_id: v.guild_id,
            message_id: v.message_id,
            fail_if_not_exists: v.fail_if_not_exists,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum StickerType {
    /// Official sticker in a pack.
    ///
    /// Part of nitro or in a removed purchasable pack.
    Standard = 1,
    /// Sticker uploaded to a boosted guild for the guild's members.
    Guild = 2,
}

impl From<twilight_model::channel::message::sticker::StickerType> for StickerType {
    fn from(v: twilight_model::channel::message::sticker::StickerType) -> Self {
        match v {
            twilight_model::channel::message::sticker::StickerType::Standard => Self::Standard,
            twilight_model::channel::message::sticker::StickerType::Guild => Self::Guild,
        }
    }
}

impl From<StickerType> for twilight_model::channel::message::sticker::StickerType {
    fn from(v: StickerType) -> Self {
        match v {
            StickerType::Standard => Self::Standard,
            StickerType::Guild => Self::Guild,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sticker {
    /// Whether the sticker is available.
    pub available: bool,
    /// Description of the sticker.
    pub description: Option<String>,
    /// Format type.
    pub format_type: StickerFormatType,
    /// ID of the guild that owns the sticker.
    pub guild_id: Option<GuildId>,
    /// Unique ID of the sticker.
    pub id: StickerId,
    /// Name of the sticker.
    pub name: String,
    /// Unique ID of the pack the sticker is in.
    pub pack_id: Option<StickerPackId>,
    /// Sticker's sort order within a pack.
    pub sort_value: Option<u64>,
    /// CSV list of tags the sticker is assigned to, if any.
    pub tags: String,
    /// ID of the user that uploaded the sticker.
    pub user: Option<User>,

    pub kind: StickerType,
}

impl From<twilight_model::channel::message::Sticker> for Sticker {
    fn from(v: twilight_model::channel::message::Sticker) -> Self {
        Self {
            description: v.description,
            format_type: v.format_type.into(),
            id: v.id,
            name: v.name,
            pack_id: v.pack_id,
            tags: v.tags,
            available: v.available,
            guild_id: v.guild_id,
            sort_value: v.sort_value,
            user: v.user.map(|u| u.into()),
            kind: v.kind.into(),
        }
    }
}
impl From<Sticker> for twilight_model::channel::message::Sticker {
    fn from(v: Sticker) -> Self {
        Self {
            description: v.description,
            format_type: v.format_type.into(),
            id: v.id,
            name: v.name,
            pack_id: v.pack_id,
            tags: v.tags,
            available: v.available,
            guild_id: v.guild_id,
            sort_value: v.sort_value,
            user: v.user.map(|u| u.into()),
            kind: v.kind.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum StickerFormatType {
    /// Sticker format is a PNG.
    Png,
    /// Sticker format is an APNG.
    Apng,
    /// Sticker format is a LOTTIE.
    Lottie,
}

impl From<twilight_model::channel::message::sticker::StickerFormatType> for StickerFormatType {
    fn from(v: twilight_model::channel::message::sticker::StickerFormatType) -> Self {
        match v {
            twilight_model::channel::message::sticker::StickerFormatType::Apng => Self::Apng,
            twilight_model::channel::message::sticker::StickerFormatType::Png => Self::Png,
            twilight_model::channel::message::sticker::StickerFormatType::Lottie => Self::Lottie,
        }
    }
}
impl From<StickerFormatType> for twilight_model::channel::message::sticker::StickerFormatType {
    fn from(v: StickerFormatType) -> Self {
        match v {
            StickerFormatType::Apng => Self::Apng,
            StickerFormatType::Png => Self::Png,
            StickerFormatType::Lottie => Self::Lottie,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDelete {
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub id: MessageId,
}

impl From<twilight_model::gateway::payload::incoming::MessageDelete> for MessageDelete {
    fn from(v: twilight_model::gateway::payload::incoming::MessageDelete) -> Self {
        Self {
            channel_id: v.channel_id,
            guild_id: v.guild_id,
            id: v.id,
        }
    }
}
