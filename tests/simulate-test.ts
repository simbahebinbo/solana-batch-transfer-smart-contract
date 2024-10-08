// import * as anchor from "@coral-xyz/anchor";
// import {Program} from "@coral-xyz/anchor";
// import {BatchTransfer} from "../target/types/batch_transfer";
// import {getOrCreateAssociatedTokenAccount} from "@solana/spl-token";
//
// const {SystemProgram} = anchor.web3;
// const {TOKEN_PROGRAM_ID, Token, createMint} = require('@solana/spl-token');
//
// describe('batch-transfer', () => {
//     // Configure the client to use the local cluster.
//     let provider = anchor.AnchorProvider.env();
//     anchor.setProvider(provider);
//
//     const program = anchor.workspace.BatchTransfer as Program<BatchTransfer>;
//
//
//     it("Is Simulate", async () => {
//         const payer = anchor.web3.Keypair.generate();
//
//         let initLamports = 1000000000
//
//         let airdrop_tx = await provider.connection.requestAirdrop(
//             payer.publicKey,
//             initLamports
//         );
//         const latestBlockHash = await provider.connection.getLatestBlockhash();
//         await provider.connection.confirmTransaction({
//             blockhash: latestBlockHash.blockhash,
//             lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
//             signature: airdrop_tx
//         })
//
//
//         const mintAuthority = anchor.web3.Keypair.generate();
//         const mint = await createMint(
//             provider.connection,
//             payer,
//             mintAuthority.publicKey,
//             null,
//             9, // Decimal places
//         );
//         console.log("Mint created: ", mint.toBase58());
//
//         const tokenAccountAuthority = anchor.web3.Keypair.generate();
//         const tokenAccount = await getOrCreateAssociatedTokenAccount(
//             provider.connection,
//             payer,
//             mint,
//             tokenAccountAuthority.publicKey,
//         );
//         console.log("Associated Token Account: ", tokenAccount.address.toBase58());
//
//         await program.methods
//             .simulate()
//             .accounts({
//                 payer: payer.publicKey,
//                 //@ts-ignore
//                 tokenProgram: TOKEN_PROGRAM_ID,
//                 tokenAccount: tokenAccount.address,
//                 mint: mint,
//             })
//             .signers([payer])
//             .rpc();
//     });
// });
