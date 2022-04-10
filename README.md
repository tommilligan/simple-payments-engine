# simple-payments-engine

A simple payments engine, to accrue transactions into final account totals.

## Usage

To run:

```bash
cargo run -- actions.csv > output.csv
```

To test:

```bash
./check
```

## A brief word on terminology

As described in the challenge, the terminology conflates `transaction` (in the sense of an initial deposit/withdrawal),
and follow up actions such as a dispute, that reference that initial action.

For example, a `dispute` is described as a `transaction`, but does not have a unique transaction id.

For my own sanity, and to avoid confustion, the terms used in this implementation are:

- `action`: an entry from the input CSV describing one of: deposit/withdrawal/dispute/resolve/chargeback
  - `transfer`: an initial change of funds; one of: deposit/withdrawal
  - `update`: a state change pointing at an existing `transfer`; one of: dispute/resolve/chargeback

## Model

As described, the actions make up the following state machine:

```txt
                         dispute            chargeback
           ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
           │              ├───►              │   │              │
deposit ───► Transferred  │   │ Disputed     ├───► Chargebacked │
withdrawal │              ◄───┤              │   │              │
           └──────────────┘   └──────────────┘   └──────────────┘
                         resolve
```

Some optimisations fall out of this:

- We don't care about a deposit vs. a withdrawal for anything except the value sign. So let's just store it as the sign in a f64.
- Once a transfer reaches the state of `Chargebacked`, it is effectively dead.
  - There are no further actions we can take on it.
  - We only need to retain it, if we are to ignore duplicate transaction ids in the input/handle retries.
    - This could be a potential optimisation in future - can drop all state information, and just filter out the given ids from the input

Some annoying things:

- We have to retain all actions for all time. In reality, I imagine each transfer has a timestamp,
  and after a certain window (28 days ish?) the transfer is deemed valid forever.
  - Even if not valid forever, there could be some time-based archive to disk cache/cold storage etc.

## Design

### Initial Thoughts

Before beginning implementation, I have the following thoughts/assumptions:

- We need to maintain state for:
  - Each transaction: value, status
  - Each client: funds available, funds held, lock status
- For large volumes, we will be constrained by the amount of data we can fit in memory. Therefore, this should be optimised as much as possible.

### Precision

I used f64 everywhere to ensure the precision was reasonable. See [this blog post](https://blog.demofox.org/2017/11/21/floating-point-precision/) for a pretty nice overview.

f32 will only provide four decimal precision up to around 2^10 or ±1E3. f64 will provide four decimal precision up to around 2^39 or ±5E11. This corresponds to tens of trillions as a max value.

### Data sizes

As mentioned in the brief, `tx` is a u32. As we need to hold state for all transactions, that will be our resource limit.

If we were sure we would use all transaction ids, the most efficient memory model would be store a `Vec` where the index is the `tx`. This gives us a max size of about **44 GiB** (11 bytes per transfer), which would be just about doable on my 64GB RAM laptop:

```python
value = 8 # f64 is 8 bytes
state = 1 # enum stored as u8, single byte (can enforce with `#[repr(u8)]`
client_id = 2 # u16 is 2 bytes
per_tx = value + state + client_id
num_tx = 2 ** 32
total = per_tx * num_tx
total_gi = total / 1024**3
```

This is probably not a reasonable scenario. Let's say we use a large amount of data, such that `tx`
can be randomly generated without having collisions. The upper bound for this is something like
[sqrt-n](https://www.johndcook.com/blog/2017/01/10/probability-of-secure-hash-collisions) values, so
for a u32, let's say 2^16 values.

We also need to allow for storing:

- the key
- the overhead of the lookup structure (HashMap)
  - [1 byte metadata per key](https://www.reddit.com/r/rust/comments/prirpw/memory_efficient_hashmap/hdkjpsc/)
  - unused space in the container; this comment indicates [a factor of 11/10, then the next biggest power of 2](https://github.com/servo/servo/issues/6908#issuecomment-127729009).
    We can make a simple worst case assumption of a factor of 2.

This gives us **2 MiB** (32 bytes per transfer, inlucluding overhead):

```python
key = 4 # u32 is 4 bytes
value = 8 # f64 is 8 bytes
state = 1 # enum stored as u8, single byte (can enforce with `#[repr(u8)]`
client_id = 2 # u16 is 2 bytes
collection_metadata = 1
per_tx = key + value + state + client_id + collection_metadata
num_tx = 2 ** 16
collection_overhead = 2
total = (per_tx * num_tx) * collection_overhead
total_mi = total / 1024**2
total_per_tx = total / num_tx
```

Even assuming the overhead of a HashMap to store the relevant information in, that's still very tiny.

So we should be good to just store everything in memory!

Alternative thoughts if we can't fit everything in memory:

- run in memory, but sharded (horizontally by `tx`, for instance split into 16 shards where each cares about `tx`s with `tx % 16 = <my_shard_id>`)
- back a large buffer with disk using something like `memmap`
- use something like RocksDB for a very simple, fast database-in-a-file
  - see also Redis etc. though that will be significantly slower

### Scaling

Already mentioned in the data sizes section, for scaling I'd move to a horizontally sharded model.

If running as a microservice, I'd have:

- Pool of `read` workers, to accept an incoming CSV, deserialise, shard actions by `tx` and forward to:
- Pool of `ledger` workers to handle collating actions related to a range of `tx`s
- Small pool of `write` workers to handle returning the current state across many `ledger` workers
  - This would actually be kind of interesting in terms of consistency

### Other grab bag of notes

- Input data format is pretty inefficient (at the very least, `type` should be an enum of 0-4)
- Client id is pretty small? Only u16?

## Assumptions

Big list of assumptions I made:

- The purpose of the `client` field on the dispute/resolve/chargeback actions was not specified.
  As it is not required (only `tx` is required to uniquely refernce a transfer), I have ignored it.
- No error API was described for the program to implement.
  In an unrecoverable error state, the program will exit with code `1`.
  Logs including error messages will be printed to `stderr`.
- Invalid actions should be silently dropped from the input. This includes cases such as:
  - unknown action type
  - invalid value for field type
- There were no negative `amount` values in the data. I assumed that such a action is invalid (a negative withdrawal should be a deposit, and vv).
  - Amounts will be in the range `0 <= amount <= 1E11` (see precision above).
- Assumed it was fine to create clients and output their state, even if all actions failed (e.g. one `withdrawal` for a client, results in that client appearing in the ouput with funds of `0`).
- Assumed that we desire consistency in the results - i.e. they should be sorted.
  - I chose to back the underlying store with an `IndexMap`, rather than sorting a lot of data at output time.
    In practice these rows would be in a database, and you'd just have an index on the id.
- Transfers can be infinitely disputed, as long as they are resolved each time.

## Time taken

- 1:10:00 for writeup/design ahead of beginning implementation
- 2:30:00 for core implementation, such that the example input/ouput pair works
- 1:00:00 for additional tidy up, test coverage, some bug fixes
- 0:30:00 to change the design slightly to not trust the input data even more
