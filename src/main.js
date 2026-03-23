const invoke = window.__TAURI__?.core?.invoke;
const tauriEvent = window.__TAURI__?.event;

const historyList = document.querySelector("#history-list");
const refreshButton = document.querySelector("#refresh-btn");
const seedButton = document.querySelector("#seed-btn");
const searchInput = document.querySelector("#search-input");
const debugStatus = document.querySelector("#debug-status");

let entries = [];
let filteredEntries = [];
let selectedIndex = 0;

function setDebugStatus(message, tone = "muted") {
  if (!debugStatus) {
    return;
  }

  debugStatus.textContent = message;
  debugStatus.dataset.tone = tone;
}

async function refreshDebugClipboard() {
  if (!invoke) {
    setDebugStatus("调试：Tauri invoke 不可用", "error");
    return;
  }

  try {
    const text = await invoke("read_current_clipboard");
    if (text && text.trim()) {
      setDebugStatus(`调试：当前系统剪贴板 = ${text.trim().slice(0, 80)}`, "ok");
    } else {
      setDebugStatus("调试：当前系统剪贴板为空，或 Rust 没读到文本", "error");
    }
  } catch (error) {
    setDebugStatus(`调试：读取系统剪贴板失败 - ${String(error)}`, "error");
  }
}

async function confirmSelection() {
  const selectedItem = filteredEntries[selectedIndex];
  if (!selectedItem || !invoke) {
    return;
  }

  try {
    await invoke("paste_history_item", { itemId: selectedItem.id });
  } catch (error) {
    setDebugStatus(`调试：paste_history_item 失败 - ${String(error)}`, "error");
  }
}

async function hideWindow() {
  if (!invoke) {
    return;
  }

  try {
    await invoke("hide_window");
  } catch (error) {
    console.error("Failed to hide window", error);
  }
}

function formatTime(value) {
  const date = new Date(value);
  return new Intl.DateTimeFormat("zh-CN", {
    hour: "2-digit",
    minute: "2-digit"
  }).format(date);
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function ensureSelectedIndex(list) {
  if (!list.length) {
    selectedIndex = 0;
    return;
  }

  selectedIndex = Math.min(Math.max(selectedIndex, 0), list.length - 1);
}

function scrollSelectedItemIntoView() {
  const selectedItem = historyList?.querySelector(".history-item.selected");
  selectedItem?.scrollIntoView({ block: "nearest" });
}

function render(list) {
  if (!historyList) {
    return;
  }

  if (!list.length) {
    filteredEntries = [];
    selectedIndex = 0;
    historyList.innerHTML = [
      '<div class="empty-state">',
      '  <strong>还没有历史内容</strong>',
      '  <p>复制文本后会自动出现在这里。</p>',
      '</div>'
    ].join("");
    return;
  }

  filteredEntries = list.slice(0, 20);
  ensureSelectedIndex(filteredEntries);

  historyList.innerHTML = filteredEntries
    .map(
      (item, index) => `
        <article class="history-item ${item.is_pinned ? "pinned" : ""} ${index === selectedIndex ? "selected" : ""}" data-index="${index}">
          <div class="history-main">
            <header>
              <span class="type-pill">${item.kind}</span>
              <span class="meta">${item.source_app ?? "Clipboard"} · ${formatTime(item.created_at)}</span>
            </header>
            <strong>${escapeHtml(item.preview)}</strong>
            <p>${escapeHtml(item.content)}</p>
          </div>
          <button class="pin-toggle" data-id="${item.id}">
            ${item.is_pinned ? "已收" : "收藏"}
          </button>
        </article>
      `
    )
    .join("");

  for (const button of historyList.querySelectorAll(".pin-toggle")) {
    button.addEventListener("click", async () => {
      const { id } = button.dataset;
      if (!id || !invoke) {
        return;
      }

      await invoke("toggle_pin", { itemId: id });
      await loadHistory(searchInput?.value ?? "");
    });
  }

  scrollSelectedItemIntoView();
}

function moveSelection(offset) {
  if (!filteredEntries.length) {
    return;
  }

  selectedIndex = (selectedIndex + offset + filteredEntries.length) % filteredEntries.length;
  render(filteredEntries);
}

async function loadHistory(query = "") {
  if (!invoke) {
    render([]);
    setDebugStatus("调试：Tauri invoke 不可用", "error");
    return;
  }

  try {
    entries = await invoke("get_history", { query: query.trim() || null });
    render(entries);

    if (entries.length) {
      setDebugStatus(`调试：历史条数 ${entries.length}`, "ok");
    } else {
      await refreshDebugClipboard();
    }
  } catch (error) {
    render([]);
    setDebugStatus(`调试：get_history 调用失败 - ${String(error)}`, "error");
  }
}

refreshButton?.addEventListener("click", async () => {
  await loadHistory(searchInput?.value ?? "");
});

seedButton?.addEventListener("click", async () => {
  if (!invoke) {
    return;
  }

  try {
    await invoke("seed_demo_history");
    await loadHistory(searchInput?.value ?? "");
  } catch (error) {
    setDebugStatus(`调试：seed_demo_history 失败 - ${String(error)}`, "error");
  }
});

searchInput?.addEventListener("input", () => {
  selectedIndex = 0;
  void loadHistory(searchInput.value);
});

window.addEventListener("focus", () => {
  void loadHistory(searchInput?.value ?? "");
});

document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "visible") {
    void loadHistory(searchInput?.value ?? "");
  }
});

window.addEventListener("keydown", (event) => {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    moveSelection(1);
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    moveSelection(-1);
    return;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    void confirmSelection();
    return;
  }

  if (event.key !== "Escape") {
    return;
  }

  event.preventDefault();
  void hideWindow();
});

window.setInterval(() => {
  void loadHistory(searchInput?.value ?? "");
}, 1200);

if (tauriEvent?.listen) {
  tauriEvent.listen("overlay-opened", async () => {
    selectedIndex = 0;
    await loadHistory(searchInput?.value ?? "");
    searchInput?.focus();
    searchInput?.select();
  });

  tauriEvent.listen("history-updated", () => {
    void loadHistory(searchInput?.value ?? "");
  });
}

void loadHistory();

