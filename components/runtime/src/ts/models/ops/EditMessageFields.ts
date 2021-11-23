import type { AllowedMentions } from "./AllowedMentions";
import type { Embed } from "../Embed";

export interface OpEditMessageFields {
  content?: string;
  embeds?: Array<Embed>;
  allowedMentions?: AllowedMentions;
}
