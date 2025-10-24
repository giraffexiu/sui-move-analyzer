/// Simple marketplace module for trading NFTs
/// This module provides additional examples for function analysis
module simple_nft::marketplace {
    use sui::object::{Self, UID};
    use sui::coin::{Self, Coin};
    use sui::sui::SUI;
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use simple_nft::nft::{Self, SimpleNFT};

    /// A listing for an NFT in the marketplace
    public struct Listing has key {
        id: UID,
        nft: SimpleNFT,
        price: u64,
        seller: address,
    }

    /// Event emitted when an NFT is listed for sale
    public struct NFTListed has copy, drop {
        listing_id: address,
        nft_name: std::string::String,
        price: u64,
        seller: address,
    }

    /// Event emitted when an NFT is purchased
    public struct NFTPurchased has copy, drop {
        listing_id: address,
        buyer: address,
        price: u64,
    }

    /// Create a new listing for an NFT
    /// This function demonstrates complex parameter handling and event emission
    public fun create_listing(
        nft: SimpleNFT,
        price: u64,
        ctx: &mut TxContext
    ): Listing {
        let seller = tx_context::sender(ctx);
        let nft_name = nft::name(&nft);
        
        let listing = Listing {
            id: object::new(ctx),
            nft,
            price,
            seller,
        };

        // Emit listing event
        sui::event::emit(NFTListed {
            listing_id: object::uid_to_address(&listing.id),
            nft_name,
            price,
            seller,
        });

        listing
    }

    /// Purchase an NFT from a listing
    /// This function demonstrates complex function calls and coin handling
    public fun purchase(
        listing: Listing,
        payment: Coin<SUI>,
        ctx: &mut TxContext
    ): SimpleNFT {
        let buyer = tx_context::sender(ctx);
        let Listing { id, nft, price, seller } = listing;

        // Verify payment amount
        assert!(coin::value(&payment) >= price, 1);

        // Transfer payment to seller
        transfer::public_transfer(payment, seller);

        // Emit purchase event
        sui::event::emit(NFTPurchased {
            listing_id: object::uid_to_address(&id),
            buyer,
            price,
        });

        // Clean up listing object
        object::delete(id);

        nft
    }

    /// Get the price of a listing
    public fun price(listing: &Listing): u64 {
        listing.price
    }

    /// Get the seller of a listing
    public fun seller(listing: &Listing): address {
        listing.seller
    }

    /// Cancel a listing and return the NFT to the seller
    /// This function demonstrates access control and object destruction
    public fun cancel_listing(listing: Listing, ctx: &TxContext): SimpleNFT {
        let Listing { id, nft, price: _, seller } = listing;
        
        // Only seller can cancel
        assert!(seller == tx_context::sender(ctx), 2);
        
        object::delete(id);
        nft
    }

    /// Create a listing and share it publicly
    /// This function demonstrates entry functions and public sharing
    public entry fun list_nft(
        nft: SimpleNFT,
        price: u64,
        ctx: &mut TxContext
    ) {
        let listing = create_listing(nft, price, ctx);
        transfer::share_object(listing);
    }

    /// Purchase an NFT and transfer it to the buyer
    /// This function demonstrates complex transaction flows
    public entry fun buy_nft(
        listing: Listing,
        payment: Coin<SUI>,
        ctx: &mut TxContext
    ) {
        let nft = purchase(listing, payment, ctx);
        let buyer = tx_context::sender(ctx);
        nft::transfer(nft, buyer);
    }
}