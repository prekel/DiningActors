open System
open DiningActors.Core.Say
open Akkling

[<EntryPoint>]
let main argv =
    let n = 5

    let table, system =
        m n (TimeSpan.FromMilliseconds 1000.) (TimeSpan.FromMilliseconds 5000.)

    let spawnMinTime = TimeSpan.FromMilliseconds 500.
    let spawnMaxTime = TimeSpan.FromMilliseconds 1500.

    let random = Random()

    for i in [ 1 .. 100 ] do
        Async.Sleep(randomTime random spawnMinTime spawnMaxTime)
        |> Async.RunSynchronously

        let p = random.Next(1, n)
        table.Philosophers.[p] <! StartEating

    //    system.WhenTerminated
//    |> Async.AwaitTask
//    |> Async.RunSynchronously

    0
