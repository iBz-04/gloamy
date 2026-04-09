// JXA helper library template (inline or concatenate)
// Keep this small and copyable into scripts.

function waitUntil(fn, timeoutSeconds = 5, stepSeconds = 0.2) {
  const deadline = Date.now() + timeoutSeconds * 1000;
  while (Date.now() < deadline) {
    try { if (fn()) return true; } catch (_) {}
    delay(stepSeconds);
  }
  throw new Error("Timeout in waitUntil");
}

function jsonOut(data) {
  return JSON.stringify(data);
}

function shQuote(s) {
  return "'" + String(s).replace(/'/g, "'\\''") + "'";
}

function retryWithBackoff(fn, maxRetries = 3, initialDelayMs = 100) {
  let lastError;
  for (let i = 0; i < maxRetries; i++) {
    try { return fn(); } catch (e) {
      lastError = e;
      delay((initialDelayMs * Math.pow(2, i)) / 1000);
    }
  }
  throw lastError;
}

function ensureActive(appName, timeoutSeconds = 10) {
  const app = Application(appName);
  app.activate();
  const ok = waitUntil(() => {
    try { return Application("System Events").processes.byName(appName).frontmost(); }
    catch (_) { return false; }
  }, timeoutSeconds);
  if (!ok) throw new Error("Failed to activate " + appName);
  return app;
}

