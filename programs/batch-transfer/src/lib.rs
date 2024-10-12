use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Epoch;
use anchor_lang::solana_program::program_error::ProgramError;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use std::collections::HashMap;

declare_id!("De1JkAKuuvfrMhKKai7u53w8Ap8ufvF2QPigQMSMTyEh");

#[program]
pub mod batch_transfer {
    use super::*;

    // 初始化合约账户
    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let bank_account = &mut ctx.accounts.bank_account;

        bank_account.admin = admin;

        Ok(())
    }

    // 设置手续费
    pub fn set_fee(ctx: Context<SetFee>, fee: u64) -> Result<()> {
        let bank_account = &mut ctx.accounts.bank_account;

        require!(
            ctx.accounts.admin.key() == bank_account.admin,
            ErrorCode::Unauthorized
        );

        bank_account.fee = fee;

        Ok(())
    }

    // // 提现合约中的 Native SOL
    // pub fn withdraw_native(ctx: Context<WithdrawNative>, amount: u64) -> Result<()> {
    //     let bank_account = &ctx.accounts.bank_account;
    //     let recipient = &ctx.accounts.recipient;
    //
    //     require!(
    //         bank_account.owner == ctx.accounts.owner.key(),
    //         ErrorCode::Unauthorized
    //     );
    //
    //     let contract_balance = **bank_account.to_account_info().lamports.borrow();
    //     require!(contract_balance >= amount, ErrorCode::InsufficientFunds);
    //
    //     **bank_account.to_account_info().lamports.borrow_mut() -= amount;
    //     **recipient.to_account_info().lamports.borrow_mut() += amount;
    //
    //     Ok(())
    // }
    //
    // // 提现合约中的 SPL Token
    // pub fn withdraw_spl_token(
    //     ctx: Context<WithdrawSPLToken>,
    //     amount: u64,
    // ) -> Result<()> {
    //     let bank_account = &ctx.accounts.bank_account;
    //
    //     require!(
    //         bank_account.owner == ctx.accounts.owner.key(),
    //         ErrorCode::Unauthorized
    //     );
    //
    //     let cpi_accounts = Transfer {
    //         from: ctx.accounts.contract_token_account.to_account_info(),
    //         to: ctx.accounts.recipient_token_account.to_account_info(),
    //         authority: ctx.accounts.bank_account.to_account_info(),
    //     };
    //
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::transfer(cpi_ctx, amount)?;
    //
    //     Ok(())
    // }


    // 批量转账 SOL 的函数
    // 键是接收者账户，值是转账金额
    // pub fn batch_transfer_sol(ctx: Context<BatchTransferSol>, transfers: HashMap<Pubkey, u64>) -> Result<()> {
    //     let sender = &ctx.accounts.sender;
    //     let bank_account = &ctx.accounts.bank_account;
    //
    //     // 求转账总金额
    //     let sum_ret = safe_sum(&transfers);
    //
    //     require!(
    //     sum_ret.is_ok(),
    //     ErrorCode::Overflow
    // );
    //     let total_amount: u64 = sum_ret.unwrap();
    //
    //     let fee = bank_account.fee;
    //
    //     let add_ret = safe_add(total_amount, fee);
    //
    //     require!(
    //     add_ret.is_ok(),
    //     ErrorCode::Overflow
    // );
    //
    //     // 校验 sender 账户是否有足够的 SOL 来支付所有转账金额和手续费
    //     let sender_balance = **sender.to_account_info().lamports.borrow();
    //     let required_balance = add_ret.unwrap();
    //
    //     require!(
    //     sender_balance >= required_balance,
    //     ErrorCode::InsufficientFunds
    // );
    //
    //     // 扣除手续费到合约地址
    //     **sender.to_account_info().lamports.borrow_mut() -= fee;
    //     **bank_account.to_account_info().lamports.borrow_mut() += fee;
    //
    //     // 进行批量转账
    //     for (recipient, amount) in transfers.iter() {
    //         let ix = anchor_lang::solana_program::system_instruction::transfer(
    //             &sender.key(),
    //             recipient,
    //             *amount,
    //         );
    //
    //         // 获取 sender 的 account_info 变量
    //         let sender_account_info = sender.to_account_info();
    //
    //         // 获取 recipient 的账户信息
    //         let recipient_account_info = next_account_info(&mut ctx.remaining_accounts.iter())?;
    //
    //         // 调用 Solana 的 transfer 系统指令
    //         anchor_lang::solana_program::program::invoke(
    //             &ix,
    //             &[
    //                 sender_account_info.clone(),
    //                 recipient_account_info.clone(),
    //             ],
    //         )?;
    //     }
    //
    //     Ok(())
    // }

    // pub fn transfer_sol(ctx: Context<TransferSol>) -> Result<()> {
    //     let from_account = &ctx.accounts.from;
    //     let system_program = &ctx.accounts.system_program;
    //     let bank_account = &mut ctx.accounts.bank_account;
    //
    //     // Transfer SOL to multiple 'to' accounts with specified amounts
    //     for (to_account, &amount) in ctx.accounts.to.iter() {
    //         let transfer_amount_ix = anchor_lang::solana_program::system_instruction::transfer(&from_account.key, &to_account.key(), amount);
    //         anchor_lang::solana_program::program::invoke(
    //             &transfer_amount_ix,
    //             &[
    //                 from_account.to_account_info(),
    //                 to_account.to_account_info(),
    //                 system_program.to_account_info(),
    //             ],
    //         )?;
    //     }
    //
    //     // Transfer fee to the bank account
    //     let transfer_fee_ix = anchor_lang::solana_program::system_instruction::transfer(&from_account.key, &bank_account.to_account_info().key, bank_account.fee);
    //     anchor_lang::solana_program::program::invoke(
    //         &transfer_fee_ix,
    //         &[
    //             from_account.to_account_info(),
    //             bank_account.to_account_info(),
    //             system_program.to_account_info(),
    //         ],
    //     )?;
    //
    //     // Emit the TransferEvent after successful transfers
    //     emit!(TransferEvent {
    //         from: from_account.key(),
    //         to: ctx.accounts.to.iter().map(|(a, _)| a.key()).collect(),
    //         total_amount: ctx.accounts.to.iter().map(|(_, amount)| amount).sum::<u64>(),
    //         fee: bank_account.fee,
    //     });
    //
    //     Ok(())
    // }

    // // 批量转账 SPL Token 的函数
    // pub fn batch_transfer_token(
    //     ctx: Context<BatchTransferToken>,
    //     transfers: HashMap<Pubkey, u64>, // 键是接收者账户，值是转账金额
    // ) -> Result<()> {
    //     let sender = &ctx.accounts.sender;
    //     let bank_account = &ctx.accounts.bank_account;
    //     let sender_token_account = &ctx.accounts.token_account;
    //     let mut total_amount: u64 = 0;
    //     let transfers_clone1 = transfers.clone();
    //     let transfers_clone2 = transfers.clone();
    //
    //     // 计算总的转账金额
    //     for amount in transfers_clone1.values() {
    //         total_amount = total_amount.checked_add(*amount).ok_or(ErrorCode::Overflow)?;
    //     }
    //
    //     let fee = bank_account.fee;
    //
    //     // 校验 SPL Token 余额是否足够
    //     let sender_token_balance = token::accessor::amount(&sender_token_account.to_account_info())?;
    //     require!(
    //         sender_token_balance >= total_amount,
    //         ErrorCode::InsufficientTokenBalance
    //     );
    //
    //     // 校验 sender 是否有足够的 SOL 支付手续费
    //     let sender_sol_balance = **sender.to_account_info().lamports.borrow();
    //     require!(
    //         sender_sol_balance >= fee,
    //         ErrorCode::InsufficientFundsForFee
    //     );
    //
    //     // 扣除手续费到合约地址
    //     **sender.to_account_info().lamports.borrow_mut() -= fee;
    //     **bank_account.to_account_info().lamports.borrow_mut() += fee;
    //
    //     let mut lamports = 0;
    //     let default_pubkey = Pubkey::default();
    //
    //     // 进行批量转账 SPL Token
    //     for (recipient, amount) in transfers_clone2.iter() {
    //         let cpi_accounts = Transfer {
    //             from: sender_token_account.to_account_info(),
    //             to: AccountInfo::new(
    //                 recipient,
    //                 false,
    //                 true,
    //                 &mut lamports,
    //                 &mut [],
    //                 &default_pubkey,
    //                 false,
    //                 Epoch::default(),
    //             ),
    //             authority: sender.to_account_info(),
    //         };
    //         let cpi_program = ctx.accounts.token_program.to_account_info();
    //         let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //
    //         token::transfer(cpi_ctx, *amount)?;
    //     }
    //
    //     Ok(())
    // }

    // 查询账户余额的函数
    // pub fn check_balance_sol(ctx: Context<CheckBalanceSol>) -> Result<u64> {
    //     let account_balance = **ctx.accounts.account.to_account_info().lamports.borrow();
    //     Ok(account_balance)
    // }

    // pub fn check_balance_token(ctx: Context<CheckBalanceToken>) -> Result<u64> {
    //     let token_balance = token::accessor::amount(&ctx.accounts.token_account.to_account_info())?;
    //     Ok(token_balance)
    // }
    pub fn simulate(ctx: Context<Simulate>) -> Result<()> {
        Ok(())
    }
}

#[account]
pub struct BankAccount {
    pub admin: Pubkey,
    pub fee: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = deployer, space = 8 + 32 + 8)]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub deployer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetFee<'info> {
    #[account(mut)]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

// #[derive(Accounts)]
// pub struct WithdrawNative<'info> {
//     #[account(mut)]
//     pub bank_account: Account<'info, BankAccount>,
//     #[account(mut)]
//     pub owner: Signer<'info>,
//     #[account(mut)]
//     pub recipient: SystemAccount<'info>,
// }
//
// #[derive(Accounts)]
// pub struct WithdrawSPLToken<'info> {
//     #[account(mut)]
//     pub bank_account: Account<'info, BankAccount>,
//     #[account(mut)]
//     pub contract_token_account: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub owner: Signer<'info>,
//     #[account(mut)]
//     pub recipient_token_account: Account<'info, TokenAccount>,
//     pub token_program: Program<'info, Token>,
// }

// #[derive(Accounts)]
// pub struct BatchTransferSol<'info> {
//     #[account(mut, signer)]
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     pub from: AccountInfo<'info>,
//     /// Each entry includes an AccountInfo for the recipient and the amount to transfer.
//     #[account(mut)]
//     pub to: Vec<(AccountInfo<'info>, u64)>,
//     #[account(mut)]
//     pub bank_account: Account<'info, BankAccount>,
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     pub system_program: AccountInfo<'info>,
// }

// #[derive(Accounts)]
// pub struct TransferSol<'info> {
//     #[account(mut, signer)]
//     pub from: AccountInfo<'info>,
//     #[account(mut)]
//     pub to: Vec<(AccountInfo<'info>, u64)>,  // Each 'to' account comes with its transfer amount
//     #[account(mut)]
//     pub bank_account: Account<'info, BankAccount>,
//     pub admin: Signer<'info>, // Admin who has permission to update fee
//     pub system_program: Program<'info, System>,
// }


// #[derive(Accounts)]
// pub struct BatchTransferToken<'info> {
//     #[account(mut)]
//     pub sender: Signer<'info>,
//     #[account(mut)]
//     pub bank_account: Account<'info, BankAccount>,
//     #[account(mut)]
//     pub token_account: Account<'info, TokenAccount>, // 发起者的 Token 账户
//     pub token_program: Program<'info, Token>,
//     pub system_program: Program<'info, System>,
// }

// #[derive(Accounts)]
// pub struct CheckBalanceSol<'info> {
//     #[account(mut)]
//     pub account: SystemAccount<'info>,
// }

#[derive(Accounts)]
pub struct Simulate<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
}


// #[derive(Accounts)]
// pub struct CheckBalanceToken<'info> {
//     #[account(mut)]
//     pub token_account: Account<'info, TokenAccount>,
// }

#[event]
pub struct TransferEvent {
    pub from: Pubkey,
    pub to: Vec<Pubkey>,
    pub total_amount: u64,
    pub fee: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow during calculation.")]
    Overflow,
    #[msg("Insufficient SOL funds to complete the transfer.")]
    InsufficientFunds,
    #[msg("Insufficient SPL Token balance to complete the transfer.")]
    InsufficientTokenBalance,
    #[msg("Insufficient SOL funds to cover the transfer fee.")]
    InsufficientFundsForFee,
    #[msg("Unauthorized access.")]
    Unauthorized,
}


// Helper function: 求和并检查溢出
fn safe_sum(transfers: &HashMap<Pubkey, u64>) -> std::result::Result<u64, ProgramError> {
    transfers.values().try_fold(0u64, |acc, &value| {
        acc.checked_add(value).ok_or(ProgramError::InvalidArgument)
    })
}

fn safe_add(a: u64, b: u64) -> std::result::Result<u64, ProgramError> {
    a.checked_add(b).ok_or(ProgramError::InvalidArgument)
}

