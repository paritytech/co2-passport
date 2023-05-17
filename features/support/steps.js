const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");

/* User Story 1 (us1) */

Given('I have the environment prepared.', async function () {
    await this.prepareEnvironment();
});

When('{string} creates an asset with metadata: {string} and {string} emissions with the amount: {int} Grams per kilo CO2 emitted from date: {int}.', async function (caller, metadata, emission_category, emissions, date) {
    await this.blastAsset(caller, metadata, emission_category, emissions, date);
});

Then('the following events will be emitted:', function (jsonString) {
  let events = JSON.parse(jsonString);
  expect(this.events).to.deep.equal(events);
});

Given('the {string} has blasted the asset with the following parameters:', async function (caller, jsonString) {
  let asset = JSON.parse(jsonString);

  await this.prepareEnvironment();
  await this.blastAsset(caller, asset.metadata, asset.emission_category, asset.emissions, asset.date);

});

When('{string} transfers asset with ID {int} to {string} with new {string} emission with the amount of {int} grams per kilo on the date {int}',
async function (seller, assetId, buyer, emission_category, emissions, date) {
      await this.transferAsset(seller, assetId, buyer, emission_category, emissions, date);
});

Then('the following transfer events will be emitted:', function (jsonString) {
  let events = JSON.parse(jsonString);
  expect(this.events).to.deep.equal(events);
});


/* User Story 2 (us2) */

Given('A {string} creates an asset that is split into child assets where the asset is defined as:', async function (string, jsonString) {
  let asset = JSON.parse(jsonString);

  await this.prepareEnvironment();
  await this.blastAsset(caller, asset.metadata, asset.emission_category, asset.emissions, asset.date);
});

When('{string} performs a query on the asset with ID {int}', function (string, int) {
  // When('{string} performs a query on the asset with ID {float}', function (string, float) {
    // Write code here that turns the phrase above into concrete actions
    return 'pending';
});

Then('the following result should be returned', function (docString) {
  // Write code here that turns the phrase above into concrete actions
  return 'pending';
});