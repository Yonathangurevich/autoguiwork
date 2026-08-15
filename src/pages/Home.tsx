import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { listAutomations, createAutomation } from "../services";
import "./Home.css";

export function Home() {
  const [automations, setAutomations] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // "New automation" flow: reveal a name input, then create + navigate.
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const navigate = useNavigate();

  async function load() {
    setLoading(true);
    setError(null);
    try {
      setAutomations(await listAutomations());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function create() {
    setError(null);
    try {
      const app = await createAutomation(newName);
      // Jump straight into the new automation's dashboard.
      navigate(`/automation/${encodeURIComponent(app.name)}`);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="home">
      <div className="home-head">
        <div>
          <h1>your automations</h1>
          <p className="subtitle">build one, then run it or expose it as an API</p>
        </div>
        {!creating && (
          <button className="new-btn" onClick={() => setCreating(true)}>
            + New automation
          </button>
        )}
      </div>

      {creating && (
        <form
          className="new-form"
          onSubmit={(e) => {
            e.preventDefault();
            create();
          }}
        >
          <input
            autoFocus
            className="new-input"
            placeholder="automation name…"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
          />
          <button type="submit" className="new-create">
            create
          </button>
          <button
            type="button"
            className="new-cancel"
            onClick={() => {
              setCreating(false);
              setNewName("");
              setError(null);
            }}
          >
            cancel
          </button>
        </form>
      )}

      {error && <p className="error">{error}</p>}
      {loading && <p>loading…</p>}

      {!loading && (
        <div className="cards">
          {automations.map((name) => (
            <button
              className="card"
              key={name}
              onClick={() => navigate(`/automation/${encodeURIComponent(name)}`)}
            >
              <span className="card-name">{name}</span>
            </button>
          ))}

          {automations.length === 0 && !creating && (
            <p className="empty">no automations yet — click “+ New automation”.</p>
          )}
        </div>
      )}
    </div>
  );
}
