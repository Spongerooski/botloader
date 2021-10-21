import { BotGuild, User } from "../ApiClient";
import { BuildConfig } from "../BuildConfig";
import { useCurrentGuild } from "./GuildsProvider";
import { useSession } from "./Session";

export function TopNav() {
    let session = useSession();
    let currentGuild = useCurrentGuild();

    return <div className="top-nav">
        <div className="current-user">
            {session.user ? <UserLoggedIn user={session.user} /> : <UserNotLoggedIn />}
        </div>

        <div className="current-server">
            {currentGuild ? <CurrentGuild guild={currentGuild} /> : <NoCurrentGuild />}
        </div>
    </div>
}

function UserLoggedIn(props: { user: User }) {
    return <p>Logged in as {props.user.username}</p>
}

function UserNotLoggedIn() {
    return <p>Not logged in, log in using <a href={BuildConfig.botloaderApiBase + "/login"}>this link</a></p>
}

function CurrentGuild(props: { guild: BotGuild }) {
    return <p>On guild: {props.guild.guild.name}</p>
}

function NoCurrentGuild() {
    return <p>No current guild</p>
}