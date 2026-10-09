//! Secrets derived from other secrets are built without revealing anything,
//! computed when revealed, and get the most restrictive policy of their inputs.
//! A secret derived from the environment or a file is never plainly revealed.
//! Revealed values are masked in logs, so they are checked through the data
//! sent by `reveal`, captured in detailed debug events.

use super::common::{self, message};
use melodium_common::descriptor::DataType;
use melodium_common::executive::{
    Log, Secret, SecretOrigin, SecretPolicy, SecretReveal, SecretTransmission, Value,
};
use melodium_engine::debug::{DataContent, Event, EventKind};
use std::collections::HashMap;

const SCRIPT: &str = include_str!("scripts/secret_derivation.mel");

const TOKEN: &str = "to:k/en@1";

fn run() -> (Vec<Log>, Vec<Event>) {
    let token = std::env::temp_dir().join(format!(
        "melodium_secret_derivation_{}_token",
        std::process::id()
    ));
    std::fs::write(&token, TOKEN).unwrap();

    common::run(
        SCRIPT,
        HashMap::from([
            (
                "token".to_string(),
                // Inline, as secrets from files are never plainly revealed.
                Value::Secret(
                    Secret::new(
                        "token".to_string(),
                        DataType::String,
                        SecretPolicy {
                            transmission: SecretTransmission::Local,
                            reveal: SecretReveal::Any,
                            plain_reveal: true,
                        },
                        SecretOrigin::Inline(Value::String(TOKEN.to_string())),
                    )
                    .unwrap(),
                ),
            ),
            (
                "file_token_locator".to_string(),
                Value::String(format!("file:{}", token.display())),
            ),
        ]),
    )
}

#[test]
fn derived_secrets_are_computed_when_revealed() {
    let (logs, events) = run();

    let revealed: Vec<Value> = events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::DataSent {
                output,
                data: DataContent::Values { values },
                ..
            } if output.label == "revealSecret" && output.name == "value" => Some(values.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    let expected = [
        format!("Bearer {TOKEN}"),
        "https://gitlab-ci-token:to%3Ak%2Fen%401@gitlab.com/group/project.git".to_string(),
        // base64 of "ci:to:k/en@1"
        "Y2k6dG86ay9lbkAx".to_string(),
        // base64 of "to:k/en@1"
        "dG86ay9lbkAx".to_string(),
    ];
    for value in &expected {
        assert!(
            revealed.contains(&Value::String(value.clone())),
            "{} not in {:?}",
            value,
            revealed
        );
    }

    // Revealed values, and the values they come from, are masked in logs.
    for label in ["bearer", "repository", "basic", "bytes"] {
        let logged = message(&logs, label);
        assert!(
            logged.starts_with("<secret \"") && logged.ends_with("\">"),
            "{}",
            logged
        );
    }
    for log in &logs {
        assert!(!log.message.contains(TOKEN), "{}", log.message);
        for value in &expected {
            assert!(!log.message.contains(value.as_str()), "{}", log.message);
        }
    }

    // Building derived secrets reveals nothing: every reveal comes from `reveal`,
    // and revealing a derived secret also records the secrets it comes from.
    for event in &events {
        if let EventKind::SecretRevealed { element, .. } = &event.kind {
            assert_eq!(element.to_string(), "std/secret::reveal");
        }
    }
    let revealed = |name: &str| {
        events
            .iter()
            .filter(|event| matches!(&event.kind, EventKind::SecretRevealed { secret_name, .. } if secret_name == name))
            .count()
    };
    assert_eq!(revealed("authorization"), 1);
    assert_eq!(revealed("url_token"), 1);
    assert_eq!(revealed("token"), 4);
    assert_eq!(revealed("closed"), 0);
}

#[test]
fn derived_secrets_get_the_most_restrictive_policy() {
    let (logs, events) = run();

    assert!(message(&logs, "mixed").contains("does not allow plain reveal"));
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretDenied { secret_name, .. } if secret_name == "mixed"
    )));
}

#[test]
fn derivation_failures_never_show_values() {
    let (logs, _) = run();

    let broken = message(&logs, "broken");
    assert!(
        broken.contains("the template uses an entry that is not given"),
        "{}",
        broken
    );
    for log in &logs {
        if log.label == "broken" || log.label == "secret" {
            assert!(!log.message.contains(TOKEN), "{}", log.message);
        }
    }
}

#[test]
fn secrets_derived_from_files_are_never_plainly_revealed() {
    let (logs, events) = run();

    let refused = message(&logs, "file-bearer");
    assert!(
        refused.contains("secrets from 'file:' cannot be plainly revealed"),
        "{}",
        refused
    );
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        EventKind::SecretDenied { secret_name, .. } if secret_name == "file_authorization"
    )));
    for log in &logs {
        assert!(!log.message.contains(TOKEN) || log.label != "file-bearer");
    }
}
