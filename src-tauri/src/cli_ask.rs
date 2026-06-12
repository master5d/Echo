use anyhow::{Context, Result};

pub fn run_ask(
    question: &str,
    options: Option<&str>,
    timeout_s: u64,
    speak: bool,
    port: u16,
) -> Result<i32> {
    let token_path = crate::portable::data_dir()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| dirs_path().expect("app data dir"))
        .join(crate::agent_bridge::token::TOKEN_FILE);
    let token = std::fs::read_to_string(&token_path)
        .with_context(|| {
            format!(
                "token not found at {} — is Echo running?",
                token_path.display()
            )
        })?
        .trim()
        .to_string();
    let opts: Vec<String> = options
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
        .unwrap_or_default();
    let body = serde_json::json!({
        "question": question,
        "kind": if opts.is_empty() { "text" } else { "choice" },
        "options": opts,
        "timeout_s": timeout_s,
        "speak": speak,
        "source": "cli",
    });
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let resp: serde_json::Value = rt.block_on(async {
        reqwest::Client::new()
            .post(format!("http://127.0.0.1:{port}/v1/ask"))
            .bearer_auth(&token)
            .json(&body)
            .timeout(std::time::Duration::from_secs(timeout_s + 30))
            .send()
            .await?
            .json()
            .await
            .map_err(anyhow::Error::from)
    })?;
    match resp["status"].as_str() {
        Some("answered") => {
            println!("{}", resp["answer"].as_str().unwrap_or_default());
            Ok(0)
        }
        Some("timeout") => Ok(2),
        _ => Ok(3),
    }
}

/// %APPDATA%\com.sovern.echo equivalent without an AppHandle.
fn dirs_path() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|a| std::path::PathBuf::from(a).join("com.sovern.echo"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| {
            std::path::PathBuf::from(h).join("Library/Application Support/com.sovern.echo")
        })
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        std::env::var_os("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".local/share/com.sovern.echo"))
    }
}
