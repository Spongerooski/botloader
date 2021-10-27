import { mkdirSync }  from 'fs';
import fse from 'fs-extra';

console.log("Hello im prebuild!");

try { 
    mkdirSync("out");
}catch{
    // probably already created, too lazy to figure out how to do proper error handling
} 

fse.removeSync("out/typings");

fse.copySync("../typings", "out/typings", {
    overwrite: true,
    recursive: true,
});