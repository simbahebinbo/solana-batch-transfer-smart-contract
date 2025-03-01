use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_error::ProgramError;
use anchor_spl::token::{self, Token, TokenAccount};

declare_id!("CRYsXNCjnhR1dgrpd3rEtY8ec7DNKa2659iCaN5tatXK");

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
        require!(!bank_account.is_initialized, ErrorCode::AlreadyInitialized);
        
        // 确保部署者就是指定的管理员
        require!(ctx.accounts.deployer.key() == admin, ErrorCode::Unauthorized);

        bank_account.admin = admin;
        bank_account.fee = 0; // 初始手续费设为0
        bank_account.is_initialized = true;
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
     * @param transfers 转账信息数组
     */
    pub fn batch_transfer_sol<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, BatchTransferSol<'info>>, 
        transfers: Vec<TransferInfo>
    ) -> Result<()> {
        // 检查转账列表不能为空
        if transfers.is_empty() {
            return Err(ErrorCode::EmptyTransfers.into());
        }

        // 计算总转账金额
        let total_amount = safe_sum_transfer_info(&transfers)?;
        let fee = ctx.accounts.bank_account.fee;
        let required_balance = safe_add(total_amount, fee)?;

        // 检查发送者余额是否足够
        let sender_balance = ctx.accounts.sender.lamports();
        require!(
            sender_balance >= required_balance,
            ErrorCode::InsufficientFunds
        );

        // 扣除手续费
        if fee > 0 {
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.sender.to_account_info(),
                    to: ctx.accounts.bank_account.to_account_info(),
                },
            );
            anchor_lang::system_program::transfer(cpi_context, fee)?;
        }

        // 执行批量转账
        let remaining_accounts = ctx.remaining_accounts;
        for (i, transfer_info) in transfers.iter().enumerate() {
            if i >= remaining_accounts.len() {
                return Err(ProgramError::NotEnoughAccountKeys.into());
            }
            
            let recipient_account_info = &remaining_accounts[i];

            // 验证接收者账户是否存在
            require!(
                recipient_account_info.key() == transfer_info.recipient,
                ErrorCode::InvalidRecipient
            );

            // 使用系统程序进行转账
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.sender.to_account_info(),
                    to: recipient_account_info.clone(),
                },
            );
            anchor_lang::system_program::transfer(cpi_context, transfer_info.amount)?;
        }

        // 发送转账事件
        emit!(SolTransferEvent {
            from: ctx.accounts.sender.key(),
            recipients: transfers.iter().map(|info| info.recipient).collect(),
            amounts: transfers.iter().map(|info| info.amount).collect(),
            total_amount,
            fee,
        });

        Ok(())
    }

    /**
     * @notice 批量转账SPL Token
     * @param ctx 上下文
     * @param transfers 转账信息数组
     */
    pub fn batch_transfer_token<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, BatchTransferToken<'info>>, 
        transfers: Vec<TransferInfo>
    ) -> Result<()> {
        // 检查转账列表不能为空
        if transfers.is_empty() {
            return Err(ErrorCode::EmptyTransfers.into());
        }

        // 计算总转账金额
        let total_amount = safe_sum_transfer_info(&transfers)?;
        let fee = ctx.accounts.bank_account.fee;

        // 检查发送者余额是否足够
        let token_balance = token::accessor::amount(&ctx.accounts.token_account.to_account_info())?;
        if token_balance < total_amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // 检查发送者SOL余额是否足够支付手续费
        let sender_balance = ctx.accounts.sender.lamports();
        if sender_balance < fee {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // 扣除手续费
        if fee > 0 {
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.sender.to_account_info(),
                    to: ctx.accounts.bank_account.to_account_info(),
                },
            );
            anchor_lang::system_program::transfer(cpi_context, fee)?;
        }

        // 执行批量转账
        let remaining_accounts = ctx.remaining_accounts;
        for (i, transfer_info) in transfers.iter().enumerate() {
            if i >= remaining_accounts.len() {
                return Err(ProgramError::NotEnoughAccountKeys.into());
            }
            
            let recipient_token_account = &remaining_accounts[i];

            // 验证接收者账户是否存在
            require!(
                recipient_token_account.key() == transfer_info.recipient,
                ErrorCode::InvalidRecipient
            );

            // 使用 token 程序进行转账
            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_account.to_account_info(),
                    to: recipient_token_account.clone(),
                    authority: ctx.accounts.sender.to_account_info(),
                },
            );
            token::transfer(cpi_context, transfer_info.amount)?;
        }

        // 发送转账事件
        emit!(TokenTransferEvent {
            from: ctx.accounts.sender.key(),
            token_account: ctx.accounts.token_account.key(),
            recipients: transfers.iter().map(|info| info.recipient).collect(),
            amounts: transfers.iter().map(|info| info.amount).collect(),
            total_amount,
            fee,
        });

        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct BankAccount {
    pub admin: Pubkey, // 管理员地址
    pub fee: u64,      // 手续费金额
    pub is_initialized: bool, // 是否已初始化
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = deployer,
        space = 8 + 32 + 8 + 1, // 增加1字节存储is_initialized
        seeds = [b"bank_account"],
        bump
    )]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub deployer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetFee<'info> {
    #[account(
        mut,
        seeds = [b"bank_account"],
        bump
    )]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct BatchTransferSol<'info> {
    /// CHECK: 发送者账户，必须是签名者且可变
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(
        mut,
        seeds = [b"bank_account"],
        bump
    )]
    pub bank_account: Account<'info, BankAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchTransferToken<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(
        mut,
        seeds = [b"bank_account"],
        bump
    )]
    pub bank_account: Account<'info, BankAccount>,
    #[account(
        mut,
        constraint = token_account.owner == sender.key()
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferInfo {
    pub recipient: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("SOL余额不足")]
    InsufficientFunds,
    #[msg("算术溢出")]
    ArithmeticOverflow,
    #[msg("未授权")]
    Unauthorized,
    #[msg("转账列表不能为空")]
    EmptyTransfers,
    #[msg("账户已初始化")]
    AlreadyInitialized,
    #[msg("接收者账户无效")]
    InvalidRecipient,
}

/// 安全求和函数，防止溢出
pub fn safe_sum_transfer_info(transfers: &Vec<TransferInfo>) -> std::result::Result<u64, ProgramError> {
    transfers.iter().try_fold(0u64, |acc, info| {
        acc.checked_add(info.amount).ok_or(ProgramError::InvalidArgument)
    })
}

/// 安全加法函数，防止溢出
pub fn safe_add(a: u64, b: u64) -> std::result::Result<u64, ProgramError> {
    a.checked_add(b).ok_or(ProgramError::InvalidArgument)
}
