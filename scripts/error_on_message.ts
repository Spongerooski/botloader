import { Bot, console, CreateMessage, Timers } from './bot/index';

Bot.registerMeta({
    name: "error_on_message",
    context: "Guild",
})

Bot.on("MESSAGE_CREATE", async evt => {
    if (!evt.author.bot) {
        counter++;
    }
})
