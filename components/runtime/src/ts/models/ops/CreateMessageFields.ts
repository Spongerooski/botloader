import type { AllowedMentions } from "./AllowedMentions";
import type { Embed } from "../Embed";

export interface OpCreateMessageFields {
  content: string;
  embeds?: Array<Embed>;
  allowedMentions?: AllowedMentions;
}
