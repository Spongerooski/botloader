# Status


This project is currently a **VERY EARLY WORK IN PROGRESS**.

Contact me on discord for more details: Jonas747#0001 (105487308693757952)

https://botloader.io

# BotLoader (or loader) 

BotLoader is a discord bot where the server admins can program the bot through typescript, it takes care of all the low level things and provides an api for things such as storage, commands, timers and so on.

In the future you could imagine a steam workshop like marketplace of modules you could add to the server. Want a leveling system? There will probably be multiple available on the marketplace. That is the ultimate goal of this project, right now however it is very far away from that.

# Project layout

 - cmd
   - bot: Bot entrypoint, currently its almost the entire thing but in the future it will be split up into smaller services
   - webapi: http API for managing the bot, scripts etc... (API docs are TODO)
   - rs2ts-cli: Cli tool to transpile rust structs/enums to typescript, with scripts to run on relevant folder
 - components
   - stores: Configuration and other stores
   - botrpc: RPC interface to communicate with the bot from somewhere else (such as telling it to reload scripts after they have been updated in the db, or streaming logs)
   - runtime: VM Runtime, essentially this is what provides all the functions to interact with the outside world
   - vm: Manages a single v8 isolate
   - vmthread: Manages a thread, running vm's on it
   - vm-manager: Manages all the vm's and threads, also acts as a event muxer to send events to appropriate vms
   - isolatecell: provides a safe way to manage enter and exit states of v8 isolates
   - rs2ts: Generates typescript types from rust structs/enums
   - scriptscheduler: Provides various timers for triggering scripts
   - tscompiler: compiler for ts to js, done by swc. Note: no typechecking is performed
   - discordoauthwrapper: Simple wrapper that also handles caching for bearer discord api clients
 - botloader-vscode: vs code extension for botloader
 - frontend: https://botloader.io website