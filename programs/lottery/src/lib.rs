// Import necessary modules and traits from the anchor_lang crate
use anchor_lang::{
    prelude::*, // Brings common types like AccountInfo, ProgramResult, etc., into scope
    solana_program::{clock::Clock, hash::hash, program::invoke, system_instruction::transfer}, // Importing specific Solana program modules
};

// Import constants and error definitions
mod constants;
mod error;
use crate::{constants::*, error::*};

// Declare the program ID - this is the unique address of this smart contract program
declare_id!("FpDJiceCWU5Zdyd8arskS9fvpZY9kzypC4q3Ak6jadmB");

// Define the lottery module, which will contain all program instructions
#[program]
mod lottery {
    use super::*; // Bring all items from the parent module into the scope of the current module

    // Function to initialize the master account
    // The master account keeps track of the last lottery ID
    pub fn init_master(_ctx: Context<InitMaster>) -> Result<()> {
        // This function currently doesn't do anything beyond successful initialization
        Ok(()) // Return an Ok result to indicate success
    }

    // Function to create a new lottery
    // Initializes a lottery account and sets up its parameters
    pub fn create_lottery(ctx: Context<CreateLottery>, ticket_price: u64) -> Result<()> {
        // Create a lottery account to hold information about the current lottery
        let lottery = &mut ctx.accounts.lottery; // Get a mutable reference to the lottery account
        let master = &mut ctx.accounts.master; // Get a mutable reference to the master account

        // Set up the lottery account with relevant details
        lottery.id = master.last_id; // Assign the new lottery ID
        lottery.authority = ctx.accounts.authority.key(); // Set the authority for the lottery
        lottery.ticket_price = ticket_price; // Set the price for lottery tickets

        // Increment the last lottery ID stored in the master account
        master.last_id += 1;

        // Log information about the newly created lottery
        msg!("Lottery with ID : {}", lottery.id);
        msg!("Authority: {}", lottery.authority);
        msg!("Lottery ticket price: {}", lottery.ticket_price);

        Ok(()) // Return an Ok result to indicate success
    }

    // Function to buy a ticket for a lottery
    // Creates a ticket account and transfers the ticket price to the lottery PDA
    pub fn buy_ticket(ctx: Context<BuyTicket>, lottery_id: u32) -> Result<()> {
        // Get references to the accounts involved
        let lottery = &mut ctx.accounts.lottery;
        let ticket = &mut ctx.accounts.ticket;
        let buyer = &mut ctx.accounts.buyer;

        // Check if a winner already exists, return an error if so
        if lottery.winner_id.is_some() {
            return err!(LotteryError::WinnerAlreadyExists);
        }

        // Transfer SOL from the buyer to the lottery account using a system instruction
        invoke(
            &transfer(&buyer.key(), &lottery.key(), lottery.ticket_price),
            &[
                buyer.to_account_info(),
                lottery.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        ticket.id = lottery.last_ticket_id;
        ticket.lottery_id = lottery_id;
        ticket.authority = buyer.key();

        // Increment the last ticket ID and assign it to the new ticket
        lottery.last_ticket_id += 1;

        // Log information about the newly created ticket
        msg!("Ticket ID: {}", ticket.id);
        msg!("Ticket authority: {}", ticket.authority);

        Ok(()) // Return an Ok result to indicate success
    }

    // Function to select a winner for the lottery
    pub fn pick_winner(ctx: Context<PickWinner>, _lottery_id: u32) -> Result<()> {
        // Get a mutable reference to the lottery account
        let lottery = &mut ctx.accounts.lottery;

        // Check if a winner has already been selected
        if lottery.winner_id.is_some() {
            return err!(LotteryError::WinnerAlreadyExists);
        }

        // Check if there are any tickets purchased
        if lottery.last_ticket_id == 0 {
            return err!(LotteryError::NoTickets);
        }

        // Retrieve the current clock data from the Solana runtime
        let clock = Clock::get()?;

        // Generate a pseudo-random number based on the current timestamp and slot
        // Note: This method is deterministic and predictable, and should be replaced by a secure random number generator (e.g., an oracle)
        let pseudo_random_number = ((u64::from_le_bytes(
            <[u8; 8]>::try_from(&hash(&clock.unix_timestamp.to_be_bytes()).to_bytes()[..8])
                .unwrap(),
        ) * clock.slot)
            % u32::MAX as u64) as u32;

        // Calculate the winner ticket ID
        // The '+1' ensures the winner_id is within the range of ticket IDs (1 to last_ticket_id)
        let winner_id = (pseudo_random_number % lottery.last_ticket_id) + 1;

        // Set the winner_id in the lottery account
        lottery.winner_id = Some(winner_id);

        // Log the winner ID
        msg!("Winner id: {}", winner_id);
        Ok(())
    }

    // Function for the winner to claim the price
    pub fn claim_price(ctx: Context<ClaimPrice>, _lottery_id: u32, _ticket_id: u32) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery; // Get a mutable reference to the lottery account
        let ticket = &mut ctx.accounts.ticket; // Get a mutable reference to the ticket account
        let winner = &mut ctx.accounts.authority; // Get a mutable reference to the winner's account

        // Check if the price has already been claimed
        if lottery.claimed {
            return err!(LotteryError::AlreadyClaimed);
        }

        // Check if the ticket ID matches the winner ID
        match lottery.winner_id {
            Some(winner_id) => {
                if winner_id != ticket.id {
                    return err!(LotteryError::InvalidWinner);
                }
            }
            None => return err!(LotteryError::WinnerNotChosen),
        }

        // Calculate the total price amount
        let price = lottery
            .ticket_price
            .checked_mul(lottery.last_ticket_id.into())
            .unwrap();

        // Transfer the price amount from the lottery account to the winner's account
        **lottery.to_account_info().try_borrow_mut_lamports()? -= price;
        **winner.to_account_info().try_borrow_mut_lamports()? += price;

        // Mark the price as claimed
        lottery.claimed = true;

        // Log the price claim
        msg!(
            "{} claimed {} lamports from lottery id {} with ticket id {}",
            winner.key(),
            price,
            lottery.id,
            ticket.id
        );
        Ok(())
    }
}

// Define the accounts context for the `init_master` function
// Specifies the accounts that need to be provided to this instruction
#[derive(Accounts)]
pub struct InitMaster<'info> {
    // Define the master account, which is initialized here
    #[account(
        init, // This attribute indicates that this account is being initialized
        payer = payer, // The payer is responsible for covering the fees for creating this account
        space = 4 + 8, // Allocate enough space for the account (4 bytes for u32 + 8 bytes for extra space)
        seeds = [MASTER_SEED.as_bytes()], // Use MASTER_SEED as the seed for generating a program-derived address (PDA)
        bump, // The bump seed used to create a valid PDA; prevents collision
    )]
    pub master: Account<'info, Master>, // Define the master account of type `Master`

    // Define the payer account, which will pay the rent for initializing the master account
    #[account(mut)] // `mut` means this account is mutable (its balance can change)
    pub payer: Signer<'info>, // The signer is the account that authorizes this transaction

    // Reference to the system program, used to interact with Solana's native features
    pub system_program: Program<'info, System>,
}

// Define the data structure that will be stored in the master account
#[account]
pub struct Master {
    pub last_id: u32, // Track the last lottery ID created (4 bytes for a u32 integer)
}

// Define the accounts context for the `create_lottery` function
// Specifies the accounts that need to be provided to this instruction
#[derive(Accounts)]
pub struct CreateLottery<'info> {
    // Define the lottery account, which is initialized here
    #[account(
        init, // This attribute indicates that this account is being initialized
        payer = authority, // The authority is responsible for covering the fees for creating this account
        space = 8 + 4 + 32 + 8 + 4 + 1 + 4 + 1, // Allocate enough space for the account (total 62 bytes)
        // 8 +  // Account discriminator
        // 4 +  // id: u32
        // 32 + // authority: Pubkey
        // 8 +  // ticket_price: u64
        // 4 +  // last_ticket_id: u32
        // 1 + 4 + // winner_id: Option<u32> (1 byte for option tag + 4 bytes for u32)
        // 1;   // claimed: bool
        seeds = [LOTTERY_SEED.as_bytes(), &master.last_id.to_le_bytes()], // Use LOTTERY_SEED and current last_id as seeds for generating a PDA
        bump, // The bump seed used to create a valid PDA; prevents collision
    )]
    pub lottery: Account<'info, Lottery>, // Define the lottery account of type `Lottery`

    // Define the master account, which keeps track of the lottery ID
    #[account(
        mut, // The master account is mutable, as the lottery ID will be updated
        seeds = [MASTER_SEED.as_bytes()], // Use MASTER_SEED as the seed for generating the PDA
        bump, // The bump seed used to create a valid PDA
    )]
    pub master: Account<'info, Master>, // Define the master account of type `Master`

    // Define the authority account, which will be responsible for managing the lottery
    #[account(mut)] // The authority account is mutable (e.g., its balance can change)
    pub authority: Signer<'info>, // The signer is the account that authorizes this transaction

    // Reference to the system program, used to interact with Solana's native features
    pub system_program: Program<'info, System>,
}

// Define the data structure that will be stored in the lottery account
#[account]
pub struct Lottery {
    pub id: u32,                // The ID of the lottery (4 bytes for a u32 integer)
    pub authority: Pubkey,      // The public key of the authority managing the lottery (32 bytes)
    pub ticket_price: u64,      // The price of a lottery ticket (8 bytes for a u64 integer)
    pub last_ticket_id: u32,    // The ID of the last issued ticket (4 bytes for a u32 integer)
    pub winner_id: Option<u32>, // The ID of the winning ticket, if any (wrapped in Option)
    pub claimed: bool,          // Indicates whether the price has been claimed (1 byte for a boolean)
}

// Define the accounts context for the `buy_ticket` function
// Specifies the accounts that need to be provided to this instruction
#[derive(Accounts)]
#[instruction(lottery_id: u32)]
pub struct BuyTicket<'info> {
    // Define the lottery account, which the ticket will be associated with
    #[account(
        mut, // The lottery account is mutable, as the last_ticket_id will be updated
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()], // Use LOTTERY_SEED and lottery_id as seeds for generating the PDA
        bump, // The bump seed used to create a valid PDA
    )]
    pub lottery: Account<'info, Lottery>, // Define the lottery account of type `Lottery`

    // Define the ticket account, which is initialized here
    #[account(
        init, // This attribute indicates that this account is being initialized
        payer = buyer, // The buyer is responsible for covering the fees for creating this account
        space = 8 + 4 + 32 + 4, // Allocate enough space for the account (total 48 bytes)
        // 8 +  // Account discriminator
        // 4 +  // id: u32
        // 32 + // authority: Pubkey
        // 4;   // lottery_id: u32
        seeds = [
            TICKET_SEED.as_bytes(), // Use TICKET_SEED as part of the seed for generating a PDA
            lottery.key().as_ref(), // Include the lottery key as part of the seed
            &lottery.last_ticket_id.to_le_bytes(), // Include the current last_ticket_id as part of the seed
        ],
        bump, // The bump seed used to create a valid PDA
    )]
    pub ticket: Account<'info, Ticket>, // Define the ticket account of type `Ticket`

    // Define the buyer account, which will purchase the ticket
    #[account(mut)] // The buyer account is mutable (e.g., its balance will be deducted)
    pub buyer: Signer<'info>, // The signer is the account that authorizes this transaction

    // Reference to the system program, used to interact with Solana's native features
    pub system_program: Program<'info, System>,
}

// Define the data structure that will be stored in the ticket account
#[account]
pub struct Ticket {
    pub id: u32,           // The ID of the ticket (4 bytes for a u32 integer)
    pub authority: Pubkey, // The public key of the ticket owner (32 bytes)
    pub lottery_id: u32,   // The ID of the lottery that this ticket belongs to (4 bytes for a u32 integer)
}

// Define the accounts context for the `pick_winner` function
// Specifies the accounts that need to be provided to this instruction
#[derive(Accounts)]
#[instruction(lottery_id: u32)]
pub struct PickWinner<'info> {
    // Define the lottery account, which will have its winner_id updated
    #[account(
        mut, // The lottery account is mutable, as the winner_id will be set
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()], // Use LOTTERY_SEED and lottery_id as seeds for generating the PDA
        bump, // The bump seed used to create a valid PDA
        has_one = authority // Ensure that the authority is the same as the lottery's authority
    )]
    pub lottery: Account<'info, Lottery>, // Define the lottery account of type `Lottery`

    // Define the authority account, which must sign the transaction
    pub authority: Signer<'info>, // The signer is the account that authorizes this transaction
}

// Define the accounts context for the `claim_price` function
// Specifies the accounts that need to be provided to this instruction
#[derive(Accounts)]
#[instruction(lottery_id: u32, ticket_id: u32)]
pub struct ClaimPrice<'info> {
    // Define the lottery account from which the price will be claimed
    #[account(
        mut, // The lottery account is mutable, as lamports will be deducted
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>, // Define the lottery account of type `Lottery`

    // Define the ticket account that must match the winning ticket
    #[account(
        seeds = [
            TICKET_SEED.as_bytes(),
            lottery.key().as_ref(),
            &ticket_id.to_le_bytes()
        ],
        bump,
        has_one = authority, // Ensure that the authority is the owner of this ticket
    )]
    pub ticket: Account<'info, Ticket>, // Define the ticket account of type `Ticket`

    // Define the authority account, which must be the winner
    #[account(mut)] // The authority account is mutable (e.g., its balance will increase)
    pub authority: Signer<'info>, // The signer is the account that authorizes this transaction

    // Reference to the system program, used to interact with Solana's native features
    pub system_program: Program<'info, System>,
}
