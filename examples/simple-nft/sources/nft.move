/// Simple NFT module demonstrating basic NFT functionality
/// This module provides examples for testing the Move Function Analyzer
module simple_nft::nft {
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use std::string::{Self, String};

    /// The NFT object that can be owned and transferred
    public struct SimpleNFT has key, store {
        id: UID,
        name: String,
        description: String,
        creator: address,
    }

    /// Event emitted when an NFT is minted
    public struct NFTMinted has copy, drop {
        nft_id: address,
        name: String,
        creator: address,
    }

    /// Create a new NFT with the given name and description
    /// This function demonstrates parameter handling and object creation
    public fun mint(
        name: vector<u8>,
        description: vector<u8>,
        ctx: &mut TxContext
    ): SimpleNFT {
        let sender = tx_context::sender(ctx);
        let nft = SimpleNFT {
            id: object::new(ctx),
            name: string::utf8(name),
            description: string::utf8(description),
            creator: sender,
        };

        // Emit minting event
        sui::event::emit(NFTMinted {
            nft_id: object::uid_to_address(&nft.id),
            name: nft.name,
            creator: sender,
        });

        nft
    }

    /// Transfer an NFT to a new owner
    /// This function demonstrates function calls and parameter passing
    public fun transfer(nft: SimpleNFT, recipient: address) {
        transfer::public_transfer(nft, recipient);
    }

    /// Get the name of an NFT
    /// This function demonstrates simple getter functionality
    public fun name(nft: &SimpleNFT): String {
        nft.name
    }

    /// Get the description of an NFT
    public fun description(nft: &SimpleNFT): String {
        nft.description
    }

    /// Get the creator of an NFT
    public fun creator(nft: &SimpleNFT): address {
        nft.creator
    }

    /// Burn an NFT, destroying it permanently
    /// This function demonstrates object destruction
    public fun burn(nft: SimpleNFT) {
        let SimpleNFT { id, name: _, description: _, creator: _ } = nft;
        object::delete(id);
    }

    /// Create and transfer an NFT in one transaction
    /// This function demonstrates multiple function calls and complex logic
    public entry fun mint_and_transfer(
        name: vector<u8>,
        description: vector<u8>,
        recipient: address,
        ctx: &mut TxContext
    ) {
        let nft = mint(name, description, ctx);
        transfer(nft, recipient);
    }

    /// Update the description of an NFT (only creator can do this)
    /// This function demonstrates mutable references and access control
    public fun update_description(
        nft: &mut SimpleNFT,
        new_description: vector<u8>,
        ctx: &TxContext
    ) {
        assert!(nft.creator == tx_context::sender(ctx), 0);
        nft.description = string::utf8(new_description);
    }
}