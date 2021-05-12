use std::borrow::Borrow;
use std::fmt;
use std::time::Duration;

use actix::prelude::*;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::fork::*;

#[derive(Debug)]
enum PhilosopherState {
    Eating,
    Waiting(Option<Addr<ForkActor>>, Option<Addr<ForkActor>>),
    NotEating,
}

impl fmt::Display for PhilosopherState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.borrow() {
            PhilosopherState::Waiting(_, _) => write!(f, "Waiting"),
            _ => write!(f, "{:?}", self)
        }
    }
}

#[derive(Debug)]
pub struct PhilosopherActor {
    id: usize,
    left_fork: Addr<ForkActor>,
    right_fork: Addr<ForkActor>,
    rng: ThreadRng,
    eating_min: Duration,
    eating_max: Duration,
    state: PhilosopherState,
}

impl Actor for PhilosopherActor {
    type Context = Context<Self>;
}

impl PhilosopherActor {
    pub fn new(id: usize, left_fork: Addr<ForkActor>, right_fork: Addr<ForkActor>, eating_min: Duration, eating_max: Duration) -> Self {
        PhilosopherActor {
            id,
            left_fork,
            right_fork,
            rng: rand::thread_rng(),
            eating_min,
            eating_max,
            state: PhilosopherState::NotEating,
        }
    }


    fn start_eating(&mut self, ctx: &mut Context<Self>) {
        let rand_time = self.rng.gen_range(self.eating_min..self.eating_max);
        ctx.notify_later(PhilosopherMsg::StopEating, rand_time);
        self.state = PhilosopherState::Eating;
        println!("Philosopher {} started eating {:?}", self.id, rand_time);
    }
}

#[derive(Debug)]
#[derive(Message)]
#[rtype(result = "()")]
pub enum PhilosopherMsg {
    StartEating,
    StopEating,
    SuccessfullyTaken(Addr<ForkActor>),
    ForkIsFree(Addr<ForkActor>),
}

impl fmt::Display for PhilosopherMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.borrow() {
            PhilosopherMsg::SuccessfullyTaken(_) => write!(f, "SuccessfullyTaken"),
            PhilosopherMsg::ForkIsFree(_) => write!(f, "ForkIsFree"),
            _ => write!(f, "{:?}", self)
        }
    }
}

impl Handler<PhilosopherMsg> for PhilosopherActor {
    type Result = ();

    fn handle(&mut self, msg: PhilosopherMsg, ctx: &mut Context<Self>) -> Self::Result {
        println!("Philosopher {}, state: {}, msg: {}", self.id, self.state, msg);

        match msg {
            PhilosopherMsg::StartEating => {
                match self.state {
                    PhilosopherState::Eating => (),
                    PhilosopherState::Waiting(_, _) => (),
                    PhilosopherState::NotEating => {
                        self.left_fork.do_send(ForkMsg::TryTake(ctx.address()));
                        self.right_fork.do_send(ForkMsg::TryTake(ctx.address()));
                        self.state = PhilosopherState::Waiting(Some(self.left_fork.clone()), Some(self.right_fork.clone()))
                    }
                }
            }
            PhilosopherMsg::StopEating => {
                match self.state {
                    PhilosopherState::Eating => {
                        self.left_fork.do_send(ForkMsg::TakeOff);
                        self.right_fork.do_send(ForkMsg::TakeOff);
                        self.state = PhilosopherState::NotEating;
                    }
                    PhilosopherState::Waiting(_, _) => unreachable!(),
                    PhilosopherState::NotEating => unreachable!()
                }
            }
            PhilosopherMsg::SuccessfullyTaken(fork) => {
                match self.state.borrow() {
                    PhilosopherState::Eating => unreachable!(),
                    PhilosopherState::Waiting(left, right) => {
                        match (left, right)
                        {
                            (Some(l), Some(r)) => {
                                if l == fork.borrow() {
                                    self.state = PhilosopherState::Waiting(None, Some(r.clone()))
                                } else if r == fork.borrow() {
                                    self.state = PhilosopherState::Waiting(Some(l.clone()), None)
                                } else {
                                    unreachable!()
                                }
                            }
                            (None, Some(r)) => {
                                if r == fork.borrow() {
                                    self.start_eating(ctx);
                                } else {
                                    unreachable!()
                                }
                            }
                            (Some(l), None) => {
                                if l == fork.borrow() {
                                    self.start_eating(ctx);
                                } else {
                                    unreachable!()
                                }
                            }
                            (None, None) => unreachable!(),
                        }
                    }
                    PhilosopherState::NotEating => unreachable!(),
                }
            }
            PhilosopherMsg::ForkIsFree(fork) => {
                match self.state {
                    PhilosopherState::Eating => (),
                    PhilosopherState::Waiting(_, _) => {
                        if fork == self.left_fork || fork == self.right_fork {
                            fork.do_send(ForkMsg::TryTake(ctx.address()))
                        } else {
                            unreachable!()
                        }
                    }
                    PhilosopherState::NotEating => {
                        self.state = PhilosopherState::NotEating
                    }
                }
            }
        }
    }
}