// use anchor_lang::prelude::*;
// use anchor_lang::InstructionData;
// use anchor_spl::token::{self, spl_token, Mint, TokenAccount};
// use solana_program::instruction::Instruction;
// use solana_program_test::ProgramTest;
// use solana_sdk::account::Account;
// use solana_sdk::signature::Signer;
// use solana_sdk::transaction::Transaction;
//
// #[tokio::test]
// async fn test_simulate() {
//     let program_id = batch_transfer::ID;
//     let mint = Pubkey::new_unique();
//     let token_account = Pubkey::new_unique();
//
//     // 初始化上下文
//     let mut program = ProgramTest::new(
//         "batch_transfer",
//         program_id,
//         None,
//     );
//
//     // 添加mint账户
//     program.add_account(
//         mint,
//         Account {
//             lamports: 1000000000000,
//             data: vec![0; Mint::LEN],
//             owner: token::ID,
//             executable: false,
//             rent_epoch: 0,
//         },
//     );
//
//     // 添加token账户
//     program.add_account(
//         token_account,
//         Account {
//             lamports: 10000000000000,
//             data: vec![0; TokenAccount::LEN],
//             owner: token::ID,
//             executable: false,
//             rent_epoch: 0,
//         },
//     );
//
//     let (mut banks_client, payer, recent_blockhash) = program.start().await;
//
//     // 创建并发送初始化 mint 的指令
//     let init_mint_ix = spl_token::instruction::initialize_mint(
//         &token::ID,
//         &mint,
//         &payer.pubkey(),
//         None,
//         9, // decimals
//     ).unwrap();
//
//     let tx = Transaction::new_signed_with_payer(
//         &[init_mint_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(tx).await.unwrap();
//
//     // 创建并发送初始化 token account 的指令
//     let init_token_account_ix = spl_token::instruction::initialize_account(
//         &token::ID,
//         &token_account,
//         &mint,
//         &payer.pubkey(),
//     ).unwrap();
//
//     let tx = Transaction::new_signed_with_payer(
//         &[init_token_account_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(tx).await.unwrap();
//
//     let simulate_ix = Instruction {
//         program_id: program_id,
//         accounts: batch_transfer::accounts::Simulate {
//             payer: payer.pubkey(),
//             token_program: token::ID,
//             token_account: token_account,
//             mint: mint,
//         }
//             .to_account_metas(None),
//         data: batch_transfer::instruction::Simulate {}.data(),
//     };
//
//     let simulate_tx = Transaction::new_signed_with_payer(
//         &[simulate_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(simulate_tx).await.unwrap();
// }
