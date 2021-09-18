import { Bot, console, CreateMessage, Timers } from './bot/index';

Bot.registerMeta({
    name: "runaway_validate",
    context: "Guild",
})

let a = 0;
let b = a;

while (true) {
    a++;
    b = a;
}
