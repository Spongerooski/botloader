import { UserGuild } from "./apiclient";

export interface IndexFile {
    guild: UserGuild,
    openScripts: IndexScript[],
}

export interface IndexScript {
    id: number,
    name: string,
}