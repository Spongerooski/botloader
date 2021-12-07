import type { CommandInteractionOption } from "./CommandInteractionOption";
import type { InteractionMember } from "../discord/InteractionMember";

export interface CommandInteraction {
  channelId: string;
  id: string;
  member: InteractionMember;
  token: string;
  name: string;
  parentName: string | null;
  parentParentName: string | null;
  options: Array<CommandInteractionOption>;
}
