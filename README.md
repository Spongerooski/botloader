# Jack bot

What is jack bot? 

Jack bot is a user discord bot where the server admins can program the bot through javascript, the initial prototype will include a basic runtime API for interacting with discord and basic storage.

In the future you could imagine a steam workshop like marketplace of modules you could add to the server, want a leveling system? there will probably be multiple available on the marketplace and if none suits your needs you can modify an existing one to do exactly what you want, wihtout having to worry about things like the ever changing discord API, the pain of running and scaling bots and all the low level stuff.

Jack bot will provide a simple high level API that will strive to be backwards compatible.

# Technical details

At the core it uses deno, which is a layer above v8 that's secure by default, meaning we don't have to worry about all the different knobs on v8.

The prototype wont have this but the release will also have seperate script running processes which if your script/module is caught using a lot of CPU time will killl it, and if it happens repeatedly it will eventually mark it as bad. This should be enough to deal with bitcoin miners as the higher level javascript code shouldn't realistically have a reason for having more than a second's worth of cpu time at most for handling most events, all the time is usually spent outside the script doing things such as api calls, database queries and so on.

# Project layout

 - cmd
   - jack-bot: the bot itself, currently its the entire thing but in the future it may be split up
 - components
   - configstore: the core configuration store for jack bot, currently this only handles storing scripts and link themselves, there will probably be something else for custom user storage through scripts.
   - jack-sandbox: the javascript sandboxing portion, this is essentially just a thin wrapper around deno-core which does some of the heavy lifting, the sandbox does not have any capabilities in and of itself, that's provided by the runtime
   - jack-runtime: the jack javascript runtime, essentially this is what provides all the functions to interact with the outside world

# Protoype Architechture

Everything runs in 1 process, the bot, using v8

**Script contexts and their events**

 - Guild
    - Role events
    - Channel events
    - Member events
    - Time fired
 - Role(roleid)
    - Role updated
    - Role assigned to member
    - Role removed from member
    - Time fired
 - Channel(channelid)
    - Message events
    - Reaction events
    - Channel update
    - Time fired



**General Events:**

Guild:

 - Guild update

Messages:

 - Message create
 - Message edit
 - Message deletes

Members:

 - Member add
 - Member update
 - Member remove

Reactions:

 - TODO

Roles:

 - Role add
 - Role update
 - Role remove
 
Channels:

 - Channel add
 - Channel update
 - Channel remove

**Bot namespace**

The bot context is a global namespace always available to perform various actions

From there you can:

 - Get global context data:
    - Guild object
    - Roles
    - Channels
    - Emojis
 - Fetch additional discord data:
    - Members
    - Messages
    - Reactions
 - Interact with the discord API through the bot:
    - SendMessage
    - DeleteMessage
    - EditMessage
