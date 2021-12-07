import { execSync } from 'child_process';
import { mkdirSync }  from 'fs';
import fse from 'fs-extra';
// import {}

console.log("Hello im prebuild!");

execSync('"../components/runtime/src/ts/typedecls.sh"');

try { 
    mkdirSync("out");
}catch{
    // probably already created, too lazy to figure out how to do proper error handling
} 

fse.removeSync("out/typings");

fse.copySync("../components/runtime/src/ts/typings", "out/typings", {
    overwrite: true,
    recursive: true,
});