#! /usr/bin/env node

import { writeFileSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { execSync, spawn } from 'node:child_process';
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

        /** @type {boolean} */
        #skipLogin = false;

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

            const cliArgsStartIndex = (() => {
                /** this script is executed via `node -e ...` and no CLI arguments are given */
                if (process.argv.length === 1) return 1;

                const argv1 = process.argv[1];
                if (argv1.startsWith("/") || argv1.startsWith("C:") || argv1.startsWith("D:")) {
                    /** this script is executed by `node workers_openapi.js` */
                    return 2;
                } else {
                    /** this script is executed via `node -e ...` */
                    return 1;
                }
            })();
            if (process.argv.length > cliArgsStartIndex) {
                let i = cliArgsStartIndex;
                while (i < process.argv.length) {
                    switch (process.argv[i]) {
                        case "--out":
                        case "-o":
                            this.#outputPath = process.argv[i + 1];
                            i += 2;
                            break;
                        case "--skip-login":
                            this.#skipLogin = true;
                            i += 1;
                            break;
                        case "--":
                            this.#additionalOptions = process.argv.slice(i + 1);
                            i = process.argv.length;
                            break;
                        default:
                            this.#additionalOptions = process.argv.slice(i);
                            i = process.argv.length;
                            break;
                    }
                }
            }
        }

        /** @returns {string} */
        get outputPath() {
            return this.#outputPath;
        }

        /** @returns {boolean} */
        get skipLogin() {
            return this.#skipLogin;
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

        /** @returns {string | undefined} */
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
         */
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
    if (app.skipLogin) {
        /* goto `catch` and skip `wrangler whoami` */
        throw "specified `--skip-login`";
    }

    const e = new TextDecoder();

    const wrangler_whoami = spawn("npx", ["wrangler", "whoami"]);
    await new Promise((resolve, reject) => {
        wrangler_whoami.on("close", (code) => {
            if (code === 0) {resolve()} else {reject(`'npx wrangler whoami' closed with ${code}`)}
        });
        wrangler_whoami.on("exit", (code) => {
            if (code === 0) {resolve()} else {reject(`'npx wrangler whoami' exited with ${code}`)}
        });
        wrangler_whoami.on("error", (err) => {
            reject(`'npx wrangler whoami' failed: ${err}`);
        });
        wrangler_whoami.on("disconnect", () => {
            reject(`'npx wrangler whoami' disconnected`);
        });

        //////////////////////////////////////////////////////////////

        wrangler_whoami.stdout.on("data", (data) => {
            for (const line of e.decode(data).trimEnd().split("\n")) {
                if (/^🔓|Scope|-/.test(line)) break;

                console.log(line);

                /**
                 * ```e.g.
                 * │ kanarus      │ 0xx000x000x0000000x0xxx0x000xx00 │
                 * ```
                 */
                if (/^│ .* \s*│ [0-9a-z]{32} │$/.test(line)) {                
                    app.cloudflareAccountName = line.split("│")[1].trim();
                    resolve();
                }
            }
        });
    });
} catch (e) {
    app.warn(`skipping login: ${e}`);
}

try {
    const wasmpack_is_installed = (() => {
        try {
            execSync("which wasm-pack");
            return true;
        } catch (e) {
            return false;
        }
    })();
    if (!wasmpack_is_installed) {
        const cargo_install_wasmpack = spawn("cargo", [
            "install",
            "wasm-pack"
        ], { stdio: "inherit" });
        await new Promise((resolve, reject) => {
            cargo_install_wasmpack.on("close", (code) => {
                if (code === 0) {resolve()} else {reject(`'cargo install' closed with ${code}`)}
            });
            cargo_install_wasmpack.on("exit", (code) => {
                if (code === 0) {resolve()} else {reject(`'cargo install' exited with ${code}`)}
            });
            cargo_install_wasmpack.on("error", (err) => {
                reject(`'cargo install' failed: ${err}`);
            });
            cargo_install_wasmpack.on("disconnect", () => {
                reject(`'cargo install' disconnected`);
            });
        });
    }

    if (existsSync(app.WASMPACK_OUT_DIR)) {
        rmSync(app.WASMPACK_OUT_DIR, { recursive: true, force: true });
    }

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
            reject(`'wasm-pack build' disconnected`);
        });
    });
} catch (e) {
    app.exit(151, `Build failed: ${e}`);
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
    let OpenAPIDocumentBytes = await wasmpack_js.OpenAPIDocumentBytes();

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
    app.exit(152, `Generation failed: ${e}`);
}

try {
    rmSync(app.WASMPACK_OUT_DIR, { recursive: true, force: true });
} catch (e) {
    app.exit(153, `Cleaning up failed: ${e}`);
}
