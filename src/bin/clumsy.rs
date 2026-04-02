use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use serde_json::Value;

fn settings_path() -> PathBuf {
    #[cfg(windows)]
    {
        env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join(".claude").join("settings.json"))
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\.claude\\settings.json"))
    }
    #[cfg(not(windows))]
    {
        env::var("HOME")
            .map(|p| PathBuf::from(p).join(".claude").join("settings.json"))
            .unwrap_or_else(|_| PathBuf::from("/home/.claude/settings.json"))
    }
}

#[allow(clippy::enum_variant_names)]
enum Profile {
    ClaudeMax,
    ClaudePro,
    ClaudeFree,
}

impl Profile {
    fn display(&self) -> &'static str {
        match self {
            Profile::ClaudeMax => "Claude Max",
            Profile::ClaudePro => "Claude Pro",
            Profile::ClaudeFree => "Claude Free",
        }
    }

    fn env_vars(&self) -> Vec<(String, String)> {
        match self {
            Profile::ClaudeMax => vec![
                ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "claude-opus-4.5".into()),
                ("ANTHROPIC_MODEL".into(), "claude-sonnet-4.5".into()),
                ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "claude-haiku-4.5".into()),
            ],
            Profile::ClaudePro => vec![
                ("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), "claude-opus-4.5".into()),
                ("ANTHROPIC_MODEL".into(), "claude-sonnet-4.5".into()),
                ("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), "gpt-5-mini".into()),
            ],
            Profile::ClaudeFree => vec![("ANTHROPIC_MODEL".into(), "gpt-5-mini".into())],
        }
    }
}

fn prompt_menu() -> io::Result<Profile> {
    let options = [Profile::ClaudeMax, Profile::ClaudePro, Profile::ClaudeFree];

    println!("select a profile to apply:");
    for (i, p) in options.iter().enumerate() {
        println!("  {}) {}", i + 1, p.display());
    }
    print!("enter number (1-3): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim().parse::<usize>().ok();

    match choice {
        Some(1) => Ok(Profile::ClaudeMax),
        Some(2) => Ok(Profile::ClaudePro),
        Some(3) => Ok(Profile::ClaudeFree),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid selection")),
    }
}

fn load_or_create_settings(path: &PathBuf) -> io::Result<Value> {
    if path.exists() {
        let s = fs::read_to_string(path)?;
        let v: Value = serde_json::from_str(&s)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(v)
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let default_env = serde_json::json!({
            "ANTHROPIC_BASE_URL": "http://localhost:4141",
            "ANTHROPIC_AUTH_TOKEN": "sk-dummy",
            "ANTHROPIC_DEFAULT_OPUS_MODEL": "claude-opus-4.5",
            "ANTHROPIC_MODEL": "claude-sonnet-4.5",
            "ANTHROPIC_DEFAULT_HAIKU_MODEL": "claude-haiku-4.5",
            "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "1",
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
            "CLAUDE_CODE_ATTRIBUTION_HEADER": "0"
        });
        Ok(serde_json::json!({ "env": default_env }))
    }
}

fn save_atomic(path: &PathBuf, v: &Value) -> io::Result<()> {
    let json = serde_json::to_string_pretty(v).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json.as_bytes())?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn set_profile(path: &PathBuf, profile: &Profile) -> io::Result<()> {
    let mut settings = load_or_create_settings(path)?;

    if !settings.is_object() {
        settings = serde_json::json!({});
    }

    if settings.get("env").is_none() {
        let default_env = serde_json::json!({
            "ANTHROPIC_BASE_URL": "http://localhost:4141",
            "ANTHROPIC_AUTH_TOKEN": "sk-dummy",
            "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "1",
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
            "CLAUDE_CODE_ATTRIBUTION_HEADER": "0",
        });
        settings["env"] = default_env;
    }

    let env_obj = settings["env"].as_object_mut().unwrap();

    for (k, v) in profile.env_vars() {
        env_obj.insert(k, Value::String(v));
    }

    save_atomic(path, &settings)
}

fn print_usage() {
    println!("clumsy - simple profile switcher for claude settings");
    println!("usage: clumsy --p");
    println!("  --p    interactive profile selector");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // if no args provided, show the clumsy usage only
    if args.len() < 2 {
        print_usage();
        return;
    }

    // explicitly handle help flags so we don't surface upstream/cargo or ratatui help
    if args.iter().any(|a| a == "-h" || a == "--help" || a == "--h") {
        print_usage();
        return;
    }

    if args.iter().any(|a| a == "--p" || a == "--profile") {
        match prompt_menu() {
            Ok(profile) => {
                let path = settings_path();
                match set_profile(&path, &profile) {
                    Ok(()) => println!(
                        "applied profile: {}\nsettings written to {}",
                        profile.display(),
                        path.display()
                    ),
                    Err(e) => eprintln!("failed to write settings.json: {}", e),
                }
            }
            Err(e) => {
                eprintln!("invalid selection: {}", e);
            }
        }
    } else {
        print_usage();
    }
}
