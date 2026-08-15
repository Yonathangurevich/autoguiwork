import { useEffect, useState } from "react";
import { saveImage, readImage } from "../../../services";
import "./FindImageEditor.css";

// The value shape of a FindImageLoop action.
interface FindImage {
  image_path: string;
  waited_ms: number;
  store_as: string;
}

interface Props {
  value: FindImage;
  appName: string;
  stepIndex: number;
  onChange: (v: FindImage) => void;
}

export function FindImageEditor({ value, appName, stepIndex, onChange }: Props) {
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  // A data URL to preview the template image. Set on paste (immediately) and
  // loaded from disk on mount, so it survives navigating away / reopening.
  const [preview, setPreview] = useState<string | null>(null);

  // When the step already has a saved image, load it from disk as a data URL
  // so the preview shows even after switching steps or reopening the app.
  useEffect(() => {
    let cancelled = false;
    if (value.image_path) {
      readImage(appName, value.image_path)
        .then((dataUrl) => {
          if (!cancelled) setPreview(dataUrl);
        })
        .catch(() => {
          /* leave preview null; the path text still shows */
        });
    }
    return () => {
      cancelled = true;
    };
    // Re-run if the saved path changes (e.g. a fresh paste overwrote it).
  }, [appName, value.image_path]);

  // Read an image Blob (from the clipboard) → bytes → save_image → set path.
  async function handleBlob(blob: Blob) {
    setError(null);
    setSaving(true);
    try {
      const buf = new Uint8Array(await blob.arrayBuffer());
      const relPath = await saveImage(appName, stepIndex, buf);
      onChange({ ...value, image_path: relPath });
      setPreview(URL.createObjectURL(blob));
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  // Paste handler: pull the first image off the clipboard.
  function onPaste(e: React.ClipboardEvent) {
    const item = Array.from(e.clipboardData.items).find((i) =>
      i.type.startsWith("image/")
    );
    if (!item) {
      setError("no image in the clipboard — copy a screenshot first");
      return;
    }
    const blob = item.getAsFile();
    if (blob) handleBlob(blob);
  }

  return (
    <div className="findimage">
      <div
        className="paste-zone"
        tabIndex={0}
        onPaste={onPaste}
        title="click here, then paste (Ctrl+V) a screenshot"
      >
        {saving ? (
          <span>saving…</span>
        ) : preview ? (
          <img className="paste-preview" src={preview} alt="template" />
        ) : value.image_path ? (
          <span className="paste-has">image set: {value.image_path}</span>
        ) : (
          <span className="paste-empty">
            click here &amp; paste a screenshot (Ctrl+V)
          </span>
        )}
      </div>

      {error && <p className="findimage-error">{error}</p>}

      <label className="field">
        <span>save position as (variable name)</span>
        <input
          value={value.store_as}
          onChange={(e) => onChange({ ...value, store_as: e.target.value })}
          placeholder="found"
        />
      </label>

      <label className="field">
        <span>timeout (ms)</span>
        <input
          type="number"
          value={value.waited_ms}
          onChange={(e) =>
            onChange({ ...value, waited_ms: Number(e.target.value) })
          }
        />
      </label>
    </div>
  );
}
