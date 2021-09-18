(function($window){
    $window.$jackGlobal = {}

    $window.$jackGlobal.handleDispath = function(){};

    $window.$jackGlobal.disaptchEvent = function(evt){
        $window.$jackGlobal.handleDispath(evt);
    };
})(this);
