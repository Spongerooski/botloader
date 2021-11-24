import { Application, TSConfigReader, TypeDocReader } from "typedoc";

async function main() {
    const app = new Application();

    // If you want TypeDoc to load tsconfig.json / typedoc.json files
    app.options.addReader(new TSConfigReader());
    app.options.addReader(new TypeDocReader());

    app.bootstrap({
        // typedoc options here
        entryPoints: ["../src/ts/index.ts"],
        tsconfig: "../src/ts/tsconfig.json",
        readme: "src/README.md",
        name: "Botloader",
    });

    const project = app.convert();

    if (project) {
        // Project may not have converted correctly
        const outputDir = "docs";

        // Rendered docs
        await app.generateDocs(project, outputDir);
    }
}

main().catch(console.error);