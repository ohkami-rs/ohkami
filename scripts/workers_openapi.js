#! /usr/bin/env node

import { writeFileSync, existsSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { exit as __raw_exit__ } from 'node:process';

const app = (() => {
    class App {
        /** @type {string} @readonly */
        WORKER_BUILD_OUTPUT_DIR = "workers_openapi-worker_build-output";

        /** @type {string} */
        #outputPath;

        /** @type {number} */
        #code;

        constructor() {
            /**
             * Based on https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/iniscrptact.html,
             * we'll use codes in *reserved for application use* range as exit codes
             * */
            this.#code = 150;

            let [, , path] = process.argv;
            this.#outputPath = path || "openapi.json";

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
    // `wasm-pack` is expected to be available
    // because it's a dependency of `worker-build`.
    execSync(`
        wasm-pack build
            --dev
            --no-opt
            --no-pack
            --no-typescript
            --target nodejs
            --out-dir ${app.WORKER_BUILD_OUTPUT_DIR}
    `);
} catch (e) {
    app.exit();
}


// ..


try {
    writeFileSync(app.outputPath, );
} catch (e) {

    CLI.exit(1);
}
