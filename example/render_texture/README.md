# BLP / DDS / PCX / TGA WebGL2 example

```sh
cd example/render_texture
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
```

然后打开 <http://localhost:8080>，分别通过四个入口选择 `.blp`、`.dds`、`.pcx` 或 `.tga` 文件。
