import type { Embed } from "../Embed";
import type { AllowedMentions } from "./AllowedMentions";
export interface OpCreateMessageFields {
    content: string;
    embeds?: Array<Embed>;
    allowedMentions?: AllowedMentions;
}
