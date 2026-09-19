import init, { render_blp, render_dds, render_pcx, render_tga } from "./pkg/webrpg_draw_image_example.js?v=20260920-1";

const canvas = document.querySelector("#canvas");
const status = document.querySelector("#status");

await init({
  module_or_path: fetch("./pkg/webrpg_draw_image_example_bg.wasm?v=20260920-1", {
    cache: "no-store",
  }),
});

function bindFileInput(selector, render) {
  const input = document.querySelector(selector);
  input.addEventListener("change", async () => {
    const file = input.files[0];
    if (!file) return;

    try {
      render(canvas, new Uint8Array(await file.arrayBuffer()));
      status.textContent = `${file.name} 已通过 DrawImage 渲染`;
    } catch (error) {
      status.textContent = `加载失败：${error}`;
    }
  });
}

bindFileInput("#blp-file", render_blp);
bindFileInput("#dds-file", render_dds);
bindFileInput("#pcx-file", render_pcx);
bindFileInput("#tga-file", render_tga);
