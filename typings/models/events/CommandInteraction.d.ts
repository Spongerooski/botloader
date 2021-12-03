import type { PartialMember } from "../PartialMember";
import type { CommandInteractionOption } from "./CommandInteractionOption";
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
