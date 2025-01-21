#! /usr/bin/env node

import { writeFileSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { cwd, exit as __raw_exit__ } from 'node:process';
import { join } from 'node:path';

const app = (() => {
    class App {
        /** @type {string} @readonly */
        WASMPACK_OUT_DIR = ".ohkami";

        /** @type {string} @readonly */
        WASMPACK_OUT_NAME = "workers_openapi";

        /** @type {string} */
        #outputPath = "openapi.json";

        /** @type {[string]} */
        #additionalOptions = [];

        /** @type {boolean} */
        #noWorkersDevDomain = false;

        /** @type {string | undefined} */
        #workerName;

        /** @type {string | undefined} */
        #cloudflareAccountName;

        constructor() {
            try {
                const wrangler_toml = readFileSync("wrangler.toml");
                for (const line of wrangler_toml.toString('utf-8').split("\n")) {
                    if (/^workers_dev\s*=\s*false\s*(#.*)?$/.test(line)) {
                        this.#noWorkersDevDomain = true;
                    } else {
                        const nameMatch = /^name\s*=\s*"([a-zA-Z0-9_\-]+)"\s*(#.*)?$/.exec(line);
                        if (nameMatch?.length >= 2) {
                            this.#workerName = nameMatch[1];
                        }
                    }
                }
            } catch (e) {
                this.exit(150, `Expected a wrangler project, but ${e}`)
            }

            if (process.argv.length > 2) {
                let i = 2;
                while (i < process.argv.length) {
                    switch (process.argv[i]) {
                        case "--out":
                        case "-o":
                            this.#outputPath = process.argv[i + 1];
                            i += 2;
                            break;
                        case "--":
                            this.#additionalOptions = process.argv.slice(i + 1);
                            i = process.argv.length;
                            break;
                        default:
                            this.exit(151, `Unexpected flag specified: ${process.argv[i]}`);
                    }
                }
            }
        }

        /** @returns {string} */
        get outputPath() {
            return this.#outputPath;
        }

        /** @returns {string} */
        get additionalOptions() {
            return this.#additionalOptions;
        }

        /** @returns {boolean} */
        get noWorkersDevDomain() {
            return this.#noWorkersDevDomain;
        }

        /** @returns {string} */
        get workerName() {
            return this.#workerName;
        }

        /** @returns {string | null} */
        get cloudflareAccountName() {
            return this.#cloudflareAccountName;
        }
        /** @param {string} AccountName */
        set cloudflareAccountName(AccountName) {
            this.#cloudflareAccountName = AccountName;
        }

        /**
         * @param {number} code 
         * @param {string | undefined} message 
         * @returns {void}
         * */
        exit(code, message) {
            if (existsSync(this.WASMPACK_OUT_DIR)) {
                rmSync(this.WASMPACK_OUT_DIR, { recursive: true, force: true });
            }
        
            if (typeof message == 'string') {
                console.error("[workers_openapi.js] Fatal:", message);
            }
            
            __raw_exit__(code);
        }

        /**
         * @param {string} message 
         * @returns {void}
         */
        warn(message) {
            if (typeof message == 'string') {
                console.error("[workers_openapi.js] Warning:", message);
            }
        }
    }

    return new App();
})();

try {
    /**
     * ```e.g.
     * │ kanarus      │ 0xx000x000x0000000x0xxx0x000xx00 │
     * ```
     */
    const wrangler_whoami = spawn("wrangler", ["whoami"]);

    const e = new TextDecoder();

    await new Promise((resolve, reject) => {
        wrangler_whoami.on("close", (code) => {
            if (code === 0) {resolve()} else {reject(`'wrangler whoami' closed with ${code}`)}
        });
        wrangler_whoami.on("exit", (code) => {
            if (code === 0) {resolve()} else {reject(`'wrangler whoami' exited with ${code}`)}
        });
        wrangler_whoami.on("error", (err) => {
            reject(`'wrangler whoami' failed: ${err}`);
        });
        wrangler_whoami.on("disconnect", () => {
            reject(`'wasm-pack build' disconnected`);
        });

        /////////////////////////////////////////////

        wrangler_whoami.stdout.on("data", (data) => {
            for (const line of e.decode(data).trimEnd().split("\n")) {
                if (/^🔓|Scope|-/.test(line)) break;
                console.log(line);
                if (/^│ .* \s*│ [0-9a-z]{32} │$/.test(line)) {                
                    app.cloudflareAccountName = line.split("│")[1].trim();
                    resolve();
                }
            }
        });
    });
} catch (e) {
    app.warn(`Error or unexpected output of wrangler: ${e}`);
}

try {
    if (existsSync(app.WASMPACK_OUT_DIR)) {
        rmSync(app.WASMPACK_OUT_DIR, { recursive: true, force: true });
    }

    /**
     * `wasm-pack` is expected to be available because
     * it's a dependency of `worker-build`.
     * */
    const wasmpack_build = spawn("wasm-pack", [
        "build",
        "--dev",
        "--no-opt",
        "--no-pack",
        "--no-typescript",
        "--target", "nodejs",
        "--out-dir", app.WASMPACK_OUT_DIR,
        "--out-name", app.WASMPACK_OUT_NAME,
        "--", ...app.additionalOptions
    ], { stdio: "inherit" });

    await new Promise((resolve, reject) => {
        wasmpack_build.on("close", (code) => {
            if (code === 0) {resolve()} else {reject(`'wasm-pack build' closed with ${code}`)}
        });
        wasmpack_build.on("exit", (code) => {
            if (code === 0) {resolve()} else {reject(`'wasm-pack build' exited with ${code}`)}
        });
        wasmpack_build.on("error", (err) => {
            reject(`'wasm-pack build' failed: ${err}`);
        });
        wasmpack_build.on("disconnect", () => {
            reject(`'wasm-pack' build disconnected`);
        });
    });
} catch (e) {
    app.exit(153, `Build failed: ${e}`);
}

try {
    const wasmpack_js = await import(join(
        cwd(),
        app.WASMPACK_OUT_DIR,
        `${app.WASMPACK_OUT_NAME}.js`
    ));
    if (!wasmpack_js.OpenAPIDocumentBytes) {
        throw new Error("Not activating Ohkami's `openapi` feature flag");
    }

    /** @type {Uint8Array} */
    let OpenAPIDocumentBytes = wasmpack_js.OpenAPIDocumentBytes();

    {
        let OpenAPIDocumentJSON = JSON.parse(
            (new TextDecoder()).decode(OpenAPIDocumentBytes)
        );

        if (OpenAPIDocumentJSON.servers
            .filter((s) => s.url.includes("localhost"))
            .length === 0
        ) {
            OpenAPIDocumentJSON.servers.push({
                url: `http://localhost:8787`,
                description: "local dev",
            });
        }

        /**
         * This process should not be done in `#[ohkami::worker]` attribute's
         * background becasue of heavy latency of `wrangler whoami`, causing
         * too bad developer experience for rust-analyzer users.
        */
        if (app.workerName && app.cloudflareAccountName && !app.noWorkersDevDomain) {
            if (OpenAPIDocumentJSON.servers
                .filter((s) => !(s.url.includes("localhost")))
                .length === 0
            ) {
                OpenAPIDocumentJSON.servers.push({
                    url: `https://${app.workerName}.${app.cloudflareAccountName}.workers.dev`,
                    description: "production",
                });
            }
        }

        OpenAPIDocumentBytes = (new TextEncoder()).encode(
            JSON.stringify(OpenAPIDocumentJSON, null, 2)
            + "\n"
        );
    }

    writeFileSync(app.outputPath, OpenAPIDocumentBytes);

} catch (e) {
    app.exit(154, `Generation failed: ${e}`);
}

try {
    rmSync(app.WASMPACK_OUT_DIR, { recursive: true, force: true });
} catch (e) {
    app.exit(155, `Cleaning up failed: ${e}`);
}
