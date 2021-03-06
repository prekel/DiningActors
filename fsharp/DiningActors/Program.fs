open System
open DiningActors.Core.Say
open Akkling

[<EntryPoint>]
let main argv =
    let times n minEat maxEat minSpawn maxSpawn =
        n,
        (TimeSpan.FromMilliseconds(float minEat)),
        (TimeSpan.FromMilliseconds(float maxEat)),
        (TimeSpan.FromMilliseconds(minSpawn)),
        (TimeSpan.FromMilliseconds(maxSpawn))

    let n, minEat, maxEat, minSpawn, maxSpawn = times 5 1000 5000 500. 1500.
    //let n, minEat, maxEat, minSpawn, maxSpawn = times 5000 100 50000 0. 0.005


    let table, system = m n minEat maxEat

    let spawnMinTime = minSpawn
    let spawnMaxTime = maxSpawn

    let random = Random()

    let rec numbersFrom n =
        seq {
            yield n
            yield! numbersFrom (n + 1.)
        }

    for i in seq { 1 .. 100000 } do
        Async.Sleep(randomTime random spawnMinTime spawnMaxTime)
        |> Async.RunSynchronously

        let p = random.Next(1, n + 1)
        table.Philosophers.[p - 1] <! StartEating

    //    system.WhenTerminated
//    |> Async.AwaitTask
//    |> Async.RunSynchronously

    0
