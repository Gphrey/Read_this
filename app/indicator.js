const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const indicator = document.querySelector("#indicator");
const title = document.querySelector("#title");
const message = document.querySelector("#message");
const stopBtn = document.querySelector("#stopBtn");

function setStatus(status) {
  const state = status?.state || "Idle";
  const text = status?.message || "";

  indicator.classList.toggle("error", state === "Error");
  indicator.classList.toggle("fetching", state === "FetchingVoice");
  indicator.classList.toggle("reading", state === "Reading");

  if (state === "FetchingVoice") {
    title.textContent = "Preparing voice";
    message.textContent = text || "Fetching Edge TTS audio...";
    stopBtn.style.visibility = "hidden";
  } else if (state === "Reading") {
    title.textContent = "Reading aloud";
    message.textContent = text || "Playing selected text.";
    stopBtn.style.visibility = "visible";
  } else if (state === "Error") {
    title.textContent = "Could not read";
    message.textContent = text || "Something went wrong.";
    stopBtn.style.visibility = "hidden";
  } else {
    title.textContent = "Readtis";
    message.textContent = text || "Idle.";
    stopBtn.style.visibility = "hidden";
  }
}

stopBtn.addEventListener("click", async () => {
  try {
    await invoke("stop_reading");
  } catch (error) {
    setStatus({ state: "Error", message: String(error) });
  }
});

await listen("status-changed", (event) => setStatus(event.payload));

try {
  setStatus(await invoke("get_status"));
} catch {
  setStatus({ state: "Idle", message: "Ready." });
}
