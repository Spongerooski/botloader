# Status

This project is currently a **VERY EARLY WORK IN PROGRESS**.

Contact me on discord for more details: Jonas747#0001 (105487308693757952)

# BotLoader (or loader)

What is BotLoader? 

BotLoader is a discord bot where the server admins can program the bot through typescript, it takes care of all the low level things and provides an api for things such as storage, commands, timers and so on.

In the future you could imagine a steam workshop like marketplace of modules you could add to the server. Want a leveling system? There will probably be multiple available on the marketplace and if none suit your needs you can modify an existing one to do exactly what you want, without having to worry about things like the ever changing discord API, the pain of running and scaling bots and all the low level stuff.

BotLoader will provide a simple high level API that will strive to be backwards compatible where possible (of course this can't be a 100% guarantee as discord's changes aren't always backwards compatible themselves).

# Technical details

At the core it uses deno, which is a layer above v8 that's secure by default, meaning we don't have to worry about all the different knobs on v8.

# Project layout

 - cmd
   - bot: the bot itself, currently its the entire thing but in the future it may be split up
   - webapi: the frontend API
 - components
   - configstore: the core configuration store for BotLoader, currently this only handles storing scripts and link themselves, there will probably be something else for custom user storage through scripts.
   - runtime: the bot runtime, essentially this is what provides all the functions to interact with the outside world
   - vm: manages a single v8 isolate
   - vmthread: manages a thread, running vm's on it
   - isolatecell: provides a safe way to manage enter and exit states of v8 isolates
   - rs2ts: Generates typescript types from rust structs
   - scriptscheduler: Provides various timers for triggering scripts

# Script packs and guild scripts

The current plan is as follows:
 
 - 1x v8 isolate to run standalone guild scripts, that is 1 isolate in total for all the standalone guild scripts
 - 1x v8 isolate per script pack

This way, bugs in script packs wont affect other vm's and so on. Failures are somewhat isolated.

This also makes it easier to namespace certain things
