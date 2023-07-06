#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod asset_co2_emissions {
    use ink::prelude::collections::{BTreeMap, BTreeSet};

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    // Max size of the Metadata vector.
    pub const MAX_METADATA_LENGTH: u16 = 1024; // 1KB

    // Max emissions per asset
    pub const MAX_EMISSIONS_PER_ASSET: u8 = 100;

    // Max size of DataSource
    pub const MAX_DATA_SOURCE_LENGTH: u8 = 128;

    /// Asset ID type.
    // TODO proper ID type
    pub type AssetId = u128;

    // Metadata represented by a vector of bytes/characters.
    pub type Metadata = Vec<u8>;

    // CO2 Emissions Data Source represented by vector of bytes/characters.
    pub type DataSource = Vec<u8>;

    // Optional argument for referencing a parent asset that is split into child assets.
    pub type ParentDetails = Option<AssetId>;

    // The type returned when querying for an Asset.
    #[derive(Debug, PartialEq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct AssetDetails {
        asset_id: AssetId,
        metadata: Metadata,
        emissions: Vec<CO2Emissions>,
        parent: ParentDetails,
    }

    #[derive(Copy, Clone, Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    enum EmissionsCategory {
        Process,
        Transport,
        Upstream,
    }

    #[derive(Clone, Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CO2Emissions {
        // Type of CO2 Emissions (bucket).
        category: EmissionsCategory,
        // Emissions source of data
        data_source: DataSource,
        // If CO2 Emissions are balanced (per record).
        balanced: bool,
        // Emissions in kg CO2 (to avoid fractions).
        value: u128,
        // Real CO2 emissions date as UNIX timestamp, not block creation time.
        date: u64,
    }

    /// The AssetCO2Emissions Error types.
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
        // When the data source vector contains too many characters
        DataSourceOverflow,
        // When an Asset with ID already exists.
        AssetAlreadyExists,
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
        data_source: DataSource,
        balanced: bool,
        date: u64,
        value: u128,
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
        /// * `parent` - Information about asset creation from the existing Asset (in the case of e.g. splitting steel) - identifier of the Asset's parent
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
        /// * `id` - The Asset to be transferred
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
    pub struct InfinityAsset {
        // privileged contract owner
        contract_owner: AccountId,
        // the next asset id to assign
        next_id: AssetId,
        // mapping asset id to its owner
        asset_owner: Mapping<AssetId, AccountId>,
        // mapping to find what assets an account has
        owned_assets: BTreeMap<AccountId, BTreeSet<AssetId>>,
        // emissions of an asset
        co2_emissions: Mapping<AssetId, Vec<CO2Emissions>>,
        // metadata of an asset
        metadata: Mapping<AssetId, Metadata>,
        // what assets are paused
        paused: Mapping<AssetId, bool>,
        // child asset's parent
        parent: Mapping<AssetId, ParentDetails>,
    }

    impl Default for InfinityAsset {
        fn default() -> Self {
            Self::new()
        }
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

        /// Sets the new contract owner. Must be called by current contract owner
        #[ink(message)]
        pub fn set_contract_owner(
            &mut self,
            new_owner: AccountId,
        ) -> Result<(), AssetCO2EmissionsError> {
            // Only the owner of the contract may set the new owner
            self.ensure_contract_owner(self.env().caller())?;
            self.contract_owner = new_owner;
            Ok(())
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

        /// Insert new asset in the assets of `owner`
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

        /// Remove asset from the assets of `owner`
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

        /// Ensure that asset does not exist.
        fn ensure_not_exist(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.asset_owner.contains(id) {
                false => Ok(()),
                true => Err(AssetCO2EmissionsError::AssetAlreadyExists),
            }
        }

        /// Ensure asset does exist
        fn ensure_exists(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.asset_owner.contains(id) {
                true => Ok(()),
                false => Err(AssetCO2EmissionsError::AssetNotFound),
            }
        }

        /// Ensure the calling origin is the contract owner
        fn ensure_contract_owner(&self, caller: AccountId) -> Result<(), AssetCO2EmissionsError> {
            match caller.eq(&self.contract_owner) {
                true => Ok(()),
                false => Err(AssetCO2EmissionsError::NotContractOwner),
            }
        }

        /// Ensure the calling origin is the asset owner
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

        /// Ensure the asset is `Paused`
        fn ensure_paused(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.has_paused(*id) {
                None => Err(AssetCO2EmissionsError::AssetNotFound),
                Some(false) => Err(AssetCO2EmissionsError::NotPaused),
                Some(true) => Ok(()),
            }
        }

        /// Ensure the asset is not `Paused`
        fn ensure_not_paused(&self, id: &AssetId) -> Result<(), AssetCO2EmissionsError> {
            match self.has_paused(*id) {
                None => Err(AssetCO2EmissionsError::AssetNotFound),
                Some(true) => Err(AssetCO2EmissionsError::AlreadyPaused),
                Some(false) => Ok(()),
            }
        }

        /// Ensure the parent details of child asset are correct
        fn ensure_proper_parent(
            &self,
            parent: &ParentDetails,
            caller: &AccountId,
        ) -> Result<(), AssetCO2EmissionsError> {
            match parent {
                None => Ok(()),
                Some(parent_id) => {
                    self.ensure_owner(parent_id, caller)?;
                    self.ensure_paused(parent_id)
                }
            }
        }

        /// Ensure emissions are correct: not empty, not unbounded, and all items are correct
        fn ensure_emissions_correct(
            &self,
            emissions: &Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            self.ensure_emissions_not_empty(emissions)?;
            self.ensure_emissions_not_unbounded(emissions)?;

            // ensure all emissions items are correct
            emissions.iter().try_for_each(|item| {
                self.ensure_emissions_item_correct(item)?;
                Ok(())
            })
        }

        /// Ensure emissions are not empty
        fn ensure_emissions_not_empty(
            &self,
            emissions: &Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            match emissions.len() {
                0 => Err(AssetCO2EmissionsError::EmissionsEmpty),
                _ => Ok(()),
            }
        }

        /// Ensure length of emissions vec is not greater than `MAX_EMISSIONS_PER_ASSET`
        fn ensure_emissions_not_unbounded(
            &self,
            emissions: &Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            if emissions.len() > MAX_EMISSIONS_PER_ASSET as usize {
                return Err(AssetCO2EmissionsError::EmissionsOverflow);
            }
            Ok(())
        }

        fn ensure_emissions_data_src_not_unbounded(
            &self,
            data_source: &DataSource,
        ) -> Result<(), AssetCO2EmissionsError> {
            if data_source.len() > MAX_DATA_SOURCE_LENGTH as usize {
                return Err(AssetCO2EmissionsError::DataSourceOverflow);
            }
            Ok(())
        }

        /// Ensure emissions item is correct
        fn ensure_emissions_item_correct(
            &self,
            item: &CO2Emissions,
        ) -> Result<(), AssetCO2EmissionsError> {
            self.ensure_emissions_data_src_not_unbounded(&item.data_source)?;
            self.ensure_emissions_item_not_zero(item)?;
            Ok(())
        }

        /// ensure emissions value is non-zero
        fn ensure_emissions_item_not_zero(
            &self,
            emissions: &CO2Emissions,
        ) -> Result<(), AssetCO2EmissionsError> {
            match emissions.value {
                0 => Err(AssetCO2EmissionsError::ZeroEmissionsItem),
                _ => Ok(()),
            }
        }

        /// Ensure metadata does not exceed `MAX_METADATA_LENGTH`
        fn ensure_proper_metadata(
            &self,
            metadata: &Metadata,
        ) -> Result<(), AssetCO2EmissionsError> {
            if metadata.len() > MAX_METADATA_LENGTH as usize {
                return Err(AssetCO2EmissionsError::MetadataOverflow);
            }
            Ok(())
        }

        /// Save new emissions for asset and emit an event for each emission
        fn save_new_co2_emissions(
            &mut self,
            id: &AssetId,
            emissions: &[CO2Emissions],
        ) -> Result<(), AssetCO2EmissionsError> {
            let mut updated_emissions = self.co2_emissions.get(id).unwrap_or(Vec::new());
            updated_emissions.extend_from_slice(emissions);

            self.ensure_emissions_not_unbounded(&updated_emissions)?;

            self.co2_emissions.insert(id, &updated_emissions);
            // emit an event for each emission
            emissions.iter().for_each(|emission| {
                self.env().emit_event(Emission {
                    id: *id,
                    category: emission.category,
                    data_source: emission.data_source.clone(),
                    balanced: emission.balanced,
                    date: emission.date,
                    value: emission.value,
                })
            });
            Ok(())
        }

        /// Return the next id and increase by 1
        fn next_id(&mut self) -> Result<AssetId, AssetCO2EmissionsError> {
            let asset_id = self.next_id;
            self.next_id = self
                .next_id
                .checked_add(1)
                .ok_or(AssetCO2EmissionsError::AssetIdOverflow)?;
            Ok(asset_id)
        }

        /// Build asset tree from child to parent
        fn build_asset_tree(&self, id: AssetId) -> Vec<AssetDetails> {
            let mut asset_id = id;
            let mut tree_path: Vec<AssetDetails> = Vec::new();
            loop {
                // This function is called after initial check if asset exists
                // So it must contain asset and its children -- unwrap must be safe
                // It has been confirmed in previous test cases
                // If not, we need to capture that sth is wrong with the smart contract
                let asset: AssetDetails = self
                    .get_asset(asset_id)
                    .expect("Asset existence already checked");
                let parent_details = asset.parent.clone();
                tree_path.push(asset);
                match parent_details {
                    None => break,
                    Some(parent_id) => asset_id = parent_id,
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
            let caller = self.env().caller();

            self.ensure_proper_metadata(&metadata)?;
            self.ensure_emissions_correct(&emissions)?;
            self.ensure_proper_parent(&parent, &caller)?;

            let asset_id: u128 = self.next_id()?;
            self.ensure_not_exist(&asset_id)?;

            self.insert_owned_asset(&to, &asset_id)?;

            self.asset_owner.insert(asset_id, &to);
            self.metadata.insert(asset_id, &metadata);
            self.paused.insert(asset_id, &false);
            self.parent.insert(asset_id, &parent);

            self.env().emit_event(Blasted {
                id: asset_id,
                metadata,
                owner: to,
                parent,
            });

            // Save CO2 emissions & emit corresponding events
            self.save_new_co2_emissions(&asset_id, &emissions)?;

            Ok(())
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            id: AssetId,
            emissions: Vec<CO2Emissions>,
        ) -> Result<(), AssetCO2EmissionsError> {
            let from = self.env().caller();

            self.ensure_exists(&id)?;
            self.ensure_owner(&id, &from)?;
            self.ensure_not_paused(&id)?;
            self.ensure_emissions_correct(&emissions)?;

            self.remove_owned_asset(&from, &id)?;
            self.insert_owned_asset(&to, &id)?;

            self.asset_owner.insert(id, &to);

            self.env().emit_event(Transfer { from, to, id });

            // Save CO2 emissions & emit corresponding events
            self.save_new_co2_emissions(&id, &emissions)?;

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
            self.save_new_co2_emissions(&id, &Vec::from([emissions]))?;
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
                // Asset does not exist, return None
                None => None,
                // Asset must exist, fetch and unpack attributes
                Some(metadata) => {
                    let emissions = self.get_asset_emissions(id).expect("Emissions must exist");
                    let parent = self
                        .get_parent_details(id)
                        .expect("Parent Details must exist");

                    Some(AssetDetails {
                        asset_id: id,
                        metadata,
                        emissions,
                        parent,
                    })
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

    /// Unit tests
    #[cfg(test)]
    mod tests {
        use ink::env::test;
        use ink::env::test::DefaultAccounts;
        use ink::env::DefaultEnvironment;

        use super::*;

        use ink::primitives::{Clear, Hash};

        type Event = <InfinityAsset as ::ink::reflect::ContractEventBase>::Type;

        fn get_accounts() -> DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }

        fn prepare_env() -> (DefaultAccounts<DefaultEnvironment>, InfinityAsset) {
            (get_accounts(), InfinityAsset::new())
        }

        fn env_with_default_asset() -> (
            (DefaultAccounts<DefaultEnvironment>, InfinityAsset),
            (AssetId, AccountId),
        ) {
            let (accounts, mut contract) = prepare_env();
            let asset_owner = accounts.django;
            let asset_id = blast_default_asset(&mut contract, &asset_owner);
            ((accounts, contract), (asset_id, asset_owner))
        }

        fn set_caller(sender: AccountId) {
            test::set_caller::<DefaultEnvironment>(sender);
        }

        fn new_emission(
            category: EmissionsCategory,
            data_source: DataSource,
            balanced: bool,
            value: u128,
            date: u64,
        ) -> CO2Emissions {
            CO2Emissions {
                category,
                data_source,
                balanced,
                value,
                date,
            }
        }

        fn default_data_source() -> Vec<u8> {
            Vec::from([0u8, 1u8, 2u8, 3u8])
        }

        fn new_emissions(items: u8) -> Vec<CO2Emissions> {
            let mut emissions = Vec::new();
            for i in 0..items {
                emissions.push(new_emission(
                    EmissionsCategory::Upstream,
                    default_data_source(),
                    true,
                    i as u128 + 1, // avoid Zero Emissions Item
                    default_timestamp(),
                ));
            }
            emissions
        }

        fn default_metadata() -> Vec<u8> {
            Vec::from([0u8, 1u8, 2u8, 3u8])
        }

        fn default_timestamp() -> u64 {
            // 28.04.2023 00:00:00
            1682632800
        }

        fn default_emission_item() -> CO2Emissions {
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_data_source = default_data_source();
            let emissions_balanced = true;
            let emissions_value = 1;
            let timestamp: u64 = default_timestamp();

            new_emission(
                emissions_category,
                emissions_data_source,
                emissions_balanced,
                emissions_value,
                timestamp,
            )
        }

        fn blast_default_asset(contract: &mut InfinityAsset, owner: &AccountId) -> AssetId {
            let metadata = default_metadata();
            let parent = None;

            let emissions: Vec<CO2Emissions> = new_emissions(1);

            assert!(contract.blast(*owner, metadata, emissions, parent).is_ok());

            contract.next_id - 1
        }

        /// For calculating the event topic hash.
        struct PrefixedValue<'a, 'b, T> {
            pub prefix: &'a [u8],
            pub value: &'b T,
        }

        impl<X> scale::Encode for PrefixedValue<'_, '_, X>
        where
            X: scale::Encode,
        {
            #[inline]
            fn size_hint(&self) -> usize {
                self.prefix.size_hint() + self.value.size_hint()
            }

            #[inline]
            fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
                self.prefix.encode_to(dest);
                self.value.encode_to(dest);
            }
        }

        fn encoded_into_hash<T>(entity: &T) -> Hash
        where
            T: scale::Encode,
        {
            use ink::{
                env::hash::{Blake2x256, CryptoHash, HashOutput},
                primitives::Clear,
            };

            let mut result = Hash::CLEAR_HASH;
            let len_result = result.as_ref().len();
            let encoded = entity.encode();
            let len_encoded = encoded.len();
            if len_encoded <= len_result {
                result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                return result;
            }
            let mut hash_output = <<Blake2x256 as HashOutput>::Type as Default>::default();
            <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
            result
        }

        fn assert_blasted_event(
            event: &test::EmittedEvent,
            expected_id: AssetId,
            expected_metadata: Metadata,
            expected_owner: AccountId,
            expected_parent: ParentDetails,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("Encountered invalid contract event data buffer");
            if let Event::Blasted(Blasted {
                id,
                metadata,
                owner,
                parent,
            }) = decoded_event
            {
                assert_eq!(id, expected_id, "encountered invalid Blasted.id");
                assert_eq!(
                    metadata, expected_metadata,
                    "encountered invalid Blasted.metadata"
                );
                assert_eq!(owner, expected_owner, "encountered invalid Blasted.owner");
                assert_eq!(
                    parent, expected_parent,
                    "encountered invalid Blasted.parent"
                );
            } else {
                panic!("encountered unexpected event kind: expected a Blasted event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"InfinityAsset::Blasted",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"InfinityAsset::Blasted::id",
                    value: &expected_id,
                }),
            ];
            assert_event_topics(expected_topics, event.topics.clone());
        }

        fn assert_paused_event(event: &test::EmittedEvent, expected_id: AssetId) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("Encountered invalid contract event data buffer");
            if let Event::Paused(Paused { id }) = decoded_event {
                assert_eq!(id, expected_id, "encountered invalid Paused.id");
            } else {
                panic!("encountered unexpected event kind: expected a Paused event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"InfinityAsset::Paused",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"InfinityAsset::Paused::id",
                    value: &expected_id,
                }),
            ];
            assert_event_topics(expected_topics, event.topics.clone());
        }

        fn assert_emissions_event(
            event: &test::EmittedEvent,
            expected_id: AssetId,
            expected_category: EmissionsCategory,
            expected_data_source: DataSource,
            expected_balanced: bool,
            expected_date: u64,
            expected_value: u128,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("Encountered invalid contract event data buffer");
            if let Event::Emission(Emission {
                id,
                category,
                data_source,
                balanced,
                date,
                value,
            }) = decoded_event
            {
                assert_eq!(id, expected_id, "encountered invalid Emission.id");
                assert_eq!(
                    category, expected_category,
                    "encountered invalid Emission.category"
                );
                assert_eq!(
                    data_source, expected_data_source,
                    "encountered invalid Emission.data_source"
                );
                assert_eq!(
                    balanced, expected_balanced,
                    "encountered invalid Emission.balanced"
                );
                assert_eq!(date, expected_date, "encountered invalid Emission.date");
                assert_eq!(value, expected_value, "encountered invalid Emission.value");
            } else {
                panic!("encountered unexpected event kind: expected an Emission event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"InfinityAsset::Emission",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"InfinityAsset::Emission::id",
                    value: &expected_id,
                }),
            ];
            assert_event_topics(expected_topics, event.topics.clone());
        }

        fn assert_transfer_event(
            event: &test::EmittedEvent,
            expected_id: AssetId,
            expected_from: AccountId,
            expected_to: AccountId,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("Encountered invalid contract event data buffer");
            if let Event::Transfer(Transfer { id, from, to }) = decoded_event {
                assert_eq!(id, expected_id, "encountered invalid Transfer.id");
                assert_eq!(from, expected_from, "encountered invalid Transfer.from");
                assert_eq!(to, expected_to, "encountered invalid Transfer.to");
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"InfinityAsset::Transfer",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"InfinityAsset::Transfer::id",
                    value: &expected_id,
                }),
            ];
            assert_event_topics(expected_topics, event.topics.clone());
        }

        fn assert_event_topics(expected: Vec<Hash>, topics: Vec<Vec<u8>>) {
            for (n, (actual_topic, expected_topic)) in topics.iter().zip(expected).enumerate() {
                let mut topic_hash = Hash::CLEAR_HASH;
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(
                    topic_hash, expected_topic,
                    "encountered invalid topic at {n}"
                );
            }
        }

        #[ink::test]
        fn should_set_new_owner() {
            let (accounts, mut contract) = prepare_env();

            // Check if proper contract owner set
            assert_eq!(contract.contract_owner, accounts.alice);

            // Set a new contract owner
            assert!(contract.set_contract_owner(accounts.bob).is_ok());

            // Check if new contract owner is properly set
            assert_eq!(contract.contract_owner, accounts.bob);
        }

        #[ink::test]
        fn should_reject_set_new_owner() {
            let (accounts, mut contract) = prepare_env();

            // Check initial contract owner
            assert_eq!(contract.contract_owner, accounts.alice);

            set_caller(accounts.bob);

            // Check if proper error is returned
            // While trying to set a new contract owner as nonowner
            assert_eq!(
                contract.set_contract_owner(accounts.bob),
                Err(AssetCO2EmissionsError::NotContractOwner)
            );

            // Check if contract owner remain unchanged
            assert_eq!(contract.contract_owner, accounts.alice);
        }

        #[ink::test]
        #[should_panic(expected = "Only contract owner can set code hash")]
        fn should_reject_set_code() {
            let (accounts, mut contract) = prepare_env();

            // Check initial contract owner
            assert_eq!(contract.contract_owner, accounts.alice);

            set_caller(accounts.bob);

            // Check if panic
            // While trying to set contract code as nonowner
            contract.set_code([0x0; 32]);
        }

        #[ink::test]
        fn should_reject_empty_emissions_during_blast() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;
            let emissions: Vec<CO2Emissions> = Vec::new();

            // Check if proper error is returned
            // While trying to blast asset without CO2 Emissions items
            assert_eq!(
                contract.blast(accounts.alice, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::EmissionsEmpty)
            );
        }

        #[ink::test]
        fn should_reject_single_zero_emissions_item_during_blast() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;

            let mut item = default_emission_item();
            item.value = 0;

            let mut emissions: Vec<CO2Emissions> = new_emissions(1);
            emissions[0].value = 0;

            // Check if proper error is returned
            // While trying to blast asset with Zero emissions item
            assert_eq!(
                contract.blast(accounts.alice, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_reject_zero_emissions_item_in_array_during_blast() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;

            let mut emissions = new_emissions(3);
            emissions[1].value = 0;

            // Check if proper error is returned
            // While trying to blast asset with a Zero emissions item in many items
            assert_eq!(
                contract.blast(accounts.alice, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_blast_with_single_emissions_item() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;

            let emissions: Vec<CO2Emissions> = new_emissions(1);

            let owner = accounts.bob;

            // Check if asset is blasted
            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent.clone())
                .is_ok());

            let expected_asset_id = 1;

            let emitted_events = test::recorded_events().collect::<Vec<_>>();

            // Check events count
            // 1 * Blasted + 1 * Emission
            assert_eq!(1 + 1, emitted_events.len());

            // Check Blasted event
            assert_blasted_event(
                &emitted_events[0],
                expected_asset_id,
                metadata,
                owner,
                parent,
            );

            // Check Emission event
            assert_emissions_event(
                &emitted_events[1],
                expected_asset_id,
                emissions[0].category,
                emissions[0].data_source.clone(),
                emissions[0].balanced,
                emissions[0].date,
                emissions[0].value,
            );
        }

        #[ink::test]
        fn should_blast_with_multiple_emissions_items() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;

            let emissions_items_count: u8 = 3;
            let emissions = new_emissions(emissions_items_count);

            let owner = accounts.eve;

            // Check if asset is blasted
            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent.clone())
                .is_ok());

            let expected_asset_id = 1;

            let emitted_events = test::recorded_events().collect::<Vec<_>>();

            // Check proper events count
            // 1 * Blasted + 3 * Emission
            assert_eq!(
                1 + emissions_items_count,
                test::recorded_events().count() as u8
            );

            // Check Blasted event for the asset
            assert_blasted_event(
                &emitted_events[0],
                expected_asset_id,
                metadata,
                owner,
                parent,
            );

            // Check Emission event for each emission item
            for i in 0..(emissions_items_count as usize) {
                assert_emissions_event(
                    &emitted_events[i + 1],
                    expected_asset_id,
                    emissions[i].category,
                    emissions[i].data_source.clone(),
                    emissions[i].balanced,
                    emissions[i].date,
                    emissions[i].value,
                );
            }
        }

        #[ink::test]
        fn should_nonexistent_get_emissions_work_properly() {
            let contract = InfinityAsset::new();

            // Check if contract return proper value for nonexistent asset
            assert!(contract.get_asset_emissions(1000).is_none());
        }

        #[ink::test]
        fn should_get_emissions_work_properly() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;

            let emissions = new_emissions(3);

            let asset_id = 1;

            // Check if asset is blasted
            assert!(contract
                .blast(accounts.eve, metadata, emissions.clone(), parent)
                .is_ok());

            let emissions_from_state = contract.get_asset_emissions(asset_id);

            // Check if contract return proper emissions data
            assert!(emissions_from_state.is_some());
            assert!(emissions.iter().eq(emissions_from_state.unwrap().iter()));
        }

        #[ink::test]
        fn should_nonexistent_get_metadata_work_properly() {
            let contract = InfinityAsset::new();

            // Check if contract return proper value for nonexistent asset
            assert!(contract.get_metadata(1000).is_none());
        }

        #[ink::test]
        fn should_get_metadata_work_properly() {
            let ((_accounts, contract), (asset_id, _asset_owner)) = env_with_default_asset();

            let metadata = default_metadata();

            let metadata_from_state = contract.get_metadata(asset_id);

            // Check if contract return proper metadata
            assert!(metadata_from_state.is_some());
            assert!(metadata.iter().eq(metadata_from_state.unwrap().iter()));
        }

        #[ink::test]
        fn should_nonexistent_get_parent_work_properly() {
            let contract = InfinityAsset::new();

            // Check if contract return proper value for nonexistent asset
            assert!(contract.get_parent_details(1000).is_none());
        }

        #[ink::test]
        fn should_get_parent_for_root_asset_work_properly() {
            let ((_accounts, contract), (asset_id, _asset_owner)) = env_with_default_asset();

            let parent = None;

            let parent_from_state = contract.get_parent_details(asset_id);

            // Check if contract return proper asset's parent
            assert!(parent_from_state.is_some());
            assert_eq!(parent, parent_from_state.unwrap());
        }

        #[ink::test]
        fn should_owner_of_work_properly() {
            let ((_accounts, contract), (asset_id, asset_owner)) = env_with_default_asset();

            let owner_from_state = contract.owner_of(asset_id);

            // Check if contract return proper asset's owner
            assert!(owner_from_state.is_some());
            assert_eq!(asset_owner, owner_from_state.unwrap());
        }

        #[ink::test]
        fn should_already_blasted_asset_not_be_paused() {
            let ((_accounts, contract), (asset_id, _asset_owner)) = env_with_default_asset();

            let paused = contract.has_paused(asset_id);

            // Check if asset is not paused
            assert!(paused.is_some());
            assert!(!paused.unwrap());
        }

        #[ink::test]
        fn should_not_owner_not_be_able_to_set_paused_state() {
            let ((accounts, mut contract), (asset_id, _asset_owner)) = env_with_default_asset();

            set_caller(accounts.eve);

            // Check if proper error is returned
            // While trying to pause an asset as nonowner
            assert_eq!(
                contract.pause(asset_id),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_owner_be_able_to_set_paused_state() {
            let ((_accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            // Check if `pause` work
            assert!(contract.pause(asset_id).is_ok());

            let emitted_events = test::recorded_events().collect::<Vec<_>>();

            // Check events count
            // 1* Blasted + 1 * Emission + 1 * Paused
            assert_eq!(1 + 1 + 1, test::recorded_events().count());

            // Check Paused event
            assert_paused_event(&emitted_events[2], asset_id);
        }

        #[ink::test]
        fn should_owner_not_be_able_to_set_paused_state_while_already_paused() {
            let ((_accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            // Pause the asset
            assert!(contract.pause(asset_id).is_ok());

            // Check if proper error is returned
            // While trying to pause already paused asset
            assert_eq!(
                contract.pause(asset_id),
                Err(AssetCO2EmissionsError::AlreadyPaused)
            );
        }

        #[ink::test]
        fn should_reject_non_existent_parent_in_blast() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();

            let emissions: Vec<CO2Emissions> = new_emissions(1);

            let owner = accounts.alice;

            let parent: ParentDetails = Some(1000);

            set_caller(owner);

            // Check if proper error is returned
            // While trying to blast asset with nonexistent parent
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_reject_not_owner_creating_child_in_blast() {
            let ((accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            let emissions = new_emissions(1);
            let metadata = default_metadata();

            let parent: ParentDetails = Some(asset_id);

            set_caller(accounts.alice);

            // Check if proper error is returned
            // While trying to blast child asset as parent nonowner
            assert_eq!(
                contract.blast(asset_owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_reject_not_paused_in_blast() {
            let ((_accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            let emissions = new_emissions(1);
            let metadata = default_metadata();

            set_caller(asset_owner);

            let parent: ParentDetails = Some(asset_id);

            // Check if proper error is returned
            // While trying to blast a child asset for not paused parent
            assert_eq!(
                contract.blast(asset_owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::NotPaused)
            );
        }

        #[ink::test]
        fn should_blast_child() {
            let ((_accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            let metadata = default_metadata();
            let emissions = new_emissions(1);
            let parent: ParentDetails = Some(asset_id);

            // Pause parent asset
            assert!(contract.pause(asset_id).is_ok());

            // Blast child asset
            assert!(contract
                .blast(
                    asset_owner,
                    metadata.clone(),
                    emissions.clone(),
                    parent.clone()
                )
                .is_ok());

            let expected_asset_id = 2;

            let emitted_events = test::recorded_events().collect::<Vec<_>>();

            // Check events count
            // 1 * Blasted + 1 * Emission + 1 * Paused + 1 * Blasted + 1 * Emission
            assert_eq!(1 + 1 + 1 + 1 + 1, emitted_events.len());

            // Check Blasted event
            assert_blasted_event(
                &emitted_events[3],
                expected_asset_id,
                metadata,
                asset_owner,
                parent.clone(),
            );

            // Check Emission event for already blasted child asset
            assert_emissions_event(
                &emitted_events[4],
                expected_asset_id,
                emissions[0].category,
                emissions[0].data_source.clone(),
                emissions[0].balanced,
                emissions[0].date,
                emissions[0].value,
            );

            // Check child asset's parent
            let parent_from_state = contract.get_parent_details(expected_asset_id);
            assert!(parent_from_state.is_some());
            assert_eq!(parent, parent_from_state.unwrap());
        }

        #[ink::test]
        fn should_not_add_emissions_to_nonexistent_asset() {
            let mut contract = InfinityAsset::new();

            let item = default_emission_item();

            let asset_id = 1;

            // Check if proper error is returned
            // While trying to add CO2 Emissions to nonexistent asset
            assert_eq!(
                contract.add_emissions(asset_id, item),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_not_owner_not_be_able_to_add_emissions() {
            let ((accounts, mut contract), (asset_id, _asset_owner)) = env_with_default_asset();

            set_caller(accounts.bob);

            // Check if proper error is returned
            // While trying to add CO2 Emissions as asset's nonowner
            assert_eq!(
                contract.add_emissions(asset_id, default_emission_item()),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_reject_paused_in_add_emissions() {
            let ((_accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            // Pause the asset first
            assert!(contract.pause(asset_id).is_ok());

            // Check if proper error is returned
            // While trying to add CO2 Emissions in Paused state
            assert_eq!(
                contract.add_emissions(asset_id, default_emission_item()),
                Err(AssetCO2EmissionsError::AlreadyPaused)
            );
        }

        #[ink::test]
        fn should_reject_zero_emissions_item_in_add_emissions() {
            let ((_accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            // Prepare Zero emissions item
            let mut new_emission_item = default_emission_item();
            new_emission_item.value = 0u128;

            // Check if proper error is returned
            // While trying to add Zero emissions item in `add_emissions`
            assert_eq!(
                contract.add_emissions(asset_id, new_emission_item),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_owner_be_able_to_add_emissions() {
            let ((_accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            let emission_item = default_emission_item();

            // Add CO2 Emissions item
            assert!(contract
                .add_emissions(asset_id, emission_item.clone())
                .is_ok());

            let emitted_events = test::recorded_events().collect::<Vec<_>>();

            // Check events count
            // 1 * Blasted + 1 * Emissions + 1 * Emissions
            assert_eq!(1 + 1 + 1, emitted_events.len());
            assert_emissions_event(
                &emitted_events[2],
                asset_id,
                emission_item.category,
                emission_item.data_source.clone(),
                emission_item.balanced,
                emission_item.date,
                emission_item.value,
            );

            let expected_emissions: Vec<CO2Emissions> =
                Vec::from([default_emission_item(), emission_item]);
            let emissions_from_state = contract.get_asset_emissions(asset_id);

            // Check if contract return proper CO2 Emissions data
            assert!(emissions_from_state.is_some());
            assert!(expected_emissions
                .iter()
                .eq(emissions_from_state.unwrap().iter()));
        }

        #[ink::test]
        fn should_not_transfer_nonexistent_asset() {
            let (accounts, mut contract) = prepare_env();

            let item = default_emission_item();

            let asset_id = 1;

            set_caller(accounts.alice);

            // Check if proper error is returned
            // While trying to transfer nonexistent asset
            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::from([item])),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_not_owner_not_be_able_to_transfer() {
            let ((accounts, mut contract), (asset_id, _asset_owner)) = env_with_default_asset();

            set_caller(accounts.bob);

            // Check if proper error is returned
            // While trying to transfer asset as nonowner
            assert_eq!(
                contract.transfer(accounts.bob, asset_id, new_emissions(1)),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_reject_transfer_in_paused() {
            let ((accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            // Pause an asset
            assert!(contract.pause(asset_id).is_ok());

            // Check if proper error is returned
            // While trying to transfer asset in Paused state
            assert_eq!(
                contract.transfer(accounts.bob, asset_id, new_emissions(2)),
                Err(AssetCO2EmissionsError::AlreadyPaused)
            );
        }

        #[ink::test]
        fn should_reject_empty_emissions_in_transfer() {
            let ((accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            // Check if proper error is returned
            // While trying to transfer asset with empty vector of CO2 Emissions items
            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::new()),
                Err(AssetCO2EmissionsError::EmissionsEmpty)
            );
        }

        #[ink::test]
        fn should_reject_too_many_emissions_on_blast() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;

            let emissions: Vec<CO2Emissions> = new_emissions(MAX_EMISSIONS_PER_ASSET + 1);

            let expected_asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            // Check if proper error is returned
            // While trying to blast asset with too many emissions items
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::EmissionsOverflow)
            );

            // Check if asset is not blasted
            assert!(contract.get_asset(expected_asset_id).is_none());
        }

        #[ink::test]
        fn should_reject_too_many_emissions_on_add() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();
            let parent = None;

            let emissions: Vec<CO2Emissions> = new_emissions(MAX_EMISSIONS_PER_ASSET);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            // Blast asset with max emissions items count
            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());

            let item = default_emission_item();

            // Check if proper error is returned
            // While trying to add emission item causing overflow
            assert_eq!(
                contract.add_emissions(asset_id, item.clone()),
                Err(AssetCO2EmissionsError::EmissionsOverflow)
            );

            // Check if proper error is returned
            // While trying to transfer with emission item causing overflow
            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::from([item])),
                Err(AssetCO2EmissionsError::EmissionsOverflow)
            );
        }

        #[ink::test]
        fn should_reject_too_many_chars_in_data_source() {
            let (accounts, mut contract) = prepare_env();

            let metadata = default_metadata();

            let parent = None;

            let emissions_data_source = vec![1u8; MAX_DATA_SOURCE_LENGTH as usize + 1];
            let mut item = default_emission_item();
            item.data_source = emissions_data_source;

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let owner = accounts.alice;

            set_caller(owner);

            let expected_asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            // Check if proper error is returned
            // While trying to blast asset with emissions item `data_source` overflow
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::DataSourceOverflow)
            );

            // Check if asset is not blasted
            assert!(contract.get_asset(expected_asset_id).is_none());
        }

        #[ink::test]
        fn should_reject_zero_emissions_item_in_transfer() {
            let ((accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            let mut emissions = new_emissions(4);
            emissions[2].value = 0;

            // Check if proper error is returned
            // While trying to transfer with Zero emission item
            assert_eq!(
                contract.transfer(accounts.bob, asset_id, emissions),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_reject_too_many_chars_in_metadata() {
            let (accounts, mut contract) = prepare_env();

            let metadata: Metadata = vec![0u8; MAX_METADATA_LENGTH as usize + 1];

            let parent = None;

            let emissions = new_emissions(1);

            let expected_asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            // Check if proper error is returned
            // While trying to blast asset with `metadata` overflow
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::MetadataOverflow)
            );

            // Check if asset is not blasted
            assert!(contract.get_asset(expected_asset_id).is_none());
        }

        #[ink::test]
        fn should_owner_be_able_to_transfer() {
            let ((accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            let _item = default_emission_item();

            let new_owner = accounts.bob;

            let emissions = new_emissions(1);

            // Transfer asset
            assert!(contract
                .transfer(new_owner, asset_id, emissions.clone())
                .is_ok());

            let emitted_events = test::recorded_events().collect::<Vec<_>>();

            // Check emitted evetns count
            // 1 * Blasted + 1 * Emission + 1 * Transfer + 1 * Emission
            assert_eq!(1 + 1 + 1 + 1, emitted_events.len());

            // Check Transfer event
            assert_transfer_event(&emitted_events[2], asset_id, asset_owner, new_owner);

            // Check Emission event
            assert_emissions_event(
                &emitted_events[3],
                asset_id,
                emissions[0].category,
                emissions[0].data_source.clone(),
                emissions[0].balanced,
                emissions[0].date,
                emissions[0].value,
            );

            let expected_emissions: Vec<CO2Emissions> =
                Vec::from([default_emission_item(), emissions[0].clone()]);
            let emissions_from_state = contract.get_asset_emissions(asset_id);

            // Check asset's CO2 Emissions items
            assert!(emissions_from_state.is_some());
            assert!(expected_emissions
                .iter()
                .eq(emissions_from_state.unwrap().iter()));

            // Check asset's owner
            let owner_from_state = contract.owner_of(asset_id);
            assert!(owner_from_state.is_some());
            assert_eq!(new_owner, owner_from_state.unwrap());
        }

        #[ink::test]
        fn should_nonexistent_asset_query_emissions_work_properly() {
            let contract = InfinityAsset::new();
            assert!(contract.query_emissions(69).is_none());
        }

        #[ink::test]
        fn should_query_emissions_for_single_asset_work_properly() {
            let ((_accounts, contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            let emissions = Vec::from([default_emission_item()]);
            let metadata: Metadata = default_metadata();
            let parent = None;

            let expected_value: Vec<AssetDetails> = Vec::from([AssetDetails {
                asset_id,
                metadata,
                emissions,
                parent,
            }]);

            let details_from_state = contract.query_emissions(asset_id);

            // Check single asset tree
            assert!(details_from_state.is_some());
            assert_eq!(expected_value, details_from_state.unwrap());
        }

        #[ink::test]
        fn should_query_emissions_for_longer_path_work_properly() {
            let ((_accounts, mut contract), (mut asset_id, asset_owner)) = env_with_default_asset();

            let metadata = default_metadata();

            let emissions: Vec<CO2Emissions> = Vec::from([default_emission_item()]);

            let mut expected_tree_path: Vec<AssetDetails> = Vec::from([AssetDetails {
                asset_id,
                metadata: metadata.clone(),
                emissions,
                parent: None,
            }]);

            let timestamp = 1_000_000_000u64;

            set_caller(asset_owner);
            // create long token tree path
            for i in 1..1_000 {
                let parent: ParentDetails = Some(asset_id);

                let mut emissions = new_emissions(1);
                emissions[0].value = i;
                emissions[0].date = timestamp + i as u64;

                // Pause an asset
                assert!(contract.pause(asset_id).is_ok());

                // Blast child asset
                assert!(contract
                    .blast(
                        asset_owner,
                        metadata.clone(),
                        emissions.clone(),
                        parent.clone()
                    )
                    .is_ok());

                asset_id += 1;
                expected_tree_path.insert(
                    0,
                    AssetDetails {
                        asset_id,
                        metadata: metadata.clone(),
                        emissions,
                        parent,
                    },
                );
            }

            let details_from_state = contract.query_emissions(asset_id);

            // Check extended asset tree
            assert!(details_from_state.is_some());
            assert_eq!(expected_tree_path, details_from_state.unwrap());
        }

        #[ink::test]
        fn should_list_asset_for_empty_account_work_properly() {
            let (accounts, contract) = prepare_env();

            // Check if contract return proper value for `query_emissions`
            assert_eq!(contract.list_assets(accounts.alice), Vec::<AssetId>::new());
        }

        #[ink::test]
        fn should_list_asset_for_single_asset_work_properly() {
            let ((_accounts, contract), (asset_id, asset_owner)) = env_with_default_asset();

            set_caller(asset_owner);

            // Check if contract return proper value for `list_assets`
            assert_eq!(
                Vec::<AssetId>::from([asset_id]),
                contract.list_assets(asset_owner)
            );
        }

        #[ink::test]
        fn should_list_assets_for_many_assets_work_properly() {
            let ((_accounts, mut contract), (mut asset_id, asset_owner)) = env_with_default_asset();

            let metadata = default_metadata();

            let timestamp = 1_000_000_000u64;

            set_caller(asset_owner);

            // create long token tree path
            for i in 1..1_000 {
                let parent: ParentDetails = Some(asset_id);
                let mut emissions = new_emissions(1);
                emissions[0].value = i;
                emissions[0].date = timestamp + i as u64;

                // Pause an asset
                assert!(contract.pause(asset_id).is_ok());
                // Blast a child
                assert!(contract
                    .blast(asset_owner, metadata.clone(), emissions.clone(), parent)
                    .is_ok());

                asset_id += 1;
            }

            let mut assets_from_state = contract.list_assets(asset_owner);
            assets_from_state.sort();

            // Check if contract return proper value
            assert_eq!((1..1_001).collect::<Vec<AssetId>>(), assets_from_state);
        }

        #[ink::test]
        fn should_list_asset_after_transfer_work_properly() {
            let ((accounts, mut contract), (asset_id, asset_owner)) = env_with_default_asset();

            let new_owner = accounts.bob;

            set_caller(asset_owner);

            // Check owned assets before `transfer`
            assert_eq!(Vec::from([asset_id]), contract.list_assets(asset_owner));
            assert_eq!(Vec::<AssetId>::new(), contract.list_assets(new_owner));

            // Transfer asset
            assert!(contract
                .transfer(new_owner, asset_id, new_emissions(1))
                .is_ok());

            // Check owned assets after `transfer`
            assert_eq!(Vec::<AssetId>::new(), contract.list_assets(asset_owner));
            assert_eq!(Vec::from([asset_id]), contract.list_assets(new_owner));
        }
    }
}
