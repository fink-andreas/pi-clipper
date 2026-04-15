pub mod context;
pub mod dedupe;
pub mod logger;
pub mod observer;
pub mod sanitizer;
pub mod watcher;
pub mod writer;

use anyhow::Result;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::app_state::AppState;
use crate::context::detect_active_context;
use crate::pipeline::context::ContextDecision;
use crate::pipeline::logger::{ContextLog, EventLogger, EventLog};
use crate::pipeline::observer::ClipboardObserver;
use crate::pipeline::sanitizer::{sanitize, SanitizeResult};
use crate::pipeline::writer::ClipboardWriter;
use crate::rules::builtins::default_rules;
use sha2::{Digest, Sha256};

pub async fn start_background_workers(app: AppHandle, state: AppState) -> Result<()> {
    let log_dir = state
        .log_dir()
        .ok_or_else(|| anyhow::anyhow!("log dir unavailable"))?;

    let mut logger = EventLogger::new(log_dir, 7)?;
    let mut writer = ClipboardWriter::new(10, 500)?;
    let mut observer = ClipboardObserver::new(200);

    tokio::spawn(async move {
        tracing::info!("pipeline workers started");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if app.state::<AppState>().monitoring_enabled() {
                let _ = process_event(&mut observer, &mut logger, &mut writer).await;
            }
        }
    });

    tracing::info!("pipeline workers initialized");
    Ok(())
}

async fn process_event(
    observer: &mut ClipboardObserver,
    logger: &mut EventLogger,
    writer: &mut ClipboardWriter,
) -> Result<()> {
    let Some(change) = observer.next_change().await else {
        return Ok(());
    };

    let context = detect_active_context().unwrap_or_else(|_| ContextDecision::unknown());
    if !context.is_terminal {
        return Ok(());
    }

    sanitize_and_write(&change, &context, logger, writer)
}

fn sanitize_and_write(
    change: &crate::pipeline::watcher::ClipboardChanged,
    context: &ContextDecision,
    logger: &mut EventLogger,
    writer: &mut ClipboardWriter,
) -> Result<()> {
    let start = std::time::Instant::now();
    let rules = default_rules();
    let SanitizeResult { output, changed, actions } = sanitize(&change.text, &rules);

    let output_hash = hash_text(&output);

    if changed {
        writer.write_clipboard(&output)?;
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    let event_log = EventLog {
        timestamp: change.timestamp,
        event_id: uuid::Uuid::new_v4().to_string(),
        context: Some(ContextLog {
            is_terminal: context.is_terminal,
            confidence: context.confidence,
            process_name: context.process_name.clone(),
            window_title: context.window_title.clone(),
        }),
        input_hash: Some(change.hash.clone()),
        output_hash: Some(output_hash),
        input_preview: Some(EventLogger::truncate_preview(&change.text)),
        output_preview: if changed {
            Some(EventLogger::truncate_preview(&output))
        } else {
            None
        },
        changed,
        actions: actions.into_iter().map(|a| a.rule_id).collect(),
        duration_ms,
        status: "ok".to_string(),
        error: None,
    };

    logger.log(&event_log)?;
    Ok(())
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}