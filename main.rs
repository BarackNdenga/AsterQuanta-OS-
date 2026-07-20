// aqmctl : CLI d'administration reelle d'AsterQuanta OS.
// Pilote de vrais outils systeme (systemctl, journalctl, rauc, aqm-dtnd)
// - aucune simulation, ce sont de vrais appels sur le systeme installe.

use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::process::{exit, Command};

const SUPERVISOR_SOCKET: &str = "/run/aqm/aqm-supervisor.sock";
const DTN_AAP_SOCKET: &str = "/run/aqm/aqm-dtnd.aap.sock";

fn usage() {
    eprintln!(
        "aqmctl <commande>\n\n\
         status                 Rapport de sante (via aqm-supervisor)\n\
         services               Liste des services systemd surveilles\n\
         restart <service>      systemctl restart <service>\n\
         logs [n]                journalctl -n <n> (defaut 20)\n\
         update install <bundle> rauc install <bundle> (update A/B reelle)\n\
         update status           rauc status\n\
         recovery                bascule vers aqm-recovery.target\n\
         safe-mode on|off        bascule aqm-safe.target / multi-user.target\n\
         dtn status              statut du noeud DTN (aqm-dtnd)\n"
    );
}

fn read_supervisor() -> String {
    match UnixStream::connect(SUPERVISOR_SOCKET) {
        Ok(mut s) => {
            let _ = s.write_all(b"status");
            let mut buf = String::new();
            let _ = s.read_to_string(&mut buf);
            buf
        }
        Err(e) => format!("{{\"error\":\"aqm-supervisor injoignable: {}\"}}", e),
    }
}

fn run(cmd: &str, args: &[&str]) {
    match Command::new(cmd).args(args).status() {
        Ok(s) if s.success() => {}
        Ok(s) => exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("erreur d'execution de {}: {}", cmd, e);
            exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        exit(1);
    }

    match args[1].as_str() {
        "status" => println!("{}", read_supervisor().trim()),
        "services" => run("systemctl", &["list-units", "aqm-*.service", "rauc.service", "--no-pager"]),
        "restart" => {
            if args.len() < 3 {
                eprintln!("Usage: aqmctl restart <service>");
                exit(1);
            }
            run("systemctl", &["restart", &args[2]]);
        }
        "logs" => {
            let n = args.get(2).map(|s| s.as_str()).unwrap_or("20");
            run("journalctl", &["-n", n, "--no-pager"]);
        }
        "update" => match args.get(2).map(|s| s.as_str()) {
            Some("install") => {
                let Some(bundle) = args.get(3) else {
                    eprintln!("Usage: aqmctl update install <bundle.raucb>");
                    exit(1);
                };
                run("rauc", &["install", bundle]);
            }
            Some("status") => run("rauc", &["status"]),
            _ => usage(),
        },
        "recovery" => run("systemctl", &["isolate", "aqm-recovery.target"]),
        "safe-mode" => match args.get(2).map(|s| s.as_str()) {
            Some("on") => run("systemctl", &["isolate", "aqm-safe.target"]),
            Some("off") => run("systemctl", &["isolate", "multi-user.target"]),
            _ => usage(),
        },
        "dtn" => match args.get(2).map(|s| s.as_str()) {
            Some("status") => match UnixStream::connect(DTN_AAP_SOCKET) {
                Ok(_) => println!("aqm-dtnd: socket AAP joignable ({})", DTN_AAP_SOCKET),
                Err(e) => println!("aqm-dtnd injoignable: {}", e),
            },
            _ => usage(),
        },
        _ => {
            usage();
            exit(1);
        }
    }
}
