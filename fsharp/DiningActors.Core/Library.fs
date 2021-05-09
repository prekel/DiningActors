namespace DiningActors.Core

open Akkling

module Say =

    type PhilosopherState =
        | Eating
        | WaitingLeft
        | WaitingRight
        | WaitingBoth
        | NotEating

    and PhilosopherMsg =
        | StartEating
        | StopEating
        | ForkIsTaken
        | ForkIsFreeNow
        | ForkIsFree

    and ForkState =
        | Taken
        | ForkFree

    and ForkMsg =
        | TryTake of IActorRef<PhilosopherMsg>
        | TakeOff


    type Table = unit


    let philosopherBehavior
        (leftFork: IActorRef<ForkMsg>)
        (rightFork: IActorRef<ForkMsg>)
        initial
        (ctx: Actor<PhilosopherMsg>)
        =
        let rec loop (state: PhilosopherState) =
            actor {
                let! msg = ctx.Receive()

                match msg with
                | StartEating ->
                    match state with
                    | NotEating ->
                        let! ans = leftFork <? TryTake
                        let a = ctx.Self
                        return! loop WaitingLeft
                    | _ -> return Unhandled
                | StopEating -> return! loop state
                | ForkIsTaken ->
                    match state with
                    | WaitingLeft ->

                        return! loop state
                | ForkIsFree ->
                    match state with
                    | WaitingLeft ->
                        rightFork <! TryTake
                        return loop WaitingRight
                    | WaitingRight ->
                        rightFork <! TryTake
                        return loop state
                    | _ -> return Unhandled
            }

        loop initial

    let forkBehavior (ctx: Actor<_>) =
        let rec loop (state: ForkState) =
            actor {
                let! n = ctx.Receive()
                let s = ctx.Sender()
                let! a = s <? "asd"

                match n with
                | TryTake -> printfn "Current state: %A" state
                | Free -> return! loop state
            }

        loop ForkFree

    let m () =
        let system =
            System.create "basic-sys"
            <| Configuration.defaultConfig ()

        ()

    let hello name = printfn "Hello %s" name
