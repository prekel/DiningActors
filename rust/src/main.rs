use std::borrow::Borrow;
use std::fmt;
use std::time::Duration;

use actix::clock::delay_for;
use actix::prelude::*;
use rand::Rng;
use rand::rngs::ThreadRng;

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
struct PhilosopherActor {
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
    fn new(id: usize, left_fork: Addr<ForkActor>, right_fork: Addr<ForkActor>, eating_min: Duration, eating_max: Duration) -> Self {
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
}

#[derive(Debug)]
#[derive(Message)]
#[rtype(result = "()")]
enum PhilosopherMsg {
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
                                    let rand_time = self.rng.gen_range(self.eating_min..self.eating_max);
                                    ctx.notify_later(PhilosopherMsg::StopEating, rand_time);
                                    self.state = PhilosopherState::Eating;
                                    println!("Philosopher {} started eating {:?}", self.id, rand_time);
                                } else {
                                    unreachable!()
                                }
                            }
                            (Some(l), None) => {
                                if l == fork.borrow() {
                                    let rand_time = self.rng.gen_range(self.eating_min..self.eating_max);
                                    ctx.notify_later(PhilosopherMsg::StopEating, rand_time);
                                    self.state = PhilosopherState::Eating;
                                    println!("Philosopher {} started eating {:?}", self.id, rand_time);
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
struct ForkActor {
    id: usize,
    ph1: Option<Addr<PhilosopherActor>>,
    state: ForkState,
}

impl ForkActor {
    fn new(id: usize) -> Self {
        ForkActor {
            id,
            ph1: None,
            state: ForkState::ForkFree,
        }
    }
}

impl Actor for ForkActor {
    type Context = Context<Self>;
}

#[derive(Debug)]
#[derive(Message)]
#[rtype(result = "()")]
enum ForkMsg {
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

    fn handle(&mut self, msg: ForkMsg, ctx: &mut Context<Self>) -> Self::Result {
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

#[actix_rt::main]
async fn main() {
    //let f = (1..5).map(|i| ForkActor::new(1)).collect();

    let f1 = ForkActor::new(1).start();
    let f2 = ForkActor::new(2).start();
    let f3 = ForkActor::new(3).start();
    let f4 = ForkActor::new(4).start();
    let f5 = ForkActor::new(5).start();

    let eating_min = Duration::from_millis(1000);
    let eating_max = Duration::from_millis(5000);

    let p1 = PhilosopherActor::new(1, f5.clone(), f1.clone(), eating_min, eating_max).start();
    let p2 = PhilosopherActor::new(2, f1.clone(), f2.clone(), eating_min, eating_max).start();
    let p3 = PhilosopherActor::new(3, f2.clone(), f3.clone(), eating_min, eating_max).start();
    let p4 = PhilosopherActor::new(4, f3.clone(), f4.clone(), eating_min, eating_max).start();
    let p5 = PhilosopherActor::new(5, f4.clone(), f5.clone(), eating_min, eating_max).start();

    let spawn_min = Duration::from_millis(500);
    let spawn_max = Duration::from_millis(1500);

    let phs = vec![p1, p2, p3, p4, p5];

    let mut rng = rand::thread_rng();

    for _ in 1..100000 {
        let sleep_time = rng.gen_range(spawn_min..spawn_max);
        delay_for(sleep_time).await;
        let p = rng.gen_range(1..5);
        phs[p - 1].do_send(PhilosopherMsg::StartEating);
    }

    System::current().stop();
}
