import init, { render_blp, render_dds, render_pcx, render_tga } from "./pkg/webrpg_example.js";

const canvas = document.querySelector("#canvas");
const status = document.querySelector("#status");

await init();

function bindFileInput(selector, render) {
  const file = document.querySelector(selector);
  file.addEventListener("change", async () => {
    const selected = file.files[0];
    if (!selected) return;

    try {
      render(canvas, new Uint8Array(await selected.arrayBuffer()));
      status.textContent = `${selected.name} 已渲染`;
    } catch (error) {
      status.textContent = `加载失败：${error}`;
    }
  });
}

bindFileInput("#blp-file", render_blp);
bindFileInput("#dds-file", render_dds);
bindFileInput("#pcx-file", render_pcx);
bindFileInput("#tga-file", render_tga);
