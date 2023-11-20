use std::time::Duration;
use winit::platform::pump_events::EventLoopExtPumpEvents;

pub struct EventLoopManager {
    pub window_event_loop: winit::event_loop::EventLoop<()>,
    window_exit_code: Option<i32>,
}

impl EventLoopManager {
    pub fn new() -> Self {
        EventLoopManager {
            window_event_loop: winit::event_loop::EventLoop::new().unwrap(),
            window_exit_code: None,
        }
    }

    pub fn set_window_closed(&mut self, exit_code: i32) {
        self.window_exit_code = Some(exit_code);
    }

    pub fn run<F>(&mut self, mut callback: F)
        where
            F: 'static + FnMut(winit::event::Event<()>, &winit::event_loop::EventLoopWindowTarget<()>)
    {
        loop {
            // Run the winit event loop once
            let winit_status = self.window_event_loop.pump_events(Some(Duration::new(0, 10000000)), |event, window_target| {
                callback(event, window_target);
            });
            if let winit::platform::pump_events::PumpStatus::Exit(exit_code) = winit_status {
                self.set_window_closed(exit_code);
            }

            // Check if there are remaining events to be handled
            if self.window_exit_code.is_some() {
                break;
            }
        }
    }
}
