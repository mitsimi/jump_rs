document.body.addEventListener("htmx:beforeSwap", (event) => {
  if (event.detail.xhr.status >= 400 && event.detail.xhr.status < 500) {
    event.detail.shouldSwap = true;
    event.detail.isError = false;
  }
});

let modalTrigger = null;

document.addEventListener("click", (event) => {
  const trigger = event.target instanceof Element
    ? event.target.closest('[hx-target="#modal-root"]')
    : null;
  if (trigger) modalTrigger = trigger;
});

document.body.addEventListener("htmx:afterSwap", () => {
  document.querySelectorAll(".toast__toast").forEach((toast) => {
    if (toast.dataset.dismissScheduled) return;
    toast.dataset.dismissScheduled = "true";
    setTimeout(() => {
      toast.classList.remove("toast__show");
      setTimeout(() => toast.remove(), 450);
    }, 3800);
  });
});

document.body.addEventListener("htmx:afterSettle", () => {
  const dialog = document.querySelector("#modal-root dialog");
  if (dialog && !dialog.open) {
    dialog.addEventListener("cancel", jumpCancelModal, { once: true });
    dialog.showModal();
  } else if (!dialog) jumpRestoreModalTrigger();
});

function jumpRestoreModalTrigger() {
  if (modalTrigger?.isConnected) modalTrigger.focus();
  modalTrigger = null;
}

function jumpCloseModal() {
  const dialog = document.querySelector("#modal-root dialog");
  if (dialog?.open) dialog.close();
  jumpHandleModalClose();
}

function jumpCancelModal(event) {
  event.preventDefault();
  jumpCloseModal();
}

function jumpCloseModalOnBackdrop(event) {
  const dialog = event.currentTarget;
  const bounds = dialog.getBoundingClientRect();
  const outside = event.clientX < bounds.left
    || event.clientX > bounds.right
    || event.clientY < bounds.top
    || event.clientY > bounds.bottom;
  if (outside) jumpCloseModal();
}

function jumpHandleModalClose() {
  const modalRoot = document.getElementById("modal-root");
  if (modalRoot) modalRoot.replaceChildren();
  jumpRestoreModalTrigger();
}

function jumpShowTransferTab(tab) {
  const exportTab = document.getElementById("transfer-export-tab");
  const importTab = document.getElementById("transfer-import-tab");
  const exportPanel = document.getElementById("transfer-export-panel");
  const importPanel = document.getElementById("transfer-import-panel");
  const showImport = tab === "import";

  exportTab?.classList.toggle("transfer__tab--active", !showImport);
  importTab?.classList.toggle("transfer__tab--active", showImport);
  if (exportPanel) exportPanel.hidden = showImport;
  if (importPanel) importPanel.hidden = !showImport;
}

function jumpImportDragOver(event) {
  event.preventDefault();
  document
    .getElementById("transfer-drop-zone")
    ?.classList.add("transfer__drop-zone--active");
}

function jumpImportDragLeave(event) {
  event.preventDefault();
  document
    .getElementById("transfer-drop-zone")
    ?.classList.remove("transfer__drop-zone--active");
}

function jumpImportDrop(event) {
  event.preventDefault();
  document
    .getElementById("transfer-drop-zone")
    ?.classList.remove("transfer__drop-zone--active");
  jumpLoadImportFile(event.dataTransfer.files[0]);
}

async function jumpLoadImportFile(file) {
  if (!file) return;
  if (!file.name.endsWith(".json")) {
    alert("Only JSON files are supported");
    return;
  }

  const payload = document.getElementById("import-payload");
  const fileName = document.getElementById("transfer-file-name");
  if (payload) payload.value = await file.text();
  if (fileName) fileName.textContent = file.name;
}
