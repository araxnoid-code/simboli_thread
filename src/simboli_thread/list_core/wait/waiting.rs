use std::{
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    },
};

pub struct Waiting<O>
where
    O: 'static,
{
    pub(crate) data_ptr: &'static AtomicPtr<O>,
}

impl<O> Waiting<O> {
    pub fn block(&self) -> Option<&O> {
        while self.data_ptr.load(Ordering::Acquire).is_null() {
            spin_loop();
        }

        unsafe { Some(&*self.data_ptr.load(Ordering::Acquire)) }
    }

    pub fn get(&self) -> Option<&O> {
        unsafe { Some(&*self.data_ptr.load(Ordering::Acquire)) }
    }

    pub fn collect(self) -> O {
        unsafe {
            while self.data_ptr.load(Ordering::Acquire).is_null() {
                spin_loop();
            }

            let data_box = Box::from_raw(self.data_ptr.load(Ordering::Acquire));
            *data_box
        }
    }
}
