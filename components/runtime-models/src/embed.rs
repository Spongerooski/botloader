use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::datetime::Timestamp;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Embed {
    pub author: Option<EmbedAuthor>,
    pub color: Option<u32>,
    pub description: Option<String>,
    pub fields: Vec<EmbedField>,
    pub footer: Option<EmbedFooter>,
    pub image: Option<EmbedImage>,
    pub kind: String,
    pub provider: Option<EmbedProvider>,
    pub thumbnail: Option<EmbedThumbnail>,
    pub timestamp: Option<u64>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub video: Option<EmbedVideo>,
}
impl From<Embed> for twilight_model::channel::embed::Embed {
    fn from(v: Embed) -> Self {
        Self {
            author: v.author.map(From::from),
            color: v.color,
            description: v.description,
            fields: v.fields.into_iter().map(From::from).collect(),
            footer: v.footer.map(From::from),
            image: v.image.map(From::from),
            kind: v.kind,
            provider: v.provider.map(From::from),
            thumbnail: v.thumbnail.map(From::from),
            timestamp: v.timestamp.map(Timestamp::from_secs).flatten(),
            title: v.title,
            url: v.url,
            video: v.video.map(From::from),
        }
    }
}

impl From<twilight_model::channel::embed::Embed> for Embed {
    fn from(v: twilight_model::channel::embed::Embed) -> Self {
        Self {
            author: v.author.map(From::from),
            color: v.color,
            description: v.description,
            fields: v.fields.into_iter().map(From::from).collect(),
            footer: v.footer.map(From::from),
            image: v.image.map(From::from),
            kind: v.kind,
            provider: v.provider.map(From::from),
            thumbnail: v.thumbnail.map(From::from),
            timestamp: v.timestamp.map(|ts| ts.as_secs()),
            title: v.title,
            url: v.url,
            video: v.video.map(From::from),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EmbedAuthor {
    pub icon_url: Option<String>,
    pub name: Option<String>,
    pub proxy_icon_url: Option<String>,
    pub url: Option<String>,
}
impl From<EmbedAuthor> for twilight_model::channel::embed::EmbedAuthor {
    fn from(v: EmbedAuthor) -> Self {
        Self {
            icon_url: v.icon_url,
            name: v.name,
            proxy_icon_url: v.proxy_icon_url,
            url: v.url,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedAuthor> for EmbedAuthor {
    fn from(v: twilight_model::channel::embed::EmbedAuthor) -> Self {
        Self {
            icon_url: v.icon_url,
            name: v.name,
            proxy_icon_url: v.proxy_icon_url,
            url: v.url,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EmbedField {
    #[serde(default)]
    pub inline: bool,
    pub name: String,
    pub value: String,
}
impl From<EmbedField> for twilight_model::channel::embed::EmbedField {
    fn from(v: EmbedField) -> Self {
        Self {
            inline: v.inline,
            name: v.name,
            value: v.value,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedField> for EmbedField {
    fn from(v: twilight_model::channel::embed::EmbedField) -> Self {
        Self {
            inline: v.inline,
            name: v.name,
            value: v.value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EmbedFooter {
    pub icon_url: Option<String>,
    pub proxy_icon_url: Option<String>,
    pub text: String,
}

impl From<EmbedFooter> for twilight_model::channel::embed::EmbedFooter {
    fn from(v: EmbedFooter) -> Self {
        Self {
            icon_url: v.icon_url,
            proxy_icon_url: v.proxy_icon_url,
            text: v.text,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedFooter> for EmbedFooter {
    fn from(v: twilight_model::channel::embed::EmbedFooter) -> Self {
        Self {
            icon_url: v.icon_url,
            proxy_icon_url: v.proxy_icon_url,
            text: v.text,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EmbedImage {
    pub height: Option<u64>,
    pub proxy_url: Option<String>,
    pub url: Option<String>,
    pub width: Option<u64>,
}
impl From<EmbedImage> for twilight_model::channel::embed::EmbedImage {
    fn from(v: EmbedImage) -> Self {
        Self {
            height: v.height,
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedImage> for EmbedImage {
    fn from(v: twilight_model::channel::embed::EmbedImage) -> Self {
        Self {
            height: v.height,
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EmbedProvider {
    pub name: Option<String>,
    pub url: Option<String>,
}

impl From<EmbedProvider> for twilight_model::channel::embed::EmbedProvider {
    fn from(v: EmbedProvider) -> Self {
        Self {
            name: v.name,
            url: v.url,
        }
    }
}

impl From<twilight_model::channel::embed::EmbedProvider> for EmbedProvider {
    fn from(v: twilight_model::channel::embed::EmbedProvider) -> Self {
        Self {
            name: v.name,
            url: v.url,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EmbedThumbnail {
    pub height: Option<u64>,
    pub proxy_url: Option<String>,
    pub url: Option<String>,
    pub width: Option<u64>,
}

impl From<EmbedThumbnail> for twilight_model::channel::embed::EmbedThumbnail {
    fn from(v: EmbedThumbnail) -> Self {
        Self {
            height: v.height,
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width,
        }
    }
}
impl From<twilight_model::channel::embed::EmbedThumbnail> for EmbedThumbnail {
    fn from(v: twilight_model::channel::embed::EmbedThumbnail) -> Self {
        Self {
            height: v.height,
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EmbedVideo {
    pub height: Option<u64>,
    pub proxy_url: Option<String>,
    pub url: Option<String>,
    pub width: Option<u64>,
}

impl From<twilight_model::channel::embed::EmbedVideo> for EmbedVideo {
    fn from(v: twilight_model::channel::embed::EmbedVideo) -> Self {
        Self {
            height: v.height,
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width,
        }
    }
}

impl From<EmbedVideo> for twilight_model::channel::embed::EmbedVideo {
    fn from(v: EmbedVideo) -> Self {
        Self {
            height: v.height,
            proxy_url: v.proxy_url,
            url: v.url,
            width: v.width,
        }
    }
}
