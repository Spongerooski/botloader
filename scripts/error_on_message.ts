import { Bot, console, CreateMessage, Timers } from 'botloader';

Bot.registerMeta({
    name: "error_on_message",
})

Bot.on("MESSAGE_CREATE", async evt => {
    if (!evt.author.bot) {
        counter++;
    }
})
