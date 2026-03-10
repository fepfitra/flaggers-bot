use daemonize::Daemonize;
use std::fs::File;

pub fn stop_daemon(pid_file: &str) {
    // try to kill by PID file
    if let Ok(pid_str) = std::fs::read_to_string(pid_file)
        && let Ok(pid) = pid_str.trim().parse::<u32>()
    {
        println!("Stopping daemon (PID: {})", pid);
        std::process::Command::new("kill")
            .arg(pid.to_string())
            .output()
            .ok();
        std::fs::remove_file(pid_file).ok();
    }
}

pub fn daemonize(pid_file: &str) {
    let _pid_file = File::create(pid_file).expect("Failed to create PID file");
    let stdout = File::create("flaggers_bot.out.log").expect("Failed to create stdout log");
    let stderr = File::create("flaggers_bot.err.log").expect("Failed to create stderr log");

    let daemonize = Daemonize::new()
        .pid_file(pid_file)
        .working_directory(std::env::current_dir().unwrap())
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => tracing::info!("Daemon started successfully"),
        Err(e) => eprintln!("Error starting daemon: {}", e),
    }
}
