import init, { render_blp, render_dds, render_pcx, render_tga } from "./pkg/webrpg_draw_image_example.js?v=20260920-1";

const canvas = document.querySelector("#canvas");
const status = document.querySelector("#status");
const intervalInput = document.querySelector("#interval");
const positionInput = document.querySelector("#position");
const jumpButton = document.querySelector("#jump");

await init({
  module_or_path: fetch("./pkg/webrpg_draw_image_example_bg.wasm?v=20260920-1", {
    cache: "no-store",
  }),
});

const delay = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));
let playbackId = 0;
let activePlaylist = null;

async function play(files, extension, render, startIndex = 0) {
  const currentPlaybackId = ++playbackId;

  for (let index = startIndex; index < files.length; index += 1) {
    if (currentPlaybackId !== playbackId) return;

    const file = files[index];
    const path = file.webkitRelativePath || file.name;
    try {
      const data = new Uint8Array(await file.arrayBuffer());
      if (currentPlaybackId !== playbackId) return;
      render(canvas, data);
      positionInput.value = index + 1;
      status.textContent = `[${index + 1}/${files.length}] 正在展示 ${path}`;
    } catch (error) {
      console.error(`加载 ${path} 失败：`, error);
      status.textContent = `加载 ${path} 失败：${error}`;
      return;
    }

    if (index + 1 < files.length) {
      const seconds = Number(intervalInput.value);
      await delay((Number.isFinite(seconds) ? Math.max(0, seconds) : 3) * 1000);
    }
  }

  if (currentPlaybackId === playbackId) {
    status.textContent = `${files.length} 张 ${extension} 图片展示完成`;
  }
}

function bindDirectoryInput(selector, extension, render) {
  const input = document.querySelector(selector);
  input.addEventListener("change", async () => {
    const files = Array.from(input.files)
      .filter((file) => file.name.toLowerCase().endsWith(extension))
      .sort((left, right) =>
        (left.webkitRelativePath || left.name).localeCompare(
          right.webkitRelativePath || right.name,
        ),
      );

    if (files.length === 0) {
      status.textContent = `目录中没有 ${extension} 图片`;
      return;
    }

    activePlaylist = { files, extension, render };
    positionInput.value = 1;
    positionInput.max = files.length;
    await play(files, extension, render);
  });
}

jumpButton.addEventListener("click", () => {
  if (!activePlaylist) {
    status.textContent = "请先选择图片目录";
    return;
  }

  const position = Number.parseInt(positionInput.value, 10);
  if (position < 1 || position > activePlaylist.files.length) {
    status.textContent = `图片序号应在 1 到 ${activePlaylist.files.length} 之间`;
    return;
  }

  play(
    activePlaylist.files,
    activePlaylist.extension,
    activePlaylist.render,
    position - 1,
  );
});

bindDirectoryInput("#blp-file", ".blp", render_blp);
bindDirectoryInput("#dds-file", ".dds", render_dds);
bindDirectoryInput("#pcx-file", ".pcx", render_pcx);
bindDirectoryInput("#tga-file", ".tga", render_tga);
