import init, { render_blp } from "./pkg/webrpg_example.js";

const file = document.querySelector("#file");
const canvas = document.querySelector("#canvas");
const status = document.querySelector("#status");

await init();

file.addEventListener("change", async () => {
  const selected = file.files[0];
  if (!selected) return;

  try {
    render_blp(canvas, new Uint8Array(await selected.arrayBuffer()));
    status.textContent = `${selected.name} 已渲染`;
  } catch (error) {
    status.textContent = `加载失败：${error}`;
  }
});
