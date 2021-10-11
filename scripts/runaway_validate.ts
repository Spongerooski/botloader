import { Bot, console, CreateMessage, Timers } from 'botloader';

let a = 0;
let b = a;

while (true) {
    a++;
    b = a;
}


Bot.registerMeta({
    name: "runaway_validate",
    context: "Guild",
})

