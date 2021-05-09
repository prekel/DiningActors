namespace DiningActors.Core

open System
open Akkling

module Say =

    type PhilosopherState =
        | Eating
        | Waiting of IActorRef<ForkMsg> option * IActorRef<ForkMsg> option
        | NotEating

    and PhilosopherMsg =
        | StartEating
        | StopEating
        | SuccessfullyTaken of IActorRef<ForkMsg>
        //| AlreadyTaken of IActorRef<ForkMsg>
        | ForkIsFree of IActorRef<ForkMsg>

    and ForkState =
        | Taken
        | ForkFree

    and ForkMsg =
        | TryTake of IActorRef<PhilosopherMsg>
        | TakeOff


    type Table =
        { Philosophers: IActorRef<PhilosopherMsg> array
          PhilosopherAndRightFork: (IActorRef<PhilosopherMsg> * IActorRef<ForkMsg>) array }

    let randomTime (random: Random) (min: TimeSpan) (max: TimeSpan) =
        random.NextDouble()
        * (max.TotalMilliseconds - min.TotalMilliseconds)
        |> TimeSpan.FromMilliseconds
        |> (+) min

    let randomBehavior (ctx: Actor<_>) =
        let rec loop random =
            actor {
                let! min, max = ctx.Receive()

                ctx.Sender() <! randomTime random min max

                return! loop random
            }

        loop (Random())

    let philosopherBehavior
        (leftFork: IActorRef<ForkMsg>)
        (rightFork: IActorRef<ForkMsg>)
        (random: IActorRef<TimeSpan * TimeSpan>)
        (eatingMinTime: TimeSpan)
        (eatingMaxTime: TimeSpan)
        (ctx: Actor<PhilosopherMsg>)
        =

        let rec loop (state: PhilosopherState) =
            actor {
                let! msg = ctx.Receive()

                ctx.Log.Value.Info("Philosopher: msg {0}; state {1}", msg, state)

                match msg with
                | StartEating ->
                    match state with
                    | NotEating ->
                        leftFork <! TryTake(ctx.Self)
                        rightFork <! TryTake(ctx.Self)
                        return! loop (Waiting((Some leftFork), (Some rightFork)))
                    | _ -> return loop state
                | StopEating ->
                    match state with
                    | Eating ->
                        leftFork <! TakeOff
                        rightFork <! TakeOff
                        return! loop NotEating
                    | Waiting _ -> return Unhandled
                    | NotEating -> return Unhandled
                | SuccessfullyTaken fork ->
                    match state with
                    | Eating -> return Unhandled
                    | Waiting (left, right) ->
                        match left, right with
                        | Some left, Some right ->
                            match left, right with
                            | _, right when left = fork -> return! loop (Waiting(None, Some right))
                            | left, _ when right = fork -> return! loop (Waiting(Some left, None))
                            | _ -> return Unhandled
                        | None, Some right ->
                            if right = fork then
                                let! r = random <? (eatingMinTime, eatingMaxTime)
                                do ctx.Schedule r ctx.Self StopEating |> ignore
                                ctx.Log.Value.Info("Philosopher started eating {0}", r)
                                return! loop Eating
                            else
                                return Unhandled
                        | Some left, None ->
                            if left = fork then
                                let! r = random <? (eatingMinTime, eatingMaxTime)
                                do ctx.Schedule r ctx.Self StopEating |> ignore
                                ctx.Log.Value.Info("Philosopher started eating {0}", r)
                                return! loop Eating
                            else
                                return Unhandled
                        | None, None -> return Unhandled
                    | NotEating -> return Unhandled
                | ForkIsFree fork ->
                    match state with
                    | Eating -> return Unhandled
                    | Waiting _ ->
                        if fork = leftFork || fork = rightFork then
                            fork <! TryTake(ctx.Self)
                            return! loop state
                        else
                            return Unhandled
                    | NotEating -> return! loop NotEating
            }

        loop NotEating

    let forkBehavior (ctx: Actor<_>) =
        let rec loop ph1 (state: ForkState) =
            actor {
                let! msg = ctx.Receive()
                ctx.Log.Value.Info("Fork: msg {0}; state {1}", msg, state)

                match msg with
                | TryTake ph ->
                    match state with
                    | Taken -> return! loop (Some ph) state
                    | ForkFree ->
                        ph <! SuccessfullyTaken(ctx.Self)
                        return! loop ph1 Taken
                | TakeOff ->
                    match state with
                    | Taken ->
                        match ph1 with
                        | Some ph1 -> ph1 <! ForkIsFree(ctx.Self)
                        | None -> ()

                        return! loop ph1 ForkFree
                    | ForkFree -> return Unhandled
            }

        loop None ForkFree

    let m n (eatingMinTime: TimeSpan) (eatingMaxTime: TimeSpan) =
        let system =
            System.create "sys"
            <| Configuration.defaultConfig ()

        let forks =
            [| for i in [ 1 .. n ] do
                   yield spawn system $"fork%d{i}" (props forkBehavior) |]

        let forkLast = forks.[forks.Length - 1]

        let random =
            spawn system "random" (props randomBehavior)

        let philosophers =
            [| for i in [ 1 .. n ] do
                   yield
                       spawn
                           system
                           $"philosopher%d{i}"
                           (props (
                               philosopherBehavior
                                   (if i = 1 then
                                        forkLast
                                    else
                                        forks.[i - 2])
                                   forks.[i - 1]
                                   random
                                   eatingMinTime
                                   eatingMaxTime
                           )) |]

        let table =
            { Philosophers = philosophers
              PhilosopherAndRightFork = Array.zip philosophers forks }

        table, system
