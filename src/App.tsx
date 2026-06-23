import { Navigate, Route, Router } from "@solidjs/router";
import "solid-sonner/styles.css";
import "./App.css";
import {
  CapsLockLanguageSwitchRoute,
  CurrentLanguageIndicatorRoute,
  EyeRestRoute,
  MainApp,
  PersistentNotificationsRoute,
  SettingsRoute,
} from "./MainApp";
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
  return (
    <Router root={MainApp}>
      <Route path="/" component={() => <Navigate href="/notifications" />} />
      <Route path="/notifications" component={PersistentNotificationsRoute} />
      <Route path="/eye-rest" component={EyeRestRoute} />
      <Route path="/caps-lock-language-switch" component={CapsLockLanguageSwitchRoute} />
      <Route path="/current-language-indicator" component={CurrentLanguageIndicatorRoute} />
      <Route path="/settings" component={SettingsRoute} />
    </Router>
  );
}

export default App;
