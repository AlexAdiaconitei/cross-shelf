//! Native read of the page text with WebView2 `ExecuteScript` (spec 5.2: no bridge for the page,
//! cookies and content are read through the webview's own API from Rust).

use tauri::WebviewWindow;
use webview2_com::ExecuteScriptCompletedHandler;
use windows::core::HSTRING;

/// Returns `document.body.innerText` of the window's current page.
pub async fn body_text(win: &WebviewWindow) -> Result<String, String> {
    eval_string(win, "document.body ? document.body.innerText : ''").await
}

/// Evaluates `js` in the page and returns its string result. The script must evaluate to a string.
pub async fn eval_string(win: &WebviewWindow, js: &str) -> Result<String, String> {
    let js = js.to_owned();
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, String>>();
    let tx = std::sync::Mutex::new(Some(tx));
    win.with_webview(move |wv| unsafe {
        let send = move |r: Result<String, String>| {
            if let Some(t) = tx.lock().ok().and_then(|mut g| g.take()) {
                let _ = t.send(r);
            }
        };
        let core = match wv.controller().CoreWebView2() {
            Ok(c) => c,
            Err(e) => return send(Err(e.to_string())),
        };
        let send = std::sync::Arc::new(std::sync::Mutex::new(Some(send)));
        let on_done = send.clone();
        let handler = ExecuteScriptCompletedHandler::create(Box::new(move |err, result| {
            if let Some(s) = on_done.lock().ok().and_then(|mut g| g.take()) {
                s(match err {
                    Ok(()) => Ok(result),
                    Err(e) => Err(e.to_string()),
                });
            }
            Ok(())
        }));
        if let Err(e) = core.ExecuteScript(&HSTRING::from(js.as_str()), &handler)
            && let Some(s) = send.lock().ok().and_then(|mut g| g.take())
        {
            s(Err(e.to_string()));
        }
    })
    .map_err(|e| e.to_string())?;
    let raw = rx.await.map_err(|_| "webview closed".to_string())??;
    // ExecuteScript returns the JSON encoding of the value: a quoted, escaped string.
    serde_json::from_str::<String>(&raw).map_err(|e| e.to_string())
}
