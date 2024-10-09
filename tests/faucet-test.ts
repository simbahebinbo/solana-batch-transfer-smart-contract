// import {Program} from "@coral-xyz/anchor";
// import {BatchTransfer} from "../target/types/batch_transfer";
//
// const anchor = require('@coral-xyz/anchor');
// const {
//     TOKEN_PROGRAM_ID,
//     AccountLayout,
//     MintLayout,
//     createInitializeMintInstruction,
//     createInitializeAccountInstruction
// } = require('@solana/spl-token');
// const assert = require('chai').assert;
//
// describe('batch-transfer', () => {
//     // Configure the client to use the local cluster.
//     const provider = anchor.AnchorProvider.env();
//     anchor.setProvider(provider);
//
//     const program = anchor.workspace.BatchTransfer as anchor.Program<BatchTransfer>;
//
//
//     it('Is Faucet', async () => {
//         const accounts = [];
//         // 创建一个新的 Keypair 作为费用支付者
//         const admin = anchor.web3.Keypair.generate();
//         accounts.push(admin);
//
//         let initLamports = 1000000000
//
//         // 发放一定数量的 SOL 给费用支付者
//         // 发起多个空投请求
//         const airdropPromises = accounts.map(async (account) => {
//             const airdrop_tx = await provider.connection.requestAirdrop(account.publicKey, initLamports);
//             return {airdrop_tx, account};
//         });
//
//         // 等待所有空投请求完成
//         const airdropResults = await Promise.all(airdropPromises);
//
//         // 获取最新的区块哈希
//         const latestBlockHash = await provider.connection.getLatestBlockhash();
//
//         // 确认每个空投交易
//         const confirmationPromises = airdropResults.map(({airdrop_tx, account}) =>
//             provider.connection.confirmTransaction({
//                 blockhash: latestBlockHash.blockhash,
//                 lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
//                 signature: airdrop_tx
//             }).then((confirmation) => {
//                 console.log(`Airdrop to ${account.publicKey.toBase58()} confirmed`, confirmation);
//                 return confirmation;
//             }).catch((error) => {
//                 console.error(`Error confirming airdrop to ${account.publicKey.toBase58()}`, error);
//                 throw error;
//             })
//         );
//
//         // 等待所有确认请求完成
//         await Promise.all(confirmationPromises);
//
//         console.log("All airdrops confirmed.");
//
//         // 创建一个新的 Keypair 作为代币 mint
//         const mint = anchor.web3.Keypair.generate();
//         // 获取创建账户所需的最小租金
//         const mintRent = await provider.connection.getMinimumBalanceForRentExemption(MintLayout.span);
//
//         // 创建代币 mint 账户
//         const createMintAccountIx = anchor.web3.SystemProgram.createAccount({
//             fromPubkey: admin.publicKey,
//             newAccountPubkey: mint.publicKey,
//             lamports: mintRent,
//             space: MintLayout.span, // 代币 mint 账户的大小
//             programId: TOKEN_PROGRAM_ID,
//         });
//
//         // 初始化代币 mint 的指令
//         const initMintIx = createInitializeMintInstruction(
//             mint.publicKey, // 代币 mint 的公钥
//             9,  // 小数位数
//             admin.publicKey, // 代币的铸造权限
//             null, // 冻结权限（可选）
//             TOKEN_PROGRAM_ID
//         );
//
//         // 构建交易并添加指令
//         const tx1 = new anchor.web3.Transaction().add(createMintAccountIx, initMintIx);
//
//         // 签署并发送交易
//         await provider.sendAndConfirm(tx1, [admin, mint]);
//
//         console.log('token mint created:', mint.publicKey.toBase58());
//
//
//         // Create token account for the user
//         // 创建一个新的 Keypair 作为代币账户
//         const userTokenAccount = anchor.web3.Keypair.generate();
//         // 获取创建账户所需的最小租金
//         const userTokenAccountRent = await provider.connection.getMinimumBalanceForRentExemption(AccountLayout.span);
//
//         // 创建代币账户
//         const createUserTokenAccountIx = anchor.web3.SystemProgram.createAccount({
//             fromPubkey: admin.publicKey,
//             newAccountPubkey: userTokenAccount.publicKey,
//             lamports: userTokenAccountRent,
//             space: AccountLayout.span, // 代币账户的大小
//             programId: TOKEN_PROGRAM_ID,
//         });
//
//         // 初始化代币账户的指令
//         const initUserTokenAccountIx = createInitializeAccountInstruction(
//             userTokenAccount.publicKey, // 代币账户的公钥
//             mint.publicKey, // 代币 mint 的公钥
//             admin.publicKey, // 代币账户的所有者
//             TOKEN_PROGRAM_ID
//         );
//
//         // 构建交易并添加指令
//         const tx2 = new anchor.web3.Transaction()
//             .add(createUserTokenAccountIx)
//             .add(initUserTokenAccountIx);
//
//         // 签署并发送交易
//         await provider.sendAndConfirm(tx2, [admin, userTokenAccount], {commitment: 'confirmed'});
//         console.log('user token account created:', userTokenAccount.publicKey.toBase58());
//
//
//         // Set the amount to be minted
//         const mintAmount = 1000000000;
//
//         // Call the faucet function
//         await program.methods.faucet(new anchor.BN(mintAmount))
//             .accounts({
//                 mint: mint.publicKey,
//                 userTokenAccount: userTokenAccount.publicKey,
//                 admin: admin.publicKey,
//                 // @ts-ignore
//                 tokenProgram: TOKEN_PROGRAM_ID,
//             })
//             .signers([admin])
//             .rpc();
//
//         // Check the amount of the user token account
//         const userTokenAccountInfo = await provider.connection.getAccountInfo(userTokenAccount.publicKey);
//         const accountData = AccountLayout.decode(Buffer.from(userTokenAccountInfo.data));
//         assert.equal(accountData.amount, mintAmount);
//     });
// });
//
//
