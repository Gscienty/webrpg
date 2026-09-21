import init, { TextRenderer } from "./pkg/webrpg_render_text_example.js";

const canvas = document.querySelector("#canvas");
const fontFile = document.querySelector("#font-file");
const fontSize = document.querySelector("#font-size");
const fontColor = document.querySelector("#font-color");
const text = document.querySelector("#text");
const status = document.querySelector("#status");
let renderer;

await init();

function render() {
  if (!renderer) return;

  const color = fontColor.value.match(/[0-9a-f]{2}/gi).map(value => parseInt(value, 16));
  try {
    renderer.render(text.value, Number(fontSize.value), ...color);
    status.textContent = `${fontFile.files[0].name} · ${fontSize.value}px`;
  } catch (error) {
    status.textContent = `渲染失败：${error}`;
  }
}

fontFile.addEventListener("change", async () => {
  const file = fontFile.files[0];
  if (!file) return;
  try {
    const nextRenderer = new TextRenderer(canvas, new Uint8Array(await file.arrayBuffer()));
    renderer?.free();
    renderer = nextRenderer;
    render();
  } catch (error) {
    status.textContent = `字体加载失败：${error}`;
  }
});

fontSize.addEventListener("input", render);
fontColor.addEventListener("input", render);
text.addEventListener("input", render);
