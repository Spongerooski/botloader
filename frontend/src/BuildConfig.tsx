export const BuildConfig = {
    botloaderApiBase: process.env["REACT_APP_BOTLOADER_API_BASE"] || "http://localhost:7447",
}

console.log(`using api base: ${BuildConfig.botloaderApiBase}`);