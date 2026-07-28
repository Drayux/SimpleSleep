use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::process::exit;

use users::{ get_current_uid, get_user_by_uid };

use pam_client::{ Context, Flag };
use pam_client::conv_cli::Conversation;

enum Mode {
    Check,      // Print the pam status and exit
    Sleep,      // Soft sleep (S2)
    Hibernate,  // Deep sleep (S3)
    Shutdown,   // Complete shutdown
    Reboot,     // System reboot (partially depends on BIOS firmware)
}

fn main() {
    let mode = match resolve_mode() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Usage: {} <mode: sleep, hibernate, reboot, shutdown>", e);
            exit(2);
        },
    };

    if let Mode::Check = mode {
        println!("Dry-run, checking pam configuration...");
    }

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

    if let Err(_) = dispatch_action(mode) {
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
            "reboot" => Mode::Reboot,
            "restart" => Mode::Reboot,
            "shutdown" => Mode::Shutdown,
            "poweroff" => Mode::Shutdown,
            "check" => Mode::Check,
            _ => {
                return Err(args[0].clone());
            },
        },
        None => Mode::Check,
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
    
    Ok(())
}

fn write_sysfs(payload: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .open("/sys/power/state")?;

    file.write_all(payload)
}

fn dispatch_action(mode: Mode) -> Result<(), String> {
    let res = match mode {
        Mode::Sleep => write_sysfs(b"mem"),
        Mode::Hibernate => write_sysfs(b"disk"),
        Mode::Reboot => match Command::new("reboot").status() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Mode::Shutdown => match Command::new("poweroff").status() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
        Mode::Check => {
            // Pam validation already passed if we made it here.
            // (A better solution probably exists as this makes assumptions about the caller)
            println!("pam authentication success!");
            exit(0);
        },
    };

    if let Err(e) = res {
        let e_str = e.to_string();
        eprintln!("system error: {}", e_str);
        return Err(e_str)
    };
    Ok(())
}
