import React from "react";
import { createRoot } from "react-dom/client";
import App from "./Reshape";
import "./reshape.css";
import "./brand.css";
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
