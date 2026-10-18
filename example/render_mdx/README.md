# MPQ / MDX renderer example

```sh
cd example/render_mdx
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
```

打开 <http://localhost:8080>。上传的 MPQ、W3M、W3X 会保存在浏览器 OPFS 中；选择归档后可浏览其 `(listfile)` 目录，点击 `.mdx` 文件即可渲染。
