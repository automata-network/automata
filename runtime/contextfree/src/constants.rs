// SPDX-License-Identifier: Apache-2.0

/// Money matters.
pub mod currency {
    use automata_primitives::Balance;

    pub const MILLICENTS: Balance = 1_000_000_000_000;
    pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
    pub const DOLLARS: Balance = 1_000 * CENTS;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

/// Time.
pub mod time {
    use automata_primitives::BlockNumber;

    pub const MILLISECS_PER_BLOCK: u64 = 5000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 2 * MINUTES;
    pub const EPOCH_DURATION_IN_SLOTS: u64 = {
        const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;
        (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
    };

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}
