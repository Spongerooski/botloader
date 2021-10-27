import { console, CreateMessage, Script, Timers } from 'botloader';

const script = new Script("should trigger a error on messages");

script.on("MESSAGE_CREATE", async evt => {
    if (!evt.author.bot) {
        throw new Error("woo fancy error appeared");
    }
})

script.run();
