# akropolisos-chain-node

A new SRML-based Substrate node, ready for hacking.

# Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build the WebAssembly binary:

```bash
./scripts/build.sh
```

Build all native code:

```bash
cargo build
```

# Run

You can start a full node in AkropolisOS chain with:

```bash
cargo run -- --name node-name
```

You can start a validator node in AkropolisOS chain with:

```bash
cargo run -- --name node-name --validator
```

# Development

You can start a development chain with:

```bash
cargo run -- --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units. Give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet). You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.


# How it works

## Account creation


This guide will walk you through how to create account and how to connect to AkropolisOSChain Testnet.

1) Open [Akropolis UI](https://wallet.akropolis.io) (it’s polkadotJS app working with substrate v.1.0). You can also use [Polkadot UI](https://polkadot.js.org/apps/#/explorer).

2) Go to *Settings*, open *Developer* tab. Insert in textbox description of types (copy&paste from here) and Save it.


```bash

{
    "Count": "u64",
    "DaoId": "u64",
    "MemberId": "u64",
    "ProposalId": "u64",
    "VotesCount": "MemberId",
    "TokenId": "u32",
    "Days": "u32",
    "Rate": "u32",
    "Dao": {
      "address": "AccountId",
      "name": "Text",
      "description": "Bytes",
      "founder": "AccountId"
    },
    "Action": {
      "_enum": {
        "EmptyAction": null,
        "AddMember": "AccountId",
        "RemoveMember": "AccountId",
        "GetLoan": "(Vec<u8>, Days, Rate, Balance)",
        "Withdraw": "(AccountId, Balance, Vec<u8>)",
        "ChangeTimeout": "(DaoId, BlockNumber)",
        "ChangeMaximumNumberOfMembers": "(DaoId, MemberId)"
      }
    },
    "Proposal": {
      "dao_id": "DaoId",
      "action": "Action",
      "open": "bool",
      "accepted": "bool",
      "voting_deadline": "BlockNumber",
      "yes_count": "VotesCount",
      "no_count": "VotesCount"
    },
    "Token": {
      "token_id": "u32",
      "decimals": "u16",
      "symbol": "Vec<u8>"
    },
    "Limits": {
      "max_tx_value": "u128",
      "day_max_limit": "u128",
      "day_max_limit_for_one_address": "u128",
      "max_pending_tx_limit": "u128",
      "min_tx_value": "u128"
    },
    "Status": {
        "_enum":[
          "Revoked",
          "Pending",
          "PauseTheBridge",
          "ResumeTheBridge",
          "UpdateValidatorSet",
          "UpdateLimits",
          "Deposit",
          "Withdraw",
          "Approved",
          "Canceled",
          "Confirmed"
        ]
    },
    "Kind" :{
      "_enum":[
      "Transfer",
      "Limits",
      "Validator",
      "Bridge"
      ]
    },
      "TransferMessage": {
        "message_id": "H256",
        "eth_address": "H160",
        "substrate_address": "AccountId",
        "amount": "Balance",
        "status": "Status",
        "direction": "Status"
    },
      "LimitMessage": {
        "id": "H256",
        "limits": "Limits",
        "status": "Status"
    },
      "BridgeMessage": {
        "message_id": "H256",
        "account": "AccountId",
        "status": "Status",
        "action": "Status"
    },
      "ValidatorMessage": {
        "message_id": "H256",
        "quorum":"u64",
        "accounts": "Vec<AccountId>",
        "status": "Status",
        "action": "Status"
    },
    "BridgeTransfer": {
      "transfer_id": "ProposalId",
      "message_id": "H256",
      "open": "bool",
      "votes": "MemberId",
      "kind": "Kind"
    }
  }



```


3) If you use [Akropolis UI](https://wallet.akropolis.io) skip this step, and go to the step 4. If you use [Polkadot UI](https://polkadot.js.org/apps/#/explorer), go to *Settings' General* tab, choose *custom endpoint* (top right corner), and set:

- remote node/endpoint to connect to: wss://node1-chain.akropolis.io or wss://node2-chain.akropolis.io,

- address prefix: Default for the connected node,

- default interface theme: Substrate,

- interface operation mode: Fully featured.

Then push to *Save&Reload* button.

4) Create Account:

- Navigate to the *Accounts* tab and click on the *Add account* button.

- Enter a name for your account and create a secure password. This password will be used to decrypt your account.

- Click *Save* and *Create and backup account*.

- Save your encrypted keystore locally.

- The account now appears in your *Accounts* tab and is backed up to the keystore you just saved.

5) Fill [the form](https://forms.gle/QjcccF6WWxSrbe9Z7) to get test AKRO tokens.

##Staking

This guide will walk you through how to nominate your AKROs to a validator node so that you can take part in the staking system.

We will assume that you will be starting with two fresh accounts. Click [here](https://wiki.polkadot.network/en/latest/polkadot/learn/staking/#accounts) to learn more about what stash and controller accounts mean.

1) The first step is to create two accounts by going to the *Accounts* tab. Make sure to use *stash* and *controller* in the names of your accounts to identify them easily.

2) Once you've created your accounts you will need to acquire some AKROs. Each of your accounts should have at least 150 milliAKROs to cover the existential deposit and transaction fees.

To nominate and validate follow [this instructions](https://wiki.polkadot.network/en/latest/polkadot/node/guides/how-to-nominate/#nominating).



## Working with DAOs


### Creation of DAOs

For creation of DAO you will need account with some AKROs.

1) Go to *Extrinsics* tab, select in *using the selected account* your account address.

2) Select "dao" in *submit the following extrinsic*

3) Insert *name* of new DAO and it's *description* in the HEX format. Use [utility](https://www.rapidtables.com/convert/number/ascii-to-hex.html) to convert ASCII symbols to HEX (please remove space symbols). Dao's name should have only "a" - "z", "A" - "Z", "0" - "9", "_" and "-" symbols. Length of DAO's name is between 10 and 255 symbols, length of description is between 10 and 4096 symbols.

3) Click *Submit* button

After DAO is created you will see DAO page with minimal balance:


4) See DAO stats you can in *Chain state* tab. Select *dao* in *selected state query* and select what kind of data you want to get:

- daosCount(): number of DAOs

- daos(DaoId): get information about DAO. DaoId is a number, starts from 0.

- membersCount(): number of members in DAO

- members(DaoId, MemberId): infromation about DAO member, where DaoId and MemberId is a numbers-identifiers.

### Add new members to DAO

Adding new members to DAO works through voting. To start voting you should make a proposal to add candidate. Candidate needs an account with some AKROs. This account should not be a member of this DAO to do a proposal.

1) Go to 'Extrinsics' tab and insert candidate's address to "using the selected account", select "dao" in "submit the following extrinsic" and "proposeToAddMemeber(dao_id)" function. Then insert dao id and click "Submit Transaction".

2) Check the status of proposal you can in *Chain state* tab. Select *dao* in *selected state query*.

- daoProposalsCount(DaoId) will show number of existing proposals

- daoProposals(DaoId, ProposalId) will show status of proposal ProposalId in DAO DaoId: open:true/false, voting_deadline - block number when voting is over, yes_count & no_count - number of DAO members voted yes or no for proposal).

### Remove member from DAO

Excluding DAO member happens through voting. Only existing DAO members can be removed from DAO. If DAO has only one member, this member can't be removed from DAO.

1) Go to 'Extrinsics' tab and insert candidate's address to "using the selected account", select "dao" in "submit the following extrinsic" and "proposeToRemoveMemeber(dao_id)" function. Then insert dao id and click "Submit Transaction".

2) Check the status of proposal you can in *Chain state* tab. Select *dao* in *selected state query*.

- daoProposalsCount(DaoId) will show number of existing proposals

- daoProposals(DaoId, ProposalId) will show status of proposal ProposalId in DAO DaoId: open:true/false, voting_deadline - block number when voting is over, yes_count & no_count - number of DAO members voted yes or no for proposal).



### Voting

Only DAO member can take participation in voting (one time for proposal).

To take participation in voting go to 'Extrinsics' tab and insert your address to "using the selected account", select "dao" in "submit the following extrinsic" and "vote(dao_id, proposal_id, vote)" function, where vote is boolean (Yes/No).  Then insert dao id and click "Submit Transaction".
