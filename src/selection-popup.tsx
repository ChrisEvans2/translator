import React from "react";
import ReactDOM from "react-dom/client";
import { SelectionPopup } from "./components/SelectionPopup";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SelectionPopup />
  </React.StrictMode>,
);
