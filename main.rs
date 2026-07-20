// aqm-supervisor : supervise les VRAIS services systemd de l'image
// (pas une simulation) et publie un score de sante sur un socket UNIX
// que aqm-shell (aqmctl) et aqm-ui consomment.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;
use std::thread;
use std::time::Duration;

const SOCKET_PATH: &str = "/run/aqm/aqm-supervisor.sock";

const CRITICAL_SERVICES: &[&str] = &[
    "aqm-dtnd.service",
    "aqm-recovery.service",
    "sshd.service",
];

const WATCHED_SERVICES: &[&str] = &[
    "aqm-dtnd.service",
    "aqm-recovery.service",
    "aqm-ui.service",
    "sshd.service",
    "rauc.service",
];

struct ServiceState {
    name: String,
    active_state: String,
    sub_state: String,
    n_restarts: String,
}

fn query_service(name: &str) -> ServiceState {
    let out = Command::new("systemctl")
        .args([
            "show",
            name,
            "--property=ActiveState,SubState,NRestarts",
            "--value",
        ])
        .output();

    let (active, sub, restarts) = match out {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            let mut lines = text.lines();
            (
                lines.next().unwrap_or("unknown").to_string(),
                lines.next().unwrap_or("unknown").to_string(),
                lines.next().unwrap_or("0").to_string(),
            )
        }
        Err(_) => ("unreachable".into(), "unreachable".into(), "0".into()),
    };

    ServiceState {
        name: name.to_string(),
        active_state: active,
        sub_state: sub,
        n_restarts: restarts,
    }
}

fn health_report() -> String {
    let mut healthy = 0usize;
    let mut critical_down = false;
    let mut lines = Vec::new();

    for &svc in WATCHED_SERVICES {
        let s = query_service(svc);
        let ok = s.active_state == "active";
        if ok {
            healthy += 1;
        }
        if CRITICAL_SERVICES.contains(&svc) && !ok {
            critical_down = true;
        }
        lines.push(format!(
            "{{\"service\":\"{}\",\"active_state\":\"{}\",\"sub_state\":\"{}\",\"restarts\":{}}}",
            s.name, s.active_state, s.sub_state, s.n_restarts
        ));
    }

    let score = healthy as f32 / WATCHED_SERVICES.len() as f32;
    let system_state = if critical_down {
        "critical"
    } else if score < 1.0 {
        "degraded"
    } else {
        "nominal"
    };

    format!(
        "{{\"score\":{:.2},\"system_state\":\"{}\",\"services\":[{}]}}\n",
        score,
        system_state,
        lines.join(",")
    )
}

fn handle_client(mut stream: UnixStream) {
    let mut buf = [0u8; 64];
    let _ = stream.read(&mut buf);
    let report = health_report();
    let _ = stream.write_all(report.as_bytes());
}

fn main() {
    std::fs::create_dir_all("/run/aqm").ok();
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).expect("bind aqm-supervisor socket");

    // Boucle de fond: journalise l'etat toutes les 30s (visible via
    // `journalctl -u aqm-supervisor`), en plus de servir les requetes.
    thread::spawn(|| loop {
        println!("health_report {}", health_report().trim());
        thread::sleep(Duration::from_secs(30));
    });

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_client(stream);
        }
    }
}
