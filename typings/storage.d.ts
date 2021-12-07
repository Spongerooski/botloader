import { OpStorageBucketValue, OpStorageBucketEntry } from "./models/index";
/**
 * The storage namespace contains API's for persistent storage within botloader.
 */
export declare namespace Storage {
    interface SetValueOptions {
        /**
         * Optional time to live in seconds for the value, after which it is deleted.
         */
        ttl?: number;
    }
    interface ListOptions {
        /**
         * Only return entires after this key.
         *
         * Use this to paginate through results, using the key of the last entry in the list call.
         */
        after?: string;
        /**
         * Number of entries to return, max 100.
         *
         * Defaults to 25 as of writing.
         */
        limit?: number;
        /**
         * Only return entries that match the pattern,
         *
         *  - `%`: A percent sign (%) matches any sequence of zero or more characters.
         *  - `_`: An underscore (_) matches any single character.
         *
         * To match _ and % as literals escape them with backslash (`\_` and `\%`).
         *
         * @example
         * `user\_%` will match `user_123123`
         */
        keyPattern?: string;
    }
    interface SortedListOptions {
        /**
         * How many entries to skip, useful for paginating through the list
         */
        offset?: number;
        /**
         * Number of entries to return, max 100.
         *
         * Defaults to 25 as of writing.
         */
        limit?: number;
    }
    interface Entry<T> {
        /**
         * The bucket this entry was in
         */
        bucket: string;
        /**
         * Key where this entry was stored at the time of fetching
         */
        key: string;
        /**
         * Value this entry holds
         */
        value: T;
        /**
         * If a ttl was set, when this entry expires
         */
        expiresAt?: Date;
    }
    /**
     *
     * A Bucket provides persistent storage to botloader, using this you can store data and have it persist across vm reloads and bot restarts.
     *
     * Buckets are namespaces, A Bucket with the name `a` holds different values from another Bucket with the name `b` even though the keys might be the same.
     *
     * @remark this bucket should be registered with your script or plugin (example: `script.registerStorageBucket(...)`).
     *
     * @typeParam T - The type of values stored in this bucket
     */
    class Bucket<T> {
        name: string;
        /**
         * Create a new storage bucket.
         *
         * @remark this bucket should be registered with your script or plugin (example: `script.registerStorageBucket(...)`),.
         *
         * @param name The name of the bucket. Note that this is not unique across your scripts and
         * the same name in one script will have the same values as in another and is perfectly safe.
         */
        constructor(name: string);
        protected intoInternalValue(v: T): OpStorageBucketValue;
        protected fromInternalValue(v: OpStorageBucketValue): T | undefined;
        protected entryFromInternal(entry: OpStorageBucketEntry): Entry<T>;
        protected entryFromInternalOptional(entry?: OpStorageBucketEntry | null): Entry<T> | undefined;
        /**
         * Store a value at the provided key in the bucket, this will overwrite the previous value stored there, if any.
         *
         * @param key The key that you're storing the value in
         * @param value The value you're storing, for objects this will be converted to json behind the scenes
         * @param options Optional options
         * @returns The storage entry
         */
        set(key: string, value: T, options?: SetValueOptions): Promise<Entry<T>>;
        /**
         * Similar to {@link set} but stores the value conditionally.
         *
         * @param key The key where you're storing the value
         * @param value The value you're storing, for objects this will be converted to json behind the scenes
         * @param cond The condition that has to pass to store the value.
         *  - IfExists: will only store the value if the key existed beforehand.
         *  - IfNotExists: will only store the value if the key did not exist.
         * @param options Optional options
         * @returns Either the new entry, or undefined if the condition failed.
         */
        setIf(key: string, value: T, cond: "IfExists" | "IfNotExists", options?: SetValueOptions): Promise<Entry<T> | undefined>;
        /**
         * Fetches a entry from the bucket.
         *
         * @param key The key for the value you want returned
         * @returns The entry, or undefined if it did not exist
         */
        get(key: string): Promise<Entry<T> | undefined>;
        /**
         * Deletes an entry from the bucket permanently.
         *
         * @param key The key to delete
         * @returns The deleted entry, or undefined if none
         */
        del(key: string): Promise<Entry<T> | undefined>;
        /**
         * Retrieve a list of entries from the database, you can use `after` to paginate through all the items in the bucket.
         *
         * @param options Pagination options
         * @returns A list of entries
         */
        list(options: ListOptions): Promise<Entry<T>[]>;
    }
    /**
     * A Bucked holding number values
     *
     * The values being numbers allows them to be sorted easily giving you access to {@link incr} and {@link sortedList}.
     *
     * See {@link Bucket} for more info on buckets.
     */
    class NumberBucket extends Bucket<number> {
        protected intoInternalValue(v: number): OpStorageBucketValue;
        protected fromInternalValue(v: OpStorageBucketValue): number | undefined;
        /**
         * Atomically increments the value stored at key. If the entry did not exist beforehand a new one is created and set to `amount`
         *
         * @param key The key whose value to increment
         * @param amount The amount to increment the value by
         * @returns The entry after incrementing the value
         */
        incr(key: string, amount: number): Promise<Entry<number>>;
        /**
         * Returns a list of entries in the bucket sorted by values
         *
         * @param order The order of which you want the entries in the bucket sorted by
         * @param options Pagination options
         */
        sortedList(order: "Ascending" | "Descending", options?: SortedListOptions): Promise<Entry<number>[]>;
    }
}
