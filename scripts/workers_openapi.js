#! /usr/bin/env node

import { writeFileSync, existsSync, rmSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { cwd, exit as __raw_exit__ } from 'node:process';
import { join } from 'node:path';

const app = (() => {
    class App {
        /** @type {string} @readonly */
        WASMPACK_OUT_DIR = ".ohkami-workers_openapi-worker_build-out";

        /** @type {string} @readonly */
        WASMPACK_OUT_NAME = ".ohkami-workers_openapi-worker_build";

        /** @type {string} */
        #outputPath = "openapi.json";

        /** @type {[string]} */
        #additionalOptions = [];

        /**
         * Based on https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/iniscrptact.html,
         * we'll use codes in *reserved for application use* range as exit codes
         * 
         * @type {number}
         * */
        #code = 150;

        constructor() {
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
                            this.exit(`Unexpected flag specified: ${process.argv[i]}`);
                    }
                }
            }

            try {
                if (!existsSync("Cargo.toml")) throw new Error("`Cargo.toml` directory not found");
                if (!existsSync("src")) throw new Error("`src` directory not found");
            } catch (e) {
                this.exit(`Expected to be called at the top of a Rust package, but ${e}`)
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

        /**
         * @param {string | undefined} message 
         * @returns {void}
         * */
        exit(message) {
            if (existsSync(app.WASMPACK_OUT_DIR)) {
                rmSync(app.WASMPACK_OUT_DIR, { recursive: true, force: true });
            }
        
            if (typeof message == 'string') {
                console.error(message);
            }
            
            const code = this.#code;
            this.#code += 1;
            __raw_exit__(code);
        }
    }

    return new App();
})();

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
            if (code === 0) {resolve()} else {app.exit()}
        });
        wasmpack_build.on("exit", (code) => {
            if (code === 0) {resolve()} else {app.exit()}
        });
        wasmpack_build.on("error", (err) => {
            reject(err);
        });
        wasmpack_build.on("disconnect", () => {
            reject("disconnected");
        });
    });

} catch (e) {
    app.exit(`Build failed: ${e}`);
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
    const OpenAPIDocumentBytes = wasmpack_js.OpenAPIDocumentBytes();

    writeFileSync(app.outputPath, OpenAPIDocumentBytes);

} catch (e) {
    app.exit(`Generation failed: ${e}`);
}
