function notifyActiveWindow(client) {
    callDBus(
        "com.k0kubun.Xremap",
        "/com/k0kubun/Xremap",
        "com.k0kubun.Xremap",
        "NotifyActiveWindow",
        "caption" in client ? client.caption : "",
        "resourceClass" in client ? client.resourceClass : "",
        "resourceName" in client ? client.resourceName : ""
    );
}

if (workspace.windowList) {
    // kde 6
    workspace.windowActivated.connect(notifyActiveWindow);
} else {
    // kde 5
    workspace.clientActivated.connect(notifyActiveWindow);
}
