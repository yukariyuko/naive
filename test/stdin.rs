use std::cell::RefCell;

use trySimpleAsyncRuntime::{
    reactor::Reactor,
    task::{Iotask, Iotype},
};

fn main() {
    let mut reactor = Reactor::new();
    let buf = RefCell::new(Vec::new());
    let mut future = Iotask::new(reactor.next_id(), 0, buf, Iotype::READ);
    reactor.regist(future);
    reactor.event_loop();
}
