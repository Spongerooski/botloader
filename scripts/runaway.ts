import { Script } from 'botloader';

const script = new Script("runaway script");


let a = 0;
let b = a;

script.on("MESSAGE_CREATE", async evt => {
    while (true) {
        a++;
        b = a;
    }
})

script.run();