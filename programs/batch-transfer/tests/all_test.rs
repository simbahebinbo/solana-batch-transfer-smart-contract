use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::token::{self, Mint, TokenAccount};
use solana_program::instruction::Instruction;
use solana_program::system_program;
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

use batch_transfer::{self, BankAccount};

#[tokio::test]
async fn test_all() {
    let SetUpTest {
        program_id,
        pt,
        admin,
        bank_account,
        sender,
        recipient1,
        recipient2,
        mint,
        sender_token_account,
        recipient1_token_account,
        recipient2_token_account,
    } = SetUpTest::new();

    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 1. 初始化 bank_account
    let initialize_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::Initialize {
            bank_account: bank_account.pubkey(),
            deployer: admin.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::Initialize {
            admin: admin.pubkey(),
        }
        .data(),
    };

    let initialize_tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
        Some(&admin.pubkey()),
        &[&admin, &bank_account],
        recent_blockhash,
    );

    banks_client.process_transaction(initialize_tx).await.unwrap();

    // 2. 设置手续费
    let fee = 10_000_000; // 0.01 SOL
    let set_fee_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::SetFee {
            bank_account: bank_account.pubkey(),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee { fee }.data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(set_fee_tx).await.unwrap();

    // 3. 初始化代币相关账户
    // 3.1 初始化 mint
    let init_mint_ix = token::spl_token::instruction::initialize_mint(
        &token::ID,
        &mint,
        &admin.pubkey(),
        None,
        9, // decimals
    ).unwrap();

    let init_mint_tx = Transaction::new_signed_with_payer(
        &[init_mint_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(init_mint_tx).await.unwrap();

    // 3.2 初始化所有 token accounts
    let token_accounts = vec![
        (sender_token_account, sender.pubkey()),
        (recipient1_token_account, recipient1.pubkey()),
        (recipient2_token_account, recipient2.pubkey()),
    ];

    for (token_account, owner) in token_accounts {
        let init_token_account_ix = token::spl_token::instruction::initialize_account(
            &token::ID,
            &token_account,
            &mint,
            &owner,
        ).unwrap();

        let init_token_account_tx = Transaction::new_signed_with_payer(
            &[init_token_account_ix],
            Some(&admin.pubkey()),
            &[&admin],
            recent_blockhash,
        );

        banks_client.process_transaction(init_token_account_tx).await.unwrap();
    }

    // 3.3 铸造代币给发送者
    let mint_amount = 1_000_000_000;
    let mint_to_ix = token::spl_token::instruction::mint_to(
        &token::ID,
        &mint,
        &sender_token_account,
        &admin.pubkey(),
        &[],
        mint_amount,
    ).unwrap();

    let mint_to_tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_to_tx).await.unwrap();

    // 4. 执行 SOL 批量转账
    let sol_transfer_amount1 = 100_000_000; // 0.1 SOL
    let sol_transfer_amount2 = 200_000_000; // 0.2 SOL
    let sol_transfers = vec![
        (recipient1.pubkey(), sol_transfer_amount1),
        (recipient2.pubkey(), sol_transfer_amount2),
    ];

    let batch_transfer_sol_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: sol_transfers.clone() }.data(),
    };

    // 添加接收者账户到 remaining_accounts
    let mut accounts = batch_transfer_sol_ix.accounts;
    accounts.extend(
        sol_transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_sol_ix = Instruction {
        program_id: batch_transfer_sol_ix.program_id,
        accounts,
        data: batch_transfer_sol_ix.data,
    };

    let batch_transfer_sol_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_sol_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client.process_transaction(batch_transfer_sol_tx).await.unwrap();

    // 5. 执行代币批量转账
    let token_transfer_amount1 = 1_000_000; // 0.001 tokens
    let token_transfer_amount2 = 2_000_000; // 0.002 tokens
    let token_transfers = vec![
        (recipient1_token_account, token_transfer_amount1),
        (recipient2_token_account, token_transfer_amount2),
    ];

    let batch_transfer_token_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferToken {
            sender: sender.pubkey(),
            token_account: sender_token_account,
            bank_account: bank_account.pubkey(),
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferToken { transfers: token_transfers.clone() }.data(),
    };

    // 添加接收者账户到 remaining_accounts
    let mut accounts = batch_transfer_token_ix.accounts;
    accounts.extend(
        token_transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_token_ix = Instruction {
        program_id: batch_transfer_token_ix.program_id,
        accounts,
        data: batch_transfer_token_ix.data,
    };

    let batch_transfer_token_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_token_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client.process_transaction(batch_transfer_token_tx).await.unwrap();

    // 6. 验证结果
    // 6.1 验证 SOL 转账结果
    let total_sol_transfer = sol_transfer_amount1 + sol_transfer_amount2;
    let transaction_fee = 5000; // 预估的交易费用

    // 检查接收者 SOL 余额
    let recipient1_balance = banks_client
        .get_account(recipient1.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient1_balance, sol_transfer_amount1);

    let recipient2_balance = banks_client
        .get_account(recipient2.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient2_balance, sol_transfer_amount2);

    // 6.2 验证代币转账结果
    // 检查接收者代币余额
    let recipient1_token_account_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        recipient1_token_account,
    ).await;
    assert_eq!(recipient1_token_account_data.amount, token_transfer_amount1);

    let recipient2_token_account_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        recipient2_token_account,
    ).await;
    assert_eq!(recipient2_token_account_data.amount, token_transfer_amount2);

    // 6.3 验证手续费收取
    let bank_account_data: BankAccount = load_and_deserialize(
        banks_client.clone(),
        bank_account.pubkey(),
    ).await;
    assert_eq!(bank_account_data.fee, fee);
}

pub struct SetUpTest {
    pub program_id: Pubkey,
    pub pt: ProgramTest,
    pub admin: Keypair,
    pub bank_account: Keypair,
    pub sender: Keypair,
    pub recipient1: Keypair,
    pub recipient2: Keypair,
    pub mint: Pubkey,
    pub sender_token_account: Pubkey,
    pub recipient1_token_account: Pubkey,
    pub recipient2_token_account: Pubkey,
}

impl SetUpTest {
    pub fn new() -> Self {
        let program_id = batch_transfer::ID;
        let mut pt: ProgramTest = ProgramTest::new("batch_transfer", program_id, None);
        pt.set_compute_max_units(3_600_000);

        // 创建所有需要的密钥对
        let admin = Keypair::new();
        let bank_account = Keypair::new();
        let sender = Keypair::new();
        let recipient1 = Keypair::new();
        let recipient2 = Keypair::new();

        // 为账户添加初始 SOL 余额
        let accounts = vec![
            (&admin, 10_000_000_000),     // 10 SOL
            (&sender, 1_000_000_000),     // 1 SOL
            (&recipient1, 0),             // 0 SOL
            (&recipient2, 0),             // 0 SOL
        ];

        for (keypair, lamports) in accounts {
            pt.add_account(
                keypair.pubkey(),
                Account {
                    lamports,
                    ..Account::default()
                },
            );
        }

        // 创建代币相关账户
        let mint = Pubkey::new_unique();
        pt.add_account(
            mint,
            Account {
                lamports: 1_000_000_000,
                data: vec![0; Mint::LEN],
                owner: token::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let sender_token_account = Pubkey::new_unique();
        let recipient1_token_account = Pubkey::new_unique();
        let recipient2_token_account = Pubkey::new_unique();

        for token_account in [&sender_token_account, &recipient1_token_account, &recipient2_token_account] {
            pt.add_account(
                *token_account,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0; TokenAccount::LEN],
                    owner: token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            );
        }

        Self {
            program_id,
            pt,
            admin,
            bank_account,
            sender,
            recipient1,
            recipient2,
            mint,
            sender_token_account,
            recipient1_token_account,
            recipient2_token_account,
        }
    }
}

pub async fn load_and_deserialize<T: AccountDeserialize>(
    mut banks_client: BanksClient,
    address: Pubkey,
) -> T {
    let account = banks_client
        .get_account(address)
        .await
        .unwrap()
        .unwrap();

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}
