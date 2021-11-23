import type { ReactionType } from "./ReactionType";

export interface MessageReaction {
  count: bigint;
  emoji: ReactionType;
  me: boolean;
}
