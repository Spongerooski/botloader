import { Bot, console, CreateMessage, Timers } from 'botloader';

// import { bot } 

Bot.registerMeta({
    name: "pog",
    context: "Guild",
    timers: [
        new Timers.IntervalTimerCron("epic", "aaa"),
        new Timers.IntervalTimerSeconds("epic", 123),
    ]
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