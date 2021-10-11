import { ChannelType } from "./commonmodels";

export namespace Commands {

    export interface CommandDef<T extends OptionsMap> {
        name: string;
        description: string;
        options: T;
        callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<T>) => void,
        kind?: "chat" | "user" | "message";
    }

    type OptionsMap = {
        [key: string]: BaseOption<boolean>;
    }

    type ParsedOptionsMap<T extends OptionsMap> = {
        [Property in keyof T]:
        T[Property] extends BaseOption<false> ? OptionTypeToParsedType<T[Property]> | undefined : OptionTypeToParsedType<T[Property]>;
    }

    interface BaseOption<TRequired extends boolean | undefined> {
        description: string;
        kind: OptionType;
        required?: TRequired;
    }


    export interface StringOption<T extends boolean> extends BaseOption<T> {
        kind: "STRING";
    };

    export interface NumberOption<T extends boolean> extends BaseOption<T> {
        kind: "NUMBER";
    };

    export type OptionType =
        "STRING" |
        "INTEGER" | //	Any integer between -2^53 and 2^53
        "BOOLEAN" |
        "USER" |
        "CHANNEL" | //	Includes all channel types + categories
        "ROLE" |
        "MENTIONABLE" | //	Includes users and roles
        "NUMBER"; //	Any double between -2^53 and 2^53

    type OptionTypeToParsedType<T extends BaseOption<boolean>> =
        T extends StringOption<boolean> ? string :
        T extends NumberOption<boolean> ? number :
        any;

    export function registerCommand<T extends OptionsMap>(cmd: CommandDef<T>) { }
    export function registerGroup(group: Group) { }

    export class ExecutedCommandContext {
        async sendResponse(resp: string) { }
    }

    export interface Group {
        commands: CommandDef<any>[],
    }

    export class Group {
        name: string;
        description: string;
        protected isSubGroup: boolean;

        commands: CommandDef<any>[];
        groups: Group[];

        constructor(name: string, description: string) {
            this.name = name;
            this.description = description;
        }

        subGroup(name: string, description: string) {
            if (this.isSubGroup) {
                throw "cant make sub groups of sub groups";
            }

            let group = new Group(name, description);
            group.isSubGroup = true;
            this.groups.push(group);
            return group;
        }

        registerCommand<T extends OptionsMap>(cmd: CommandDef<T>) {
            this.commands.push(cmd);
            return this;
        }
    }
}