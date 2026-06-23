import "solid-sonner/styles.css";
import "./App.css";
import { MainApp } from "./MainApp";
import { EyeRestOverlay } from "./overlays/EyeRestOverlay";
import { LanguageIndicatorOverlay } from "./overlays/LanguageIndicatorOverlay";
import { ToastOverlay } from "./overlays/ToastOverlay";

function App() {
  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");

  if (view === "toast") {
    document.documentElement.dataset.view = "toast";
    return <ToastOverlay />;
  }

  if (view === "eye-rest") {
    document.documentElement.dataset.view = "eye-rest";
    return <EyeRestOverlay />;
  }

  if (view === "language-indicator") {
    document.documentElement.dataset.view = "language-indicator";
    return <LanguageIndicatorOverlay />;
  }

  delete document.documentElement.dataset.view;
  return <MainApp />;
}

export default App;
