use std::{
    cell::Cell,
    fmt,
    process::{Command, exit},
    sync::{Arc, Mutex},
    thread, time,
};
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{self, Menu},
};

enum NetworkProxyStatus {
    On,
    Off,
}

impl fmt::Display for NetworkProxyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkProxyStatus::Off => write!(f, "OFF"),
            NetworkProxyStatus::On => write!(f, "ON"),
        }
    }
}

fn configure_proxy(status: NetworkProxyStatus) {
    let output = match Command::new("./scripts/proxy_loader_mac.sh")
        .arg(status.to_string())
        .output()
    {
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1)
        }
        Ok(val) => val,
    };

    if output.stderr.len() > 0 {
        eprintln!("Error: {:?}", output.stderr);
        exit(1)
    }
}

#[derive(Debug)]
enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
    UpdateTitle(String),
}

struct TimerState {
    elapsed_before: Cell<u64>,
    start_time: Cell<time::Instant>,
    is_running: Cell<bool>,
}

fn main() {
    configure_proxy(NetworkProxyStatus::On);

    let proxy_process = Arc::new(Mutex::new(
        Command::new("mitmdump")
            .arg("-s")
            .arg("redirect.py")
            // .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    ));

    let proxy_process_clone = proxy_process.clone();

    ctrlc::set_handler(move || {
        let mut guard = proxy_process_clone.lock().unwrap();
        let _ = (*guard).kill();
        configure_proxy(NetworkProxyStatus::Off);
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let state = Arc::new(Mutex::new(TimerState {
        elapsed_before: Cell::new(0),
        is_running: Cell::new(true),
        start_time: Cell::new(time::Instant::now()),
    }));

    let quit_item = menu::MenuItem::with_id("quit", "Quit", true, None);
    let stop_item = menu::MenuItem::with_id("stop", "Stop", true, None);
    let start_item = menu::MenuItem::with_id("start", "Start", true, None);
    let tray_menu = Menu::with_items(&[&start_item, &stop_item, &quit_item]).unwrap();
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("system-tray - tray icon library!")
        .with_title("00:00:00")
        .with_icon({
            const WIDTH: u32 = 16;
            const HEIGHT: u32 = 16;
            let mut rgba = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
            let center_x = (WIDTH / 2) as i32;
            let center_y = (HEIGHT / 2) as i32;
            let radius = 6.0;
            for y in 0..HEIGHT as i32 {
                for x in 0..WIDTH as i32 {
                    let dx = (x - center_x) as f32;
                    let dy = (y - center_y) as f32;
                    if (dx * dx + dy * dy).sqrt() <= radius {
                        let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                        rgba[idx + 0] = 1; // red
                        rgba[idx + 1] = 1; // green
                        rgba[idx + 2] = 1; // blue
                        rgba[idx + 3] = 255; // alpha
                    }
                }
            }
            Icon::from_rgba(rgba, WIDTH, HEIGHT).unwrap()
        })
        .build()
        .unwrap();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let proxy = event_loop.create_proxy();

    tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    let proxy = event_loop.create_proxy();

    tray_icon::menu::MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let state_clone = Arc::clone(&state);

    let proxy = event_loop.create_proxy();

    thread::spawn(move || {
        loop {
            let state = state_clone.lock().unwrap();
            if !state.is_running.get() {
                drop(state);
                continue;
            }
            let time_string = {
                let elapsed =
                    state.start_time.get().elapsed().as_secs() + state.elapsed_before.get();
                let hours = elapsed / 3600;
                let minutes = (elapsed % 3600) / 60;
                let seconds = elapsed % 60;
                let _time_string = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
                _time_string
            };
            let _ = proxy.send_event(UserEvent::UpdateTitle(time_string));
            drop(state);

            thread::sleep(time::Duration::from_millis(1000))
        }
    });

    let state_clone = Arc::clone(&state);

    let proxy_process_clone = proxy_process.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::MenuEvent(e)) => {
                let state = state_clone.lock().unwrap();
                match e.id.as_ref() {
                    "stop" => {
                        state
                            .elapsed_before
                            .update(|x| x + state.start_time.get().elapsed().as_secs());
                        state.is_running.set(false);
                    }
                    "start" => {
                        state.is_running.set(true);
                        state.start_time.set(time::Instant::now());
                    }
                    "quit" => {
                        let mut guard = proxy_process_clone.lock().unwrap();
                        let _ = (*guard).kill();
                        configure_proxy(NetworkProxyStatus::Off);
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::UserEvent(UserEvent::TrayIconEvent(_e)) => {
                // println!("[TrayIconEvent] {e:?}");
            }
            Event::UserEvent(UserEvent::UpdateTitle(new_title)) => tray.set_title(Some(new_title)),
            _ => (),
        }
    });
}
