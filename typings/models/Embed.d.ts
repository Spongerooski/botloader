import type { EmbedFooter } from "./EmbedFooter";
import type { EmbedImage } from "./EmbedImage";
import type { EmbedProvider } from "./EmbedProvider";
import type { EmbedField } from "./EmbedField";
import type { EmbedThumbnail } from "./EmbedThumbnail";
import type { EmbedAuthor } from "./EmbedAuthor";
import type { EmbedVideo } from "./EmbedVideo";
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
