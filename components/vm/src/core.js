(function($window){
    $window.$jackGlobal = {}

    $window.$jackGlobal.runEventLoop = async function(cb){
        while(true){
            const next = await Deno.core.opAsync("op_botloader_rcv_event");
            if (next.name === "STOP"){
                return;
            }
            cb(next);
        }
    }
})(this);