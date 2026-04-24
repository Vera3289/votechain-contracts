# JavaScript Examples — Governance Contract

These examples use `@stellar/stellar-sdk` to interact with a deployed VoteChain governance contract on Stellar Testnet.

---

## Setup

```bash
npm install @stellar/stellar-sdk
```

```js
import {
  Contract,
  Keypair,
  Networks,
  SorobanRpc,
  TransactionBuilder,
  nativeToScVal,
  xdr,
} from "@stellar/stellar-sdk";

const RPC_URL = "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = Networks.TESTNET;
const GOVERNANCE_CONTRACT_ID = "C..."; // from config/testnet.toml

const server = new SorobanRpc.Server(RPC_URL);
const contract = new Contract(GOVERNANCE_CONTRACT_ID);
const keypair = Keypair.fromSecret("S..."); // signer

/** Build, simulate, sign, and submit a contract call. */
async function invoke(operation) {
  const account = await server.getAccount(keypair.publicKey());
  const tx = new TransactionBuilder(account, {
    fee: "100",
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const simResult = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }

  const preparedTx = SorobanRpc.assembleTransaction(tx, simResult).build();
  preparedTx.sign(keypair);

  const sendResult = await server.sendTransaction(preparedTx);
  if (sendResult.status === "ERROR") throw new Error(sendResult.errorResult);

  // Poll for confirmation
  let response;
  do {
    await new Promise((r) => setTimeout(r, 1000));
    response = await server.getTransaction(sendResult.hash);
  } while (response.status === "NOT_FOUND");

  return response;
}
```

---

## initialize

```js
async function initialize(adminAddress, tokenContractId) {
  const op = contract.call(
    "initialize",
    nativeToScVal(adminAddress, { type: "address" }),
    nativeToScVal(tokenContractId, { type: "address" })
  );
  return invoke(op);
}

// Usage
await initialize(keypair.publicKey(), "C...<TOKEN_CONTRACT_ID>");
```

---

## create_proposal

```js
async function createProposal(proposerKeypair, title, description, quorum, durationSeconds) {
  const op = contract.call(
    "create_proposal",
    nativeToScVal(proposerKeypair.publicKey(), { type: "address" }),
    nativeToScVal(title, { type: "string" }),
    nativeToScVal(description, { type: "string" }),
    nativeToScVal(quorum, { type: "i128" }),
    nativeToScVal(durationSeconds, { type: "u64" })
  );
  return invoke(op);
}

// Usage — quorum of 1000 tokens, 7-day voting window
await createProposal(
  keypair,
  "Increase treasury allocation",
  "Allocate 10% more to the dev fund",
  1000n,
  604800n
);
```

---

## cast_vote

```js
// Vote enum: { Yes: null } | { No: null } | { Abstain: null }
function voteScVal(env, choice) {
  const variants = { Yes: 0, No: 1, Abstain: 2 };
  return xdr.ScVal.scvVec([
    xdr.ScVal.scvSymbol(choice),
  ]);
}

async function castVote(voterKeypair, proposalId, vote) {
  const voteVal = xdr.ScVal.scvVec([xdr.ScVal.scvSymbol(vote)]); // "Yes" | "No" | "Abstain"
  const op = contract.call(
    "cast_vote",
    nativeToScVal(voterKeypair.publicKey(), { type: "address" }),
    nativeToScVal(proposalId, { type: "u64" }),
    voteVal
  );
  return invoke(op);
}

// Usage
await castVote(keypair, 1n, "Yes");
```

---

## finalise

```js
async function finalise(proposalId) {
  const op = contract.call(
    "finalise",
    nativeToScVal(proposalId, { type: "u64" })
  );
  return invoke(op);
}

// Usage — call after the voting period has ended
await finalise(1n);
```

---

## execute

```js
async function execute(adminKeypair, proposalId) {
  const op = contract.call(
    "execute",
    nativeToScVal(adminKeypair.publicKey(), { type: "address" }),
    nativeToScVal(proposalId, { type: "u64" })
  );
  return invoke(op);
}

// Usage — proposal must be in Passed status
await execute(keypair, 1n);
```

---

## cancel

```js
async function cancel(adminKeypair, proposalId) {
  const op = contract.call(
    "cancel",
    nativeToScVal(adminKeypair.publicKey(), { type: "address" }),
    nativeToScVal(proposalId, { type: "u64" })
  );
  return invoke(op);
}

// Usage — proposal must be Active
await cancel(keypair, 1n);
```

---

## Read-only calls

```js
/** Returns the full Proposal object for a given ID. */
async function getProposal(proposalId) {
  const result = await server.simulateTransaction(
    new TransactionBuilder(await server.getAccount(keypair.publicKey()), {
      fee: "100",
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(contract.call("get_proposal", nativeToScVal(proposalId, { type: "u64" })))
      .setTimeout(30)
      .build()
  );
  return result.result?.retval;
}

/** Returns true if the voter has already voted on the proposal. */
async function hasVoted(proposalId, voterAddress) {
  const result = await server.simulateTransaction(
    new TransactionBuilder(await server.getAccount(keypair.publicKey()), {
      fee: "100",
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        contract.call(
          "has_voted",
          nativeToScVal(proposalId, { type: "u64" }),
          nativeToScVal(voterAddress, { type: "address" })
        )
      )
      .setTimeout(30)
      .build()
  );
  return result.result?.retval;
}
```

---

## Notes

- Replace `"C..."` and `"S..."` with values from `config/testnet.toml` and your funded testnet keypair.
- Fund a testnet account at [https://friendbot.stellar.org](https://friendbot.stellar.org/?addr=<YOUR_ADDRESS>).
- The `invoke` helper handles simulation, fee estimation, signing, and polling in one call.
- Vote enum values must be passed as `ScvVec([ScvSymbol("Yes")])` to match the Soroban contract type.
