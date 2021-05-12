use std::time::Duration;

use actix::clock::delay_for;
use actix::prelude::*;
use rand::Rng;

use crate::fork::*;
use crate::philosopher::*;

mod philosopher;
mod fork;

async fn run(n: usize, eating_min: Duration, eating_max: Duration, spawn_min: Duration, spawn_max: Duration)
{
    let fs = (1..n + 1).map(|i| ForkActor::new(i).start()).collect::<Vec<Addr<ForkActor>>>();

    let phs = (1..n + 1).map(|i| {
        let left = if i == 1 { fs[n - 1].clone() } else { fs[i - 2].clone() };

        PhilosopherActor::new(i, left, fs[i - 1].clone(), eating_min, eating_max).start()
    }).collect::<Vec<Addr<PhilosopherActor>>>();

    let mut rng = rand::thread_rng();

    for _ in 1..100000 {
        let sleep_time = rng.gen_range(spawn_min..spawn_max);
        let p = rng.gen_range(1..n + 1);
        println!("Spawn philosopher {}, sleep {:?}", p, sleep_time);
        delay_for(sleep_time).await;
        phs[p - 1].do_send(PhilosopherMsg::StartEating);
    }
}

#[actix_rt::main]
async fn main() {
    let n = 5;
    let eating_min = Duration::from_millis(1000);
    let eating_max = Duration::from_millis(5000);
    let spawn_min = Duration::from_millis(500);
    let spawn_max = Duration::from_millis(1500);

    // let n = 5000;
    // let eating_min = Duration::from_millis(100);
    // let eating_max = Duration::from_millis(50000);
    // let spawn_min = Duration::from_millis(0);
    // let spawn_max = Duration::from_micros(5);


    run(n, eating_min, eating_max, spawn_min, spawn_max).await;

    System::current().stop();
}
