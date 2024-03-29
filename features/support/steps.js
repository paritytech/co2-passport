const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");
const fs = require("fs");

/* User Story 1 (us1) */

Given("I have the environment prepared.", async function () {
	await this.prepareEnvironment();
});

When(
	"{string} blasts an asset defined as the following:",
	async function (caller, jsonString) {
		let asset = JSON.parse(jsonString);
		let assetParent = null;
		await this.blastAsset(
			caller,
			JSON.stringify(asset.metadata),
			assetParent,
			asset.emissions
		);
	}
);

Then(
	"The asset {int} and emitted events will be the following:",
	async function (assetId, jsonString) {
		let { asset, events } = JSON.parse(jsonString);
		let assetDetails = await this.getAsset("Eve", assetId);

		expect(assetDetails).to.deep.equal(asset);
		expect(this.events).to.deep.equal(events);
	}
);

Given(
	"The {string} has blasted the asset with the following parameters:",
	async function (caller, jsonString) {
		let asset = JSON.parse(jsonString);

		let assetParent = null;

		await this.prepareEnvironment();
		await this.blastAsset(
			caller,
			asset.metadata,
			assetParent,
			asset.emissions
		);
	}
);

When(
	"{string} transfers asset with ID {int} to {string} with emissions of:",
	async function (seller, assetId, buyer, jsonString) {
		let emissions = JSON.parse(jsonString);

		let err = await this.transferAsset("Eve", assetId, buyer, emissions);
		expect(err).to.equal("NotOwner");

		await this.transferAsset(seller, assetId, buyer, emissions);
	}
);

Then(
	"{string} will be the new owner of asset {int}, the emissions and transfer events will be the following:",
	async function (sender, assetId, jsonString) {
		let { emissions, events } = JSON.parse(jsonString);

		expect(this.events).to.deep.equal(events);

		let ownerOf = await this.getOwnerOf(sender, assetId);
		let assetDetails = await this.getAsset(sender, assetId);
		expect(ownerOf).to.equal(this.accounts[sender].address);
		expect(assetDetails.emissions).to.deep.equal(emissions);
	}
);

Given(
	"The {string} has blasted the following asset:",
	async function (caller, jsonString) {
		let asset = JSON.parse(jsonString);

		let assetParent = null;

		await this.prepareEnvironment();
		await this.blastAsset(
			caller,
			asset.metadata,
			assetParent,
			asset.emissions
		);
	}
);

When(
	"{string} adds the following emission to the asset with ID {int}:",
	async function (seller, assetId, jsonString) {
		let emission = JSON.parse(jsonString);

		let err = await this.addEmission("Eve", assetId, emission);
		expect(err).to.equal("NotOwner");
		await this.addEmission(seller, assetId, emission);
	}
);

Then("The asset {int} will be:", async function (assetId, jsonString) {
	let { emissions, events } = JSON.parse(jsonString);

	let assetDetails = await this.getAsset("Eve", assetId);

	expect(this.events).to.deep.equal(events);
	expect(assetDetails.emissions).to.deep.equal(emissions);
});

Given(
	"The {string} blasts the following parent asset:",
	async function (caller, jsonString) {
		let asset = JSON.parse(jsonString);

		let assetParent = null;

		await this.prepareEnvironment();
		await this.blastAsset(
			caller,
			JSON.stringify(asset.metadata),
			assetParent,
			asset.emissions
		);
	}
);

When(
	"{string} pauses the parent asset {int} and creates a child asset, which creates a child, defined as:",
	async function (caller, assetId, jsonString) {
		let assets = JSON.parse(jsonString);

		let err = await this.pauseAsset("Eve", assetId);
		expect(err).to.equal("NotOwner");

		// try to blast child asset with un-paused parent
		err = await this.blastAsset(
			caller,
			JSON.stringify(assets[0].metadata),
			assetId,
			[
				{
					category: "Upstream",
					balanced: true,
					date: 1755040054,
					value: 10,
				},
			]
		);
		// ensure error is thrown
		expect(err).to.equal("NotPaused");

		// Process to blast child asset:
		// 1: Pause Parent Asset
		// 2: Blast child asset with parent id as parent and weight relation
		// 3. Repeat to create child of child

		// Use helper to create asset tree
		await this.createAssetTree(caller, assets, assetId);
	}
);

Then(
	"The asset {int} when queried will equal the following asset tree:",
	async function (assetId, jsonString) {
		let assets = JSON.parse(jsonString);

		await this.queryEmissions("Eve", assetId);

		expect(this.readOutput).to.deep.equal(assets);
	}
);

/* User Story 2 (us2) */

Given(
	"A {string} creates an asset that is split into child assets where the assets are defined as:",
	async function (caller, jsonString) {
		let assets = JSON.parse(jsonString);

		await this.prepareEnvironment();
		await this.createAssetTree(caller, assets);
	}
);

When(
	"{string} performs a query on the asset with ID {int}",
	async function (caller, assetId) {
		await this.queryEmissions(caller, assetId);
	}
);
Then(
	"The emissions can be calculated offchain for {string} emissions between the dates {int} and {int} and weight {int} with the total equal to {float} based on the following:",
	function (
		filterCategory,
		filterDateFrom,
		filterDateTo,
		weight,
		expectedEmissions,
		docString
	) {
		expect(this.readOutput).to.deep.equal(JSON.parse(docString));

		// The recursive formula is defined as:
		// let e_n = the emissions of the asset at the nth level
		// let w_n = the weight of the asset at nth level
		// let Te_n = the total emissions of the asset at the nth level
		// Te_0 = e_0 * w_0
		// Te_1 = e_1 * w_1 + Te_0
		// Te_2 = e_2 * w_2 + Te_1 + Te_2
		// ...
		// Te_n = e_n * w_n + Te_n-1 + Te_n-2 + ... + Te_0

		let totalEmissions = 0;
		for (let asset of this.readOutput.reverse()) {
			let emissions = asset.emissions;

			// Calculate total emissions for the current asset.
			// Removes emissions outside the date range of filterDateFrom and filterDateTo
			// and filters by category
			const totalAssetEmissions = emissions
				.filter((emission) => {
					return (
						emission.date > filterDateFrom &&
						emission.date < filterDateTo &&
						emission.category === filterCategory
					);
				})
				.reduce(
					(total, emission) => total + emission.value * weight,
					0
				);

			totalEmissions = totalEmissions + totalAssetEmissions;
		}

		expect(totalEmissions).to.equal(expectedEmissions);
	}
);

/* User Story 3 (us3) */
Given("The original contract is deployed", async function () {
	await this.prepareEnvironment();
});

When("The contract owner upgrades the contract", async function () {
	const contract = JSON.parse(
		fs.readFileSync(
			"./integration-tests/updated-contract/target/ink/updated_contract.contract"
		)
	);
	await this.deploySmartContract(contract);
	await this.upgradeContract(contract);
});

Then("The contract will be upgraded", async function () {
	// This should return `AlreadyPaused` error to showcase the contract upgraded
	let err = await this.setContractOwner("Seller", "Seller");
	expect(err).to.equal("AlreadyPaused");
});
