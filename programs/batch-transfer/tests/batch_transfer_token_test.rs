use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::token::{self, Mint, TokenAccount};
use solana_program::instruction::{Instruction, AccountMeta};
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

use batch_transfer::{BankAccount, self};

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

// 辅助函数：初始化测试环境
async fn setup_token_test_env() -> (
    ProgramTest,
    Keypair,
    Keypair,
    Keypair,
    Pubkey,
    Pubkey,
) {
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    let admin = Keypair::new();
    let sender = Keypair::new();
    let bank_account = Keypair::new();

    // 为管理员和发送者添加初始余额
    let initial_balance = 100 * LAMPORTS_PER_SOL;
    for account in [&admin, &sender] {
        pt.add_account(
            account.pubkey(),
            Account {
                lamports: initial_balance,
                ..Account::default()
            },
        );
    }

    // 创建代币相关账户
    let mint = Keypair::new();
    let sender_token_account = Keypair::new();

    // 添加 mint 账户
    pt.add_account(
        mint.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL,
            data: vec![0; Mint::LEN],
            owner: token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // 添加发送者代币账户
    pt.add_account(
        sender_token_account.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL,
            data: vec![0; TokenAccount::LEN],
            owner: token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    (pt, admin, sender, bank_account, mint.pubkey(), sender_token_account.pubkey())
}

// 辅助函数：初始化代币账户
async fn initialize_token_account(
    banks_client: &mut solana_program_test::BanksClient,
    payer: &Keypair,
    token_account: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let init_token_account_ix = token::spl_token::instruction::initialize_account(
        &token::ID,
        token_account,
        mint,
        owner,
    )
    .unwrap();

    let init_token_account_tx = Transaction::new_signed_with_payer(
        &[init_token_account_ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    banks_client
        .process_transaction(init_token_account_tx)
        .await
        .unwrap();
}

// 辅助函数：铸造代币
async fn mint_tokens(
    banks_client: &mut solana_program_test::BanksClient,
    payer: &Keypair,
    mint: &Pubkey,
    token_account: &Pubkey,
    amount: u64,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let mint_ix = token::spl_token::instruction::mint_to(
        &token::ID,
        mint,
        token_account,
        &payer.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let mint_tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_tx).await.unwrap();
}

// 辅助函数：初始化 mint
async fn initialize_mint(
    banks_client: &mut solana_program_test::BanksClient,
    payer: &Keypair,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    decimals: u8,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let init_mint_ix = token::spl_token::instruction::initialize_mint(
        &token::ID,
        mint,
        mint_authority,
        None,
        decimals,
    )
    .unwrap();

    let init_mint_tx = Transaction::new_signed_with_payer(
        &[init_mint_ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    banks_client.process_transaction(init_mint_tx).await.unwrap();
}

// 辅助函数：初始化银行账户
async fn initialize_bank_account(
    banks_client: &mut solana_program_test::BanksClient,
    admin: &Keypair,
    bank_account: &Keypair,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let initialize_ix = Instruction {
        program_id: batch_transfer::ID,
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
        &[admin, bank_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();
}

// 辅助函数：设置手续费
async fn set_fee(
    banks_client: &mut solana_program_test::BanksClient,
    admin: &Keypair,
    bank_account: &Keypair,
    fee: u64,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let set_fee_ix = Instruction {
        program_id: batch_transfer::ID,
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
        &[admin],
        recent_blockhash,
    );

    banks_client.process_transaction(set_fee_tx).await.unwrap();
}

#[tokio::test]
async fn test_batch_transfer_token() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let bank_account = Keypair::new();

    // 为测试账户添加初始余额
    let initial_sol_balance = 10_000_000_000; // 10 SOL
    pt.add_account(
        sender.pubkey(),
        Account {
            lamports: initial_sol_balance,
            ..Account::default()
        },
    );
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: initial_sol_balance,
            ..Account::default()
        },
    );

    // 创建代币相关账户
    let mint = Pubkey::new_unique();
    let sender_token_account = Pubkey::new_unique();
    let recipient1_token_account = Pubkey::new_unique();
    let recipient2_token_account = Pubkey::new_unique();

    // 添加 mint 账户
    pt.add_account(
        mint,
        Account {
            lamports: 1000000000000,
            data: vec![0; Mint::LEN],
            owner: token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // 添加代币账户
    for token_account in [&sender_token_account, &recipient1_token_account, &recipient2_token_account] {
        pt.add_account(
            *token_account,
            Account {
                lamports: 10000000000000,
                data: vec![0; TokenAccount::LEN],
                owner: token::ID,
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 初始化 bank_account
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

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();

    // 设置手续费
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

    // 初始化 mint
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

    // 初始化所有代币账户
    for token_account in [&sender_token_account, &recipient1_token_account, &recipient2_token_account] {
        let init_token_account_ix = token::spl_token::instruction::initialize_account(
            &token::ID,
            token_account,
            &mint,
            &sender.pubkey(),
        ).unwrap();

        let init_token_account_tx = Transaction::new_signed_with_payer(
            &[init_token_account_ix],
            Some(&admin.pubkey()),
            &[&admin],
            recent_blockhash,
        );

        banks_client.process_transaction(init_token_account_tx).await.unwrap();
    }

    // 铸造代币到发送者账户
    let mint_amount = 1_000_000_000;
    let mint_to_ix = token::spl_token::instruction::mint_to(
        &token::ID,
        &mint,
        &sender_token_account,
        &admin.pubkey(),
        &[],
        mint_amount,
    ).unwrap();

    let mint_tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_tx).await.unwrap();

    // 准备批量转账数据
    let transfer_amount1 = 100_000_000;
    let transfer_amount2 = 200_000_000;
    let transfers = vec![
        (recipient1_token_account, transfer_amount1),
        (recipient2_token_account, transfer_amount2),
    ];

    // 执行批量转账
    let batch_transfer_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferToken {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            token_account: sender_token_account,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferToken { transfers: transfers.clone() }.data(),
    };

    // 添加接收者账户到 remaining_accounts
    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client.process_transaction(batch_transfer_tx).await.unwrap();

    // 验证转账结果
    let total_transfer = transfer_amount1 + transfer_amount2;
    let expected_sender_balance = mint_amount - total_transfer;

    // 检查发送者代币余额
    let sender_token_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        sender_token_account,
    ).await;
    assert_eq!(sender_token_data.amount, expected_sender_balance);

    // 检查接收者代币余额
    let recipient1_token_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        recipient1_token_account,
    ).await;
    assert_eq!(recipient1_token_data.amount, transfer_amount1);

    let recipient2_token_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        recipient2_token_account,
    ).await;
    assert_eq!(recipient2_token_data.amount, transfer_amount2);

    // 检查手续费是否正确收取
    let bank_account_data: BankAccount = load_and_deserialize(
        banks_client.clone(),
        bank_account.pubkey(),
    ).await;
    assert_eq!(bank_account_data.fee, fee);

    // 检查发送者的 SOL 余额是否正确扣除手续费
    let sender_sol_balance = banks_client
        .get_account(sender.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    // 考虑 Solana 网络的交易费用（每笔交易约 5000 lamports）
    let transaction_fee = 5000;
    assert_eq!(sender_sol_balance, initial_sol_balance - fee - transaction_fee);
}

#[tokio::test]
async fn test_batch_transfer_token_zero_fee() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let bank_account = Keypair::new();

    // 为测试账户添加初始余额
    let initial_sol_balance = 10_000_000_000; // 10 SOL
    pt.add_account(
        sender.pubkey(),
        Account {
            lamports: initial_sol_balance,
            ..Account::default()
        },
    );
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: initial_sol_balance,
            ..Account::default()
        },
    );

    // 创建代币相关账户
    let mint = Pubkey::new_unique();
    let sender_token_account = Pubkey::new_unique();
    let recipient1_token_account = Pubkey::new_unique();
    let recipient2_token_account = Pubkey::new_unique();

    // 添加 mint 账户
    pt.add_account(
        mint,
        Account {
            lamports: 1000000000000,
            data: vec![0; Mint::LEN],
            owner: token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // 添加代币账户
    for token_account in [&sender_token_account, &recipient1_token_account, &recipient2_token_account] {
        pt.add_account(
            *token_account,
            Account {
                lamports: 10000000000000,
                data: vec![0; TokenAccount::LEN],
                owner: token::ID,
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 初始化 bank_account
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

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();

    // 设置手续费为 0
    let fee = 0;
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

    // 初始化 mint
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

    // 初始化所有代币账户
    for token_account in [&sender_token_account, &recipient1_token_account, &recipient2_token_account] {
        let init_token_account_ix = token::spl_token::instruction::initialize_account(
            &token::ID,
            token_account,
            &mint,
            &sender.pubkey(),
        ).unwrap();

        let init_token_account_tx = Transaction::new_signed_with_payer(
            &[init_token_account_ix],
            Some(&admin.pubkey()),
            &[&admin],
            recent_blockhash,
        );

        banks_client.process_transaction(init_token_account_tx).await.unwrap();
    }

    // 铸造代币到发送者账户
    let mint_amount = 1_000_000_000;
    let mint_to_ix = token::spl_token::instruction::mint_to(
        &token::ID,
        &mint,
        &sender_token_account,
        &admin.pubkey(),
        &[],
        mint_amount,
    ).unwrap();

    let mint_tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_tx).await.unwrap();

    // 准备批量转账数据
    let transfer_amount1 = 100_000_000;
    let transfer_amount2 = 200_000_000;
    let transfers = vec![
        (recipient1_token_account, transfer_amount1),
        (recipient2_token_account, transfer_amount2),
    ];

    // 执行批量转账
    let batch_transfer_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferToken {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            token_account: sender_token_account,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferToken { transfers: transfers.clone() }.data(),
    };

    // 添加接收者账户到 remaining_accounts
    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client.process_transaction(batch_transfer_tx).await.unwrap();

    // 验证转账结果
    let total_transfer = transfer_amount1 + transfer_amount2;
    let expected_sender_balance = mint_amount - total_transfer;

    // 检查发送者代币余额
    let sender_token_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        sender_token_account,
    ).await;
    assert_eq!(sender_token_data.amount, expected_sender_balance);

    // 检查接收者代币余额
    let recipient1_token_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        recipient1_token_account,
    ).await;
    assert_eq!(recipient1_token_data.amount, transfer_amount1);

    let recipient2_token_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        recipient2_token_account,
    ).await;
    assert_eq!(recipient2_token_data.amount, transfer_amount2);

    // 检查手续费是否为 0
    let bank_account_data: BankAccount = load_and_deserialize(
        banks_client.clone(),
        bank_account.pubkey(),
    ).await;
    assert_eq!(bank_account_data.fee, 0);

    // 检查发送者的 SOL 余额是否保持不变（因为手续费为 0）
    let sender_sol_balance = banks_client
        .get_account(sender.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    // 考虑 Solana 网络的交易费用（每笔交易约 5000 lamports）
    let transaction_fee = 5000;
    assert_eq!(sender_sol_balance, initial_sol_balance - transaction_fee);
}

pub async fn load_and_deserialize<T: AccountDeserialize>(
    mut banks_client: solana_program_test::BanksClient,
    address: Pubkey,
) -> T {
    let account = banks_client
        .get_account(address)
        .await
        .unwrap()
        .unwrap();

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}

#[tokio::test]
async fn test_token_insufficient_balance() {
    let (mut pt, admin, sender, bank_account, mint, sender_token_account) =
        setup_token_test_env().await;
    let recipient = Keypair::new();
    let recipient_token_account = Keypair::new();

    // 添加接收者代币账户
    pt.add_account(
        recipient_token_account.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL,
            data: vec![0; TokenAccount::LEN],
            owner: token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 初始化所有账户
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    set_fee(&mut banks_client, &admin, &bank_account, fee, recent_blockhash).await;

    // 初始化 mint
    initialize_mint(
        &mut banks_client,
        &admin,
        &mint,
        &admin.pubkey(),
        9,
        recent_blockhash,
    )
    .await;

    // 初始化代币账户
    initialize_token_account(
        &mut banks_client,
        &admin,
        &sender_token_account,
        &mint,
        &sender.pubkey(),
        recent_blockhash,
    )
    .await;
    initialize_token_account(
        &mut banks_client,
        &admin,
        &recipient_token_account.pubkey(),
        &mint,
        &recipient.pubkey(),
        recent_blockhash,
    )
    .await;

    // 铸造少量代币到发送者账户
    let initial_token_amount = 100;
    mint_tokens(
        &mut banks_client,
        &admin,
        &mint,
        &sender_token_account,
        initial_token_amount,
        recent_blockhash,
    )
    .await;

    // 尝试转账超过余额的金额
    let transfer_amount = initial_token_amount + 1;
    let transfers = vec![(recipient_token_account.pubkey(), transfer_amount)];

    let mut accounts = batch_transfer::accounts::BatchTransferToken {
        sender: sender.pubkey(),
        bank_account: bank_account.pubkey(),
        token_account: sender_token_account,
        token_program: token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false))
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts,
        data: batch_transfer::instruction::BatchTransferToken { transfers }.data(),
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(batch_transfer_tx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_insufficient_sol_fee() {
    let (mut pt, admin, sender, bank_account, mint, sender_token_account) =
        setup_token_test_env().await;
    let recipient = Keypair::new();
    let recipient_token_account = Keypair::new();

    // 添加接收者代币账户
    pt.add_account(
        recipient_token_account.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL,
            data: vec![0; TokenAccount::LEN],
            owner: token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 初始化所有账户
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;
    let fee = LAMPORTS_PER_SOL * 1000; // 设置一个很高的手续费
    set_fee(&mut banks_client, &admin, &bank_account, fee, recent_blockhash).await;

    // 初始化 mint
    initialize_mint(
        &mut banks_client,
        &admin,
        &mint,
        &admin.pubkey(),
        9,
        recent_blockhash,
    )
    .await;

    // 初始化代币账户
    initialize_token_account(
        &mut banks_client,
        &admin,
        &sender_token_account,
        &mint,
        &sender.pubkey(),
        recent_blockhash,
    )
    .await;
    initialize_token_account(
        &mut banks_client,
        &admin,
        &recipient_token_account.pubkey(),
        &mint,
        &recipient.pubkey(),
        recent_blockhash,
    )
    .await;

    // 铸造代币到发送者账户
    let initial_token_amount = 1_000_000;
    mint_tokens(
        &mut banks_client,
        &admin,
        &mint,
        &sender_token_account,
        initial_token_amount,
        recent_blockhash,
    )
    .await;

    // 尝试转账
    let transfer_amount = 100;
    let transfers = vec![(recipient_token_account.pubkey(), transfer_amount)];

    let mut accounts = batch_transfer::accounts::BatchTransferToken {
        sender: sender.pubkey(),
        bank_account: bank_account.pubkey(),
        token_account: sender_token_account,
        token_program: token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false))
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts,
        data: batch_transfer::instruction::BatchTransferToken { transfers }.data(),
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(batch_transfer_tx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_empty_transfers() {
    let (mut pt, admin, sender, bank_account, mint, sender_token_account) =
        setup_token_test_env().await;

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 初始化所有账户
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    set_fee(&mut banks_client, &admin, &bank_account, fee, recent_blockhash).await;

    // 初始化 mint
    initialize_mint(
        &mut banks_client,
        &admin,
        &mint,
        &admin.pubkey(),
        9,
        recent_blockhash,
    )
    .await;

    // 初始化发送者代币账户
    initialize_token_account(
        &mut banks_client,
        &admin,
        &sender_token_account,
        &mint,
        &sender.pubkey(),
        recent_blockhash,
    )
    .await;

    // 尝试空转账列表
    let transfers: Vec<(Pubkey, u64)> = vec![];

    let accounts = batch_transfer::accounts::BatchTransferToken {
        sender: sender.pubkey(),
        bank_account: bank_account.pubkey(),
        token_account: sender_token_account,
        token_program: token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts,
        data: batch_transfer::instruction::BatchTransferToken { transfers }.data(),
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(batch_transfer_tx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_many_recipients() {
    let (mut pt, admin, sender, bank_account, mint, sender_token_account) =
        setup_token_test_env().await;

    // 创建多个接收者账户
    let num_recipients = 10;
    let mut recipient_accounts = Vec::new();
    for _ in 0..num_recipients {
        let recipient = Keypair::new();
        let recipient_token_account = Keypair::new();

        // 添加接收者代币账户
        pt.add_account(
            recipient_token_account.pubkey(),
            Account {
                lamports: LAMPORTS_PER_SOL,
                data: vec![0; TokenAccount::LEN],
                owner: token::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        recipient_accounts.push((recipient, recipient_token_account));
    }

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 初始化所有账户
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    set_fee(&mut banks_client, &admin, &bank_account, fee, recent_blockhash).await;

    // 初始化 mint
    initialize_mint(
        &mut banks_client,
        &admin,
        &mint,
        &admin.pubkey(),
        9,
        recent_blockhash,
    )
    .await;

    // 初始化所有代币账户
    initialize_token_account(
        &mut banks_client,
        &admin,
        &sender_token_account,
        &mint,
        &sender.pubkey(),
        recent_blockhash,
    )
    .await;

    for (recipient, recipient_token_account) in &recipient_accounts {
        initialize_token_account(
            &mut banks_client,
            &admin,
            &recipient_token_account.pubkey(),
            &mint,
            &recipient.pubkey(),
            recent_blockhash,
        )
        .await;
    }

    // 铸造代币到发送者账户
    let initial_token_amount = 1_000_000;
    mint_tokens(
        &mut banks_client,
        &admin,
        &mint,
        &sender_token_account,
        initial_token_amount,
        recent_blockhash,
    )
    .await;

    // 准备批量转账数据
    let transfer_amount = 1_000;
    let transfers: Vec<(Pubkey, u64)> = recipient_accounts
        .iter()
        .map(|(_, token_account)| (token_account.pubkey(), transfer_amount))
        .collect();

    let mut accounts = batch_transfer::accounts::BatchTransferToken {
        sender: sender.pubkey(),
        bank_account: bank_account.pubkey(),
        token_account: sender_token_account,
        token_program: token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false))
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts,
        data: batch_transfer::instruction::BatchTransferToken { transfers: transfers.clone() }.data(),
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client.process_transaction(batch_transfer_tx).await.unwrap();

    // 验证转账结果
    let total_transfer = transfer_amount * num_recipients as u64;
    let expected_sender_balance = initial_token_amount - total_transfer;

    // 检查发送者代币余额
    let sender_token_data: TokenAccount = load_and_deserialize(
        banks_client.clone(),
        sender_token_account,
    ).await;
    assert_eq!(sender_token_data.amount, expected_sender_balance);

    // 检查所有接收者代币余额
    for (_, recipient_token_account) in recipient_accounts {
        let recipient_token_data: TokenAccount = load_and_deserialize(
            banks_client.clone(),
            recipient_token_account.pubkey(),
        ).await;
        assert_eq!(recipient_token_data.amount, transfer_amount);
    }
}
