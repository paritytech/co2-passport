#![cfg_attr(not(feature = "std"), no_std)]

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

    pub const MAX_METADATA_LENGTH: u16 = 250;
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
        // Invalid smart contract state
        InvalidSmartContractState,
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
    pub struct InfinityAsset {
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
                next_id: 1,
                asset_owner: Mapping::new(),
                owned_assets: BTreeMap::new(),
                co2_emissions: Mapping::new(),
                metadata: Mapping::new(),
                paused: Mapping::new(),
                parent: Mapping::new(),
            }
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
                },
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
            match self.owned_assets.get_mut(owner) {
                None => Err(AssetCO2EmissionsError::InvalidSmartContractState),
                Some(owned_assets) => {
                    owned_assets.remove(asset_id);
                    Ok(())
                }
            }
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
                    let _ = self.ensure_owner(parent_id, caller)?;
                    let _ = match relation {
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
                return Err(AssetCO2EmissionsError::EmissionsOverflow)
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
                return Err(AssetCO2EmissionsError::MetadataOverflow)
            }
            Ok(())
        }

        fn save_new_emissions(&mut self, id: &AssetId, emissions: &Vec<CO2Emissions>) -> Result<(), AssetCO2EmissionsError> {
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
                .map_or_else(|| Err(AssetCO2EmissionsError::AssetIdOverflow), |id| Ok(id))?;
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
                let asset: AssetDetails = self.get_asset(asset_id).expect("Asset existence already checked");
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
                Some(owned_assets) => owned_assets
                    .into_iter()
                    .map(|asset| *asset)
                    .collect::<Vec<AssetId>>(),
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

            let _ = self.ensure_proper_metadata(&metadata)?;
            let _ = self.ensure_emissions_correct(&emissions)?;
            let _ = self.ensure_proper_parent(&parent, &caller)?;

            let asset_id: u128 = self.next_id()?;
            let _ = self.ensure_not_exist(&asset_id)?;

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
            self.save_new_emissions(&asset_id, &emissions)?;

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

            let _ = self.ensure_exists(&id)?;
            let _ = self.ensure_owner(&id, &from)?;
            let _ = self.ensure_not_paused(&id)?;
            let _ = self.ensure_emissions_correct(&emissions)?;

            self.remove_owned_asset(&from, &id)?;
            self.insert_owned_asset(&to, &id)?;

            self.asset_owner.insert(id, &to);

            self.env().emit_event(Transfer { from, to, id });

            // Save CO2 emissions & emit corresponding events
            self.save_new_emissions(&id, &emissions)?;

            Ok(())
        }

        #[ink(message)]
        fn pause(&mut self, id: AssetId) -> Result<(), AssetCO2EmissionsError> {
            let _ = self.ensure_owner(&id, &self.env().caller())?;
            let _ = self.ensure_not_paused(&id)?;

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
            let _ = self.ensure_exists(&id)?;
            let _ = self.ensure_owner(&id, &self.env().caller())?;
            let _ = self.ensure_not_paused(&id)?;
            let _ = self.ensure_emissions_item_correct(&emissions)?;

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
                    let emissions = self.get_asset_emissions(id).expect("Must exist");
                    let parent = self.get_parent_details(id).expect("Must exist");

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

    /// Unit tests
    #[cfg(test)]
    mod tests {
        use ink::env::test;
        use ink::env::DefaultEnvironment;

        use super::*;

        use ink::primitives::{Clear, Hash};

        type Event = <InfinityAsset as ::ink::reflect::ContractEventBase>::Type;

        fn get_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }
        fn set_caller(sender: AccountId) {
            test::set_caller::<DefaultEnvironment>(sender);
        }

        fn new_emissions(
            category: EmissionsCategory,
            primary: bool,
            balanced: bool,
            emissions: u128,
            date: u64,
        ) -> CO2Emissions {
            CO2Emissions {
                category,
                primary,
                balanced,
                emissions,
                date,
            }
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
                .expect("encountered invalid contract event data buffer");
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
                .expect("encountered invalid contract event data buffer");
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
            expected_primary: bool,
            expected_balanced: bool,
            expected_date: u64,
            expected_emissions: u128,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Emission(Emission {
                id,
                category,
                primary,
                balanced,
                date,
                emissions,
            }) = decoded_event
            {
                assert_eq!(id, expected_id, "encountered invalid Emission.id");
                assert_eq!(
                    category, expected_category,
                    "encountered invalid Emission.category"
                );
                assert_eq!(
                    primary, expected_primary,
                    "encountered invalid Emission.primary"
                );
                assert_eq!(
                    balanced, expected_balanced,
                    "encountered invalid Emission.balanced"
                );
                assert_eq!(date, expected_date, "encountered invalid Emission.date");
                assert_eq!(
                    emissions, expected_emissions,
                    "encountered invalid Emission.emissions"
                );
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
                .expect("encountered invalid contract event data buffer");
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
        fn should_reject_empty_emissions_during_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;
            let emissions: Vec<CO2Emissions> = Vec::new();

            assert_eq!(
                contract.blast(accounts.alice, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::EmissionsEmpty)
            );
        }

        #[ink::test]
        fn should_reject_single_zero_emissions_item_during_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions: u128 = 0;
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                emissions,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            assert_eq!(
                contract.blast(accounts.alice, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_reject_zero_emissions_item_in_array_during_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let item0 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                0u128,
                timestamp,
            );

            let item1 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                1u128,
                timestamp,
            );

            let item2 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                2u128,
                timestamp,
            );
            let emissions: Vec<CO2Emissions> = Vec::from([item1, item0, item2]);

            assert_eq!(
                contract.blast(accounts.alice, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_blast_with_single_emissions_item() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let e: u128 = 1;
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );
            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let owner = accounts.bob;

            assert!(contract
                .blast(owner, metadata.clone(), emissions, parent)
                .is_ok());

            let expected_asset_id = 1;

            let emitted_events = test::recorded_events().collect::<Vec<_>>();
            // 1 * Blasted + 1 * Emissions
            assert_eq!(1 + 1, emitted_events.len());
            assert_blasted_event(
                &emitted_events[0],
                expected_asset_id,
                metadata,
                owner,
                parent,
            );
            assert_emissions_event(
                &emitted_events[1],
                expected_asset_id,
                emissions_category,
                emissions_primary,
                emissions_balanced,
                timestamp,
                e,
            );
        }

        #[ink::test]
        fn should_blast_with_multiple_emissions_items() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e1 = 1u128;
            let item1 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e1,
                timestamp,
            );

            let e2 = 2u128;
            let item2 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e2,
                timestamp,
            );

            let e3 = 3u128;
            let item3 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e3,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item1, item2, item3]);

            let owner = accounts.eve;

            assert!(contract
                .blast(owner, metadata.clone(), emissions, parent)
                .is_ok());

            let expected_asset_id = 1;

            let emitted_events = test::recorded_events().collect::<Vec<_>>();
            // 1 * Blasted + 3 * Emissions
            assert_eq!(1 + 3, test::recorded_events().count());
            assert_blasted_event(
                &emitted_events[0],
                expected_asset_id,
                metadata,
                owner,
                parent,
            );
            assert_emissions_event(
                &emitted_events[1],
                expected_asset_id,
                emissions_category,
                emissions_primary,
                emissions_balanced,
                timestamp,
                e1,
            );
            assert_emissions_event(
                &emitted_events[2],
                expected_asset_id,
                emissions_category,
                emissions_primary,
                emissions_balanced,
                timestamp,
                e2,
            );
            assert_emissions_event(
                &emitted_events[3],
                expected_asset_id,
                emissions_category,
                emissions_primary,
                emissions_balanced,
                timestamp,
                e3,
            );
        }

        #[ink::test]
        fn should_nonexistent_get_emissions_work_properly() {
            let contract = InfinityAsset::new();
            assert!(contract.get_asset_emissions(1000).is_none());
        }

        #[ink::test]
        fn should_get_emissions_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e1 = 1u128;
            let item1 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e1,
                timestamp,
            );

            let e2 = 2u128;
            let item2 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e2,
                timestamp,
            );

            let e3 = 3u128;
            let item3 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e3,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item1, item2, item3]);

            let asset_id = 1;

            assert!(contract
                .blast(accounts.eve, metadata, emissions.clone(), parent)
                .is_ok());

            let emissions_from_state = contract.get_asset_emissions(asset_id);
            assert!(emissions_from_state.is_some());
            assert!(emissions.iter().eq(emissions_from_state.unwrap().iter()));
        }

        #[ink::test]
        fn should_nonexistent_get_metadata_work_properly() {
            let contract = InfinityAsset::new();
            assert!(contract.get_metadata(1000).is_none());
        }

        #[ink::test]
        fn should_get_metadata_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;

            assert!(contract
                .blast(accounts.eve, metadata.clone(), emissions, parent)
                .is_ok());

            let metadata_from_state = contract.get_metadata(asset_id);
            assert!(metadata_from_state.is_some());
            assert!(metadata.iter().eq(metadata_from_state.unwrap().iter()));
        }

        #[ink::test]
        fn should_nonexistent_get_parent_work_properly() {
            let contract = InfinityAsset::new();
            assert!(contract.get_parent_details(1000).is_none());
        }

        #[ink::test]
        fn should_get_parent_for_root_asset_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;

            assert!(contract
                .blast(accounts.eve, metadata, emissions, parent)
                .is_ok());

            let parent_from_state = contract.get_parent_details(asset_id);
            assert!(parent_from_state.is_some());
            assert_eq!(parent, parent_from_state.unwrap());
        }

        #[ink::test]
        fn should_owner_of_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.eve;

            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());

            let owner_from_state = contract.owner_of(asset_id);
            assert!(owner_from_state.is_some());
            assert_eq!(owner, owner_from_state.unwrap());
        }

        #[ink::test]
        fn should_already_blasted_asset_not_be_paused() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.eve;

            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());

            let paused = contract.has_paused(asset_id);
            assert!(paused.is_some());
            assert!(!paused.unwrap());
        }

        #[ink::test]
        fn should_not_owner_not_be_able_to_set_paused_state() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.eve;

            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());

            set_caller(accounts.bob);
            assert_eq!(
                contract.pause(asset_id),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_owner_be_able_to_set_paused_state() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.eve;

            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());

            set_caller(owner);
            assert!(contract.pause(asset_id).is_ok());

            let emitted_events = test::recorded_events().collect::<Vec<_>>();
            // 1* Blasted + 1 * Emissions + 1 * Paused
            assert_eq!(1 + 1 + 1, test::recorded_events().count());
            assert_paused_event(&emitted_events[2], asset_id);
        }

        #[ink::test]
        fn should_owner_not_be_able_to_set_paused_state_while_already_paused() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.eve;

            set_caller(owner);
            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());
            assert!(contract.pause(asset_id).is_ok());
            assert_eq!(
                contract.pause(asset_id),
                Err(AssetCO2EmissionsError::AlreadyPaused)
            );
        }

        #[ink::test]
        fn should_reject_non_existent_parent_in_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;
            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let owner = accounts.alice;

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let parent: ParentDetails = Some((1000, 85));

            set_caller(owner);
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_reject_0_parent_relation_in_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            set_caller(owner);

            assert!(contract.pause(asset_id).is_ok());

            let parent: ParentDetails = Some((asset_id, 0));
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::InvalidAssetRelation)
            );
        }

        #[ink::test]
        fn should_reject_not_owner_creating_child_in_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let parent: ParentDetails = Some((asset_id, 101));

            set_caller(accounts.eve);
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_reject_not_paused_in_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            set_caller(owner);

            let parent: ParentDetails = Some((asset_id, 90));
            assert_eq!(
                contract.blast(owner, metadata, emissions, parent),
                Err(AssetCO2EmissionsError::NotPaused)
            );
        }

        #[ink::test]
        fn should_blast_child() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            set_caller(owner);

            let parent: ParentDetails = Some((asset_id, 100));
            assert!(contract.pause(asset_id).is_ok());
            assert!(contract
                .blast(owner, metadata.clone(), emissions, parent)
                .is_ok());

            let expected_asset_id = 2;

            let emitted_events = test::recorded_events().collect::<Vec<_>>();
            // 1 * Blasted + 1 * Emissions + 1 * Paused + 1 * Blasted + 1 * Emissions
            assert_eq!(1 + 1 + 1 + 1 + 1, emitted_events.len());
            assert_blasted_event(
                &emitted_events[3],
                expected_asset_id,
                metadata,
                owner,
                parent,
            );
            assert_emissions_event(
                &emitted_events[4],
                expected_asset_id,
                emissions_category,
                emissions_primary,
                emissions_balanced,
                timestamp,
                e,
            );

            let parent_from_state = contract.get_parent_details(expected_asset_id);
            assert!(parent_from_state.is_some());
            assert!(parent_from_state.unwrap().is_some());
            assert_eq!(parent, parent_from_state.unwrap());
        }

        #[ink::test]
        fn should_not_add_emissions_to_nonexistent_asset() {
            let _accounts = get_accounts();

            let mut contract = InfinityAsset::new();

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let asset_id = 1;

            assert_eq!(
                contract.add_emissions(asset_id, item),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_not_owner_not_be_able_to_add_emissions() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.eve;

            set_caller(owner);
            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());

            set_caller(accounts.bob);
            let e_1 = 100u128;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );
            assert_eq!(
                contract.add_emissions(asset_id, new_emissions_item),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_reject_paused_in_add_emissions() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            assert!(contract.pause(asset_id).is_ok());

            let e_1 = 100u128;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );
            assert_eq!(
                contract.add_emissions(asset_id, new_emissions_item),
                Err(AssetCO2EmissionsError::AlreadyPaused)
            );
        }

        #[ink::test]
        fn should_reject_zero_emissions_item_in_add_emissions() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let e_1 = 0u128;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );
            assert_eq!(
                contract.add_emissions(asset_id, new_emissions_item),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_owner_be_able_to_add_emissions() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item.clone()]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let e_1 = 69u128;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );
            assert!(contract
                .add_emissions(asset_id, new_emissions_item.clone())
                .is_ok());

            let emitted_events = test::recorded_events().collect::<Vec<_>>();
            // 1 * Blasted + 1 * Emissions + 1 * Emissions
            assert_eq!(1 + 1 + 1, emitted_events.len());
            assert_emissions_event(
                &emitted_events[2],
                asset_id,
                emissions_category,
                emissions_primary,
                emissions_balanced,
                timestamp,
                e_1,
            );

            let expected_emissions: Vec<CO2Emissions> = Vec::from([item, new_emissions_item]);
            let emissions_from_state = contract.get_asset_emissions(asset_id);
            assert!(emissions_from_state.is_some());
            assert!(expected_emissions
                .iter()
                .eq(emissions_from_state.unwrap().iter()));
        }

        #[ink::test]
        fn should_not_transfer_nonexistent_asset() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let asset_id = 1;

            set_caller(accounts.alice);

            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::from([item])),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_not_owner_not_be_able_to_transfer() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.eve;

            set_caller(owner);
            assert!(contract.blast(owner, metadata, emissions, parent).is_ok());

            set_caller(accounts.bob);
            let e_1 = 100u128;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );
            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::from([new_emissions_item])),
                Err(AssetCO2EmissionsError::NotOwner)
            );
        }

        #[ink::test]
        fn should_reject_transfer_in_paused() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            assert!(contract.pause(asset_id).is_ok());

            let e_1 = 100u128;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );

            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::from([new_emissions_item])),
                Err(AssetCO2EmissionsError::AlreadyPaused)
            );
        }

        #[ink::test]
        fn should_reject_empty_emissions_in_transfer() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::new()),
                Err(AssetCO2EmissionsError::EmissionsEmpty)
            );
        }

        #[ink::test]
        fn should_reject_too_many_emissions_on_blast() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let mut emissions: Vec<CO2Emissions> = Vec::new();
            for _ in 0..MAX_EMISSIONS_PER_ASSET + 1{
                let e = 1u128;
                let item = new_emissions(
                    emissions_category,
                    emissions_primary,
                    emissions_balanced,
                    e,
                    timestamp,
                );
               emissions.push(item);
            }

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert_eq!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent), Err(AssetCO2EmissionsError::EmissionsOverflow));

            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::new()),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_reject_too_many_emissions_on_add() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let mut emissions: Vec<CO2Emissions> = Vec::new();
            for _ in 0..MAX_EMISSIONS_PER_ASSET {
               emissions.push(item.clone());
            }

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent).is_ok());

            assert_eq!(
                contract.add_emissions(asset_id, item.clone()),
                Err(AssetCO2EmissionsError::EmissionsOverflow)
            );

            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::from([item.clone()])),
                Err(AssetCO2EmissionsError::EmissionsOverflow)
            );

        }

        #[ink::test]
        fn should_reject_zero_emissions_item_in_transfer() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let e_1 = 0u128;
            let new_emissions_item_0 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );

            let new_emissions_item_1 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );
            let new_emissions_item_2 = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );

            assert_eq!(
                contract.transfer(
                    accounts.bob,
                    asset_id,
                    Vec::from([
                        new_emissions_item_1,
                        new_emissions_item_0,
                        new_emissions_item_2
                    ])
                ),
                Err(AssetCO2EmissionsError::ZeroEmissionsItem)
            );
        }

        #[ink::test]
        fn should_reject_too_many_chars_in_metadata() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let mut metadata: Metadata = Vec::new();
            for _ in 0..MAX_METADATA_LENGTH + 1 {
                metadata.push(0u8);
            }

            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let mut emissions: Vec<CO2Emissions> = Vec::new();
            for _ in 0..MAX_EMISSIONS_PER_ASSET + 1{
                let e = 1u128;
                let item = new_emissions(
                    emissions_category,
                    emissions_primary,
                    emissions_balanced,
                    e,
                    timestamp,
                );
               emissions.push(item);
            }

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert_eq!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent), Err(AssetCO2EmissionsError::MetadataOverflow));

            assert_eq!(
                contract.transfer(accounts.bob, asset_id, Vec::new()),
                Err(AssetCO2EmissionsError::AssetNotFound)
            );
        }

        #[ink::test]
        fn should_owner_be_able_to_transfer() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item.clone()]);

            let asset_id = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let e_1 = 69u128;
            let new_owner = accounts.bob;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );

            assert!(contract
                .transfer(new_owner, asset_id, Vec::from([new_emissions_item.clone()]))
                .is_ok());

            let emitted_events = test::recorded_events().collect::<Vec<_>>();
            // 1 * Blasted + 1 * Emissions + 1 * Transfer + 1 * Emissions
            assert_eq!(1 + 1 + 1 + 1, emitted_events.len());

            assert_transfer_event(&emitted_events[2], asset_id, owner, new_owner);

            assert_emissions_event(
                &emitted_events[3],
                asset_id,
                emissions_category,
                emissions_primary,
                emissions_balanced,
                timestamp,
                e_1,
            );

            let expected_emissions: Vec<CO2Emissions> = Vec::from([item, new_emissions_item]);
            let emissions_from_state = contract.get_asset_emissions(asset_id);
            assert!(emissions_from_state.is_some());
            assert!(expected_emissions
                .iter()
                .eq(emissions_from_state.unwrap().iter()));

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
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id: AssetId = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let expected_value: Vec<AssetDetails> =
                Vec::from([(asset_id, metadata, emissions, parent)]);
            let details_from_state = contract.query_emissions(asset_id);
            assert!(details_from_state.is_some());
            assert_eq!(expected_value, details_from_state.unwrap());
        }

        #[ink::test]
        fn should_query_emissions_for_longer_path_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;

            let e = 1u128;
            let item = new_emissions(emissions_category, emissions_primary, true, e, timestamp);

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let mut asset_id: AssetId = 1;
            let owner = accounts.alice;

            set_caller(owner);
            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            let mut expected_tree_path: Vec<AssetDetails> =
                Vec::from([(asset_id, metadata.clone(), emissions, parent)]);

            // create long token tree path
            for i in 1..1_000 {
                let parent: ParentDetails = Some((asset_id, (100 - (i % 100))));
                let e = i as u128;
                let item = new_emissions(
                    EmissionsCategory::Process,
                    false,
                    false,
                    e,
                    timestamp + i as u64,
                );
                let emissions: Vec<CO2Emissions> = Vec::from([item]);
                assert!(contract.pause(asset_id).is_ok());
                assert!(contract
                    .blast(owner, metadata.clone(), emissions.clone(), parent)
                    .is_ok());

                asset_id += 1;
                expected_tree_path.insert(0, (asset_id, metadata.clone(), emissions, parent));
            }

            let details_from_state = contract.query_emissions(asset_id);

            assert!(details_from_state.is_some());

            assert_eq!(expected_tree_path, details_from_state.unwrap());
        }

        #[ink::test]
        fn should_list_asset_for_empty_account_work_properly() {
            let contract = InfinityAsset::new();
            assert_eq!(
                contract.list_assets(get_accounts().alice),
                Vec::<AssetId>::new()
            );
        }

        #[ink::test]
        fn should_list_asset_for_single_asset_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let asset_id: AssetId = 1;
            let owner = accounts.alice;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            assert_eq!(
                Vec::<AssetId>::from([asset_id]),
                contract.list_assets(owner)
            );
        }

        #[ink::test]
        fn should_list_assets_for_many_assets_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;

            let e = 1u128;
            let item = new_emissions(emissions_category, emissions_primary, true, e, timestamp);

            let emissions: Vec<CO2Emissions> = Vec::from([item]);

            let mut asset_id: AssetId = 1;
            let owner = accounts.alice;

            set_caller(owner);
            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            // create long token tree path
            for i in 1..1_000 {
                let parent: ParentDetails = Some((asset_id, (100 - (i % 100))));
                let e = i as u128;
                let item = new_emissions(
                    EmissionsCategory::Process,
                    false,
                    false,
                    e,
                    timestamp + i as u64,
                );
                let emissions: Vec<CO2Emissions> = Vec::from([item]);
                assert!(contract.pause(asset_id).is_ok());
                assert!(contract
                    .blast(owner, metadata.clone(), emissions.clone(), parent)
                    .is_ok());

                asset_id += 1;
            }

            let mut assets_from_state = contract.list_assets(owner);
            assets_from_state.sort();

            assert_eq!((1..1_001).collect::<Vec<AssetId>>(), assets_from_state);
        }

        #[ink::test]
        fn should_list_asset_after_transfer_work_properly() {
            let accounts = get_accounts();

            let mut contract = InfinityAsset::new();
            let metadata: Metadata = Vec::from([0u8, 1u8, 2u8, 3u8]);
            let parent = None;

            let timestamp: u64 = 1682632800; // 28.04.2023 00:00:00
            let emissions_category = EmissionsCategory::Upstream;
            let emissions_primary = true;
            let emissions_balanced = true;

            let e = 1u128;
            let item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e,
                timestamp,
            );

            let emissions: Vec<CO2Emissions> = Vec::from([item.clone()]);

            let asset_id = 1;
            let owner = accounts.alice;
            let new_owner = accounts.bob;

            set_caller(owner);

            assert!(contract
                .blast(owner, metadata.clone(), emissions.clone(), parent)
                .is_ok());

            assert_eq!(Vec::from([asset_id]), contract.list_assets(owner));
            assert_eq!(Vec::<AssetId>::new(), contract.list_assets(new_owner));

            let e_1 = 69u128;
            let new_emissions_item = new_emissions(
                emissions_category,
                emissions_primary,
                emissions_balanced,
                e_1,
                timestamp,
            );

            assert!(contract
                .transfer(new_owner, asset_id, Vec::from([new_emissions_item.clone()]))
                .is_ok());

            assert_eq!(Vec::<AssetId>::new(), contract.list_assets(owner));
            assert_eq!(Vec::from([asset_id]), contract.list_assets(new_owner));
        }
    }

    /// E2E tests
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// Sample e2e test
        #[ink_e2e::test]
        async fn sample_e2e_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            Ok(())
        }
    }
}
