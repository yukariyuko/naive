use std::collections::BTreeMap;

use iou::IoUring;

use crate::task::{Iotask, Iotype};

const MAX_ENTRY: u32 = 16;

pub struct Reactor {
    iou: IoUring,
    pub tasks: BTreeMap<u64, Iotask>,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            iou: IoUring::new(MAX_ENTRY).unwrap(),
            tasks: BTreeMap::new(),
        }
    }

    pub fn event_loop(&mut self) {
        loop {
            let ceq = self.iou.wait_for_cqe().unwrap();
            let id = ceq.user_data();
            self.tasks.get_mut(&id).unwrap().inner.finish = true;
        }
    }

    pub fn regist(&mut self, task: Iotask) {
        let mut sqe = self.iou.prepare_sqe().unwrap();
        let len = task.inner.buf.borrow().len();
        match task.inner._type {
            Iotype::WRITE => unsafe {
                let buf_slice = &mut task.inner.buf.borrow_mut()[..];
                sqe.prep_read(task.inner.fd, buf_slice, len as u64);
            },
            Iotype::READ => unsafe {
                let buf_slice = &task.inner.buf.borrow()[..];
                sqe.prep_write(task.inner.fd, buf_slice, len as u64);
            },
        }
        unsafe {
            sqe.set_user_data(task.inner.id);
        }
        self.tasks.insert(task.inner.id, task);
        println!("{}", self.iou.submit_sqes().unwrap())
    }

    pub fn next_id(&self) -> u64 {
        self.tasks.len() as u64
    }
}
