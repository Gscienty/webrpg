import init, { list_mpq_files, Terrain } from "./pkg/webrpg_parse_w3m_example.js";

const upload = document.querySelector("#upload");
const archives = document.querySelector("#archives");
const breadcrumbs = document.querySelector("#breadcrumbs");
const entries = document.querySelector("#entries");
const status = document.querySelector("#status");
const canvas = document.querySelector("#canvas");
const context = canvas.getContext("2d");

const colors = [
  "#638747", "#76984e", "#a49561", "#8a7048",
  "#587a43", "#9a8c66", "#6f8250", "#a56d46",
  "#526c3f", "#8b955c", "#81714e", "#677d55",
  "#a38b55", "#707b45", "#916346", "#607447",
];

await init();

const root = await navigator.storage.getDirectory();
let activeArchive = null;
let activeData = null;
let filePaths = [];
let currentPath = [];

function setStatus(message) {
  status.textContent = message;
}

async function readOpfsFile(name) {
  const handle = await root.getFileHandle(name);
  return new Uint8Array(await (await handle.getFile()).arrayBuffer());
}

async function refreshArchives() {
  archives.replaceChildren();
  const names = [];
  for await (const [name, handle] of root.entries()) {
    if (handle.kind === "file" && /\.(mpq|w3m|w3x)$/i.test(name)) names.push(name);
  }
  names.sort((a, b) => a.localeCompare(b));
  for (const name of names) {
    const row = document.createElement("li");
    const open = document.createElement("button");
    open.textContent = `📦 ${name}`;
    open.onclick = () => openArchive(name);
    const remove = document.createElement("button");
    remove.className = "danger";
    remove.textContent = "删除";
    remove.onclick = async () => {
      await root.removeEntry(name);
      if (activeArchive === name) closeArchive();
      await refreshArchives();
    };
    row.append(open, remove);
    archives.append(row);
  }
  if (!names.length) setStatus("请上传 MPQ、W3M 或 W3X 文件");
}

async function openArchive(name) {
  try {
    setStatus(`正在打开 ${name}…`);
    activeArchive = name;
    activeData = await readOpfsFile(name);
    filePaths = list_mpq_files(name, activeData)
      .split(/\r?\n/)
      .filter(Boolean)
      .map((path) => path.replaceAll("/", "\\"));
    currentPath = [];
    renderDirectory();
    const terrainPath = filePaths.find((path) => /(^|\\)war3map\.w3e$/i.test(path));
    if (terrainPath) renderTerrain(terrainPath);
    else setStatus(`${name}：${filePaths.length} 个可枚举文件`);
  } catch (error) {
    closeArchive();
    setStatus(`无法打开 ${name}：${error}`);
  }
}

function closeArchive() {
  activeArchive = null;
  activeData = null;
  filePaths = [];
  currentPath = [];
  breadcrumbs.replaceChildren();
  entries.replaceChildren();
  context.clearRect(0, 0, canvas.width, canvas.height);
}

function renderDirectory() {
  breadcrumbs.replaceChildren();
  ["根目录", ...currentPath].forEach((part, index) => {
    const button = document.createElement("button");
    button.textContent = part;
    button.onclick = () => {
      currentPath = currentPath.slice(0, index);
      renderDirectory();
    };
    breadcrumbs.append(button);
  });

  const prefix = currentPath.length ? `${currentPath.join("\\")}\\` : "";
  const dirs = new Set();
  const files = [];
  for (const path of filePaths) {
    if (!path.toLowerCase().startsWith(prefix.toLowerCase())) continue;
    const rest = path.slice(prefix.length);
    const slash = rest.indexOf("\\");
    if (slash >= 0) dirs.add(rest.slice(0, slash));
    else if (rest) files.push({ name: rest, path });
  }

  entries.replaceChildren();
  for (const name of [...dirs].sort((a, b) => a.localeCompare(b))) {
    addEntry(`📁 ${name}`, () => {
      currentPath.push(name);
      renderDirectory();
    });
  }
  files.sort((a, b) => a.name.localeCompare(b));
  for (const file of files) {
    const isTerrain = /\.w3e$/i.test(file.name);
    const isMap = /\.(w3m|w3x)$/i.test(file.name);
    addEntry(`${isTerrain || isMap ? "🗺️" : "📄"} ${file.name}`,
      isTerrain || isMap ? () => renderTerrain(file.path) : null);
  }
}

function addEntry(label, onclick) {
  const row = document.createElement("li");
  const button = document.createElement("button");
  button.textContent = label;
  button.disabled = !onclick;
  if (onclick) button.onclick = onclick;
  row.append(button);
  entries.append(row);
}

function renderTerrain(path) {
  try {
    const terrain = new Terrain(activeArchive, activeData, path);
    const cells = terrain.cells();
    canvas.width = terrain.width;
    canvas.height = terrain.height;
    const image = context.createImageData(terrain.width, terrain.height);
    for (let y = 0; y < terrain.height; y++) {
      for (let x = 0; x < terrain.width; x++) {
        const source = (y * terrain.width + x) * 3;
        const target = ((terrain.height - y - 1) * terrain.width + x) * 4;
        const color = cells[source + 1] ? "#397c9e"
          : cells[source + 2] ? "#674b78"
          : colors[cells[source] % colors.length];
        const rgb = color.match(/\w\w/g).map((value) => parseInt(value, 16));
        image.data.set([...rgb, 255], target);
      }
    }
    context.putImageData(image, 0, 0);
    setStatus(`${path}：${terrain.width} × ${terrain.height}，地形集 ${terrain.tileset}`);
    terrain.free();
  } catch (error) {
    setStatus(`地形加载失败 ${path}：${error}`);
  }
}

upload.onchange = async () => {
  for (const file of upload.files) {
    const handle = await root.getFileHandle(file.name, { create: true });
    const writable = await handle.createWritable();
    await writable.write(file);
    await writable.close();
  }
  upload.value = "";
  await refreshArchives();
};

await refreshArchives();

