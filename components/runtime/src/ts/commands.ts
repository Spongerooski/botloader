import { Command as OpCommand, CommandGroup as OpCommandGroup, CommandOption as OpCommandOption, CommandOptionType as OpCommandOptionType } from "./commonmodels";
import { ScriptEventMuxer } from "./events";

export namespace Commands {

    export interface CommandDef<T extends OptionsMap> {
        name: string;
        description: string;
        options: T;
        kind?: "chat" | "user" | "message";
        group?: Group,
        callback: (ctx: ExecutedCommandContext, args: ParsedOptionsMap<T>) => void,
    }

    export type OptionsMap = {
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
        kind: "String";
    };

    export interface NumberOption<T extends boolean> extends BaseOption<T> {
        kind: "Number";
    };

    export type OptionType = OpCommandOptionType;

    type OptionTypeToParsedType<T extends BaseOption<boolean>> =
        T extends StringOption<boolean> ? string :
        T extends NumberOption<boolean> ? number :
        any;

    export class ExecutedCommandContext {
        async sendResponse(resp: string) { }
    }

    export interface Group {
        commands: CommandDef<any>[],
    }

    export class Group {
        name: string;
        description: string;
        parent?: Group;
        protected isSubGroup: boolean = false;

        subGroups: Group[] = [];

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
            this.subGroups.push(group);
            return group;
        }
    }

    export class System {
        commands: CommandDef<OptionsMap>[] = [];

        addEventListeners(muxer: ScriptEventMuxer) {
            // TODO
        }

        genOpBinding(): [OpCommand[], OpCommandGroup[]] {

            const commands: OpCommand[] = this.commands.map(cmd => {
                const options: OpCommandOption[] = [];
                for (let prop in cmd.options) {
                    if (Object.prototype.hasOwnProperty.call(cmd.options, prop)) {
                        let entry = cmd.options[prop];
                        options.push({
                            name: prop,
                            description: entry.description,
                            kind: entry.kind,
                            required: entry.required || false,
                        })
                    }
                }

                let group = undefined;
                let subGroup = undefined;
                if (cmd.group) {
                    if (cmd.group.parent) {
                        group = cmd.group.parent.name;
                        subGroup = cmd.group.name;
                    } else {
                        group = cmd.group.name;
                    }
                }

                return {
                    name: cmd.name,
                    description: cmd.description,
                    options: options,
                    group,
                    subGroup,
                }
            });

            const groups: OpCommandGroup[] = [];

            OUTER:
            for (let cmd of this.commands) {
                if (cmd.group) {
                    if (groups.some(g => g.name === cmd.group?.name)) {
                        continue OUTER;
                    }

                    // new group
                    groups.push({
                        name: cmd.group.name,
                        description: cmd.group.description,
                        subGroups: cmd.group.subGroups.map(sg => {
                            return {
                                name: sg.name,
                                description: sg.description
                            }
                        })
                    })
                }
            }


            return [commands, groups];
        }
    }
}