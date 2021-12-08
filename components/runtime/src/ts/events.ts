import { Events, Discord } from './models';

export interface EventTypes {
    /**
     * @internal
     */
    BOTLOADER_COMMAND_INTERACTION_CREATE: Events.CommandInteraction,
    /**
     * @internal
     */
    BOTLOADER_INTERVAL_TIMER_FIRED: Events.IntervalTimerEvent,
    MESSAGE_CREATE: Discord.Message,
    MESSAGE_UPDATE: Events.MessageUpdate,
    MESSAGE_DELETE: Events.MessageDelete,
}

/**
 * @internal
 */
export namespace InternalEventSystem {

    const eventMuxers: EventMuxer[] = [];

    export function registerEventMuxer(muxer: EventMuxer) {
        eventMuxers.push(muxer)
    }

    export function dispatchEvent(evt: DispatchEvent) {
        for (let muxer of eventMuxers) {
            muxer.handleEvent(evt);
        }
    }
}

if ((typeof $jackGlobal) !== "undefined") {
    $jackGlobal.runEventLoop(InternalEventSystem.dispatchEvent)
}

interface DispatchEvent {
    name: string,
    data: any,
}

type ListenerMap = {
    [Property in keyof EventTypes]+?: ((evt: EventTypes[Property]) => void)[];
}

export class EventMuxer {

    listeners: ListenerMap = {};

    /**
     * @internal
     */
    handleEvent(evt: DispatchEvent) {
        let handlers = this.listeners[evt.name as keyof EventTypes];
        if (handlers) {
            for (let handler of handlers) {
                handler(evt.data)
            }
        }
    }

    /**
     * @internal
     */
    on<T extends keyof EventTypes>(eventType: T, cb: (evt: EventTypes[T]) => void) {
        let handlers = this.listeners[eventType];

        // we cast to any since typescript isn't able to handle this
        if (handlers) {
            handlers.push(cb as any);
        } else {
            this.listeners[eventType] = [cb as any];
        }
    }

}

