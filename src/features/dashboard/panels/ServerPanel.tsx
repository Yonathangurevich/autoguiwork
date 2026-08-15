import { useEffect, useState } from "react";
import { exposeAsServer, unexposeServer, buildCurl } from "../../../services";
import type { App, InputSpec, Trigger } from "../../../models";
import "./ServerPanel.css";

// Derive the input list an automation already declares (from a Server trigger).
function existingInputs(trigger: Trigger): InputSpec[] {
  return typeof trigger === "object" && "Server" in trigger
    ? trigger.Server.inputs
    : [];
}

function isExposed(trigger: Trigger): boolean {
  return typeof trigger === "object" && "Server" in trigger;
}

interface Props {
  app: App;
  onChanged: () => void; // reload the app after expose/unexpose
}

export function ServerPanel({ app, onChanged }: Props) {
  const [inputs, setInputs] = useState<InputSpec[]>(existingInputs(app.trigger));
  const [url, setUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    setInputs(existingInputs(app.trigger));
  }, [app]);

  function addInput() {
    setInputs([...inputs, { name: "", required: true }]);
  }
  function updateInput(i: number, patch: Partial<InputSpec>) {
    setInputs(inputs.map((inp, idx) => (idx === i ? { ...inp, ...patch } : inp)));
  }
  function removeInput(i: number) {
    setInputs(inputs.filter((_, idx) => idx !== i));
  }

  async function expose() {
    setBusy(true);
    setError(null);
    try {
      const clean = inputs.filter((i) => i.name.trim() !== "");
      const endpointUrl = await exposeAsServer(app.name, clean);
      setUrl(endpointUrl);
      onChanged();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function unexpose() {
    setBusy(true);
    setError(null);
    try {
      await unexposeServer(app.name);
      setUrl(null);
      onChanged();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  const exposed = isExposed(app.trigger);

  return (
    <div className="server-panel">
      <div className="server-head">
        <h3>API endpoint</h3>
        <span className={`server-dot ${exposed ? "on" : "off"}`}>
          {exposed ? "production" : "not exposed"}
        </span>
      </div>

      <p className="server-hint">
        declare the inputs callers must send, then expose this automation as a
        POST endpoint (call it from n8n / curl).
      </p>

      <div className="inputs">
        {inputs.map((inp, i) => (
          <div className="input-row" key={i}>
            <input
              className="input-name"
              placeholder="input name (e.g. invoice)"
              value={inp.name}
              onChange={(e) => updateInput(i, { name: e.target.value })}
            />
            <label className="input-req">
              <input
                type="checkbox"
                checked={inp.required}
                onChange={(e) => updateInput(i, { required: e.target.checked })}
              />
              required
            </label>
            <button className="input-del" onClick={() => removeInput(i)}>
              ×
            </button>
          </div>
        ))}
        <button className="add-input" onClick={addInput}>
          + add input
        </button>
      </div>

      <div className="server-actions">
        <button className="expose-btn" onClick={expose} disabled={busy}>
          {exposed ? "update endpoint" : "expose as server"}
        </button>
        {exposed && (
          <button className="unexpose-btn" onClick={unexpose} disabled={busy}>
            unexpose
          </button>
        )}
      </div>

      {error && <p className="server-error">{error}</p>}

      {url && (
        <>
          <div className="server-url">
            <span className="url-label">POST</span>
            <code>{url}</code>
          </div>

          <div className="curl-block">
            <div className="curl-head">
              <span>curl</span>
              <button
                className="copy-btn"
                title="copy curl to clipboard"
                onClick={async () => {
                  await navigator.clipboard.writeText(buildCurl(url, inputs));
                  setCopied(true);
                  setTimeout(() => setCopied(false), 1500);
                }}
              >
                {copied ? "✓ copied" : "⧉ copy"}
              </button>
            </div>
            <pre className="curl-code">{buildCurl(url, inputs)}</pre>
          </div>
        </>
      )}
    </div>
  );
}
