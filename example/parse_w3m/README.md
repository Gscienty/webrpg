# W3M terrain parser example

```sh
cd example/parse_w3m
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
```

打开 <http://localhost:8080>。上传的 MPQ、W3M、W3X 会保存在浏览器 OPFS 中；选择地图后会解析 `war3map.w3e`，并用普通色块展示完整地形。也可以从侧边栏点击 `.w3e`，或打开 MPQ 内嵌套的 `.w3m`、`.w3x` 地图。

