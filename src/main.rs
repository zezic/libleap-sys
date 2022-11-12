#![feature(get_mut_unchecked)]

use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use autocxx::prelude::*;
use autocxx::subclass::*;

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
pub struct MyListener {}

impl ffi::Leap::Listener_methods for MyListener {
    #[allow(non_snake_case)]
    fn onFrame(&mut self, controller: &ffi::Leap::Controller) {
        let frame = controller.frame(autocxx::c_int(0)).within_unique_ptr();
        let id = frame.id();
        println!("FRAME {}", id);
        let hands = frame.hands().within_unique_ptr();
        let cnt: autocxx::c_int = hands.count();
        println!("CNT {}", cnt.0);
        if cnt.0 > 0 {
            let left = hands.leftmost().within_unique_ptr();
            let pos: UniquePtr<ffi::Leap::Vector> = left.palmPosition().within_unique_ptr();
            let f32_array_ptr: *const f32 = pos.toFloatPointer();
            let slice = unsafe { std::slice::from_raw_parts(f32_array_ptr, 3) };
            println!("Palm: {:?}", slice);
        }
    }
}


fn main() {
    let mut listener = MyListener::default_rust_owned();
    let listener_real = unsafe { Rc::get_mut_unchecked(&mut listener) };
    let listener_pin = listener_real.get_mut().pin_mut();
    let mut controller = ffi::Leap::Controller::new1().within_unique_ptr();
    controller.pin_mut().addListener(listener_pin);
    sleep(Duration::from_secs(30));
}
