use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_error::ProgramError;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("De1JkAKuuvfrMhKKai7u53w8Ap8ufvF2QPigQMSMTyEh");

#[program]
pub mod batch_transfer {
    use super::*;

    /**
     * @notice 初始化合约账户并设置管理员
     * @param ctx 上下文
     * @param admin 管理员地址
     */
    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let bank_account = &mut ctx.accounts.bank_account;
        bank_account.admin = admin;
        bank_account.fee = 0; // 初始手续费设为0
        Ok(())
    }

    /**
     * @notice 设置手续费
     * @param ctx 上下文
     * @param fee 手续费金额(lamports)
     */
    pub fn set_fee(ctx: Context<SetFee>, fee: u64) -> Result<()> {
        let bank_account = &mut ctx.accounts.bank_account;

        require!(
            ctx.accounts.admin.key() == bank_account.admin,
            ErrorCode::Unauthorized
        );

        bank_account.fee = fee;
        Ok(())
    }

    /**
     * @notice 批量转账SOL
     * @param ctx 上下文
     * @param transfers 转账信息，包含接收地址和转账金额的元组数组
     */
    pub fn batch_transfer_sol<'info>(
        ctx: Context<'_, '_, '_, 'info, BatchTransferSol<'info>>,
        transfers: Vec<(Pubkey, u64)>,
    ) -> Result<()> {
        // 计算总转账金额
        let total_amount = safe_sum(&transfers)?;
        let fee = ctx.accounts.bank_account.fee;
        let required_balance = safe_add(total_amount, fee)?;

        // 检查发送者余额是否足够
        let sender_balance = ctx.accounts.sender.lamports();
        require!(
            sender_balance >= required_balance,
            ErrorCode::InsufficientFunds
        );

        // 扣除手续费
        {
            let bank_account_info = &ctx.accounts.bank_account.to_account_info();
            let sender_info = &ctx.accounts.sender.to_account_info();
            let mut sender_lamports = sender_info.lamports.borrow_mut();
            let mut bank_lamports = bank_account_info.lamports.borrow_mut();
            **sender_lamports -= fee;
            **bank_lamports += fee;
        }

        // 执行批量转账
        let sender_info = &ctx.accounts.sender.to_account_info();
        let mut remaining_accounts = ctx.remaining_accounts.iter();
        for (recipient, amount) in transfers.iter() {
            let recipient_account_info = remaining_accounts
                .next()
                .ok_or(ProgramError::NotEnoughAccountKeys)?;

            anchor_lang::solana_program::program::invoke(
                &system_instruction::transfer(&sender_info.key(), recipient, *amount),
                &[sender_info.clone(), recipient_account_info.clone()],
            )?;
        }

        // 发送转账事件
        emit!(SolTransferEvent {
            from: ctx.accounts.sender.key(),
            recipients: transfers.iter().map(|(pubkey, _)| *pubkey).collect(),
            amounts: transfers.iter().map(|(_, amount)| *amount).collect(),
            total_amount,
            fee,
        });

        Ok(())
    }

    /**
     * @notice 批量转账SPL Token
     * @param ctx 上下文
     * @param transfers 转账信息，包含接收地址和转账金额的元组数组
     */
    pub fn batch_transfer_token<'info>(
        ctx: Context<'_, '_, '_, 'info, BatchTransferToken<'info>>,
        transfers: Vec<(Pubkey, u64)>,
    ) -> Result<()> {
        // 计算总转账金额
        let total_amount = safe_sum(&transfers)?;
        let fee = ctx.accounts.bank_account.fee;

        // 检查token余额
        let sender_token_balance = ctx.accounts.token_account.amount;
        require!(
            sender_token_balance >= total_amount,
            ErrorCode::InsufficientTokenBalance
        );

        // 检查SOL余额（用于支付手续费）
        let sender_sol_balance = ctx.accounts.sender.lamports();
        require!(
            sender_sol_balance >= fee,
            ErrorCode::InsufficientFundsForFee
        );

        // 扣除手续费
        {
            let bank_account_info = &ctx.accounts.bank_account.to_account_info();
            let sender_info = &ctx.accounts.sender.to_account_info();
            let mut sender_lamports = sender_info.lamports.borrow_mut();
            let mut bank_lamports = bank_account_info.lamports.borrow_mut();
            **sender_lamports -= fee;
            **bank_lamports += fee;
        }

        // 执行批量转账
        let sender_info = &ctx.accounts.sender.to_account_info();
        let token_account_info = &ctx.accounts.token_account.to_account_info();
        let token_program_info = &ctx.accounts.token_program.to_account_info();
        let mut remaining_accounts = ctx.remaining_accounts.iter();
        for (_recipient, amount) in transfers.iter() {
            let recipient_token_account = remaining_accounts
                .next()
                .ok_or(ProgramError::NotEnoughAccountKeys)?;

            token::transfer(
                CpiContext::new(
                    token_program_info.clone(),
                    Transfer {
                        from: token_account_info.clone(),
                        to: recipient_token_account.clone(),
                        authority: sender_info.clone(),
                    },
                ),
                *amount,
            )?;
        }

        // 发送转账事件
        emit!(TokenTransferEvent {
            from: ctx.accounts.sender.key(),
            token_account: ctx.accounts.token_account.key(),
            recipients: transfers.iter().map(|(pubkey, _)| *pubkey).collect(),
            amounts: transfers.iter().map(|(_, amount)| *amount).collect(),
            total_amount,
            fee,
        });

        Ok(())
    }

    // 查询账户余额的函数
    pub fn check_balance_sol(ctx: Context<CheckBalanceSol>) -> Result<u64> {
        let account_balance = **ctx.accounts.account.to_account_info().lamports.borrow();
        Ok(account_balance)
    }

    pub fn check_balance_token(ctx: Context<CheckBalanceToken>) -> Result<u64> {
        let token_balance = token::accessor::amount(&ctx.accounts.token_account.to_account_info())?;
        Ok(token_balance)
    }

    pub fn simulate(_ctx: Context<Simulate>) -> Result<()> {
        Ok(())
    }
}

#[account]
pub struct BankAccount {
    pub admin: Pubkey, // 管理员地址
    pub fee: u64,      // 手续费金额
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

#[derive(Accounts)]
pub struct BatchTransferSol<'info> {
    #[account(mut, signer)]
    pub sender: Signer<'info>,
    #[account(mut)]
    pub bank_account: Account<'info, BankAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchTransferToken<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(mut)]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckBalanceSol<'info> {
    #[account(mut)]
    pub account: SystemAccount<'info>,
}

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

#[derive(Accounts)]
pub struct CheckBalanceToken<'info> {
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
}

/**
 * @notice SOL转账事件
 * @param from 发送者地址
 * @param recipients 接收者地址列表
 * @param amounts 转账金额列表
 * @param total_amount 总转账金额
 * @param fee 手续费
 */
#[event]
pub struct SolTransferEvent {
    pub from: Pubkey,
    pub recipients: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub total_amount: u64,
    pub fee: u64,
}

/**
 * @notice SPL Token转账事件
 * @param from 发送者地址
 * @param token_account 发送者的token账户
 * @param recipients 接收者地址列表
 * @param amounts 转账金额列表
 * @param total_amount 总转账金额
 * @param fee 手续费
 */
#[event]
pub struct TokenTransferEvent {
    pub from: Pubkey,
    pub token_account: Pubkey,
    pub recipients: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub total_amount: u64,
    pub fee: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("计算过程中发生溢出")]
    Overflow,
    #[msg("SOL余额不足")]
    InsufficientFunds,
    #[msg("Token余额不足")]
    InsufficientTokenBalance,
    #[msg("SOL余额不足以支付手续费")]
    InsufficientFundsForFee,
    #[msg("未授权的操作")]
    Unauthorized,
}

/// 安全求和函数，防止溢出
fn safe_sum(transfers: &Vec<(Pubkey, u64)>) -> std::result::Result<u64, ProgramError> {
    transfers.iter().try_fold(0u64, |acc, (_, value)| {
        acc.checked_add(*value).ok_or(ProgramError::InvalidArgument)
    })
}

/// 安全加法函数，防止溢出
fn safe_add(a: u64, b: u64) -> std::result::Result<u64, ProgramError> {
    a.checked_add(b).ok_or(ProgramError::InvalidArgument)
}
