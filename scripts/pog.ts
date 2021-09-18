import { Bot, console, CreateMessage, Timers } from './bot/index';

Bot.registerMeta({
    name: "pog",
    context: "Guild",
})

let counter = 1;

console.log("Were in script: " + SCRIPT_ID + ", Full:" + SCRIPT_CONTEXT_ID);

Bot.on("MESSAGE_CREATE", async evt => {
    if (!evt.author.bot && evt.content === "pog") {
        counter++;
        await CreateMessage({
            channelId: evt.channelId,
            content: "pog #" + counter,
        })
    }
})

Bot.on("MESSAGE_DELETE", async evt => {
    counter++;
    await CreateMessage({
        channelId: evt.channelId,
        content: "someone deleted a message in this channel :( #" + counter,
    })
})

Timers.startInterval("", 100, () => { console.log("Gaming"); });