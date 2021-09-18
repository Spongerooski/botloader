export namespace console {
    export function log(msg: string, isErr: boolean = false) {
        Deno.core.print(msg + "\n", isErr);
    }
}
