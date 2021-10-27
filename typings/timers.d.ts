export declare namespace Timers {
    /**
     * Adds a event handler for when tasks with 'key' are triggered.
     *
     * Any data you pass in will be serialized to json so class info will be lost
    */
    function onTask<T>(key: string, cb: (data: T) => any): Promise<void>;
    /**
     * Schedules a task to run at the specified date.
     *
     * @param key A key for this task type, for example for a `unmute`.
     *
     * You should prepend either `SCRIPT_ID` or `SCRIPT_CONTEXT_ID` to prevent clashing with other scripts.
     *
     * Example: SCRIPT_CONTEXT_ID + "unmute"
     *
     * @param at When the task should be run
     * @param data Data that will be passed to the handler when triggered, serialized to json
     * @param id Optionally provide a ID unique to this task, if a id is not provided one will be generated for you
     * @returns The id of the task
     */
    function scheduleTask(key: string, at: Date, data: any, id?: string): Promise<string>;
    /**
     * Cancels a scheduled taskwith the provided key and id
     */
    function cancelTask(key: string, id: string): Promise<void>;
    type IntervalCB = (a: string) => any;
    /**
     * Starts a non persistent interval timer with the provided id. If you need to ensure that this runs even if the bot was down during the provided trigger interval use a persistent timer.
     *
     * @param id An id for the interval timer, used to stop the interval later.
     * @param interval The actual interval, either in minutes or as a cron style string.
     *
     * https://crontab.guru is a usefull tool for making cron intervals
     *
     * @param cb The callback function
     */
    function startInterval(id: string, interval: number | string, cb: IntervalCB): Promise<void>;
    /**
     * Starts a persistent interval timer, persistent meaning it will remember the last time it was run and run as soon after as possible, for example after the bot was down.
     *
     * @param id An id for the timer, prepend with `SCRIPT_ID` or `SCRIPT_CONTEXT_ID` to avoid clashing with other scripts.
     * @param interval The actual interval, either in minutes or as a cron style string.
     *
     * https://crontab.guru is a usefull tool for making cron intervals
     *
     * @param cb The callback function
     */
    function persistentInterval(id: string, interval: number | string, cb: IntervalCB): Promise<void>;
    /**
     * Stops the interval timer with the provided ID
     *
     * Note that this will also delete the state for a persistent timer.
     */
    function stopInterval(id: string): Promise<void>;
    class IntervalTimerSeconds {
        name: string;
        interval_seconds?: number;
        constructor(name: string, interval_seconds: number);
    }
    class IntervalTimerCron {
        name: string;
        interval_cron?: string;
        constructor(name: string, interval_cron: string);
    }
}
