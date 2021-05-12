use std::borrow::Borrow;
use std::fmt;

use actix::prelude::*;

use crate::philosopher::*;

#[derive(Debug)]
enum ForkState {
    Taken,
    ForkFree,
}

impl fmt::Display for ForkState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct ForkActor {
    id: usize,
    ph1: Option<Addr<PhilosopherActor>>,
    state: ForkState,
}

impl ForkActor {
    pub fn new(id: usize) -> Self {
        ForkActor {
            id,
            ph1: None,
            state: ForkState::ForkFree,
        }
    }
}

impl Actor for ForkActor {
    type Context = SyncContext<Self>;
}

#[derive(Debug)]
#[derive(Message)]
#[rtype(result = "()")]
pub enum ForkMsg {
    TryTake(Addr<PhilosopherActor>),
    TakeOff,
}

impl fmt::Display for ForkMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.borrow() {
            ForkMsg::TryTake(_) => write!(f, "TryTake"),
            _ => write!(f, "{:?}", self)
        }
    }
}

impl Handler<ForkMsg> for ForkActor {
    type Result = ();

    fn handle(&mut self, msg: ForkMsg, ctx: &mut Self::Context) -> Self::Result {
        println!("Fork {}, state: {}, msg: {}", self.id, self.state, msg);
        match msg {
            ForkMsg::TryTake(ph) => {
                match self.state {
                    ForkState::Taken => {
                        self.ph1 = Some(ph)
                    }
                    ForkState::ForkFree => {
                        ph.do_send(PhilosopherMsg::SuccessfullyTaken(ctx.address()));
                        self.state = ForkState::Taken
                    }
                }
            }
            ForkMsg::TakeOff =>
                {
                    match self.state {
                        ForkState::Taken => {
                            match self.ph1.borrow() {
                                Some(ph1) => {
                                    ph1.do_send(PhilosopherMsg::ForkIsFree(ctx.address()))
                                }
                                None => {}
                            }
                            self.state = ForkState::ForkFree
                        }
                        ForkState::ForkFree => unreachable!()
                    }
                }
        }
    }
}