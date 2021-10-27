export declare namespace Commands {
    export interface CommandDef<T extends OptionsMap> {
        name: string;
        description: string;
        options: T;
        callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<T>) => void;
        kind?: "chat" | "user" | "message";
    }
    type OptionsMap = {
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
        kind: "STRING";
    }
    export interface NumberOption<T extends boolean> extends BaseOption<T> {
        kind: "NUMBER";
    }
    export type OptionType = "STRING" | "INTEGER" | //	Any integer between -2^53 and 2^53
    "BOOLEAN" | "USER" | "CHANNEL" | //	Includes all channel types + categories
    "ROLE" | "MENTIONABLE" | //	Includes users and roles
    "NUMBER";
    type OptionTypeToParsedType<T extends BaseOption<boolean>> = T extends StringOption<boolean> ? string : T extends NumberOption<boolean> ? number : any;
    export function registerCommand<T extends OptionsMap>(cmd: CommandDef<T>): void;
    export function registerGroup(group: Group): void;
    export class ExecutedCommandContext {
        sendResponse(resp: string): Promise<void>;
    }
    export interface Group {
        commands: CommandDef<any>[];
    }
    export class Group {
        name: string;
        description: string;
        protected isSubGroup: boolean;
        commands: CommandDef<any>[];
        groups: Group[];
        constructor(name: string, description: string);
        subGroup(name: string, description: string): Group;
        registerCommand<T extends OptionsMap>(cmd: CommandDef<T>): this;
    }
    export {};
}
