use clap::Parser;
use crossterm::{cursor, style::Print, terminal, ExecutableCommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{
    default::Default,
    io::stdout,
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[derive(Parser, Default, Debug)]
#[command(disable_help_flag = true)]
struct Cli {
    #[arg(short, default_value_t = 0)]
    seconds: u64,
    #[arg(short, default_value_t = 0)]
    minutes: u64,
    #[arg(short, default_value_t = 0)]
    hours: u64,
    #[arg(short, default_value_t = 0)]
    days: u64,
}

const SECOND: u64 = 1;
const MINUTE: u64 = 60 * SECOND;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

const TICK_DURATION: Duration = Duration::from_secs(1);

enum WaitResult {
    Finished,
    Terminated,
}

type Seconds = u64;

fn parse_arguments() -> Seconds {
    let cli = Cli::parse();
    let seconds = cli.seconds + cli.minutes * MINUTE + cli.hours * HOUR + cli.days * DAY;
    seconds
}

fn print_duration(d: &Duration) {
    let mut secs = d.as_secs();
    let mut xs = [0; 4];
    for (i, mult) in [DAY, HOUR, MINUTE, SECOND].iter().enumerate() {
        xs[i] = secs / mult;
        secs = secs % mult;
    }

    let line = format!(
        "Sleeping after {}d {}h {}m {}s. Press Ctrl+c to cancel.",
        xs[0], xs[1], xs[2], xs[3]
    );

    stdout()
        .execute(terminal::Clear(terminal::ClearType::CurrentLine))
        .expect("could not clear terminal line")
        .execute(cursor::MoveToColumn(0))
        .expect("could not move cursor")
        .execute(Print(line))
        .expect("cannot print the left duration for sleep")
        .execute(cursor::Hide)
        .expect("could not hide the cursor");
}

fn wait(seconds: u64) -> WaitResult {
    let now = Instant::now();
    let wait_duration = Duration::from_secs(seconds);
    println!("");

    let should_terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&should_terminate))
        .expect("Cannot register SIGTERM flag");
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&should_terminate))
        .expect("Cannot register SIGINT flag");

    loop {
        if should_terminate.load(Ordering::Relaxed) {
            return WaitResult::Terminated;
        }

        let elapsed = now.elapsed();
        if elapsed < wait_duration {
            let left_duration = wait_duration - elapsed;
            print_duration(&left_duration);
            thread::sleep(TICK_DURATION);
        } else {
            break;
        }
    }

    WaitResult::Finished
}

#[cfg(target_os = "windows")]
fn put_computer_to_sleep() {
    Command::new("rundll32.exe")
        .args(["powrprof.dll", ",", "SetSuspendState", "0,1,0"])
        .status()
        .expect("failed to execute sleep command");
}

#[cfg(target_os = "linux")]
fn put_computer_to_sleep() {
    Command::new("systemctl")
        .args(["suspend"])
        .status()
        .expect("failed to execute sleep command");
}

#[cfg(target_os = "macos")]
fn put_computer_to_sleep() {
    // do macos stuff
}

fn main() {
    let seconds = parse_arguments();
    let WaitResult::Finished =wait(seconds) else {
        println!("\nSleep timeout was canceled.");
        return;
    };
    put_computer_to_sleep();
}
