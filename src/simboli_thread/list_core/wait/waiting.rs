use std::{
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    },
};

pub struct Waiting<T>
where
    T: 'static,
{
    pub(crate) data_ptr: &'static AtomicPtr<T>,
    pub(crate) data: Option<T>,
}

impl<T> Waiting<T> {
    pub fn block(&mut self) {
        while self.data_ptr.load(Ordering::Acquire).is_null() {
            spin_loop();
        }
        unsafe {
            let data = *Box::from_raw(self.data_ptr.load(Ordering::Acquire));
            self.data = Some(data);
        }
    }

    pub fn block_and_get(&mut self) -> &Option<T> {
        if let None = self.data {
            self.block();
        }
        &self.data
    }

    pub fn block_and_get_mut(&mut self) -> &mut Option<T> {
        if let None = self.data {
            self.block();
        }
        &mut self.data
    }

    pub fn get(&self) -> &Option<T> {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut Option<T> {
        &mut self.data
    }
}

impl<T> Deref for Waiting<T>
where
    T: 'static,
{
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Waiting<T>
where
    T: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

// impl<T> Drop for Waiting<T>
// where
//     T: 'static,
// {
//     fn drop(&mut self) {
//         if let None = self.data {
//             self.block();
//             unsafe {
//                 let data = Box::from_raw(self.data_ptr as *const AtomicPtr<T> as *mut AtomicPtr<T>);
//                 drop(data);
//             }
//         }
//     }
// }
