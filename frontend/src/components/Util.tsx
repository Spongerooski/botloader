import { User, UserGuild } from "../ApiClient";

export function guildIconUrl(g: UserGuild, size = 64): string {

    const extension =
        g.icon?.startsWith("a_") ? "gif" : "png";

    return `https://cdn.discordapp.com/icons/${g.id}/${g.icon}.${extension}?size=${size}`
}

export function userAvatarUrl(u: User, size = 64): string {
    if (u.avatar) {
        const extension =
            u.avatar.startsWith("a_") ? "gif" : "png";

        return `https://cdn.discordapp.com/avatars/${u.id}/${u.avatar}.${extension}?size=${size}`
    } else {
        const discriminator = parseInt(u.discriminator);

        return `https://cdn.discordapp.com/embed/avatars/${discriminator % 5}.png?size=${size}`
    }
}