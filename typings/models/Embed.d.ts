import type { EmbedVideo } from "./EmbedVideo";
import type { EmbedField } from "./EmbedField";
import type { EmbedAuthor } from "./EmbedAuthor";
import type { EmbedImage } from "./EmbedImage";
import type { EmbedThumbnail } from "./EmbedThumbnail";
import type { EmbedProvider } from "./EmbedProvider";
import type { EmbedFooter } from "./EmbedFooter";
export interface Embed {
    author?: EmbedAuthor;
    color?: number;
    description?: string;
    fields?: Array<EmbedField>;
    footer?: EmbedFooter;
    image?: EmbedImage;
    kind?: string;
    provider?: EmbedProvider;
    thumbnail?: EmbedThumbnail;
    timestamp?: number;
    title?: string;
    url?: string;
    video?: EmbedVideo;
}
