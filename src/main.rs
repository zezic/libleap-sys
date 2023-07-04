#![feature(get_mut_unchecked)]

use std::net::UdpSocket;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use autocxx::prelude::*;
use autocxx::subclass::*;
use rosc::OscMessage;
use rosc::OscPacket;
use rosc::OscType;
use rosc::encoder;

include_cpp! {
    #include "Leap.h"
    safety!(unsafe_ffi)
    generate!("Leap::Controller")
    generate!("Leap::ControllerImplementation")
    generate!("Leap::Frame")
    generate!("Leap::HandList")
    generate!("Leap::Hand")
    generate!("Leap::Vector")
    subclass!("Leap::Listener", MyListener)
}

#[is_subclass(superclass("Leap::Listener"))]
#[derive(Default)]
pub struct MyListener {
    sender: OscSender,
}

pub struct OscSender {
    socket: UdpSocket,
    notes: [Option<i32>; 2],
}

impl Default for OscSender {
    fn default() -> Self {
        let host_addr = "127.0.0.1:12345";
        let socket = UdpSocket::bind(host_addr).unwrap();
        Self { socket, notes: [None, None] }
    }
}

impl OscSender {
    fn send(&self, ns: &str, val: i32) {
        let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: ns.to_string(),
            args: vec![OscType::Int(val)],
        }))
        .expect("Can't encode OSC mes");

        let to_addr = "127.0.0.1:8000";
        self.socket
            .send_to(&msg_buf, to_addr)
            .expect("Can't send to OSC");
    }

    fn maybe_note(&mut self, chan: usize, target: bool, note: i32, velocity: i32) {
        dbg!(chan, target, note, velocity);
        self.notes[chan] = match (self.notes[chan], target) {
            (None, true) => {
                let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!("/vkb_midi/{}/note/{}", chan + 1, note),
                    args: vec![OscType::Int(velocity)],
                }))
                .expect("Can't encode OSC mes");

                let to_addr = "127.0.0.1:8000";
                self.socket
                    .send_to(&msg_buf, to_addr)
                    .expect("Can't send to OSC");
                Some(note)
            },
            (Some(current), false) => {
                let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!("/vkb_midi/{}/note/{}", chan + 1, current),
                    args: vec![OscType::Int(0)],
                }))
                .expect("Can't encode OSC mes");

                let to_addr = "127.0.0.1:8000";
                self.socket
                    .send_to(&msg_buf, to_addr)
                    .expect("Can't send to OSC");
                None
            },
            (Some(current), true) => {
                let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!("/vkb_midi/{}/pitchbend", chan + 1),
                    args: vec![OscType::Int(((note - current) * 3 + 64).clamp(0, 127))],
                }))
                .expect("Can't encode OSC mes");

                let to_addr = "127.0.0.1:8000";
                self.socket
                    .send_to(&msg_buf, to_addr)
                    .expect("Can't send to OSC");

                let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!("/vkb_midi/{}/aftertouch/{}", chan + 1, current),
                    args: vec![OscType::Int(velocity)],
                }))
                .expect("Can't encode OSC mes");

                let to_addr = "127.0.0.1:8000";
                self.socket
                    .send_to(&msg_buf, to_addr)
                    .expect("Can't send to OSC");

                return;
            }
            _ => return,
        }
    }
}

impl ffi::Leap::Listener_methods for MyListener {
    #[allow(non_snake_case)]
    fn onFrame(&mut self, controller: &ffi::Leap::Controller) {
        let frame = controller.frame(autocxx::c_int(0)).within_unique_ptr();
        let id = frame.id();
        println!("FRAME {}", id);
        let hands = frame.hands().within_unique_ptr();
        let cnt: autocxx::c_int = hands.count();
        println!("CNT {}", cnt.0);
        let left = if cnt.0 > 1 {
            let hand = hands.leftmost().within_unique_ptr();
            let pos: UniquePtr<ffi::Leap::Vector> = hand.palmPosition().within_unique_ptr();
            let f32_array_ptr: *const f32 = pos.toFloatPointer();
            let slice = unsafe { std::slice::from_raw_parts(f32_array_ptr, 3) };
            dbg!(slice);

            let remaps = [
                (-50.0, -150.0),
                (70.0, 200.0),
                (100.0, -100.0),
            ];
            for ((idx, value), (low, hi)) in slice.iter().enumerate().zip(remaps.iter()) {
                self.sender.send(&format!("/device/param/{}/value", idx + 1), remap(*value, *low, *hi, 16383));
            }
            self.sender.maybe_note(0, slice[2] < 100.0, remap(slice[1], 20.0, 800.0, 95) + 32, remap(slice[0], -50.0, -200.0, 126) + 1);
            // println!("{:?}", slice);
            Some(slice)
        } else {
            self.sender.maybe_note(0, false, 0, 0);
            None
        };
        let right = if cnt.0 > 0 {
            let hand = hands.rightmost().within_unique_ptr();
            let pos: UniquePtr<ffi::Leap::Vector> = hand.palmPosition().within_unique_ptr();
            let f32_array_ptr: *const f32 = pos.toFloatPointer();
            let slice = unsafe { std::slice::from_raw_parts(f32_array_ptr, 3) };
            dbg!(slice);
            let remaps = [
                (50.0, 150.0),
                (70.0, 200.0),
                (100.0, -100.0),
            ];
            for ((idx, value), (low, hi)) in slice.iter().enumerate().zip(remaps.iter()) {
                self.sender.send(&format!("/device/param/{}/value", idx + 1 + 4), remap(*value, *low, *hi, 16383));
            }
            self.sender.maybe_note(1, slice[2] < 100.0, remap(slice[1], 20.0, 800.0, 95) + 32, remap(slice[0], 50.0, 200.0, 126) + 1);
            // println!("{:?}", slice);
            Some(slice)
        } else {
            self.sender.maybe_note(1, false, 0, 0);
            None
        };
        // dbg!(left, right);
    }
}

fn remap(x: f32, low: f32, hi: f32, max: i32) -> i32 {
    let ((low, hi), rev) = if low < hi { ((low, hi), false) } else { ((hi, low), true) };
    let mut out = ((x.clamp(low, hi) - low) / (hi - low) * (max as f32)) as i32;
    if rev {
        out = max - out;
    }
    out
}

fn main() {
    let mut listener = MyListener::default_rust_owned();
    let listener_real = unsafe { Rc::get_mut_unchecked(&mut listener) };
    let listener_pin = listener_real.get_mut().pin_mut();
    let mut controller = ffi::Leap::Controller::new1().within_unique_ptr();
    controller.pin_mut().addListener(listener_pin);
    sleep(Duration::from_secs(3000000));
}
