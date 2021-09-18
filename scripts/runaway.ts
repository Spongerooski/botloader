import { Bot, console, CreateMessage, Timers } from './bot/index';

Bot.registerMeta({
    name: "runaway",
    context: "Guild",
})

let a = 0;
let b = a;

Bot.on("MESSAGE_CREATE", async evt => {
    while (true) {
        a++;
        b = a;
    }
})