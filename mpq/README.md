# webrpg-mpq

WebAssembly helpers for storing MPQ archives in OPFS and reading files from them.

```js
import init, {
  load_mpq,
  OpfsMpq,
  read_mpq_file,
  read_opfs_mpq_file,
  store_mpq,
} from "./pkg/webrpg_mpq.js";

await init();

await store_mpq("map.w3x", uploadedBytes);

const archive = await load_mpq("map.w3x");
const model = read_mpq_file(archive, "war3mapImported/model.mdx");

// Equivalent convenience call.
const script = await read_opfs_mpq_file("map.w3x", "war3map.j");

// Load OPFS asynchronously once, then read archive members synchronously.
const mpq = await OpfsMpq.open("map.w3x");
const first = mpq.read_file("war3map.j");
const second = mpq.read_file("war3mapImported/model.mdx");
```

Build with:

```sh
wasm-pack build mpq --target web
```

OPFS requires a secure browser context (HTTPS or localhost).
Opening an OPFS file is asynchronous by browser design. `OpfsMpq.read_file()` is
synchronous because the archive has already been loaded into memory by
`OpfsMpq.open()`.
