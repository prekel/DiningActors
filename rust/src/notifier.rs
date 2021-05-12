use std::fmt;
use std::time::Duration;

use actix::prelude::*;

use crate::philosopher::*;

#[derive(Debug)]
enum NotifierState {}

impl fmt::Display for NotifierState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct NotifierActor {}

impl NotifierActor {
    pub fn new() -> Self {
        NotifierActor {}
    }
}

impl Actor for NotifierActor {
    type Context = Context<Self>;
}

#[derive(Debug)]
#[derive(Message)]
#[rtype(result = "()")]
pub enum NotifierMsg {
    NotifyAfter(Addr<PhilosopherActor>, Duration),
    Notify(Addr<PhilosopherActor>),
}

impl fmt::Display for NotifierMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Handler<NotifierMsg> for NotifierActor {
    type Result = ();

    fn handle(&mut self, msg: NotifierMsg, ctx: &mut Self::Context) -> Self::Result {
        println!("Notifier, msg: {}", msg);

        match msg {
            NotifierMsg::NotifyAfter(ph, dur) =>
                {
                    ctx.notify_later(NotifierMsg::Notify(ph), dur);
                }
            NotifierMsg::Notify(ph) => { ph.do_send(PhilosopherMsg::StopEating) }
        }
    }
}