# DrawImage BLP / DDS / PCX / TGA example

```sh
cd example/draw_image
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
```

打开 <http://localhost:8080>，选择 BLP、DDS、PCX 或 TGA 图片目录。示例会按相对路径依次解析并渲染对应格式的全部图片；可以指定轮播间隔，也可以输入从 1 开始的图片序号进行跳转。遇到错误时会停止并显示文件路径。
