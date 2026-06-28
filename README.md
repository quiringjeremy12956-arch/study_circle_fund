# study_circle_fund

## Project Title
study_circle_fund

## Project Description
Study Circle Fund is a pooled-fund dApp that lets a small group of students chip in XLM (or other assets) toward a shared study budget, then submit and collectively approve spending requests for resources such as textbooks, lab fees, software licenses, or peer tutoring. The contract keeps an on-chain ledger of contributions, a per-circle balance, and a multi-sig approval workflow so no single member can drain the pot on their own.

## Project Vision
Make higher-education expenses fair and transparent by giving student study groups a tamper-proof treasury that anyone can verify. The long-term goal is to make it easy for clubs, dorms, online cohorts, and bootcamp teams to self-manage shared learning budgets without needing a campus bank account, a treasurer they have to trust blindly, or expensive legal paperwork.

## Key Features
- **Create study circles**: any student can spin up a named circle; the founder is auto-enrolled as the first member and a unique `circle_id` is returned.
- **Pooled contributions**: members contribute XLM-equivalent units to a circle; every contribution is recorded on-chain with the contributor's address, and per-member totals are tracked for future governance weight.
- **Spending requests**: any member can submit a request with a stated purpose (e.g. `textbooks`, `lab_fee`, `tutoring`) and an amount; the request is queued with a configurable approval threshold before any funds are debited.
- **Member-driven governance**: any circle member other than the requester can approve a pending request; once enough approvals are collected, the request auto-executes and the circle's balance is debited. The requester is blocked from self-approving and a member can only approve once.
- **Read-only helpers**: inspect circle balance, individual member contributions, membership status, and request status (approvals so far, threshold, executed flag) without sending a transaction.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** education dApp — see `contracts/study_circle_fund/src/lib.rs` for the full study_circle_fund business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CAQCM7JAKTZRAUF3SMD2XWUNVINF2ORUINXDQY7FRAJIVFXSPZDGFZ2V`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/429052456e9ae0966752f456174eb775098d4e2281f641a7e4594efa76662c1a`

## Future Scope
- **Real XLM transfers**: integrate the Stellar Asset Contract (SAC) so the circle balance becomes a genuine on-chain spendable pool, with `contribute` and `approve_disbursement` actually moving XLM instead of only updating a logical ledger.
- **Pluggable approval policies**: switch from a simple fixed-threshold count to weighted voting based on contribution share, time-locked approvals, or role-based signers (e.g. only senior members can approve tutoring payments).
- **Frontend dashboard + event indexer**: a small React/Next.js UI that surfaces circle balances, pending requests, and approval status for non-technical members, fed by an off-chain indexer listening to the contract's `circle_created`, `contributed`, `disbursement_requested`, and `disbursement_approved` events.
- **Optional yield on idle balances**: park unspent circle balances in a low-risk Stellar DEX liquidity pool so the treasury earns a small return while waiting to be spent on resources.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `study_circle_fund` (education)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
