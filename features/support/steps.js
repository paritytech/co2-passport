const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");

 Given('I have the environment prepared.', async function () {
     await this.prepareEnvironment();
 });

 When('{string} creates an asset with metadata: {string} and {string} emissions with the amount: {int} Grams per kilo CO2 emitted from date: {int}.', async function (caller, metadata, emission_category, emissions, date) {
     await this.blastAsset(caller, metadata, emission_category, emissions, date);
     this.addTo("I");
 });

 Then('the following events will be emitted:', function (jsonString) {
    let events = JSON.parse(jsonString);
    expect(this.events).to.deep.equal(events);
 });
