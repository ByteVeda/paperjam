import { useEffect, useState, useCallback } from "react";
import {
  HashRouter,
  Routes,
  Route,
  NavLink,
  Navigate,
} from "react-router-dom";
import { FileText, RefreshCw, GitBranch, Sun, Moon } from "lucide-react";
import { StoreProvider } from "./lib/store";
import { initWasm, isWasmReady } from "./lib/wasm-bridge";
import DocumentsPage from "./pages/DocumentsPage";
import ConvertPage from "./pages/ConvertPage";
import PipelinePage from "./pages/PipelinePage";

function AppShell() {
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    if (typeof window !== "undefined") {
      const stored = localStorage.getItem("pj-theme");
      if (stored === "dark" || stored === "light") return stored;
      return window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    }
    return "light";
  });

  const [wasmStatus, setWasmStatus] = useState<
    "loading" | "ready" | "error"
  >("loading");

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("pj-theme", theme);
  }, [theme]);

  useEffect(() => {
    if (isWasmReady()) {
      setWasmStatus("ready");
      return;
    }
    initWasm()
      .then(() => setWasmStatus("ready"))
      .catch((err) => {
        console.error("WASM load failed:", err);
        setWasmStatus("error");
      });
  }, []);

  const toggleTheme = useCallback(() => {
    setTheme((t) => (t === "light" ? "dark" : "light"));
  }, []);

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-brand">paperjam studio</div>
        <nav className="sidebar-nav">
          <NavLink
            to="/documents"
            className={({ isActive }) =>
              `nav-link${isActive ? " active" : ""}`
            }
          >
            <FileText size={18} />
            Documents
          </NavLink>
          <NavLink
            to="/convert"
            className={({ isActive }) =>
              `nav-link${isActive ? " active" : ""}`
            }
          >
            <RefreshCw size={18} />
            Convert
          </NavLink>
          <NavLink
            to="/pipeline"
            className={({ isActive }) =>
              `nav-link${isActive ? " active" : ""}`
            }
          >
            <GitBranch size={18} />
            Pipeline
          </NavLink>
        </nav>
      </aside>

      <div className="main-area">
        <header className="header">
          <span className="header-title">
            {wasmStatus === "loading" && "Loading WASM..."}
            {wasmStatus === "error" && "WASM failed to load"}
            {wasmStatus === "ready" && "Ready"}
          </span>
          <button
            className="theme-toggle"
            onClick={toggleTheme}
            title={`Switch to ${theme === "light" ? "dark" : "light"} theme`}
          >
            {theme === "light" ? <Moon size={16} /> : <Sun size={16} />}
          </button>
        </header>

        <main className="main-content">
          <Routes>
            <Route path="/" element={<Navigate to="/documents" replace />} />
            <Route
              path="/documents"
              element={<DocumentsPage wasmReady={wasmStatus === "ready"} />}
            />
            <Route
              path="/convert"
              element={<ConvertPage wasmReady={wasmStatus === "ready"} />}
            />
            <Route
              path="/pipeline"
              element={<PipelinePage wasmReady={wasmStatus === "ready"} />}
            />
          </Routes>
        </main>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <HashRouter>
      <StoreProvider>
        <AppShell />
      </StoreProvider>
    </HashRouter>
  );
}
