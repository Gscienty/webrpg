# Span text rendering example

```sh
cd example/render_text
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
```

打开 <http://localhost:8080>，上传 `.ttf` 或 `.ttc` 字体后即可修改文字、字号和颜色。TTC 默认使用集合中的第一个字体。
