import { useMemo } from "react";
import { Link } from "react-router-dom";
import { BotGuild, UserGuild } from "../ApiClient";
import { useGuilds } from "../components/GuildsProvider"
import { guildIconUrl } from "../components/Util";
import "./SelectServer.css"

export function SelectServerPage() {

    const guilds = useGuilds();

    const [joinedHasAdmin, notJoinedHasAdmin] = useMemo(() => {
        if (!guilds) {
            return [[], []];
        }

        const guildsAdmins = guilds.guilds.filter(g => hasAdmin(g.guild));
        const joinedHasAdmin = guildsAdmins.filter(g => g.connected);
        const notJoinedHasAdmin = guildsAdmins.filter(g => !g.connected);

        return [joinedHasAdmin, notJoinedHasAdmin];

    }, [guilds])

    if (!guilds) {
        return <p>Loading guilds.... (unless you're not logged in that is)</p>
    }

    return <div className="guild-select-page">

        <h2>Joined servers</h2>
        <div className="guild-select-list">
            {joinedHasAdmin.map(g => <GuildListItem guild={g} key={g.guild.id} />)}
        </div >
        <h2>Add to new servers</h2>
        <div className="guild-select-list">
            {notJoinedHasAdmin.map(g => <GuildListItem guild={g} key={g.guild.id} />)}
        </div >
    </div>
}

function GuildListItem({ guild: g }: { guild: BotGuild }) {
    return <Link to={`/servers/${g.guild.id}`}><div className="guild-list-item">
        <GuildIcon guild={g.guild} />
        <p>{g.guild.name}</p>
    </div></Link>
}

function GuildIcon(props: { guild: UserGuild }) {
    return <img src={guildIconUrl(props.guild)} alt={`icon for ${props.guild.name}`} />
}


const permAdmin = BigInt("0x0000000008");
const permManageServer = BigInt("0x0000000020");

function hasAdmin(g: UserGuild): boolean {
    if (g.owner) {
        return true
    }


    const perms = BigInt(g.permissions);
    if ((perms & permAdmin) === permAdmin) {
        return true
    }

    if ((perms & permManageServer) === permManageServer) {
        return true
    }

    return false
}