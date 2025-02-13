## Instructions

### Admin Actions

#### Program-Derived Addresses (PDAs)

- **config**

| Field         | Type      | Description                                  |
| --------------- | --------- | -------------------------------------------- |
| admin         | PublicKey | Admin's public key.                          |
| collection_mint | PublicKey | Mint address of the NFT collection.          |
| game_vault    | PublicKey | Vault to hold game funds (fees, rewards).      |
| fees          | u8        | Percentage of fees collected by the game.    |
| bump          | u8        | Bump seed for the `config` PDA.               |
| vault_bump    | u8        | Bump seed for the `game_vault` PDA.          |

#### `initialize_game_config`

Initializes the program state and sets up necessary accounts.

- Creates the game vault PDA.
- Sets up the configuration account.
- Defines the initial admin public key and collection mint.
- Sets initial fee percentage.

#### `update_fees`

Allows the admin to modify the fee percentage.

### Player Actions

#### Program-Derived Addresses (PDAs)

- **Pet Stats PDA**: Stores stats related to a player's pet.
- **Pet Duel PDA**: Tracks pet duel records and states.

#### `deposit_nft`

Deposits a pet NFT into the game, initializing pet stats and game accounts.

- Transfers the player's NFT to the game's associated token account.
- Initializes a **Pet Stats PDA** for the deposited NFT.

#### `withdraw_nft`

Withdraws a pet NFT from the game.

- Transfers the pet NFT back to the player's associated token account.
- Closes the game's associated token account for the NFT.
- Closes the associated **Pet Stats PDA**.

#### `pet_interaction`

Players can interact with their pets, updating the **Pet Stats PDA** accordingly.

- **Feed**: Increase pet's hunger level.
- **Bath**: Increase pet's hygiene level.

#### `init_pet_duel`

Initializes a pet duel challenge.

- Creates a **Pet Duel PDA** to track the duel.
- Sets the challenger and initial duel parameters (bet amount).
- Sets the duel status to "Challenged".

#### `accept_pet_duel`

Allows another player to accept a pet duel challenge.

- Updates the **Pet Duel PDA** with the defender's information.
- Sets the duel status to "Started" (or "Going" if the duel starts immediately).

#### `pet_attack`

Allows the current turn player to perform an attack in a pet duel.

- Updates the **Pet Duel PDA** with the attack outcome (pet health changes).
- Switches the turn to the other player.
- Checks for duel completion and determines a winner if applicable.