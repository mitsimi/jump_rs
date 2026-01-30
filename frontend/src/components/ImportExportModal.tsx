import { useState, useRef } from "react";
import { Modal, ModalContent } from "./Modal";
import styles from "./ImportExportModal.module.css";
import { useToast } from "../hooks/useToast";
import { useExportDevices } from "../hooks/useExportDevices";
import { useImportDevices } from "../hooks/useImportDevices";
import type { ImportDevice } from "../types/device";

export function ImportExportModal({
  isOpen,
  onClose,
}: {
  isOpen: boolean;
  onClose: () => void;
}) {
  const [activeTab, setActiveTab] = useState<"export" | "import">("export");
  const [jsonInput, setJsonInput] = useState("");
  const [isDragging, setIsDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { showToast } = useToast();
  const exportQuery = useExportDevices();
  const importMutation = useImportDevices();

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const file = e.dataTransfer.files?.[0];
    if (file && file.name.endsWith(".json")) {
      handleFileImport(file);
    } else if (file) {
      showToast("Only JSON files are supported", "error");
    }
  };

  const handleExport = async () => {
    const result = await exportQuery.refetch();
    if (result.data) {
      const blob = new Blob([JSON.stringify(result.data, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `jump-devices-${new Date().toISOString().split("T")[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
      showToast("Device list exported successfully", "success");
      onClose();
    } else if (result.error) {
      showToast("Failed to export devices", "error");
    }
  };

  const handleFileImport = async (file: File) => {
    try {
      const text = await file.text();
      const devices: ImportDevice[] = JSON.parse(text);

      importMutation.mutate(devices, {
        onSuccess: (result) => {
          showToast(
            `Successfully imported ${result.length} device(s)`,
            "success",
          );
          onClose();
        },
        onError: (error) => {
          showToast(
            error instanceof Error
              ? error.message
              : "Failed to import devices",
            "error",
          );
        },
      });
    } catch (e) {
      showToast(
        e instanceof Error ? e.message : "Invalid JSON format",
        "error",
      );
    }
  };

  const handlePasteImport = () => {
    if (!jsonInput.trim()) return;

    try {
      const devices: ImportDevice[] = JSON.parse(jsonInput);

      importMutation.mutate(devices, {
        onSuccess: (result) => {
          showToast(
            `Successfully imported ${result.length} device(s)`,
            "success",
          );
          onClose();
        },
        onError: (error) => {
          showToast(
            error instanceof Error
              ? error.message
              : "Failed to import devices",
            "error",
          );
        },
      });
    } catch (e) {
      showToast(
        e instanceof Error ? e.message : "Invalid JSON format",
        "error",
      );
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="DATA TRANSFER">
      <ModalContent className={styles.modalBody}>
        <div className={styles.tabs}>
          <button
            className={`${styles.tab} ${activeTab === "export" ? styles.activeTab : ""}`}
            onClick={() => setActiveTab("export")}
          >
            EXPORT
          </button>
          <button
            className={`${styles.tab} ${activeTab === "import" ? styles.activeTab : ""}`}
            onClick={() => setActiveTab("import")}
          >
            IMPORT
          </button>
        </div>

        <div className={styles.content}>
          {activeTab === "export" ? (
            <div className={styles.exportSection}>
              <div className={styles.exportIcon}>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M12 15V3"></path>
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                  <path d="m7 10 5 5 5-5"></path>
                </svg>
              </div>
              <p className={styles.description}>
                Export all registered devices to a JSON file. The file contains
                device names, MAC addresses, IP addresses, ports, and
                descriptions.
              </p>
              <button className={styles.actionBtn} onClick={handleExport}>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M12 15V3"></path>
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                  <path d="m7 10 5 5 5-5"></path>
                </svg>
                DOWNLOAD JSON
              </button>
            </div>
          ) : (
            <div className={styles.importSection}>
              <div className={styles.importOptions}>
                <div className={styles.importOption}>
                  <div
                    className={`${styles.dropZone} ${isDragging ? styles.dropZoneActive : ""}`}
                    onDragOver={handleDragOver}
                    onDragLeave={handleDragLeave}
                    onDrop={handleDrop}
                  >
                    <input
                      ref={fileInputRef}
                      type="file"
                      accept=".json"
                      onChange={(e) => {
                        const file = e.target.files?.[0];
                        if (file) handleFileImport(file);
                      }}
                      className={styles.fileInput}
                    />
                    <button
                      className={styles.uploadBtn}
                      onClick={() => fileInputRef.current?.click()}
                    >
                      <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                      >
                        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12" />
                      </svg>
                      Upload JSON File
                    </button>
                    <div className={styles.dropText}>
                      or drag and drop file here
                    </div>
                  </div>
                </div>

                <div className={styles.divider}>
                  <span>OR</span>
                </div>

                <div className={styles.importOption}>
                  <textarea
                    className={styles.jsonInput}
                    placeholder='[{"name":"Device","mac_address":"aa:bb:cc:dd:ee:ff","ip_address":"192.168.1.100","port":9}]'
                    value={jsonInput}
                    onChange={(e) => setJsonInput(e.target.value)}
                    rows={6}
                  />
                  <button
                    className={styles.actionBtn}
                    onClick={handlePasteImport}
                    disabled={!jsonInput.trim() || importMutation.isPending}
                  >
                    {importMutation.isPending
                      ? "Processing..."
                      : "Import devices"}
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>

        <div className={styles.footer}>
          <div className={styles.formatHint}>
            Expected JSON format:
            <code>{`[{"name":"Name","mac_address":"aa:bb:cc:dd:ee:ff","ip_address":"1.2.3.4","port":9,"description":"..."}]`}</code>
          </div>
        </div>
      </ModalContent>
    </Modal>
  );
}
