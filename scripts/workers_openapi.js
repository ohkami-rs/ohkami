#! /usr/bin/env node

import { writeFileSync, existsSync, rmSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { cwd, exit as __raw_exit__ } from 'node:process';
import { join } from 'node:path';

const app = (() => {
    class App {
        /** @type {string} @readonly */
        WASMPACK_OUT_DIR = "workers_openapi-worker_build-out";

        /** @type {string} @readonly */
        WASMPACK_OUT_NAME = "workers_openapi-worker_build";

        /** @type {string} */
        #outputPath = "openapi.json";

        /**
         * Based on https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/iniscrptact.html,
         * we'll use codes in *reserved for application use* range as exit codes
         * 
         * @type {number}
         * */
        #code = 150;

        constructor() {
            {
                let [, , path] = process.argv;
                if (path) this.#outputPath = path;
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

        /**
         * @param {string | undefined} message 
         * @returns {void}
         * */
        exit(message) {
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
    execSync(`
        wasm-pack build \
            --dev \
            --no-opt \
            --no-pack \
            --no-typescript \
            --target nodejs \
            --out-dir ${app.WASMPACK_OUT_DIR} \
            --out-name ${app.WASMPACK_OUT_NAME} \
    `);

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
