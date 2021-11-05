import { Command as OpCommand, CommandGroup as OpCommandGroup, CommandOptionType as OpCommandOptionType } from "./commonmodels";
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
    export type OptionType = OpCommandOptionType;
    type OptionTypeToParsedType<T extends BaseOption<boolean>> = T extends StringOption<boolean> ? string : T extends NumberOption<boolean> ? number : any;
    export class ExecutedCommandContext {
        sendResponse(resp: string): Promise<void>;
    }
    export interface Group {
        commands: CommandDef<any>[];
    }
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
        genOpBinding(): [OpCommand[], OpCommandGroup[]];
    }
    export {};
}
