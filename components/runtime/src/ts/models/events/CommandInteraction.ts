import type { CommandInteractionOption } from "./CommandInteractionOption";
import type { PartialMember } from "../discord/PartialMember";

export interface CommandInteraction {
  channelId: string;
  id: string;
  member: PartialMember;
  token: string;
  name: string;
  parentName: string | null;
  parentParentName: string | null;
  options: Array<CommandInteractionOption>;
}
