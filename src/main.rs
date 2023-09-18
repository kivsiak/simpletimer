use std::sync::mpsc::channel;
use std::time::Instant;
use std::{
    io::{stdout, Write},
    thread,
    thread::sleep,
    time::Duration,
};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, ExecutableCommand};

enum TimerEvent {
    OnTime,
    OnStop,
    OnLap,
}

fn format_duration(duration: Duration) -> String {
    let ms = duration.as_millis() % 1000 / 100;
    let seconds = duration.as_secs() % 60;
    let minutes = duration.as_secs() / 60;

    return format!(" {:0>2}:{:0>2}.{:0>1}s", minutes, seconds, ms);
}

fn main() -> anyhow::Result<()> {
    let (sender, receiver) = channel::<TimerEvent>();

    enable_raw_mode()?;
    let mut std = stdout();
    std.execute(cursor::Hide)?;

    let main_loop = thread::spawn(move || -> anyhow::Result<()> {
        let start = Instant::now();
        let mut last_lap = start.clone();
        let mut round: u32 = 0;
        while let Ok(cmd) = receiver.recv() {
            match cmd {
                TimerEvent::OnTime => {
                    std.execute(cursor::SavePosition)?;
                    let duration = Instant::now().duration_since(start);

                    std.write(format!("{: >4}: {}", round, format_duration(duration)).as_bytes())
                        .unwrap();
                    std.execute(cursor::RestorePosition)?;
                    std.flush().unwrap();
                }
                TimerEvent::OnStop => break,
                TimerEvent::OnLap => {
                    let diff = Instant::now().duration_since(last_lap);
                    last_lap = Instant::now();
                    round += 1;
                    std.write(format!("\r\n").as_bytes())?;
                    std.write(format!("Lap:  {} \r\n", format_duration(diff)).as_bytes())?;
                    std.flush().unwrap();
                }
            }
        }
        Ok(())
    });

    let timer_sender = sender.clone();
    let _timer_loop = thread::spawn(move || -> anyhow::Result<()> {
        loop {
            timer_sender.send(TimerEvent::OnTime)?;
            sleep(Duration::from_millis(50));
        }
    });

    let input_sender = sender.clone();
    let _input_loop = thread::spawn(move || -> anyhow::Result<()> {
        loop {
            match read().unwrap() {
                Event::Key(KeyEvent {
                               code: KeyCode::Enter,
                               ..
                           }) => {
                    input_sender.send(TimerEvent::OnLap)?;
                }
                Event::Key(KeyEvent {
                               code: KeyCode::Char('c'),
                               modifiers: KeyModifiers::CONTROL,
                               ..
                           }) => {
                    input_sender.send(TimerEvent::OnStop)?;
                }
                _ => {}
            }
        }
    });

    main_loop.join().unwrap()?;
    let mut std = stdout();
    std.execute(cursor::Show)?;
    disable_raw_mode()?;
    println!("\r\n");
    Ok(())
}
