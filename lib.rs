#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod asset_co2_emissions {
    use ink::prelude::vec::Vec;

    /// Asset ID type.
    // TODO proper ID type
    pub type AssetId = u128;

    pub type Metadata = Vec<u8>;
    pub type RoleId = AccountId;
    pub type ParentRelation = u128;
    pub type ParentDetails = (AssetId, ParentRelation);
    pub type AssetDetails = (AssetId, Metadata, Vec<CO2Emissions>, Option<ParentDetails>);
    pub type Description = Vec<u8>;

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    enum EmissionsCategory {
        Process,
        Transport,
        Upstream,
    }

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    enum EmissionsOrigin {
        Supplier { id: Vec<u8> },
        Hybrid,
        IndustryAverage,
    }

    /// The AssetCO2Emissions error types.
    #[derive(Debug, PartialEq, Eq, Copy, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AssetCO2EmissionsError {
        // When an Asset does not exist.
        AssetNotFound,
        // When an Asset has been already `Paused`.
        AlreadyPaused,
        // When not an Asset's Owner wants to take any action over the Asset.
        NotOwner,
        // When an Asset is not in a `Paused` state.
        NotPaused,
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
    pub struct Minted {
        #[ink(topic)]
        id: AssetId,
        #[ink(topic)]
        metadata: Metadata,
        #[ink(topic)]
        owner: AccountId,
    }

    /// This emits when ownership of any Asset changes.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
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
        #[ink(topic)]
        category: EmissionsCategory,
        #[ink(topic)]
        origin: EmissionsOrigin,
        #[ink(topic)]
        emissions: u128,
    }

    /// This emits when a new Role gets created.
    #[ink(event)]
    pub struct RoleCreated {
        #[ink(topic)]
        id: RoleId,
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        description: Description,
    }

    /// This emits when an owner of the Role has changed.
    #[ink(event)]
    pub struct RoleOwnershipChanged {
        #[ink(topic)]
        id: RoleId,
        #[ink(topic)]
        new_owner: AccountId,
    }

    /// This emits when a Role members have changed.
    #[ink(event)]
    pub struct RoleMembershipChanged {
        #[ink(topic)]
        id: RoleId,
    }

    #[derive(scale::Encode, scale::Decode)]
    pub struct CO2Emissions {
        // Type of CO2 Emissions (bucket)
        category: EmissionsCategory,
        // Origin of CO2 Emissions
        origin: EmissionsOrigin,
        // Emissions in kg CO2 (to avoid fractions).
        emissions: u128,
        // Real CO2 emissions date as UNIX timestamp, not block creation time.
        date: i64,
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
        /// * `EmptyRole` - Account list cannot be emty.
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

        /// Mint an Asset.
        ///
        /// # Arguments
        ///
        /// * `to` - The account that will own the Asset.
        /// * `metadata` - Immutable asset's metadata (physical details of steel); Can be a string, a JSON string or a link to IPFS.
        /// * `emissions` - CO2 emissions during asset creation (like minting or splitting steel).
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
        /// * `Minted` - When an Asset gets minted.
        /// * `Emissions` - When CO2 emissions are added.
        ///
        #[ink(message)]
        fn mint(
            &mut self,
            to: AccountId,
            metadata: Metadata,
            emissions: Vec<CO2Emissions>,
            parent: Option<ParentDetails>,
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
        fn get_parent_details(&self, id: AssetId) -> Option<Option<ParentDetails>>;

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
    pub struct GaiaAsset {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl GaiaAsset {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Sample unit test
        #[test]
        fn sample_unit_test() {
            assert_eq!(1 + 1, 2);
        }

        /// Sample ink! unit test
        #[ink::test]
        fn sample_ink_unit_test() {
            assert_eq!(1 + 1, 2);
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
