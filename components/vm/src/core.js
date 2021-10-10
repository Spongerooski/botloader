(function($window){
    $window.$jackGlobal = {}

    $window.$jackGlobal.handleDispatch = function(){};

    $window.$jackGlobal.dispatchEvent = function(evt){
        $window.$jackGlobal.handleDispatch(evt);
    }; 
})(this); 