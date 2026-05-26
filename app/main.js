const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const statusText = document.querySelector("#statusText");
const statusPill = document.querySelector("#statusPill");
const readBtn = document.querySelector("#readBtn");
const stopBtn = document.querySelector("#stopBtn");
const testBtn = document.querySelector("#testBtn");
const voiceSelect = document.querySelector("#voiceSelect");
const rateInput = document.querySelector("#rateInput");
const rateOutput = document.querySelector("#rateOutput");
const volumeInput = document.querySelector("#volumeInput");
const volumeOutput = document.querySelector("#volumeOutput");
const fallbackInput = document.querySelector("#fallbackInput");
const startMinimizedInput = document.querySelector("#startMinimizedInput");
const hotkeyText = document.querySelector("#hotkeyText");

let settings = null;
let saveTimer = null;

function setStatus(status) {
  statusText.textContent = status.message || "";
  statusPill.textContent = status.state || "Idle";
  statusPill.classList.toggle("reading", ["Reading", "FetchingVoice"].includes(status.state));
  statusPill.classList.toggle("error", status.state === "Error");
}

function refreshOutputs() {
  rateOutput.value = `${rateInput.value}%`;
  volumeOutput.value = `${volumeInput.value}%`;
}

function scheduleSave() {
  if (!settings) return;
  window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(async () => {
    settings.voice_id = voiceSelect.value;
    settings.rate = Number(rateInput.value);
    settings.volume = Number(volumeInput.value);
    settings.enable_local_fallback = fallbackInput.checked;
    settings.start_minimized = startMinimizedInput.checked;
    await invoke("save_settings", { settings });
  }, 180);
}

async function call(action) {
  try {
    setStatus({ state: "Working", message: "One moment..." });
    const status = await invoke(action);
    if (status) setStatus(status);
  } catch (error) {
    setStatus({ state: "Error", message: String(error) });
  }
}

async function init() {
  settings = await invoke("get_settings");
  hotkeyText.textContent = settings.hotkey;
  rateInput.value = settings.rate;
  volumeInput.value = settings.volume;
  fallbackInput.checked = settings.enable_local_fallback;
  startMinimizedInput.checked = settings.start_minimized;
  refreshOutputs();

  try {
    const voices = await invoke("list_voices");
    voiceSelect.innerHTML = "";
    for (const voice of voices) {
      const option = document.createElement("option");
      option.value = voice.id;
      option.textContent = voice.label;
      voiceSelect.append(option);
    }
    voiceSelect.value = settings.voice_id;
  } catch (error) {
    const option = document.createElement("option");
    option.value = settings.voice_id;
    option.textContent = `${settings.voice_id} (voice list unavailable)`;
    voiceSelect.append(option);
    voiceSelect.value = settings.voice_id;
    setStatus({ state: "Error", message: String(error) });
  }

  setStatus({ state: "Idle", message: "Highlight text anywhere, then press Ctrl+Alt+R." });
}

readBtn.addEventListener("click", () => call("read_selection"));
stopBtn.addEventListener("click", () => call("stop_reading"));
testBtn.addEventListener("click", () => call("test_voice"));

for (const input of [voiceSelect, rateInput, volumeInput, fallbackInput, startMinimizedInput]) {
  input.addEventListener("input", () => {
    refreshOutputs();
    scheduleSave();
  });
}

await listen("status-changed", (event) => setStatus(event.payload));
await init();
