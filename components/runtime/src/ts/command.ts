export namespace Commands {

    export class Command {
        name: string;
        description: string;
        options: Option[];
        kind?: "chat" | "user" | "message";
    }

    export class Group {
        commands: Command[];
        subGroups: Group[];
    }

    export class Option {
        // TODO
        // how far do we wanna stary to slash commands?
        name: string;
        description: string;
        kind: OptionType;

        choices: (string | number)[];
        channel_types: string;
    }

    export type OptionType =
        "STRING" |
        "INTEGER" | //	Any integer between -2^53 and 2^53
        "BOOLEAN" |
        "USER" |
        "CHANNEL" | //	Includes all channel types + categories
        "ROLE" |
        "MENTIONABLE" | //	Includes users and roles
        "NUMBER"; //	Any double between -2^53 and 2^53
}