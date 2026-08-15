import { Link } from "react-router-dom";
import type { ReactNode } from "react";
import "./Layout.css";

// The shared frame around every page: a top app bar (brand → home) plus the
// page content below. Pages render their own body inside <main>.
export function Layout({ children }: { children: ReactNode }) {
  return (
    <div className="layout">
      <header className="app-bar">
        <Link to="/" className="brand">
          autoguiwork
        </Link>
      </header>
      <main className="layout-main">{children}</main>
    </div>
  );
}
