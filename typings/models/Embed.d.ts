import type { EmbedImage } from "./EmbedImage";
import type { EmbedAuthor } from "./EmbedAuthor";
import type { EmbedVideo } from "./EmbedVideo";
import type { EmbedFooter } from "./EmbedFooter";
import type { EmbedThumbnail } from "./EmbedThumbnail";
import type { EmbedField } from "./EmbedField";
import type { EmbedProvider } from "./EmbedProvider";
export interface Embed {
    author: EmbedAuthor | null;
    color: number | null;
    description: string | null;
    fields: Array<EmbedField>;
    footer: EmbedFooter | null;
    image: EmbedImage | null;
    kind: string;
    provider: EmbedProvider | null;
    thumbnail: EmbedThumbnail | null;
    timestamp: number | null;
    title: string | null;
    url: string | null;
    video: EmbedVideo | null;
}
