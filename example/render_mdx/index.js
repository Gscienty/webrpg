import init, { list_mpq_files, MdxRenderer } from "./pkg/webrpg_render_mdx_example.js";

const upload = document.querySelector("#upload");
const archives = document.querySelector("#archives");
const breadcrumbs = document.querySelector("#breadcrumbs");
const entries = document.querySelector("#entries");
const status = document.querySelector("#status");
const canvas = document.querySelector("#canvas");

await init();

const root = await navigator.storage.getDirectory();
let activeArchive = null;
let activeData = null;
let filePaths = [];
let currentPath = [];
let mdxRenderer = null;
let renderStarted = performance.now();

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
    setStatus(`${name}：${filePaths.length} 个可枚举文件`);
  } catch (error) {
    closeArchive();
    setStatus(`无法打开 ${name}：${error}\n该 MPQ 可能没有 (listfile)。`);
  }
}

function closeArchive() {
  activeArchive = null;
  activeData = null;
  filePaths = [];
  currentPath = [];
  breadcrumbs.replaceChildren();
  entries.replaceChildren();
}

function renderDirectory() {
  breadcrumbs.replaceChildren();
  const parts = ["根目录", ...currentPath];
  parts.forEach((part, index) => {
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
    if (!rest) continue;
    const slash = rest.indexOf("\\");
    if (slash >= 0) dirs.add(rest.slice(0, slash));
    else files.push({ name: rest, path });
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
    addEntry(`${/\.mdx$/i.test(file.name) ? "🎞️" : "📄"} ${file.name}`,
      /\.mdx$/i.test(file.name) ? () => renderMdx(file.path) : null);
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

function renderMdx(path) {
  try {
    mdxRenderer?.free();
    mdxRenderer = new MdxRenderer(canvas, activeArchive, activeData, path);
    renderStarted = performance.now();
    setStatus(`正在渲染 ${path}`);
  } catch (error) {
    mdxRenderer = null;
    setStatus(`MDX 加载失败：${error}`);
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

function animate(now) {
  if (mdxRenderer) {
    try {
      mdxRenderer.render(Math.floor(now - renderStarted));
    } catch (error) {
      mdxRenderer.free();
      mdxRenderer = null;
      setStatus(`MDX 渲染失败：${error}`);
    }
  }
  requestAnimationFrame(animate);
}

await refreshArchives();
requestAnimationFrame(animate);
