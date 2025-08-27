#! /usr/bin/env node

import { join } from 'node:path';
import { cwd, exit } from 'node:process';

const wasmpack_js = await import(join(cwd(), `pkg`, `worker_bindings_test.js`));
if (!wasmpack_js) {
    exit("wasmpack_js is not found")
}

wasmpack_js.handle_dummy_env();

console.log("ok");
