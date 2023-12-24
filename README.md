
# Kelili

**Kelili** is an experimental **decentralized computer**.

`Kelili = Kindelia ∩ Nano`

Kelili is also **very tiny** (node core is about 170LOC right now without counting DHT implementation), to make experimentation and tweaking easier.

Kelili programs run on Lua right now, but they will run on **HVM**.

**Kelili does not have a consensus mechanism**. All valid decentralized blocks are part of the computer. Smart contracts that need a "canonical" fork in some situations, such as cryptocurrencies, will have to implement a consensus algorithm in the smart contract. This makes the Kelili core as small as possible.

**Kelili does not have stateful contracts**. State can be "faked" by building a chain of blocks, each one depending on the return value of the previous one (as in Bitcoin).

**Kelili does not have a native cryptocurrency**. Each smart contract is responsible for making sure that there exists an incentive for nodes to run it.
 
**Kelili allows for feeless cryptocurrencies**. The Nano cryptocurrency, for example, could be built on top of Kelili. `call` can be used to implement both `link` and the parent-child relationship, while `mark` can be used to ensure Nano blocks only receive funds from other Nano blocks

And lastly, **Kelili is an experiment**. I don't expect Kelili to be used in production. I created it to test if a cryptocomputer like Kelili is possible. I'm not an expert, and this is my first time developing anything blockchain-related.

## How to use

Edit the files in the `lua/` directory. `lua/script.lua` is the "main" file. It contains code that creates blocks and sends them to the node to be run. Code in `lua/script.lua` is "off-chain".

## API

Kelili's API surface is very small. There are only **four** interactions with the external environment. What follows is Rust pseudocode describing the API

```
// A block. T is the return value of the contained code.
struct Block<T> {
  // Maximimum amount of computation that can be done by this block
  mana_limit: u64,
  // Maximimum amount of memory that can be used by this block
  memo_limit: u64,
  contract: IO<T>
}

enum IO<T> {
  // There are two `identity` environment calls
  // Build a function that "marks" an arbitrary object, turning it into an opaque userdata object carrying the object and the hash of the current block.
  Mark { cont: Fn(Fn(U) -> Marked<U>) -> (IO T)}
  // Deconstruct a marked userdata object.
  // When `marked` is not a marked userdata object, 
  // hash is set to nil and U is set to `marked`
  Open { marked: Marked<U>, cont: Fn(U, Hash) -> (IO T) }

  // There are two `execution` environment calls
  // Run the block with hash `hash`, and call `cont` with its return value.
  Call { hash: Hash, mana_limit: u64,
  memo_limit: u64, cont: Fn(Output<hash>) -> IO<T> },
  // Finish execution of the smart contract and return `value`
  Done { value: T },
}
```
### Networking API

Kelili currently does not have a network protocol, or a way for peers to communicate with each other. Right now, all commucation is done via `tokio` channels. However, it should not be hard to write a wrapper around the DHT implementation to allow for inter-network communication.

### On block size

`Call` can be used as an equivalent to `#include` statement. This allows large blocks to be split into many tiny blocks. If these tiny blocks are less than `512` bytes long, then they could be sent as UDP packets, which would greatly increase the cryptocomputer's speed.

## TODO list

- Some smart contracts are not pure. Fix that.
- Deep clone cached data
- Complete the DHT implementation
 - Forget about dead nodes
 - Trust layer or something similar to prevent Sybil attacks.
- More examples
 - Currency that can be minted with PoW
 - Anonymous cryptocurrency
 - Exchanges
 - Currency with voting.

# Examples
## CatCoin

CatCoin is a cryptocurrency that runs on Kelili, based on Nano. Each account has a separate blockchain. The smart contract code is in `lua/blocks/catcoin.lua`. It returns a function which when given a public key, creates a new account with that public key. The Account object has the following type:

```
pub enum Message {
  Send { dest: U256, amount: u64, signature: U256 },
  Receive { from: U256, amount: u64 },
}

enum AccountResult {
  Ok { transaction: Message, update: Account }
  Error { message: Any }
}

Account = Fn(msg: Message) -> AccountResult
```
The returned `AccountResult.update` of the account is the new state of the account after the transaction has been carried out. `error` will be returned when verifying the transaction fails.

CatCoin marks and returns a transaction after verifying it. Each `Receive` message references a block with a `Send` transaction that must be marked by the original CatCoin smart contract. This is an equivalent to Nano's `link`.

Right now, each new account is gifted 100 CatCoin. This is an example, and it's obviously easy to abuse, so as an alternative, a pre-mine can be set up, or a PoW transaction that generates money.