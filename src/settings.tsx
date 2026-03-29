import React from "react";
import ReactDOM from "react-dom/client";
import { SettingsModal } from "./components/SettingsModal";
import { ThemeProvider } from "./contexts/ThemeContext";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <SettingsModal />
    </ThemeProvider>
  </React.StrictMode>,
);
