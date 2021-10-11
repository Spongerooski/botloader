import { Bot, Commands, console, CreateMessage, Timers } from 'botloader';

// import { bot } 

Bot.registerMeta({
    name: "pog",
    context: "Guild",
    timers: [
        new Timers.IntervalTimerCron("epic", "aaa"),
        new Timers.IntervalTimerSeconds("epic", 123),
    ],
})

Commands.registerCommand({
    name: "add",
    description: "add 2 numbers",
    options: {
        "a": { description: "first number", kind: "NUMBER", required: true },
        "b": { description: "second number", kind: "NUMBER", required: true },
        "optional": { description: "optional number", kind: "NUMBER" },
    },
    callback: async (ctx, args) => {
        let first_arg = args.a;
        let second_arg = args.b;
        let third = args.optional;

        let result = args.a + args.b;
        await ctx.sendResponse(`Result: ${result}`);
    }
})

Commands.registerGroup(new Commands.Group("misc", "misc").registerCommand({
    name: "add",
    description: "add 2 numbers",
    options: {
        "a": { description: "first number", kind: "NUMBER", required: true },
        "b": { description: "second number", kind: "NUMBER", required: true },
        "optional": { description: "optional number", kind: "NUMBER" },
    },
    callback: async (ctx, args) => {
        let first_arg = args.a;
        let second_arg = args.b;
        let third = args.optional;

        let result = args.a + args.b;
        await ctx.sendResponse(`Result: ${result}`);
    }
}).registerCommand({
    name: "echo",
    description: "add 2 numbers",
    options: {
        "what": { description: "what to echo", kind: "STRING", required: true },
    },
    callback: async (ctx, args) => {
        await ctx.sendResponse(`Result: ${args.what}`);
    }
}))


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