# BLP WebGL2 example

```sh
cd example/blp_render
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
```

然后打开 <http://localhost:8080>，选择一个 `.blp` 文件。
