import { Script, OpWrappers } from 'botloader';

const script = new Script("Super pog script");

let counter = 1;

script.on("MESSAGE_CREATE", async evt => {
    if (!evt.author.bot && evt.content === "pog") {
        counter++;
        await OpWrappers.createMessage({
            channelId: evt.channelId,
            content: "pog #" + counter,
        })
    }
})

script.registerCommand({
    name: "add",
    description: "add 2 numbers",
    options: {
        "a": { description: "first number", kind: "Number", required: true },
        "b": { description: "second number", kind: "Number", required: true },
        "optional": { description: "optional number", kind: "Number" },
    },
    callback: async (ctx, args) => {
        let result = args.a + args.b;
        await ctx.sendResponse(`Result: ${result}`);
    }
})

// and then something like
// script.registerIntervalTimer(...)
// script.registerConfig(...)
// script.registerStorage(...)

script.run()
