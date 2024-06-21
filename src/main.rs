#![windows_subsystem = "windows"]

use enigo::{Enigo, Mouse, Settings};
use global_hotkey::{
    hotkey::{Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        )
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

fn main() {
    let wait_time = Duration::from_millis(40);
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let (sender, receiver) = mpsc::channel::<bool>();
    let _clicker = thread::spawn(move || {
        let mut just_pressed = true;
        let mut should_click = false;
        loop {
            match receiver.try_recv() {
                Ok(_) => {
                    if just_pressed {
                        println!("press");
                        should_click = !should_click;
                        just_pressed = false;
                    }
                }
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => just_pressed = true,
            }
            if should_click {
                enigo
                    .button(enigo::Button::Left, enigo::Direction::Click)
                    .unwrap();
            }
            thread::sleep(wait_time);
        }
    });

    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(None, Code::F2);
    manager.register(hotkey).unwrap();
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    let _listener = thread::spawn(move || loop {
        if let Ok(event) = global_hotkey_channel.try_recv() {
            if hotkey.id == event.id
                && event.state == HotKeyState::Released
                && sender.send(true).is_err()
            {
                break;
            }
        }
        thread::sleep(wait_time);
    });

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App { window: None };
    let _ = event_loop.run_app(&mut app);
}
