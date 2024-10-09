// import * as anchor from "@coral-xyz/anchor";
// import {Program} from "@coral-xyz/anchor";
// import {BatchTransfer} from "../target/types/batch_transfer";
// import {assert} from "chai";
//
// const {SystemProgram} = require('@solana/web3.js');
//
// describe('batch-transfer', () => {
//     // Configure the client to use the local cluster.
//     let provider = anchor.AnchorProvider.env();
//     anchor.setProvider(provider);
//
//     const program = anchor.workspace.BatchTransfer as anchor.Program<BatchTransfer>;
//
//
//     it("Is ResetUserAccount", async () => {
//         const accounts = [];
//         const admin = anchor.web3.Keypair.generate();
//         accounts.push(admin);
//         const userAccount = anchor.web3.Keypair.generate();
//         accounts.push(userAccount);
//
//         let initLamports = 1000000000
//
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
//
//         await program.methods
//             .resetUserAccount()
//             .accounts({
//                 userAccount: userAccount.publicKey,
//                 admin: admin.publicKey,
//                 // @ts-ignore
//                 systemProgram: anchor.web3.SystemProgram.programId,
//             })
//             .signers([admin, userAccount])
//             .rpc();
//
//         const userAccountRet = await program.account.userAccount.fetch(userAccount.publicKey);
//         assert.ok(userAccountRet.amount.eq(new anchor.BN(0)));
//     });
// });
//
