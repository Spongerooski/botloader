declare let $jackGlobal: {
    runEventLoop: (cb: (evt: { name: string, data: any }) => void) => void;
};