import { Script } from "./script"

declare global {
    const script: Script;
    const console: {
        log: (...args: any[]) => void,
    };
}