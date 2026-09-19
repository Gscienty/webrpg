# DrawImage BLP / DDS / PCX / TGA example

```sh
cd example/draw_image
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
```

打开 <http://localhost:8080>，选择 `.blp`、`.dds`、`.pcx` 或 `.tga` 文件即可通过 `DrawImage` 渲染。
