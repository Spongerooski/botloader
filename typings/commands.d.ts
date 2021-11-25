import { Command as OpCommand, CommandGroup as OpCommandGroup, CommandInteraction, CommandOptionType as OpCommandOptionType, PartialMember } from "./models/index";
import { ScriptEventMuxer } from "./events";
export declare namespace Commands {
    export interface CommandDef<T extends OptionsMap> {
        name: string;
        description: string;
        options: T;
        kind?: "chat" | "user" | "message";
        group?: Group;
        callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<T>) => void;
    }
    export type OptionsMap = {
        [key: string]: BaseOption<boolean>;
    };
    type ParsedOptionsMap<T extends OptionsMap> = {
        [Property in keyof T]: T[Property] extends BaseOption<false> ? OptionTypeToParsedType<T[Property]> | undefined : OptionTypeToParsedType<T[Property]>;
    };
    interface BaseOption<TRequired extends boolean | undefined> {
        description: string;
        kind: OptionType;
        required?: TRequired;
    }
    export interface StringOption<T extends boolean> extends BaseOption<T> {
        kind: "String";
    }
    export interface NumberOption<T extends boolean> extends BaseOption<T> {
        kind: "Number";
    }
    export interface IntOption<T extends boolean> extends BaseOption<T> {
        kind: "Integer";
    }
    export interface BoolOption<T extends boolean> extends BaseOption<T> {
        kind: "Boolean";
    }
    export interface UserOption<T extends boolean> extends BaseOption<T> {
        kind: "User";
    }
    export interface ChannelOption<T extends boolean> extends BaseOption<T> {
        kind: "Channel";
    }
    export interface RoleOption<T extends boolean> extends BaseOption<T> {
        kind: "Role";
    }
    export interface MentionableOption<T extends boolean> extends BaseOption<T> {
        kind: "Mentionable";
    }
    export type OptionType = OpCommandOptionType;
    type OptionTypeToParsedType<T extends BaseOption<boolean>> = T extends StringOption<boolean> ? string : T extends NumberOption<boolean> ? number : T extends IntOption<boolean> ? number : T extends BoolOption<boolean> ? boolean : T extends UserOption<boolean> ? PartialMember : T extends ChannelOption<boolean> ? {} : T extends RoleOption<boolean> ? {} : T extends MentionableOption<boolean> ? {} : unknown;
    export class Group {
        name: string;
        description: string;
        parent?: Group;
        protected isSubGroup: boolean;
        subGroups: Group[];
        constructor(name: string, description: string);
        subGroup(name: string, description: string): Group;
    }
    export class System {
        commands: CommandDef<OptionsMap>[];
        addEventListeners(muxer: ScriptEventMuxer): void;
        handleInteractionCreate(interaction: CommandInteraction): void;
        genOpBinding(): [OpCommand[], OpCommandGroup[]];
    }
    export class ExecutedCommandContext {
        interaction: CommandInteraction;
        constructor(interaction: CommandInteraction);
        sendResponse(resp: string): Promise<void>;
    }
    export {};
}
