#![cfg_attr(not(feature = "std"), no_std)]
#[allow(unused_variables, unused_assignments, dead_code)]
#[ink::contract]
mod asset_co2_emissions {
    use ink::prelude::collections::{BTreeMap, BTreeSet};

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    /// Asset ID type.
    // TODO proper ID type
    pub type AssetId = u128;

    pub type Metadata = Vec<u8>;
    pub type RoleId = AccountId;
    pub type ParentRelation = u128;
    pub type ParentDetails = Option<(AssetId, ParentRelation)>;
    pub type AssetDetails = (AssetId, Metadata, Vec<CO2Emissions>, ParentDetails);
    pub type Description = Vec<u8>;

    pub const MAX_METADATA_LENGTH: u16 = 1024; // 1KB
    pub const MAX_EMISSIONS_PER_ASSET: u8 = 100;

    #[derive(Copy, Clone, Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    enum EmissionsCategory {
        Process,
        Transport,
        Upstream,
    }

    /// The AssetCO2Emissions error types.
    #[derive(Debug, PartialEq, Eq, Copy, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AssetCO2EmissionsError {
        // Overflow with AssetId.
        AssetIdOverflow,
        // When an Asset does not exist.
        AssetNotFound,
        // When an Asset has been already `Paused`.
        AlreadyPaused,
        // When not an Asset's Owner wants to take any action over the Asset.
        NotOwner,
        // When the calling account is not the owner of the contract
        NotContractOwner,
        // When an Asset is not in a `Paused` state.
        NotPaused,
        // When an Asset's parent is not found.
        ParentNotFound,
        // When an Asset's parent is not in `Paused` state.
        ParentNotPaused,
        // When CO2 Emissions vector is empty.
        EmissionsEmpty,
        // Too many emissions in vector
        EmissionsOverflow,
        // When CO2 Emissions item contains 0 emissions value.
        ZeroEmissionsItem,
        // When the Metadata vector contains too many characters
        MetadataOverflow,
        // When a parent <> child Asset relation is equal to 0
        InvalidAssetRelation,
        // When an Asset with ID already exists.
        AssetAlreadyExists,
    }

    /// The AccessControl error types.
    #[derive(Debug, PartialEq, Eq, Copy, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AccessControlError {
        // When an access has been already granted to the account.
        AlreadyGranted,
        // When a Role is going to be empty (no members).
        EmptyRole,
        // When an account is not a member of a Role.
        NotMember,
        // When an account is not an owner of the Role.
        NotOwner,
    }

    /// This emits when an Asset gets created.
    #[ink(event)]
    pub struct Blasted {
        #[ink(topic)]
        id: AssetId,
        metadata: Metadata,
        owner: AccountId,
        parent: ParentDetails,
    }

    /// This emits when ownership of any Asset changes.
    #[ink(event)]
    pub struct Transfer {
        from: AccountId,
        to: AccountId,
        #[ink(topic)]
        id: AssetId,
    }

    /// This emits when an Asset gets paused.
    #[ink(event)]
    pub struct Paused {
        #[ink(topic)]
        id: AssetId,
    }

    /// This emits when CO2 emission is added.
    #[ink(event)]
    pub struct Emission {
        #[ink(topic)]
        id: AssetId,
        category: EmissionsCategory,
        primary: bool,
        balanced: bool,
        date: u64,
        emissions: u128,
    }

    /// This emits when a new Role gets created.
    #[ink(event)]
    pub struct RoleCreated {
        #[ink(topic)]
        id: RoleId,
        owner: AccountId,
        description: Description,
    }

    /// This emits when an owner of the Role has changed.
    #[ink(event)]
    pub struct RoleOwnershipChanged {
        #[ink(topic)]
        id: RoleId,
        new_owner: AccountId,
    }

    /// This emits when a Role members have changed.
    #[ink(event)]
    pub struct RoleMembershipChanged {
        #[ink(topic)]
        id: RoleId,
    }

    #[derive(Clone, Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CO2Emissions {
        // Type of CO2 Emissions (bucket).
        category: EmissionsCategory,
        // Supplier primary/average; true if supplier-primary.
        primary: bool,
        // If CO2 Emissions are balanced (per record).
        balanced: bool,
        // Emissions in kg CO2 (to avoid fractions).
        emissions: u128,
        // Real CO2 emissions date as UNIX timestamp, not block creation time.
        date: u64,
    }

    #[ink::trait_definition]
    pub trait AccessControl {
        /// Function to create a new role.
        ///
        /// # Arguments
        ///
        /// * `description` - Additional role description.
        /// * `accounts` - Vector of accounts that are members of the role.
        ///
        /// # Errors
        ///
        /// * `EmptyRole` - Account list cannot be empty.
        ///
        /// # Events
        ///
        /// * `RoleCreated` - When a role gets created.
        ///
        #[ink(message)]
        fn create_role(
            &mut self,
            description: Description,
            accounts: Vec<AccountId>,
        ) -> Result<(), AccessControlError>;

        /// Transfer ownership of a role to another address.
        ///
        /// # Arguments
        ///
        /// * `role` - The Role id.
        /// * `to` - Address of the new owner.
        ///
        /// # Errors
        ///
        /// * `NotOwner` - When transaction sender is not the current role owner.
        ///
        /// # Events
        ///
        /// * `RoleOwnershipChanged` - When role ownership gets changed.
        #[ink(message)]
        fn transfer_role_ownership(
            &mut self,
            role: RoleId,
            to: AccountId,
        ) -> Result<(), AccessControlError>;

        /// Grant a role to an account.
        ///
        /// # Arguments
        ///
        /// * `role` - The Role id.
        /// * `account` - The account to be granted.
        ///
        /// # Errors
        ///
        /// * `NotOwner` - When transaction sender is not the current role owner.
        /// * `Already granted` - When the account is already role member.
        ///
        /// # Events
        ///
        /// * `RoleMembershipChanged`- When role membership gets changed.
        ///
        #[ink(message)]
        fn grant_role(
            &mut self,
            role: RoleId,
            account: AccountId,
        ) -> Result<(), AccessControlError>;

        /// Revoke a role from an account.
        ///
        /// # Arguments
        /// * `role` - The Role id.
        /// * `account` - The account to be revoked.
        ///
        /// # Errors
        ///
        /// * `NotOwner` - When transaction sender is not the current role owner.
        /// * `Already granted` - When the account is already role member.
        /// * `EmptyRole` - Account list cannot be emty.
        ///
        /// # Events
        ///
        /// * `RoleMembershipChanged`- When role membership gets changed.
        ///
        #[ink(message)]
        fn revoke_role(
            &mut self,
            role: RoleId,
            account: AccountId,
        ) -> Result<(), AccessControlError>;

        /// Checks if an account is a role member.
        ///
        /// # Arguments
        ///
        /// * `role` - The Role id.
        /// * `account` - The account id.
        ///
        #[ink(message)]
        fn has_role(&self, role: RoleId, account: AccountId) -> bool;

        /// Get role details.
        ///
        /// # Arguments
        ///
        /// * `role` - The Role id.
        ///
        #[ink(message)]
        fn get_role(&self, role: RoleId) -> Option<(Description, Vec<AccountId>)>;
    }

    #[ink::trait_definition]
    pub trait AssetCO2Emissions {
        /// List all Assets assigned to an owner.
        ///
        /// # Arguments
        ///
        /// * `owner` - An account for whom to query assets.
        ///
        #[ink(message)]
        fn list_assets(&self, owner: AccountId) -> Vec<AssetId>;

        /// Find the owner of an Asset.
        ///
        /// # Arguments
        ///
        /// * `id` - The identifier for an Asset.
        ///
        #[ink(message)]
        fn owner_of(&self, id: AssetId) -> Option<AccountId>;

        /// Blast an Asset.
        ///
        /// # Arguments
        ///
        /// * `to` - The account that will own the Asset.
        /// * `metadata` - Immutable asset's metadata (physical details of steel); Can be a string, a JSON string or a link to IPFS.
        /// * `emissions` - CO2 emissions during asset creation (like blasting or splitting steel).
        /// * `parent` - Information about asset creation from the exisitng Asset (in the case of e.g. splitting steel):
        ///                 - identifier of the Asset's parent
        ///                 - information about relation (parent's quantity used) for external systems.
        ///
        /// # Errors
        ///
        /// * `AssetNotFound` - When the Asset's parent does not exist.
        /// * `NotPaused`- When Asset's parent is not paused.
        ///
        /// # Events
        ///
        /// * `Blasted` - When an Asset gets blasted.
        /// * `Emissions` - When CO2 emissions are added.
        ///
        #[ink(message)]
        fn blast(
            &mut self,
            to: AccountId,
            metadata: Metadata,
            emissions: Vec<CO2Emissions>,
            parent: ParentDetails,
        ) -> Result<(), AssetCO2EmissionsError>;

        /// Transfers the ownership of an Asset to another account
        ///
        /// # Arguments
        ///
        /// * `to` - The new owner
        /// * `id` - The Asset to be transfered
        /// * `emissions` - CO2 emission caused by the Asset transfer
        ///
        /// # Errors
        ///
        /// * `AssetNotFound` - When the Asset does not exist.
        /// * `NotOwner` - When transaction sender is not an owner.
        /// * `AlreadyPaused` - When the Asset is paused.
        ///
        /// # Events
        ///
        /// * `Transfer` - When Asset gets transferred.
        /// * `Emissions` - When CO2 emissions are added.
        ///
        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            id: AssetId,
            emissions: Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError>;

        /// Set stopped state for an Asset.
        /// In this state no one is able to transfer/add emissions to the Asset.
        /// Should be used before splitting into smaller parts.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset to lock.
        ///
        /// # Errors
        ///
        /// * `AssetNotFound` - When the Asset does not exist.
        /// * `AlreadyPaused` - When the Asset is already paused.
        /// * `NotOwner` - When transaction sender is not an owner.
        ///
        /// # Events
        ///
        /// * `Paused` - When asset gets paused.
        #[ink(message)]
        fn pause(&mut self, id: AssetId) -> Result<(), AssetCO2EmissionsError>;

        /// Query if an Asset is paused.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset id.
        ///
        #[ink(message)]
        fn has_paused(&self, id: AssetId) -> Option<bool>;

        /// Add CO2 emissions to an Asset.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset id.
        /// * `emissions` - CO2 emissions caused by any real world action.
        ///
        /// # Errors
        ///
        /// * `AssetNotFound` - When the Asset does not exist.
        /// * `NotOwner` - When transaction sender is not an owner.
        /// * `AlreadyPaused` - When asset is paused.
        ///
        /// # Events
        ///
        /// * `Emissions` - When CO2 emissions are added.
        ///
        #[ink(message)]
        fn add_emissions(
            &mut self,
            id: AssetId,
            emissions: CO2Emissions,
        ) -> Result<(), AssetCO2EmissionsError>;

        /// Get specified Asset's CO2 emissions.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset id.
        ///
        #[ink(message)]
        fn get_asset_emissions(&self, id: AssetId) -> Option<Vec<CO2Emissions>>;

        /// Get specified Asset's metadata.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset id.
        ///
        #[ink(message)]
        fn get_metadata(&self, id: AssetId) -> Option<Metadata>;

        /// Get Asset's parent.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset id.
        ///
        #[ink(message)]
        fn get_parent_details(&self, id: AssetId) -> Option<ParentDetails>;

        /// Get asset details.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset id.
        ///
        #[ink(message)]
        fn get_asset(&self, id: AssetId) -> Option<AssetDetails>;

        /// Query Asset's emissions.
        /// This function returns CO2 emissions not only from specified Asset but also its parents.
        /// It returns full Asset's history from the Asset's tree.
        ///
        /// # Arguments
        ///
        /// * `id` - The Asset id.
        ///
        #[ink(message)]
        fn query_emissions(&self, id: AssetId) -> Option<Vec<AssetDetails>>;
    }

    #[ink(storage)]
    // #[derive(Default)]
    pub struct InfinityAsset {
        contract_owner: AccountId,
        next_id: AssetId,
        asset_owner: Mapping<AssetId, AccountId>,
        owned_assets: BTreeMap<AccountId, BTreeSet<AssetId>>,
        co2_emissions: Mapping<AssetId, Vec<CO2Emissions>>,
        metadata: Mapping<AssetId, Metadata>,
        paused: Mapping<AssetId, bool>,
        parent: Mapping<AssetId, ParentDetails>,
    }

    impl InfinityAsset {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                contract_owner: Self::env().caller(),
                next_id: 1,
                asset_owner: Mapping::new(),
                owned_assets: BTreeMap::new(),
                co2_emissions: Mapping::new(),
                metadata: Mapping::new(),
                paused: Mapping::new(),
                parent: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn set_contract_owner(
            &mut self,
            new_owner: AccountId,
        ) -> Result<(), AssetCO2EmissionsError> {
            // emit random event to showcase the update
            Err(AssetCO2EmissionsError::AlreadyPaused)
        }

        /// Modifies the code which is used to execute calls to this contract address (`AccountId`).
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: [u8; 32]) {
            self.ensure_contract_owner(self.env().caller())
                .expect("Only contract owner can set code hash");

            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
            });
            ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
        }

        fn insert_owned_asset(
            &mut self,
            owner: &AccountId,
            asset_id: &AssetId,
        ) -> Result<(), AssetCO2EmissionsError> {
            match self.owned_assets.get_mut(owner) {
                None => {
                    let mut new_owned_assets = BTreeSet::new();
                    new_owned_assets.insert(*asset_id);
                    self.owned_assets.insert(*owner, new_owned_assets);
                    Ok(())
                }
                Some(owned_assets) => {
                    owned_assets.insert(*asset_id);
                    Ok(())
                }
            }
        }

        fn remove_owned_asset(
            &mut self,
            owner: &AccountId,
            asset_id: &AssetId,
        ) -> Result<(), AssetCO2EmissionsError> {
            self.owned_assets
                .get_mut(owner)
                .expect("Owned assets must exist when removing during asset transfer")
                .remove(asset_id);
            Ok(())
        }

        fn ensure_not_exist(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.asset_owner.contains(id) {
                false => Ok(()),
                true => Err(AssetCO2EmissionsError::AssetAlreadyExists),
            }
        }

        fn ensure_exists(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.asset_owner.contains(id) {
                true => Ok(()),
                false => Err(AssetCO2EmissionsError::AssetNotFound),
            }
        }

        fn ensure_contract_owner(&self, caller: AccountId) -> Result<(), AssetCO2EmissionsError> {
            match caller.eq(&self.contract_owner) {
                true => Ok(()),
                false => Err(AssetCO2EmissionsError::NotContractOwner),
            }
        }

        fn ensure_owner(
            &self,
            id: &AssetId,
            account: &AccountId,
        ) -> Result<(), AssetCO2EmissionsError> {
            match self.asset_owner.get(id) {
                None => Err(AssetCO2EmissionsError::AssetNotFound),
                Some(owner) => {
                    if owner.eq(account) {
                        Ok(())
                    } else {
                        Err(AssetCO2EmissionsError::NotOwner)
                    }
                }
            }
        }
        fn ensure_paused(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.has_paused(*id) {
                None => Err(AssetCO2EmissionsError::AssetNotFound),
                Some(false) => Err(AssetCO2EmissionsError::NotPaused),
                Some(true) => Ok(()),
            }
        }

        fn ensure_not_paused(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.has_paused(*id) {
                None => Err(AssetCO2EmissionsError::AssetNotFound),
                Some(true) => Err(AssetCO2EmissionsError::AlreadyPaused),
                Some(false) => Ok(()),
            }
        }

        fn ensure_proper_parent(
            &self,
            parent: &ParentDetails,
            caller: &AccountId,
        ) -> Result<(), AssetCO2EmissionsError> {
            match parent {
                None => Ok(()),
                Some((parent_id, relation)) => {
                    self.ensure_owner(parent_id, caller)?;
                    match relation {
                        0 => Err(AssetCO2EmissionsError::InvalidAssetRelation),
                        _ => Ok(()),
                    }?;
                    self.ensure_paused(parent_id)
                }
            }
        }

        fn ensure_emissions_correct(
            &self,
            emissions: &Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            self.ensure_emissions_not_empty(emissions)?;
            self.ensure_emissions_not_unbounded(emissions)?;
            match emissions
                .iter()
                .all(|item| self.ensure_emissions_item_correct(item).is_ok())
            {
                false => Err(AssetCO2EmissionsError::ZeroEmissionsItem),
                true => Ok(()),
            }
        }

        fn ensure_emissions_not_empty(
            &self,
            emissions: &Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            match emissions.len() {
                0 => Err(AssetCO2EmissionsError::EmissionsEmpty),
                _ => Ok(()),
            }
        }

        fn ensure_emissions_not_unbounded(
            &self,
            emissions: &Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            if emissions.len() > MAX_EMISSIONS_PER_ASSET as usize {
                return Err(AssetCO2EmissionsError::EmissionsOverflow);
            }
            Ok(())
        }
        fn ensure_emissions_item_correct(
            &self,
            item: &CO2Emissions,
        ) -> Result<(), AssetCO2EmissionsError> {
            self.ensure_emissions_item_not_zero(item)?;
            Ok(())
        }

        fn ensure_emissions_item_not_zero(
            &self,
            item: &CO2Emissions,
        ) -> Result<(), AssetCO2EmissionsError> {
            match item.emissions {
                0 => Err(AssetCO2EmissionsError::ZeroEmissionsItem),
                _ => Ok(()),
            }
        }

        fn ensure_proper_metadata(
            &self,
            metadata: &Metadata,
        ) -> Result<(), AssetCO2EmissionsError> {
            if metadata.len() > MAX_METADATA_LENGTH as usize {
                return Err(AssetCO2EmissionsError::MetadataOverflow);
            }
            Ok(())
        }

        fn save_new_emissions(
            &mut self,
            id: &AssetId,
            emissions: &[CO2Emissions],
        ) -> Result<(), AssetCO2EmissionsError> {
            let mut updated_emissions = self.co2_emissions.get(id).unwrap_or(Vec::new());
            updated_emissions.extend_from_slice(emissions);

            self.ensure_emissions_not_unbounded(&updated_emissions)?;

            self.co2_emissions.insert(id, &updated_emissions);
            emissions.iter().for_each(|emission| {
                self.env().emit_event(Emission {
                    id: *id,
                    category: emission.category,
                    primary: emission.primary,
                    balanced: emission.balanced,
                    date: emission.date,
                    emissions: emission.emissions,
                })
            });
            Ok(())
        }

        fn next_id(&mut self) -> Result<AssetId, AssetCO2EmissionsError> {
            let asset_id = self.next_id;
            self.next_id = self
                .next_id
                .checked_add(1)
                .ok_or(AssetCO2EmissionsError::AssetIdOverflow)?;
            Ok(asset_id)
        }

        fn build_asset_tree(&self, id: AssetId) -> Vec<AssetDetails> {
            let mut asset_id = id;
            let mut tree_path: Vec<AssetDetails> = Vec::new();
            loop {
                // This function is called after initial check if asset exists
                // So it must contain asset and its childrn -- unwrap must be safe
                // It has been confirmed in previous test cases
                // If not, we need to capture that sth is wrong with the smart contract
                let asset: AssetDetails = self
                    .get_asset(asset_id)
                    .expect("Asset existence already checked");
                let parent_details = asset.3;
                tree_path.push(asset);
                match parent_details {
                    None => break,
                    Some((parent_id, _)) => asset_id = parent_id,
                }
            }

            tree_path
        }
    }

    impl AssetCO2Emissions for InfinityAsset {
        #[ink(message)]
        fn list_assets(&self, owner: AccountId) -> Vec<AssetId> {
            match self.owned_assets.get(&owner) {
                None => Vec::new(),
                Some(owned_assets) => owned_assets.iter().copied().collect::<Vec<AssetId>>(),
            }
        }

        #[ink(message)]
        fn owner_of(&self, id: AssetId) -> Option<AccountId> {
            self.asset_owner.get(id)
        }

        #[ink(message)]
        fn blast(
            &mut self,
            to: AccountId,
            metadata: Metadata,
            emissions: Vec<CO2Emissions>,
            parent: ParentDetails,
        ) -> Result<(), AssetCO2EmissionsError> {
            Ok(())
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            id: AssetId,
            emissions: Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            Ok(())
        }

        #[ink(message)]
        fn pause(&mut self, id: AssetId) -> Result<(), AssetCO2EmissionsError> {
            self.ensure_owner(&id, &self.env().caller())?;
            self.ensure_not_paused(&id)?;

            self.paused.insert(id, &true);
            self.env().emit_event(Paused { id });

            Ok(())
        }

        #[ink(message)]
        fn has_paused(&self, id: AssetId) -> Option<bool> {
            self.paused.get(id)
        }

        #[ink(message)]
        fn add_emissions(
            &mut self,
            id: AssetId,
            emissions: CO2Emissions,
        ) -> Result<(), AssetCO2EmissionsError> {
            self.ensure_exists(&id)?;
            self.ensure_owner(&id, &self.env().caller())?;
            self.ensure_not_paused(&id)?;
            self.ensure_emissions_item_correct(&emissions)?;

            // Save CO2 emissions & emit corresponding events
            self.save_new_emissions(&id, &Vec::from([emissions]))?;
            Ok(())
        }

        #[ink(message)]
        fn get_asset_emissions(&self, id: AssetId) -> Option<Vec<CO2Emissions>> {
            self.co2_emissions.get(id)
        }

        #[ink(message)]
        fn get_metadata(&self, id: AssetId) -> Option<Metadata> {
            self.metadata.get(id)
        }

        #[ink(message)]
        fn get_parent_details(&self, id: AssetId) -> Option<ParentDetails> {
            self.parent.get(id)
        }

        #[ink(message)]
        fn get_asset(&self, id: AssetId) -> Option<AssetDetails> {
            match self.get_metadata(id) {
                // Asset does not exist, retun None
                None => None,
                // Asset must exist, fetch and unpack attributes
                Some(metadata) => {
                    let emissions = self.get_asset_emissions(id).expect("Emissions must exist");
                    let parent = self.get_parent_details(id).expect("Parent must exist");

                    Some((id, metadata, emissions, parent))
                }
            }
        }

        #[ink(message)]
        fn query_emissions(&self, id: AssetId) -> Option<Vec<AssetDetails>> {
            match self.ensure_exists(&id) {
                Err(_) => None,
                Ok(_) => Some(self.build_asset_tree(id)),
            }
        }
    }
}
