#![feature(get_mut_unchecked)]

use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use autocxx::prelude::*; // use all the main autocxx functions
use autocxx::subclass::*; // use all the main autocxx functions
use autocxx::subclass::prelude::*; // use all the main autocxx functions

include_cpp! {
    #include "Leap.h"
    safety!(unsafe_ffi) // see details of unsafety policies described in the 'safety' section of the book
    // generate!("Leap::Listener") // add this line for each function or type you wish to generate
    generate!("Leap::Controller") // add this line for each function or type you wish to generate
    generate!("Leap::ControllerImplementation") // add this line for each function or type you wish to generate
    generate!("Leap::Frame") // add this line for each function or type you wish to generate
    generate!("Leap::HandList") // add this line for each function or type you wish to generate
    generate!("Leap::Hand") // add this line for each function or type you wish to generate
    generate!("Leap::Vector") // add this line for each function or type you wish to generate
    subclass!("Leap::Listener", MyListener)
}

#[is_subclass(superclass("Leap::Listener"))]
#[derive(Default)]
pub struct MyListener {}

impl ffi::Leap::Listener_methods for MyListener {
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

    // let listener_pin = unsafe {
    //     Pin::new_unchecked(listener.as_ref().borrow().as_ref())
    // };
    // let ctrl_impl = ffi::Leap::ControllerImplementation::new().within_unique_ptr();
    let mut controller = ffi::Leap::Controller::new1().within_unique_ptr();

    controller.pin_mut().addListener(listener_pin);
    // let listener = 123;
    // controller.pin_mut().addListener(listener.as_ref().borrow().as_ref());

    sleep(Duration::from_secs(30));
    println!("Hello, world!");
}
