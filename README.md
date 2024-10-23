# Lottery Program

## Overview 

**Note**
This codebase as-is, is not fit for production use. Proper due diligence, including security audits and code reviews, is required before deploying it for use in a real-world scenario.

This repository contains a Solana smart contract (program) for a simple Lottery system written in Rust using the Anchor framework.

## Actors Involved
1. Lottery Creator (Authority):
- A user who initializes the master account and creates a new lottery.
- Sets the ticket price for the lottery.

2. Buyer:
- Users who purchase tickets for a lottery.
- Each ticket represents an entry into the lottery.

3. Lottery Program:
- The smart contract that manages lotteries, tickets, winner selection, and prize distribution.

## Details

- The Lottery Creator calls `init_master` on the Lottery Program to initialize the master account, which keeps track of lottery IDs.
- The Lottery Creator invokes `create_lottery(ticket_price)` to create a new lottery with a specified ticket price.
- A Buyer purchases a ticket by calling `buy_ticket(lottery_id)`, entering the lottery.
- The Lottery Creator calls `pick_winner(lottery_id)` to select a random winner from the tickets sold.
- The Winner (a Buyer) claims the prize by invoking `claim_prize(lottery_id, ticket_id)`.

```plaintext
+---------------------+        +-----------------+        +----------------------+
| Lottery Creator     |        |      Buyer      |        |   Lottery Program    |
+---------------------+        +-----------------+        +----------------------+
          |                         |                          |
          | init_master()           |                          |
          |------------------------>|                          |
          |                         |                          |
          |                         |                          |
          | create_lottery()        |                          |
          |------------------------>|                          |
          |                         |                          |
          |                         |                          |
          |                         |                          |
          |                         |   buy_ticket()           |
          |                         |------------------------->|
          |                         |                          |
          |                         |                          | Process Ticket Purchase
          |                         |                          |----------------------->
          |                         |                          | Create Ticket Account
          |                         |                          |<-----------------------
          |                         |                          |
          |                         |                          |
          |                         |                          |
          | pick_winner()           |                          |
          |------------------------>|                          |
          |                         |                          | Random Winner Selection
          |                         |                          |----------------------->
          |                         |                          | Update Lottery with Winner
          |                         |                          |<-----------------------
          |                         |                          |
          |                         |                          |
          |                         | claim_prize()            |
          |                         |------------------------->|
          |                         |                          | Validate Winner
          |                         |                          |----------------------->
          |                         |                          | Transfer Prize to Winner
          |                         |                          |<-----------------------
          |                         |                          |
```

## Requirements 

- pnpm v9.11.0
- rust v1.78.0
- node v20.17.0
- solana v1.18.25
- anchor v0.29.0

## Installation Steps

### Install Rust and Cargo
```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Cargo to your PATH
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Install Solana CLI Tools
```bash
# Install Solana CLI Tools (replace with the latest stable version if necessary)
sh -c "$(curl -sSfL https://release.solana.com/v1.18.25/install)"

# Add Solana to your PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verify installation
solana --version
```

### Install Anchor Framework
```bash
# Install Anchor Version Manager (AVM)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

# Install the latest version of Anchor
avm install latest
avm use latest

# Verify installation
anchor --version
```

### Build the Program
```bash
anchor build
```

### Deploy the Program
Deploy the program to the local cluster using - 
```bash
anchor deploy
```

## Note
- This codebase is not suitable for production use. It lacks essential security features and has not been audited.
- The method used for selecting a lottery winner is predictable and insecure. It uses the current timestamp and slot, which can be manipulated.
- Deploying this code as-is could lead to vulnerabilities and exploitation.

## TODO
- Replace the current pseudo-random number generator with a secure oracle solution or integrate with a Verifiable Random Function (VRF) to ensure fair and unpredictable winner selection.
- Develop comprehensive unit and integration tests to cover all functionalities, edge cases, and error handling.
- Perform a thorough security audit to identify and mitigate potential vulnerabilities.
- Review the codebase for optimization opportunities and adherence to best practices.
- Provide detailed documentation for each function, including parameters, expected behavior, and potential errors.