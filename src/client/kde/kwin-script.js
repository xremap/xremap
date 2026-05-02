function notifyActiveWindow(client) {
    if (!client) {
        // Ignore when there is no active window.
        return;
    }
    callDBus(
        "com.k0kubun.Xremap",
        "/com/k0kubun/Xremap",
        "com.k0kubun.Xremap",
        "NotifyActiveWindow2",
        "caption" in client ? client.caption : "",
        "resourceClass" in client ? client.resourceClass : "",
    );
}

// Keep track of the currently focused window
var currentActiveWindow = null;

// The function to call when the title changes
function onCaptionChanged() {
    if (currentActiveWindow) {
        notifyActiveWindow(currentActiveWindow);
    }
}

function onWindowActivated(window) {
    // 1. DISCONNECT from the previous window to prevent background spam
    if (currentActiveWindow && currentActiveWindow.captionChanged) {
        try {
            currentActiveWindow.captionChanged.disconnect(onCaptionChanged);
        } catch (err) {
            // Safe fallback: Ignore errors if the previous window was closed/destroyed
        }
    }

    // 2. Update the tracker to the newly focused window
    currentActiveWindow = window;

    // 3. CONNECT the title listener only to this active window
    if (currentActiveWindow && currentActiveWindow.captionChanged) {
        currentActiveWindow.captionChanged.connect(onCaptionChanged);
    }

    // 4. Notify xremap immediately about the new window
    notifyActiveWindow(currentActiveWindow);
}

// Bind activation events based on KDE version
if (workspace.windowList) {
    // KDE 6
    workspace.windowActivated.connect(onWindowActivated);
} else {
    // KDE 5
    workspace.clientActivated.connect(onWindowActivated);
}
