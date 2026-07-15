use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;

use users::{ get_current_uid, get_user_by_uid };

use pam_client::{Context, Flag};
use pam_client::conv_cli::Conversation;

enum Mode {
    Sleep,
    Hibernate,
}

fn main() {
    let mode = match resolve_mode() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Usage: {} <mode: sleep, hibernate>", e);
            exit(2);
        },
    };

    println!("Selected: {}", match mode {
        Mode::Sleep => "sleep",
        Mode::Hibernate => "big sleep",
    });

    let real_uid = get_current_uid();
    match get_user_by_uid(real_uid)
        .map(|user| user.name().to_string_lossy().into_owned()) {
        Some(user) => {
            if let Err(e) = pam_auth(user.as_str()) {
                eprintln!("authentication failed: {e}");
                exit(1);
            }
        },
        None => {
            eprintln!("could not resolve current user");
            exit(3);
        }
    }

    if let Err(e) = write_sysfs(mode) {
        eprintln!("sysfs IO error: {}", e);
        exit(4);
    }

    exit(0);
}

fn resolve_mode() -> Result<Mode, String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        return Err(args[0].clone());
    }
    let res: Mode = match args.get(1) {
        Some(v) => match v.as_str() {
            "sleep" => Mode::Sleep,
            "hibernate" => Mode::Hibernate,
            _ => {
                return Err(args[0].clone());
            },
        },
        None => Mode::Sleep,
    };
    Ok(res)
}

fn pam_auth(real_user: &str) -> Result<(), String> {
    let mut context = Context::new(
        "power-state",
        Some(real_user),
        Conversation::new()
    ).expect("Failed to initialize PAM context!");

    if let Err(_) = context.authenticate(Flag::NONE) {
        return Err("Not authenticated".to_string())
    }
    if let Err(_) = context.acct_mgmt(Flag::NONE) {
        return Err("Validation failed".to_string())
    }
    
    println!("pam authenticated!");
    Ok(())
}

fn write_sysfs(mode: Mode) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .open("/sys/power/state")?;

    match mode {
        Mode::Sleep => { file.write_all(b"mem")?; },
        Mode::Hibernate => { file.write_all(b"disk")?; },
    }
    Ok(())
}
